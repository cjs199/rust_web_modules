use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json;

use crate::pro_json_util;

// 深拷贝
pub fn clone<T: Serialize, R: for<'de> Deserialize<'de>>(original: &T) -> R {
    pro_json_util::clone(original)
}

pub fn clone_vec_object<T: Serialize, R: for<'de> Deserialize<'de>>(original: Vec<&T>) -> Vec<R> {
    let mut vec = Vec::new();
    for item in original {
        let r: R = clone(item);
        vec.push(r);
    }
    vec
}

pub fn clone_ref_vec_object<T: Serialize, R: for<'de> Deserialize<'de>>(
    original: &Vec<T>,
) -> Vec<R> {
    let mut vec = Vec::new();
    for item in original {
        let r: R = clone(item);
        vec.push(r);
    }
    vec
}

// 将map对象转换为指定对象
pub fn map_to_object<K: Serialize, V: Serialize, R: for<'de> Deserialize<'de>>(
    map: &HashMap<K, V>,
) -> R {
    // 将传入的对象 original 序列化为 JSON 字符串。
    // 如果序列化过程中出现错误，会导致程序 panic，实际应用中应进行错误处理。
    let json = serde_json::to_string(map).unwrap();

    // 从 JSON 字符串反序列化为类型 R 的对象。
    // 如果反序列化过程中出现错误，会导致程序 panic，实际应用中应进行错误处理。
    serde_json::from_str(&json).unwrap()
}
