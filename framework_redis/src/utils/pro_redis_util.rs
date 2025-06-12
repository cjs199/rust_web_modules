use framework_utils::{pro_collection_util, pro_snowflake_util};
use lazy_static::lazy_static;
use log::error;
use r2d2::{Pool, PooledConnection};
use redis::streams::{StreamInfoGroupsReply, StreamReadOptions, StreamReadReply};
use redis::{cmd, Commands, FromRedisValue, RedisError, ToRedisArgs, Value};
use std::collections::HashMap;
use std::env;
use std::future::Future;
use std::ops::DerefMut;
use std::time::Duration;

use super::pro_redis_lock_util;

// 使用 lazy_static! 宏创建一个全局静态变量 REDIS_POOL，它是一个连接池，用于管理 Redis 连接
lazy_static! {

    pub static ref REDIS_POOL_OPTION: Option<Pool<redis::Client>> = {
        let redis_url_result = env::var("redis_url");
        if let Ok( redis_url) = redis_url_result {
            let client = redis::Client::open(redis_url).unwrap();
            let pool = r2d2::Pool::builder()
            .min_idle(Some(2))
            .max_size(50)
            .max_lifetime(Some(Duration::from_millis(30000)))
            .idle_timeout(Some(Duration::from_millis(60000)))
            .connection_timeout(Duration::from_millis(5000))
            .build(client).unwrap();
            Some(pool)
        } else {
            error!("获取redis url,初始化redis失败");
            None
        }
    };

    // 使用 lazy_static! 宏创建一个全局静态变量 EXPIRE_TIME，它表示 Redis 中数据的默认过期时间（以毫秒为单位）
    static ref EXPIRE_TIME: i64 = env::var("redis_expire_time")
        .expect("获取redis url,初始化redis失败")
        .parse::<i64>()
        .unwrap();

}

pub fn get_conn() -> PooledConnection<redis::Client> {
    REDIS_POOL_OPTION.as_ref().unwrap().get().unwrap()
}

pub fn del<K: ToRedisArgs + Clone>(key: K) -> i32 {
    let mut conn = get_conn();
    conn.del(key).unwrap()
}

// 递增操作，使用默认过期时间
pub fn incr<K: ToRedisArgs + Clone, V: ToRedisArgs>(key: K, delta: V) -> Result<i64, RedisError> {
    return incr_expire(key, delta, EXPIRE_TIME.clone());
}

// 递增操作，使用指定的过期时间
pub fn incr_expire<K: ToRedisArgs + Clone, V: ToRedisArgs>(
    key: K,
    delta: V,
    millisecond: i64,
) -> Result<i64, RedisError> {
    // 执行 Redis 的 INCR 命令，对指定的键进行递增操作，如果执行失败会导致程序 panic，实际应用中应进行错误处理
    let mut conn = get_conn();
    let result: Result<i64, RedisError> = cmd("INCRBY")
        .arg(key.clone())
        .arg(delta)
        .query(conn.deref_mut());
    if let Ok(_) = result {
        let _ = expire(key, millisecond);
    }
    result
}

// 设置指定键的过期时间为给定的毫秒数
pub fn expire<K: ToRedisArgs>(key: K, millisecond: i64) -> Result<bool, RedisError> {
    let mut conn = get_conn();
    if millisecond < 0 {
        conn.persist(key)
    } else {
        let query = cmd("PEXPIRE")
            .arg(key)
            .arg(millisecond)
            .query(conn.deref_mut());
        query
    }
}

pub fn exists<K: ToRedisArgs + Clone>(key: K) -> bool {
    let mut conn = get_conn();
    cmd("EXISTS").arg(key).query(conn.deref_mut()).unwrap()
}

pub fn kv_get<K: ToRedisArgs + Clone, V: ToRedisArgs + Clone + FromRedisValue>(key: K) -> V {
    let mut conn = get_conn();
    let get = conn.get(key).unwrap();
    let value: V = FromRedisValue::from_redis_value(&get).unwrap();
    return value;
}

pub async fn kv_get_cached<K: ToRedisArgs + Clone, T: ToRedisArgs + Clone + FromRedisValue>(
    key: K,
    func: impl Future<Output = Option<T>>,
    timeout: i64,
) -> Option<T> {
    let mut kv_get: Option<T> = kv_get(key.clone());
    if let None = kv_get {
        kv_get = func.await;
        kv_set_expire(key, kv_get.clone(), timeout);
    }
    return kv_get;
}

