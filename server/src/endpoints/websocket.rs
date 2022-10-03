use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Extension,
    },
    response::IntoResponse,
};
use dragonfly::RedisPool;
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::mpsc;
use std::sync::Arc;
use tokio::sync::broadcast;

const DEFAULT_ROOM_NAME: &'static str = "test-room";

pub struct AppState {
    redis_pool: RedisPool,
    broadcaster: broadcast::Sender<String>,
    publisher: mpsc::SyncSender<domain::models::ChatMessage>,
}

impl AppState {
    pub fn new(
        redis_pool: RedisPool,
        broadcaster: broadcast::Sender<String>,
        publisher: mpsc::SyncSender<domain::models::ChatMessage>,
    ) -> Self {
        Self {
            redis_pool,
            broadcaster,
            publisher,
        }
    }
}

pub async fn handler(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state))
}

async fn websocket(stream: WebSocket, state: Arc<AppState>) {
    // By splitting we can send and receive at the same time.
    let (mut sender, mut receiver) = stream.split();

    // Loop until a text message is found.
    let mut username = String::new();
    while let Some(Ok(message)) = receiver.next().await {
        if let Message::Text(name) = message {
            username = name;
            break;
        }
    }
    tracing::debug!("username: {}", username);

    let chat_room_user = match domain::services::chat_room::ChatRoomUser::try_new(
        state.redis_pool.clone(),
        DEFAULT_ROOM_NAME,
        &username,
    )
    .unwrap()
    {
        Some(chart_room_user) => chart_room_user,
        None => {
            // Only send our client that username is taken.
            let _ = sender
                .send(Message::Text(String::from("Username already taken.")))
                .await;
            return;
        }
    };

    // Subscribe before sending joined message.
    let mut broadcast_receiver = state.broadcaster.subscribe();

    // Send joined message to all subscribers.
    let msg = domain::models::ChatMessage::Join {
        username: username.clone(),
        room_name: DEFAULT_ROOM_NAME.to_string(),
    };
    tracing::debug!("{:?}", msg);
    let _ = state.publisher.send(msg);

    // This task will receive broadcast messages and send text message to our client.
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = broadcast_receiver.recv().await {
            // In any websocket error, break loop.
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // This task will receive messages from client and send them to broadcast subscribers.
    let name = username.clone();
    let publisher = state.publisher.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(context))) = receiver.next().await {
            let msg = domain::models::ChatMessage::Chat {
                username: name.clone(),
                room_name: DEFAULT_ROOM_NAME.to_string(),
                context,
            };
            let _ = publisher.send(msg);
        }
    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    // Send user left message.
    let msg = domain::models::ChatMessage::Leave {
        username: username.clone(),
        room_name: DEFAULT_ROOM_NAME.to_string(),
    };
    tracing::debug!("{:?}", msg);
    let _ = state.publisher.send(msg);
    drop(chat_room_user);
}
