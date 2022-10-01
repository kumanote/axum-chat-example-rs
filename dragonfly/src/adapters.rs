use crate::{RedisConnection, Result};
use redis::{Commands, ConnectionLike, FromRedisValue, PubSub, ToRedisArgs};

pub fn health_check(conn: &mut RedisConnection) -> bool {
    conn.check_connection()
}

pub fn publish<K: ToRedisArgs, V: ToRedisArgs>(
    conn: &mut RedisConnection,
    channel: K,
    value: V,
) -> Result<()> {
    conn.publish(channel, value).map_err(Into::into)
}

pub fn subscribe<K: ToRedisArgs>(conn: &mut RedisConnection, channel: K) -> Result<PubSub> {
    let mut pubsub = conn.as_pubsub();
    pubsub.subscribe(channel)?;
    Ok(pubsub)
}

pub fn getset<K: ToRedisArgs, V: ToRedisArgs + FromRedisValue>(
    conn: &mut RedisConnection,
    key: K,
    value: V,
) -> Result<Option<V>> {
    conn.getset(key, value).map_err(Into::into)
}

pub fn get<K: ToRedisArgs, T: FromRedisValue>(conn: &mut RedisConnection, key: K) -> Result<T> {
    conn.get(key).map_err(Into::into)
}

pub fn set<K: ToRedisArgs, V: ToRedisArgs>(
    conn: &mut RedisConnection,
    key: K,
    value: V,
) -> Result<()> {
    conn.set(key, value).map_err(Into::into)
}

pub fn sadd<K: ToRedisArgs, V: ToRedisArgs>(
    conn: &mut RedisConnection,
    key: K,
    value: V,
) -> Result<()> {
    conn.sadd(key, value).map_err(Into::into)
}

pub fn srem<K: ToRedisArgs, V: ToRedisArgs>(
    conn: &mut RedisConnection,
    key: K,
    value: V,
) -> Result<()> {
    conn.srem(key, value).map_err(Into::into)
}

pub fn sismember<K: ToRedisArgs, V: ToRedisArgs>(
    conn: &mut RedisConnection,
    key: K,
    value: V,
) -> Result<bool> {
    conn.sismember(key, value).map_err(Into::into)
}

pub fn smembers<K: ToRedisArgs, V: FromRedisValue>(
    conn: &mut RedisConnection,
    key: K,
) -> Result<Vec<V>> {
    conn.smembers(key).map_err(Into::into)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;

    #[test]
    #[serial_test::serial]
    fn test_health_check() {
        dotenv::dotenv().ok();
        let redis_url = std::env::var("REDIS_URL").unwrap_or("redis://localhost:6379/0".to_owned());
        let mut connection = establish_connection(redis_url).unwrap();
        assert!(health_check(&mut connection));
    }

    #[test]
    #[serial_test::serial]
    fn test_publish_subscribe() {
        dotenv::dotenv().ok();
        let redis_url = std::env::var("REDIS_URL").unwrap_or("redis://localhost:6379/0".to_owned());
        let pool = new_pool(redis_url, 2).unwrap();
        let channel = "channel1";
        // subscribe
        let pool1 = pool.clone();
        let handle1 = std::thread::spawn(move || {
            let mut connection = pool1.get().unwrap();
            let mut sub = subscribe(&mut connection, channel).unwrap();
            loop {
                let msg = sub.get_message().unwrap();
                println!("got message: {:?}", msg);
                let payload: String = msg.get_payload().unwrap();
                if &payload == "fin" {
                    break;
                }
            }
        });
        let pool2 = pool.clone();
        let handle2 = std::thread::spawn(move || {
            let mut connection = pool2.get().unwrap();
            publish(&mut connection, channel, "This is the first message.").unwrap();
            publish(&mut connection, channel, "2nd message.").unwrap();
            publish(&mut connection, channel, "3rd message.").unwrap();
            publish(&mut connection, channel, "fin").unwrap();
        });
        let _ = handle2.join().unwrap();
        let _ = handle1.join().unwrap();
    }
}