pub fn kv_set<K: ToRedisArgs + Clone, V: ToRedisArgs + Clone>(key: K, value: V) -> String {
    return kv_set_expire(key, value, EXPIRE_TIME.clone());
}

pub fn kv_set_expire<K: ToRedisArgs + Clone, V: ToRedisArgs + Clone>(
    key: K,
    value: V,
    timeout: i64,
) -> String {
    let mut conn = get_conn();
    let result: String = cmd("PSETEX")
        .arg(key)
        .arg(timeout)
        .arg(value)
        .query(conn.deref_mut())
        .unwrap();
    result
}

// 该方法是一种常用操作，用于在键值存储中原子地设置值（仅当键尚不存在时）
pub fn kv_set_if_absent<K: ToRedisArgs, V: ToRedisArgs>(key: K, value: V, ex_timeout: i64) -> bool {
    let mut conn = get_conn();
    let result: Result<Value, RedisError> = cmd("SET")
        .arg(key)
        .arg(value)
        .arg("PX")
        .arg(ex_timeout)
        .arg("NX")
        .query(conn.deref_mut());
    match result {
        Ok(result) => {
            if let Value::Okay = result {
                // 加锁成功
                true
            } else {
                // 加锁失败
                false
            }
        }
        Err(err) => {
            error!("kv_set_if_absent 异常{}", err);
            false
        }
    }
}

// 将给定的键值对存入 Redis 哈希表中，使用默认过期时间
pub fn map_put<K: ToRedisArgs + Clone, F: ToRedisArgs + Clone, V: ToRedisArgs + Clone>(
    key: K,
    field: F,
    value: V,
) -> i32 {
    return map_put_expire(key, field, value, EXPIRE_TIME.clone());
}

// 将给定的键值对存入 Redis 哈希表中，并设置过期时间。
//
// # 参数
// - `key`：哈希表的键，实现了`ToRedisArgs`和`Clone` trait。
// - `field`：哈希表中的字段，实现了`ToRedisArgs`和`Clone` trait。
// - `value`：要存入哈希表对应字段的值，实现了`ToRedisArgs`和`Clone` trait。
// - `timeout`：以秒为单位的过期时间。
//
// # 返回值
// 返回整数，表示存入操作的结果状态码。通常，非零值表示成功，零表示失败。
//
// # 可能的错误
// 如果获取 Redis 连接失败或者执行`hset`命令失败，会返回相应的错误信息。
pub fn map_put_expire<K: ToRedisArgs + Clone, F: ToRedisArgs + Clone, V: ToRedisArgs + Clone>(
    key: K,
    field: F,
    value: V,
    timeout: i64,
) -> i32 {
    // 执行 Redis 的 HSET 命令，将给定的字段和值存入哈希表中，如果执行失败会导致程序 panic，实际应用中应进行错误处理
    let mut conn = get_conn();
    let ret: i32 = cmd("HSET")
        .arg(key.clone())
        .arg(field)
        .arg(value)
        .query(conn.deref_mut())
        .unwrap();
    // 设置指定键的过期时间为给定的毫秒数
    let _ = expire(key, timeout);
    return ret;
}

pub fn map_put_all<K: ToRedisArgs + Clone, F: ToRedisArgs + Clone, V: ToRedisArgs + Clone>(
    key: K,
    map: HashMap<F, V>,
    timeout: i64,
) -> String {
    // 执行 Redis 的 HSET 命令，将给定的字段和值存入哈希表中，如果执行失败会导致程序 panic，实际应用中应进行错误处理
    let mut conn = get_conn();
    let binding = map.iter().collect::<Vec<_>>();
    let items: &[(&F, &V)] = binding.as_slice();
    let hset_multiple: String = conn.hset_multiple(key.clone(), items).unwrap();
    // 设置指定键的过期时间为给定的毫秒数
    let _ = expire(key, timeout);
    return hset_multiple;
}

// 获取指定哈希表中的所有键值对，并返回一个哈希映射
pub fn map_getall<
    K: ToRedisArgs + Clone,
    MK: FromRedisValue + std::hash::Hash + std::cmp::Eq,
    MV: FromRedisValue,
