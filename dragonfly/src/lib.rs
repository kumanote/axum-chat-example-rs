extern crate redis;

mod conn;
mod error;
mod pool;

pub use conn::*;
pub use error::*;
pub use pool::*;

pub mod adapters;

pub use redis::{FromRedisValue, RedisResult, RedisWrite, ToRedisArgs};
pub type RedisR2D2Error = r2d2::Error;
pub type RedisValue = redis::Value;
pub type RedisErrorKind = redis::ErrorKind;
pub type RedisConnection = redis::Connection;
pub type RedisClient = redis::Client;
pub type RedisPool = r2d2::Pool<RedisClient>;
pub type Result<T> = core::result::Result<T, Error>;
