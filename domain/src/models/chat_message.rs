use super::server_id::ServerId;
use dragonfly::{FromRedisValue, RedisErrorKind, RedisResult, RedisValue, RedisWrite, ToRedisArgs};
use serde::{Deserialize, Serialize};
use std::str::from_utf8;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChatMessage {
    Join {
        username: String,
        room_name: String,
    },
    Leave {
        username: String,
        room_name: String,
    },
    Chat {
        username: String,
        room_name: String,
        context: String,
    },
}

impl ChatMessage {
    pub fn message_context(&self) -> String {
        match self {
            Self::Join {
                username,
                room_name: _,
            } => format!("{} joined.", username),
            Self::Leave {
                username,
                room_name: _,
            } => format!("{} left.", username),
            Self::Chat {
                username,
                room_name: _,
                context,
            } => format!("{}: {}", username, context),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdLabeledMessage {
    pub id: ServerId,
    pub msg: ChatMessage,
}

impl FromRedisValue for IdLabeledMessage {
    fn from_redis_value(v: &RedisValue) -> RedisResult<Self> {
        match *v {
            RedisValue::Data(ref bytes) => {
                let json_string = from_utf8(bytes)?;
                match serde_json::from_str::<IdLabeledMessage>(json_string) {
                    Ok(result) => Ok(result),
                    Err(_) => Err((
                        RedisErrorKind::TypeError,
                        "illegal json value for IdLabeledMessage",
                    )
                        .into()),
                }
            }
            _ => Err((
                RedisErrorKind::TypeError,
                "Response type not json string compatible.",
            )
                .into()),
        }
    }
}

impl ToRedisArgs for IdLabeledMessage {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + RedisWrite,
    {
        let json_string = serde_json::to_string(self).unwrap();
        out.write_arg(json_string.as_bytes())
    }
}
