use crate::{models, Result};
use dragonfly::{RedisConnection, RedisPool};
use std::sync::mpsc;
use tokio::sync::broadcast;

pub fn is_username_member(
    redis_connection: &mut RedisConnection,
    room_name: &str,
    username: &str,
) -> Result<bool> {
    dragonfly::adapters::sismember(redis_connection, room_name, username).map_err(Into::into)
}

pub fn add_username_to_room(
    redis_connection: &mut RedisConnection,
    room_name: &str,
    username: &str,
) -> Result<()> {
    dragonfly::adapters::sadd(redis_connection, room_name, username).map_err(Into::into)
}

pub fn remove_username_from_room(
    redis_connection: &mut RedisConnection,
    room_name: &str,
    username: &str,
) -> Result<()> {
    dragonfly::adapters::srem(redis_connection, room_name, username).map_err(Into::into)
}

pub struct ChatRoomPublisherService {
    redis_pool: RedisPool,
    server_id: models::ServerId,
    channel_name: String,
    broadcaster: broadcast::Sender<String>,
    receiver: mpsc::Receiver<models::ChatMessage>,
}

impl ChatRoomPublisherService {
    pub fn new<S: Into<models::ServerId>>(
        redis_pool: RedisPool,
        server_id: S,
        channel_name: String,
        broadcaster: broadcast::Sender<String>,
        receiver: mpsc::Receiver<models::ChatMessage>,
    ) -> Self {
        Self {
            redis_pool,
            server_id: server_id.into(),
            channel_name,
            broadcaster,
            receiver,
        }
    }
    pub fn start(self) {
        let mut redis_connection = self.redis_pool.get().unwrap();
        for message in self.receiver {
            let _ = self.broadcaster.send(message.message_context());
            dragonfly::adapters::publish(
                &mut redis_connection,
                &self.channel_name,
                models::IdLabeledMessage {
                    id: self.server_id.clone(),
                    msg: message,
                },
            )
            .unwrap();
        }
    }
}

pub struct ChatRoomSubscriberService {
    redis_pool: RedisPool,
    server_id: models::ServerId,
    channel_name: String,
    broadcaster: broadcast::Sender<String>,
}

impl ChatRoomSubscriberService {
    pub fn new<S: Into<models::ServerId>>(
        redis_pool: RedisPool,
        server_id: S,
        channel_name: String,
        broadcaster: broadcast::Sender<String>,
    ) -> Self {
        Self {
            redis_pool,
            server_id: server_id.into(),
            channel_name,
            broadcaster,
        }
    }

    pub fn start(self) {
        let mut redis_connection = self.redis_pool.get().unwrap();
        let mut pub_sub =
            dragonfly::adapters::subscribe(&mut redis_connection, &self.channel_name).unwrap();
        while let Ok(msg) = pub_sub.get_message() {
            if let Ok(message) = msg.get_payload::<models::IdLabeledMessage>() {
                if message.id != self.server_id {
                    let _ = self.broadcaster.send(message.msg.message_context());
                }
            }
        }
    }
}
