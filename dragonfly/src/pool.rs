use crate::{RedisClient, RedisPool, Result};

pub fn new_pool<S: Into<String>>(redis_url: S, max_size: u32) -> Result<RedisPool> {
    let url_string = redis_url.into();
    let client = RedisClient::open(url_string.as_str())?;
    RedisPool::builder()
        .max_size(max_size)
        .build(client)
        .map_err(Into::into)
}
