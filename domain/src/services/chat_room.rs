use crate::{models, Result};
use dragonfly::{RedisConnection, RedisPool};
use std::sync::mpsc;
use tokio::sync::broadcast;

fn is_username_member(
    redis_connection: &mut RedisConnection,
    room_name: &str,
    username: &str,
) -> Result<bool> {
    dragonfly::adapters::sismember(redis_connection, room_name, username).map_err(Into::into)
}

fn add_username_to_room(
    redis_connection: &mut RedisConnection,
    room_name: &str,
    username: &str,
) -> Result<()> {
    dragonfly::adapters::sadd(redis_connection, room_name, username).map_err(Into::into)
}

fn remove_username_from_room(
    redis_connection: &mut RedisConnection,
    room_name: &str,
    username: &str,
) -> Result<()> {
    dragonfly::adapters::srem(redis_connection, room_name, username).map_err(Into::into)
}

pub struct ChatRoomUser {
    redis_pool: RedisPool,
    room_name: String,
    username: String,
}

impl ChatRoomUser {
    pub fn try_new(redis_pool: RedisPool, room_name: &str, username: &str) -> Result<Option<Self>> {
        let mut redis_connection = redis_pool.get()?;
        if is_username_member(&mut redis_connection, room_name, username)? {
            Ok(None)
        } else {
            add_username_to_room(&mut redis_connection, room_name, username)?;
            Ok(Some(Self {
                redis_pool,
                room_name: room_name.to_owned(),
                username: username.to_owned(),
            }))
        }
    }

    pub fn room_name(&self) -> &str {
        &self.room_name
    }

    pub fn username(&self) -> &str {
        &self.username
    }
}

impl Drop for ChatRoomUser {
    fn drop(&mut self) {
        let mut redis_connection = self.redis_pool.get().unwrap();
        remove_username_from_room(&mut redis_connection, &self.room_name, &self.username).unwrap();
    }
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
