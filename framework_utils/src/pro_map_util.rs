use std::collections::HashMap;
use serde::{Deserialize, Serialize};


// 根据键字段对向量进行分组。
//
// 该函数接受一个元素向量 `T` 和一个闭包 `key_fn`，该闭包从每个元素中提取出一个键 `V`。
// 然后，它根据提取的键对向量中的元素进行分组，并返回一个 HashMap，其中：
// * 键：从每个元素中提取的键值 `V`。
// * 值：包含对具有相同键的原始元素的引用的向量。
//
// **参数:**
//
// * `vec`: 要分组的元素向量的引用。
// * `key_fn`: 一个闭包，接受一个元素 `T` 的引用并返回相应的键 `V`。
//
// **返回:**
//
// 一个 HashMap，其中：
//
// * 键：从每个元素中提取的键 `V`。
// * 值：包含对具有相同键的原始元素的引用的向量。
pub fn group_by_key_fn<T, F, V>(vec: &Vec<T>, key_fn: F) -> HashMap<V, Vec<&T>>
where
    F: Fn(&T) -> V,
    V: std::cmp::Eq + std::hash::Hash + Clone,
    T: Serialize + for<'de> Deserialize<'de>,
{
    let mut groups: HashMap<V, Vec<&T>> = HashMap::new();
    for item in vec {
        let key = key_fn(item);
        groups.entry(key).or_default().push(item);
    }
    groups
}

pub fn group_by_key_fn_and_val_fn<T, F1, F2, K, V>(
    vec: &Vec<T>,
    key_fn: F1,
    val_fn: F2,
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
        let value = val_fn(item);
        groups.entry(key).or_default().push(value);
    }
    groups
}

pub fn group_by_key_fn_to_single_object<T, F, V>(vec: &Vec<T>, key_fn: F) -> HashMap<V, &T>
where
    F: Fn(&T) -> V,
    V: std::cmp::Eq + std::hash::Hash + Clone,
    T: Serialize + for<'de> Deserialize<'de>,
{
    let mut groups: HashMap<V, &T> = HashMap::new();
    for item in vec {
        let key = key_fn(item);
        groups.insert(key, item);
    }
    groups
}

pub fn group_by_key_fn_and_val_fn_to_single_object<T, KF, VF,K,V>(vec: &Vec<T>, key_fn: KF, val_fn: VF) -> HashMap<K, V>
where
    KF: Fn(&T) -> K,
    VF: Fn(&T) -> V,
    K: std::cmp::Eq + std::hash::Hash + Clone,
    V: std::cmp::Eq + std::hash::Hash + Clone,
{
    let mut groups: HashMap<K,V> = HashMap::new();
    for item in vec {
        let key = key_fn(item);
        let val = val_fn(item);
        groups.insert(key, val);
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
pub fn get_all_by_key_arr_and_key_fn<'a, KEY, K: std::cmp::Eq + std::hash::Hash, V>(
    map: &'a HashMap<K, V>,
    keys: &'a Vec<KEY>,
    key_fn: impl Fn(&KEY) -> K,
) -> Vec<Option<&'a V>> {
    let mut values = Vec::with_capacity(keys.len());
    for key in keys {
        let key_by_key_fn  = key_fn(key);
        if let Some(value) = map.get(&key_by_key_fn) {
            values.push(Some(value));
        } else {
            values.push(None);
        }
    }
    values
}
