use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::pro_bean_util;

/// 这个函数用于对一个切片进行分组操作。
///
/// # 参数
/// - `vec`：一个切片的引用，其中包含要进行分组的元素，这些元素必须实现`Clone` trait。
/// - `field_fn`：一个函数闭包，接受切片中的元素的引用，并返回一个用于分组的值，这个值的类型必须实现`Eq`、`Hash`和`Clone` trait。
///
/// # 返回值
/// 返回一个哈希表，键是由`field_fn`生成的用于分组的值，值是一个包含对应分组元素的动态数组。
pub fn group_by_key_field<T, F, V>(vec: &Vec<T>, key_fn: F) -> HashMap<V, Vec<T>>
where
    F: Fn(&T) -> V,
    V: std::cmp::Eq + std::hash::Hash + Clone,
    T: Serialize + for<'de> Deserialize<'de>,
{
    let mut groups: HashMap<V, Vec<T>> = HashMap::new();
    for item in vec {
        let key = key_fn(item);
        let ret: T = pro_bean_util::clone(item);
        groups.entry(key).or_default().push(ret);
    }
    groups
}

pub fn group_by_key_field_get_val<T, F1, F2, K, V>(
    vec: &Vec<T>,
    key_fn: F1,
    value_fn: F2,
) -> HashMap<K, Vec<V>>
where
    F1: Fn(&T) -> K,
    F2: Fn(&T) -> V,
    K: std::cmp::Eq + std::hash::Hash + Clone,
    T: Serialize + for<'de> Deserialize<'de>,
{
    let mut groups: HashMap<K, Vec<V>> = HashMap::new();
    for item in vec {
        let key = key_fn(item);
        let value = value_fn(item);
        groups.entry(key).or_default().push(value);
    }
    groups
}

pub fn group_by_key_field_to_single_object<T, F, V>(vec: &Vec<T>, field_fn: F) -> HashMap<V, T>
where
    F: Fn(&T) -> V,
    V: std::cmp::Eq + std::hash::Hash + Clone,
    T: Serialize + for<'de> Deserialize<'de>,
{
    let mut groups: HashMap<V, T> = HashMap::new();
    for item in vec {
        let key = field_fn(item);
        let ret: T = pro_bean_util::clone(item);
        groups.insert(key, ret);
    }
    groups
}

pub fn get_all_by_key_arr<'a, K: std::cmp::Eq + std::hash::Hash, V>(
    map: &'a HashMap<K, V>,
    keys: &'a Vec<K>,
) -> Vec<Option<&'a V>> {
    let mut values = Vec::with_capacity(keys.len());
    for key in keys {
        if let Some(value) = map.get(key) {
            values.push(Some(value));
        } else {
            values.push(None);
        }
    }
    values
}

// 因为存在引用,或者各种包装类型的原因,所以增加一个key函数处理
pub fn get_all_by_key_fn_arr<'a, KEY, K: std::cmp::Eq + std::hash::Hash, V>(
    map: &'a HashMap<K, V>,
    keys: &'a Vec<KEY>,
    k_fn: impl Fn(&KEY) -> K,
) -> Vec<Option<&'a V>> {
    let mut values = Vec::with_capacity(keys.len());
    for key in keys {
        let k_fn = k_fn(key);
        if let Some(value) = map.get(&k_fn) {
            values.push(Some(value));
        } else {
            values.push(None);
        }
    }
    values
}
