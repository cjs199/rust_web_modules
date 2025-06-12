use framework_macro::{job, redis_lock_job};
use framework_redis::utils::pro_redis_lock_util;
use framework_redis::utils::pro_redis_util;
use framework_utils::pro_json_util;
use framework_utils::pro_snowflake_util;
use framework_utils::pro_thread_util;
use framework_utils::{pro_sqlite_util, pro_time_util};
use lazy_static::lazy_static;
use log::info;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::{collections::HashMap, sync::Mutex};
use tokio::time::{interval_at, Duration, Instant};

// 硬盘缓存和内存缓存默认过期时间 ms
const TIMEOUT: i64 = 259200000;

const CACHE: &str = "cache";

const DISK: &str = "disk";

lazy_static! {
    pub static ref MAP_CACHE: Mutex<HashMap<String, Vec<u8>>> = Mutex::new(HashMap::new());
    pub static ref INSERT_VEC: Mutex<Vec<(i64, String, String)>> = Mutex::new(Vec::new());
}

// 本地缓存清理定时任务
pub struct LocalCacheJob {}

#[job]
impl LocalCacheJob {
    #[redis_lock_job(job_name = "CacheJob_delete", interval_millis = 1000)]
    pub async fn delete() {
        let current_timeout = pro_time_util::get_current_milliseconds();
        let select_all = pro_sqlite_util::select_all("SELECT `TIMEOUT`, `EXPIRE_KEY`, `STORAGE_TYPE` from `MAP_DB_EXPIRE` WHERE `TIMEOUT` < ? ", (current_timeout,),|row|{
            (
                row.get::<usize, i64>(0).unwrap(),
                row.get::<usize, String>(1).unwrap(),
                row.get::<usize, String>(2).unwrap()
            )
        });
        match select_all {
            Ok(all) => {
                for (_, expire_key, storage_type) in all {
                    if storage_type == DISK {
                        let _ = pro_sqlite_util::execute_by_params(
                            "DELETE FROM `DISK_CACHE` WHERE `KEY` = ? ;",
                            (expire_key.clone(),),
                        );
                    } else {
                        let mut map_cache = MAP_CACHE.lock().unwrap();
                        map_cache.remove(&expire_key);
                    }
                    let _ = pro_sqlite_util::execute_by_params(
                        "DELETE FROM `MAP_DB_EXPIRE` WHERE `STORAGE_TYPE` = ? AND `EXPIRE_KEY` = ? ;",
                        (storage_type, expire_key.clone()),
                    );
                }
            }
            Err(_) => println!("删除本地缓存过期数据,发生错误"),
        }
    }

    #[redis_lock_job(job_name = "CacheJob_insert", interval_millis = 1000)]
    pub async fn insert() {
        let limit = 100000;
        // 再加入新的过期时间
        let mut del_where_vec = Vec::new();
        let mut insert_where_vec = Vec::new();
        let mut data_index = 0;
        while let Some((db_timeout, key, storage_type)) = INSERT_VEC.lock().unwrap().pop() {
            data_index+=1;

            {
                del_where_vec.push(format!(
                    r#" ( `STORAGE_TYPE` = "{}" AND `EXPIRE_KEY` = "{}" ) "#,
                    storage_type, key
                ));
            }

            {
                insert_where_vec.push(format!(
                    r#" ( "{}" , "{}", "{}") "#,
                    db_timeout, key, storage_type
                ));
            }
            
            if data_index > limit{
                println!("insret 截断");
                break;
            }

        }
        if !del_where_vec.is_empty() {
            let _ = pro_sqlite_util::execute(format!(
                "DELETE FROM `MAP_DB_EXPIRE` WHERE {} ;",
                del_where_vec.join(" OR ")
            ));
        }
        if !insert_where_vec.is_empty() {
            let _ = pro_sqlite_util::execute(format!(
                "INSERT INTO `MAP_DB_EXPIRE` (`TIMEOUT`, `EXPIRE_KEY`, `STORAGE_TYPE`) VALUES {} ;",
                insert_where_vec.join(" , ")
            ));
        }

    }
}

pub fn disk_write<T>(key: impl Into<String>, t: &T)
where
    T: Serialize,
{
    disk_write_timeout(key, t, TIMEOUT);
}

pub fn disk_write_timeout<T>(key: impl Into<String>, t: &T, timeout: i64)
where
    T: Serialize,
{
    let value = serde_json::to_string(t).unwrap();
    let key = key.into();
    let _ = pro_sqlite_util::execute_by_params(
        "INSERT OR REPLACE INTO `DISK_CACHE` (`KEY`, `DATA`) VALUES (?, ?);",
        (key.clone(), value),
    );
    add_expir(key, timeout, DISK);
}

pub fn disk_read<T>(key: impl Into<String>) -> Option<T>
where
    T: for<'a> Deserialize<'a>,
{
    let key = key.into();
    let result = pro_sqlite_util::select_one(
        "SELECT `DATA` FROM `DISK_CACHE` WHERE `KEY` = ?",
        (key,),
        |row| row.get::<usize, String>(0).unwrap(),
    );

    match result {
        Ok(data) => pro_json_util::str_to_object(&data.unwrap()).unwrap(),
        Err(_) => {
            panic!("解析错误")
        }
    }
}

pub fn cache_write<T>(key: impl Into<String>, t: &T)
where
    T: Serialize,
{
    cache_write_timeout(key, t, TIMEOUT);
}

// 缓存数据,并写入超时时间
pub fn cache_write_timeout<T>(key: impl Into<String>, t: &T, timeout: i64)
where
    T: Serialize,
{
    let key = key.into();
    let mut map_cache = MAP_CACHE.lock().unwrap();
    let value = serde_json::to_vec(t).unwrap();
    map_cache.insert(key.clone(), value);
    add_expir(key, timeout, CACHE);
}

pub fn cache_read<T>(key: impl Into<String>) -> Option<T>
where
    T: for<'a> Deserialize<'a>,
{
    let key = key.into();
    let map_cache = MAP_CACHE.lock().unwrap();
    let cache_get = map_cache.get(&key);
    match cache_get {
        Some(data_vec) => {
            let from_str: T = pro_json_util::vec_to_object(data_vec.to_vec()).unwrap();
            Some(from_str)
        }
        None => None,
    }
}

pub async fn cache_read_by_fn<T>(
    key: impl Into<String>,
    get_by_fn: impl Future<Output = Option<T>>,
    timeout: i64,
) -> Option<T>
where
    T: Serialize + for<'a> Deserialize<'a>,
{
    let key = key.into();
    let cache_read: Option<T> = cache_read(key.clone());
    if let None = cache_read {
        // 如果是None, 通过函数拉取
        let get_by_fn = get_by_fn.await;
        if let Some(val) = &get_by_fn {
            // 如果拉取到数据,写入缓存携带时间
            cache_write_timeout(key, val, timeout);
        }
        get_by_fn
    } else {
        cache_read
    }
}

fn add_expir(key: impl Into<String>, timeout: i64, storage_type: &str) {
    let key = key.into();
    let db_timeout = pro_time_util::get_current_milliseconds() + timeout;
    INSERT_VEC
        .lock()
        .unwrap()
        .push((db_timeout, key.clone(), storage_type.to_string()));
}
