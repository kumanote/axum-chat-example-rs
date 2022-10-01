use crate::{RedisConnection, Result};
use redis::Client;

pub fn establish_connection<S: Into<String>>(redis_url: S) -> Result<RedisConnection> {
    let url_string = redis_url.into();
    let client = Client::open(url_string.as_str())?;
    Ok(client.get_connection()?)
}
