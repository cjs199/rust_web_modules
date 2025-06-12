use framework_utils::{pro_snowflake_util, pro_time_util};
use log::error;
use redis::cmd;
use redis::{RedisError, ToRedisArgs};
use std::future::Future;
use std::ops::DerefMut;

use super::pro_redis_util;

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
        let lock = pro_redis_util::kv_set_if_absent(key.clone(), value.clone(), ex_timeout);
        if lock {
            return true;
        }
        pro_time_util::fiber_sleep(pro_time_util::Millisecond::_200.clone() as u64).await;
        if pro_time_util::get_current_milliseconds() - current_milliseconds > wait_timeout {
            return false;
        }
    }
}

// 释放分布式锁
pub async fn release_lock(key: impl Into<String>, value: impl Into<String>) -> bool {
    let mut conn = pro_redis_util::get_conn();
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

pub async fn lock_wraper<T, F: Future<Output = T>>(
    key: impl Into<String>,
    lock_fn: F,
) -> Option<T> {
    return lock_wraper_set_val(key, pro_snowflake_util::next_id_str(), lock_fn).await;
}

pub async fn lock_wraper_set_val<T, F: Future<Output = T>>(
    key: impl Into<String>,
    value: impl Into<String>,
    lock_fn: F,
) -> Option<T> {
    let key = key.into();
    let value = value.into();
    if acquire_lock(key.clone(), value.clone()).await {
        let t = lock_fn.await;
        release_lock(key, value).await;
        Some(t)
    } else {
        None
    }
}

pub async fn full_lock_wraper<T, F: Future<Output = T>>(
    key: impl Into<String>,
    value: impl Into<String>,
    wait_timeout: i64,
    ex_timeout: i64,
    lock_fn: F,
) -> Option<T> {
    let key = key.into();
    let value = value.into();
    if acquire_lock_wait_and_expire(key.clone(), value.clone(), wait_timeout, ex_timeout).await {
        let t = lock_fn.await;
        release_lock(key, value).await;
        Some(t)
    } else {
        None
    }
}
