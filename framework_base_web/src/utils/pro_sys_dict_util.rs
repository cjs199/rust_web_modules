use crate::dto::sys_dict::RedisSysDict;
use crate::dto::sys_dict::RedisSysDictItems;
use framework_macro::redis_mq;
use framework_macro::redis_mq_pub;
use framework_redis::utils::pro_redis_mq_msg_util;
use framework_redis::utils::pro_redis_util;
use framework_utils::pro_constant_pool_util;
use framework_utils::pro_json_util;
use framework_utils::pro_thread_util;
use lazy_static::lazy_static;
use log::info;
use std::collections::HashMap;
use tokio::sync::RwLock;

lazy_static! {
    pub static ref SYS_DICT_MAP: RwLock<HashMap<String, RedisSysDict>> = RwLock::new(HashMap::new());
}

pub struct SysDictUpdatePubMQ {}

#[redis_mq]
impl SysDictUpdatePubMQ {
    // 字典更新广播
    #[redis_mq_pub("sys_dict_update_pub")]
    pub async fn sys_dict_update_pub(_: String) {
        update_dict().await;
    }
}

pub async fn update_dict() {
    info!("更新字典");
    let map_entries: HashMap<String, RedisSysDict> =
        pro_redis_util::map_getall(pro_constant_pool_util::DICT_KEY);
    for ele in map_entries {
        SYS_DICT_MAP.try_write().unwrap().insert(ele.0, ele.1);
    }
}

pub fn get_sys_dict_by_dict_code(dict_code: impl Into<String>) -> Option<RedisSysDict> {
    let dict_code_str: String = dict_code.into();
    let sys_dict_option = SYS_DICT_MAP
        .try_read()
        .unwrap()
        .get(&dict_code_str)
        .cloned();
    match sys_dict_option {
        None => None,
        Some(sys_dict) => {
            let sys_dict_clone = sys_dict.clone();
            Some(sys_dict_clone)
        }
    }
}

pub fn get_sys_dict_items_by_dict_code(dict_code: impl Into<String>) -> Option<Vec<RedisSysDictItems>> {
    let sys_dict_option = get_sys_dict_by_dict_code(dict_code);
    match sys_dict_option {
        None => None,
        Some(sys_dict) => {
            let sys_dict_items: Option<(String, Vec<RedisSysDictItems>)> = sys_dict.sysDictItems;
            match sys_dict_items {
                None => None,
                Some(sys_dict_items) => {
                    let sys_dict_items: Vec<RedisSysDictItems> = sys_dict_items.1;
                    Some(sys_dict_items)
                }
            }
        }
    }
}

pub fn get_sys_dict_item_by_dict_code(
    dict_code: impl Into<String>,
    dict_item_code: impl Into<String>,
) -> Option<RedisSysDictItems> {
    let sys_dict_option = get_sys_dict_by_dict_code(dict_code);
    let dict_item_code_str: String = dict_item_code.into();
    match sys_dict_option {
        None => None,
        Some(sys_dict) => {
            let sys_dict_items: Option<(String, Vec<RedisSysDictItems>)> = sys_dict.sysDictItems;
            match sys_dict_items {
                None => None,
                Some(sys_dict_items) => {
                    let sys_dict_items: Vec<RedisSysDictItems> = sys_dict_items.1;
                    for sys_dict_item in sys_dict_items {
                        if sys_dict_item.dictItemCode.eq(&dict_item_code_str) {
                            return Some(sys_dict_item);
                        }
                    }
                    None
                }
            }
        }
    }
}