>(
    key: K,
) -> HashMap<MK, MV> {
    // 执行 Redis 的 HGETALL 命令，获取指定哈希表中的所有键值对，如果执行失败会导致程序 panic，实际应用中应进行错误处理
    let mut conn = get_conn();
    let result: Vec<Value> = cmd("HGETALL").arg(key).query(conn.deref_mut()).unwrap();
    let mut ret: HashMap<MK, MV> = HashMap::new();
    for i in (0..result.len()).step_by(2) {
        let field: MK = FromRedisValue::from_redis_value(&result[i]).unwrap();
        let value: MV = FromRedisValue::from_redis_value(&result[i + 1]).unwrap();
        ret.insert(field, value);
    }
    return ret;
}

// 从 Redis 的哈希表中获取指定键和字段的对应值。
//
// # 参数说明
// - `key`：哈希表的键，需实现`ToRedisArgs`和`Clone` trait。
// - `field`：哈希表中的字段，需实现`ToRedisArgs`和`Clone` trait。
//
// # 返回值
// 返回一个`RedisResult`，其中包含一个`Option<V>`，表示获取的值可能存在也可能不存在。
// `V`是要获取的值的类型，需实现`redis::FromRedisValue` trait，以便从 Redis 的返回值中解析出该类型的值。
//
// # 可能的错误
// 如果无法从连接池中获取连接，则返回一个包含`IoError`的`RedisError`。
pub fn map_get<K: ToRedisArgs + Clone + std::fmt::Display, F: ToRedisArgs + Clone, V>(
    key: K,
    field: F,
) -> Option<V>
where
    V: redis::FromRedisValue,
{
    // 执行 Redis 的 HGET 命令，获取指定哈希表中指定字段的值，如果执行失败会导致程序 panic，实际应用中应进行错误处理
    let mut conn = get_conn();
    let v_result: Result<V, RedisError> = cmd("HGET").arg(key).arg(field).query(conn.deref_mut());
    if let Ok(v) = v_result {
        Some(v)
    } else {
        None
    }
}

pub fn map_del<K: ToRedisArgs + Clone, F: ToRedisArgs + Clone>(key: K, field: F) -> i32 {
    // 执行 Redis 的 HSET 命令，将给定的字段和值存入哈希表中，如果执行失败会导致程序 panic，实际应用中应进行错误处理
    let mut conn = get_conn();
    let ret: i32 = cmd("HDEL")
        .arg(key)
        .arg(field)
        .query(conn.deref_mut())
        .unwrap();
    return ret;
}

// 向 Redis 的 Stream 类型数据结构中添加数据
pub fn streams_xadd<K: ToRedisArgs + Clone, V: ToRedisArgs + Clone>(
    stream_name: K,
    items: V,
) -> String {
    // 执行 Redis 的 XADD 命令，向指定的 Stream 中添加数据，如果执行失败会导致程序 panic，实际应用中应进行错误处理
    let mut conn = get_conn();
    let ret: String = cmd("XADD")
        .arg(stream_name)
        .arg("*")
        .arg(&[("payload", items)])
        .query(conn.deref_mut())
        .unwrap();
    ret
}

// 向 Redis 的 Stream 类型数据结构中添加数据
pub fn streams_xadd_vec<K: ToRedisArgs + Clone, V: ToRedisArgs + Clone>(
    stream_name: K,
    items: Vec<V>,
) -> Vec<String> {
    let mut conn = get_conn();
    let script = r#"
        local results = {}
        for i, element in ipairs(ARGV) do
            local id = redis.call('xadd', KEYS[1], '*', 'payload',element )
            table.insert(results, id)
        end
        return results
    "#;
    // 执行 Redis 的 EVAL 命令，使用 Lua 脚本判断并释放锁，如果执行失败会导致程序 panic，实际应用中应进行错误处理
    let result: Vec<String> = cmd("eval")
        .arg(script)
        .arg(1)
        .arg(&[stream_name])
        .arg(items.to_vec())
        .query(conn.deref_mut())
        .unwrap();
    result
}

