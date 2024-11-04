use framework_utils::{pro_collection_util, pro_time_util};
use idgenerator::IdInstance;
use lazy_static::lazy_static;
use log::{error, info};
use r2d2_redis::r2d2::Pool;
use r2d2_redis::redis::streams::{StreamInfoGroupsReply, StreamReadOptions, StreamReadReply};
use r2d2_redis::redis::{self, cmd, Commands, FromRedisValue, RedisError, ToRedisArgs, Value};
use r2d2_redis::{r2d2, RedisConnectionManager};
use std::collections::HashMap;
use std::env;
use std::future::Future;
use std::ops::DerefMut;

// 使用 lazy_static! 宏创建一个全局静态变量 REDIS_POOL，它是一个连接池，用于管理 Redis 连接
lazy_static! {
    pub static ref REDIS_POOL: Pool<RedisConnectionManager> = {
        info!("开始初始化redis连接");
        // 构建一个连接池，连接到从环境变量中获取的 Redis URL，如果获取失败会导致程序 panic，实际应用中应进行错误处理
        let ret = r2d2::Pool::builder()
           .build(
                RedisConnectionManager::new(
                    env::var("redis_url").expect("获取redis url,初始化redis失败"),
                )
               .unwrap(),
            )
           .unwrap();
        info!("初始化redis连接成功");
        ret
    };
}

// 使用 lazy_static! 宏创建一个全局静态变量 EXPIRE_TIME，它表示 Redis 中数据的默认过期时间（以毫秒为单位）
lazy_static! {
    static ref EXPIRE_TIME: i64 = env::var("redis_expire_time")
        .expect("获取redis url,初始化redis失败")
        .parse::<i64>()
        .unwrap();
}

// 递增操作，使用默认过期时间
pub async fn incr<K: ToRedisArgs + Clone, V: ToRedisArgs>(
    key: K,
    delta: V,
) -> Result<i64, RedisError> {
    return incr_expire(key, delta, EXPIRE_TIME.clone()).await;
}

// 递增操作，使用指定的过期时间
pub async fn incr_expire<K: ToRedisArgs + Clone, V: ToRedisArgs>(
    key: K,
    delta: V,
    millisecond: i64,
) -> Result<i64, RedisError> {
    // 执行 Redis 的 INCR 命令，对指定的键进行递增操作，如果执行失败会导致程序 panic，实际应用中应进行错误处理
    let mut conn = REDIS_POOL.get().unwrap();
    let result: Result<i64, RedisError> = cmd("INCRBY")
        .arg(key.clone())
        .arg(delta)
        .query(conn.deref_mut());
    if let Ok(_) = result {
        let _ = expire(key, millisecond).await;
    }
    result
}

// 设置指定键的过期时间为给定的毫秒数
pub async fn expire<K: ToRedisArgs>(key: K, millisecond: i64) -> Result<bool, RedisError> {
    let mut conn = REDIS_POOL.get().unwrap();
    let query = cmd("PEXPIRE")
        .arg(key)
        .arg(millisecond)
        .query(conn.deref_mut());
    query
}

pub async fn exists<K: ToRedisArgs + Clone>(key: K) -> bool {
    let mut conn = REDIS_POOL.get().unwrap();
    cmd("EXISTS").arg(key).query(conn.deref_mut()).unwrap()
}

pub async fn kv_set<K: ToRedisArgs + Clone, V: ToRedisArgs + Clone>(key: K, value: V) -> String {
    return kv_set_expire(key, value, EXPIRE_TIME.clone()).await;
}

pub async fn kv_set_expire<K: ToRedisArgs + Clone, V: ToRedisArgs + Clone>(
    key: K,
    value: V,
    timeout: i64,
) -> String {
    let mut conn = REDIS_POOL.get().unwrap();
    let result: String = cmd("PSETEX")
        .arg(key)
        .arg(timeout)
        .arg(value)
        .query(conn.deref_mut())
        .unwrap();
    result
}