// 不指定组，进行广播式的读取 Stream 数据
pub fn streams_xread<K: ToRedisArgs + Clone + Send + Sync>(
    stream_name: K,
) -> Option<StreamReadReply> {
    // 创建一个 Stream 读取选项，设置读取数量为 1，阻塞1分钟等待数据，采用默认的读取模式（从尾部开始读取新数据）
    let options = StreamReadOptions::default().count(50).block(2000);
    // 执行 Redis 的 XREAD 命令，读取指定的 Stream 数据，如果执行失败会导致程序 panic，实际应用中应进行错误处理
    let mut conn = get_conn();
    let ret: Option<StreamReadReply> = conn
        .xread_options(&[stream_name], &["$"], &options)
        .unwrap();
    return ret;
}

// 读取指定 key 的 Stream 数据，按照分组和消费者名称进行读取
pub fn streams_xread_group<K: ToRedisArgs>(
    stream_name: K,
    group_name: impl Into<String>,
    consumer_name: impl Into<String>,
) -> Option<StreamReadReply> {
    let mut conn = get_conn();
    // 创建一个 Stream 读取选项，设置读取数量为 1，阻塞1分钟等待数据，指定组名和消费者名
    let group_name = group_name.into();
    let options = StreamReadOptions::default()
        .count(50)
        .block(2000)
        .group(group_name.clone(), consumer_name.into());
    // 执行 Redis 的 XREADGROUP 命令，读取指定组和消费者的 Stream 数据，如果执行失败会导致程序 panic，实际应用中应进行错误处理
    let xread_options = conn.xread_options(&[stream_name], &[">"], &options);
    if let Ok(ret) = xread_options {
        return ret;
    } else {
        None
    }
}

pub fn streams_xinfo_groups<K: ToRedisArgs>(stream_name: K) -> StreamInfoGroupsReply {
    let mut conn = get_conn();
    conn.xinfo_groups(stream_name).unwrap()
}

pub fn streams_xack<K: ToRedisArgs>(
    stream_name: K,
    group_name: impl Into<String>,
    id: impl Into<String>,
) -> Result<usize, RedisError> {
    let mut conn = get_conn();
    conn.xack(&[stream_name], group_name.into(), &[id.into()])
}

pub fn streams_xdel<K: ToRedisArgs>(
    stream_name: K,
    id: impl Into<String>,
) -> Result<usize, RedisError> {
    let mut conn = get_conn();
    conn.xdel(&[stream_name], &[id.into()])
}

pub async fn streams_xgroup_create_mkstream<K: ToRedisArgs + Clone>(
    stream_name: K,
    group: impl Into<String>,
) -> bool {
    let group_name = group.into();
    let next_id_str = pro_snowflake_util::next_id_str();
    let streams_xgroup_create_mkstream = "streams_xgroup_create_mkstream";
    let acquire_lock =
        pro_redis_lock_util::acquire_lock(streams_xgroup_create_mkstream, next_id_str.clone())
            .await;
    let mut xgroup_create = false;
    if acquire_lock {
        // 锁定方法,只能序列化执行
        let mut conn = get_conn();
        // 判断队列是否存在
        let exists: bool = exists(stream_name.clone());

        if !exists {
            // 执行 Redis 的 XGROUP CREATE 命令，创建一个消费者组，如果执行失败会导致程序 panic，实际应用中应进行错误处理
            xgroup_create = conn
                .xgroup_create_mkstream(stream_name.clone(), group_name.clone(), "$")
                .unwrap();
        } else {
            // 执行 Redis 的 XGROUP CREATE 命令，创建一个消费者组，如果执行失败会导致程序 panic，实际应用中应进行错误处理
            let groups: StreamInfoGroupsReply = streams_xinfo_groups(stream_name.clone());
            let groups = groups.groups;
            let collect_field_values =
                pro_collection_util::collect_field_values(&groups, |group| group.name.clone());
            if !collect_field_values.contains(&group_name.to_string()) {
                xgroup_create = conn
                    .xgroup_create_mkstream(stream_name, group_name, "$")
                    .unwrap();
            }
        }
    }
    pro_redis_lock_util::release_lock(streams_xgroup_create_mkstream, next_id_str).await;
    return xgroup_create;
}

pub fn pubsub_send<K: ToRedisArgs + Clone + Send + Sync, E: ToRedisArgs + Clone + Send + Sync>(
    channel: K,
    msg: E,
) -> i64 {
    let mut conn = get_conn();
    conn.publish(channel, msg).unwrap()
}