// 该方法是一种常用操作，用于在键值存储中原子地设置值（仅当键尚不存在时）
pub async fn kv_set_if_absent<K: ToRedisArgs, V: ToRedisArgs>(
    key: K,
    value: V,
    ex_timeout: i64,
) -> bool {
    let mut conn = REDIS_POOL.get().unwrap();
    let result: Result<Value, RedisError> = cmd("SET")
        .arg(key)
        .arg(value)
        .arg("PX")
        .arg(ex_timeout)
        .arg("NX")
        .query(conn.deref_mut());
    match result {
        Ok(result) => {
            if let r2d2_redis::redis::Value::Okay = result {
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
pub async fn map_put<K: ToRedisArgs + Clone, F: ToRedisArgs + Clone, V: ToRedisArgs + Clone>(
    key: K,
    field: F,
    value: V,
) -> i32 {
    return map_put_expire(key, field, value, EXPIRE_TIME.clone()).await;
}

/// 将给定的键值对存入 Redis 哈希表中，并设置过期时间。
///
/// # 参数
/// - `key`：哈希表的键，实现了`ToRedisArgs`和`Clone` trait。
/// - `field`：哈希表中的字段，实现了`ToRedisArgs`和`Clone` trait。
/// - `value`：要存入哈希表对应字段的值，实现了`ToRedisArgs`和`Clone` trait。
/// - `timeout`：以秒为单位的过期时间。
///
/// # 返回值
/// 返回整数，表示存入操作的结果状态码。通常，非零值表示成功，零表示失败。
///
/// # 可能的错误
/// 如果获取 Redis 连接失败或者执行`hset`命令失败，会返回相应的错误信息。
pub async fn map_put_expire<
    K: ToRedisArgs + Clone,
    F: ToRedisArgs + Clone,
    V: ToRedisArgs + Clone,
>(
    key: K,
    field: F,
    value: V,
    timeout: i64,
) -> i32 {
    // 执行 Redis 的 HSET 命令，将给定的字段和值存入哈希表中，如果执行失败会导致程序 panic，实际应用中应进行错误处理
    let mut conn = REDIS_POOL.get().unwrap();
    let ret: i32 = cmd("HSET")
        .arg(key.clone())
        .arg(field)
        .arg(value)
        .query(conn.deref_mut())
        .unwrap();
    // 设置指定键的过期时间为给定的毫秒数
    let _ = expire(key, timeout).await;
    return ret;
}

// 获取指定哈希表中的所有键值对，并返回一个哈希映射
pub async fn map_getall<
    K: ToRedisArgs + Clone,
    MK: FromRedisValue + std::hash::Hash + std::cmp::Eq,
    MV: FromRedisValue,
>(
    key: K,
) -> HashMap<MK, MV> {
    // 执行 Redis 的 HGETALL 命令，获取指定哈希表中的所有键值对，如果执行失败会导致程序 panic，实际应用中应进行错误处理
    let mut conn = REDIS_POOL.get().unwrap();
    let result: Vec<Value> = cmd("HGETALL").arg(key).query(conn.deref_mut()).unwrap();
    let mut ret: HashMap<MK, MV> = HashMap::new();
    for i in (0..result.len()).step_by(2) {
        let field: MK = FromRedisValue::from_redis_value(&result[i]).unwrap();
        let value: MV = FromRedisValue::from_redis_value(&result[i + 1]).unwrap();
        ret.insert(field, value);
    }
    return ret;
}

/// 从 Redis 的哈希表中获取指定键和字段的对应值。
///
/// # 参数说明
/// - `key`：哈希表的键，需实现`ToRedisArgs`和`Clone` trait。
/// - `field`：哈希表中的字段，需实现`ToRedisArgs`和`Clone` trait。
///
/// # 返回值
/// 返回一个`RedisResult`，其中包含一个`Option<V>`，表示获取的值可能存在也可能不存在。
/// `V`是要获取的值的类型，需实现`redis::FromRedisValue` trait，以便从 Redis 的返回值中解析出该类型的值。
///
/// # 可能的错误
/// 如果无法从连接池中获取连接，则返回一个包含`IoError`的`RedisError`。
pub async fn map_get<K: ToRedisArgs + Clone + std::fmt::Display, F: ToRedisArgs + Clone, V>(
    key: K,
    field: F,
) -> Option<V>
where
    V: redis::FromRedisValue,
{
    // 执行 Redis 的 HGET 命令，获取指定哈希表中指定字段的值，如果执行失败会导致程序 panic，实际应用中应进行错误处理
    let mut conn = REDIS_POOL.get().unwrap();
    let v_result: Result<V, RedisError> = cmd("HGET").arg(key).arg(field).query(conn.deref_mut());
    if let Ok(v) = v_result {
        Some(v)
    } else {
        None
    }
}

pub async fn map_del<K: ToRedisArgs + Clone, F: ToRedisArgs + Clone>(key: K, field: F) -> i32 {
    // 执行 Redis 的 HSET 命令，将给定的字段和值存入哈希表中，如果执行失败会导致程序 panic，实际应用中应进行错误处理
    let mut conn = REDIS_POOL.get().unwrap();
    let ret: i32 = cmd("HDEL")
        .arg(key)
        .arg(field)
        .query(conn.deref_mut())
        .unwrap();
    return ret;
}

/// 向 Redis 的 Stream 类型数据结构中添加数据
pub async fn streams_xadd<K: ToRedisArgs + Clone, V: ToRedisArgs + Clone>(
    stream_name: K,
    items: V,
) -> String {
    // 执行 Redis 的 XADD 命令，向指定的 Stream 中添加数据，如果执行失败会导致程序 panic，实际应用中应进行错误处理
    let mut conn = REDIS_POOL.get().unwrap();
    let ret: String = cmd("XADD")
        .arg(stream_name)
        .arg("*")
        .arg(&[("payload", items)])
        .query(conn.deref_mut())
        .unwrap();
    ret
}

/// 向 Redis 的 Stream 类型数据结构中添加数据
pub async fn streams_xadd_vec<K: ToRedisArgs + Clone, V: ToRedisArgs + Clone>(
    stream_name: K,
    items: Vec<V>,
) -> Vec<String> {
    let mut conn = REDIS_POOL.get().unwrap();
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
pub async fn streams_xread<K: ToRedisArgs + Clone + Send + Sync>(
    stream_name: K,
) -> Option<StreamReadReply> {
    // 创建一个 Stream 读取选项，设置读取数量为 1，阻塞1分钟等待数据，采用默认的读取模式（从尾部开始读取新数据）
    let options = StreamReadOptions::default().count(1).block(60000);
    // 执行 Redis 的 XREAD 命令，读取指定的 Stream 数据，如果执行失败会导致程序 panic，实际应用中应进行错误处理
    let mut conn = REDIS_POOL.get().unwrap();
    let ret: Option<StreamReadReply> = conn.xread_options(&[stream_name], &["$"], options).unwrap();
    return ret;
}

/// 读取指定 key 的 Stream 数据，按照分组和消费者名称进行读取
pub async fn streams_xread_group<K: ToRedisArgs>(
    stream_name: K,
    group_name: impl Into<String>,
    consumer_name: impl Into<String>,
) -> Option<StreamReadReply> {
    let mut conn = REDIS_POOL.get().unwrap();
    // 创建一个 Stream 读取选项，设置读取数量为 1，阻塞1分钟等待数据，指定组名和消费者名
    let group_name = group_name.into();
    let options = StreamReadOptions::default()
        .count(1)
        .block(60000)
        .group(group_name.clone(), consumer_name.into());
    // 执行 Redis 的 XREADGROUP 命令，读取指定组和消费者的 Stream 数据，如果执行失败会导致程序 panic，实际应用中应进行错误处理
    let ret: Option<StreamReadReply> = conn.xread_options(&[stream_name], &[">"], options).unwrap();
    return ret;
}

pub async fn streams_xinfo_groups<K: ToRedisArgs>(stream_name: K) -> StreamInfoGroupsReply {
    let mut conn = REDIS_POOL.get().unwrap();
    conn.xinfo_groups(stream_name).unwrap()
}

pub async fn streams_xack<K: ToRedisArgs>(
    stream_name: K,
    group_name: impl Into<String>,
    id: impl Into<String>,
) -> Result<usize, RedisError> {
    let mut conn = REDIS_POOL.get().unwrap();
    conn.xack(&[stream_name], group_name.into(), &[id.into()])
}

pub async fn streams_xdel<K: ToRedisArgs>(
    stream_name: K,
    id: impl Into<String>,
) -> Result<usize, RedisError> {
    let mut conn = REDIS_POOL.get().unwrap();
    conn.xdel(&[stream_name], &[id.into()])
}

pub async fn streams_xgroup_create_mkstream<K: ToRedisArgs + Clone>(
    stream_name: K,
    group: impl Into<String>,
) -> bool {
    let group_name = group.into();
    let next_id_str = IdInstance::next_id().to_string();
    let streams_xgroup_create_mkstream = "streams_xgroup_create_mkstream";
    let acquire_lock = acquire_lock(streams_xgroup_create_mkstream, next_id_str.clone()).await;
    let mut xgroup_create = false;
    if acquire_lock {
        // 锁定方法,只能序列化执行
        let mut conn = REDIS_POOL.get().unwrap();
        // 判断队列是否存在
        let exists: bool = exists(stream_name.clone()).await;

        if !exists {
            // 执行 Redis 的 XGROUP CREATE 命令，创建一个消费者组，如果执行失败会导致程序 panic，实际应用中应进行错误处理
            xgroup_create = conn
                .xgroup_create_mkstream(stream_name.clone(), group_name.clone(), "$")
                .unwrap();
        } else {
            // 执行 Redis 的 XGROUP CREATE 命令，创建一个消费者组，如果执行失败会导致程序 panic，实际应用中应进行错误处理
            let groups: StreamInfoGroupsReply = streams_xinfo_groups(stream_name.clone()).await;
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
    release_lock(streams_xgroup_create_mkstream, next_id_str).await;
    return xgroup_create;
}

// 尝试获取分布式锁，使用默认过期时间
pub async fn acquire_lock(key: impl Into<String>, value: impl Into<String>) -> bool {
    return acquire_lock_wait_and_expire(
        key.into(),
        value.into(),
        pro_time_util::Millisecond::_1_MINUTE.clone(),
        pro_time_util::Millisecond::_3_MINUTE.clone(),
    )
    .await;
}

// 尝试获取分布式锁，使用指定的过期时间
pub async fn acquire_lock_wait_and_expire<K: ToRedisArgs + Clone, V: ToRedisArgs + Clone>(
    key: K,
    value: V,
    wait_timeout: i64,
    ex_timeout: i64,
) -> bool {
    let current_milliseconds = pro_time_util::get_current_milliseconds();
    // 执行 Redis 的 SETEX 命令，尝试设置一个带有过期时间的键值对，如果执行失败会导致程序 panic，实际应用中应进行错误处理
    loop {
        let lock = kv_set_if_absent(key.clone(), value.clone(), ex_timeout).await;
        if lock {
            return true;
        }
        pro_time_util::sleep(pro_time_util::Millisecond::_200.clone() as u64);
        if pro_time_util::get_current_milliseconds() - current_milliseconds > wait_timeout {
            return false;
        }
    }
}

// 释放分布式锁
pub async fn release_lock(key: impl Into<String>, value: impl Into<String>) -> bool {
    let mut conn = REDIS_POOL.get().unwrap();
    let script = r#"
        if redis.call('get', KEYS[1]) == ARGV[1] then
            return redis.call('del', KEYS[1])
        else
            return 0
        end
    "#;
    let key: String = key.into();
    // 执行 Redis 的 EVAL 命令，使用 Lua 脚本判断并释放锁，如果执行失败会导致程序 panic，实际应用中应进行错误处理
    let eval_result: Result<i32, RedisError> = cmd("eval")
        .arg(script)
        .arg(1)
        .arg(&[key.clone()])
        .arg(&[value.into()])
        .query(conn.deref_mut());
    match eval_result {
        Ok(result) => {
            return result == 1;
        }

        Err(err) => {
            error!("{}解锁发生异常:{}", key, err);
            return false;
        }
    }
}

pub async fn lock_wraper(key: impl Into<String>, value: impl Into<String>, lock_fn: impl Future) {
    let key = key.into();
    let value = value.into();
    if acquire_lock(key.clone(), value.clone()).await {
        lock_fn.await;
        release_lock(key, value).await;
    }
}

pub async fn pubsub_send<
    K: ToRedisArgs + Clone + Send + Sync,
    E: ToRedisArgs + Clone + Send + Sync,
>(
    channel: K,
    msg: E,
) -> i64 {
    let mut conn = REDIS_POOL.get().unwrap();
    conn.publish(channel, msg).unwrap()
}
