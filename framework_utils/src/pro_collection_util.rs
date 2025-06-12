use std::{any::Any, collections::HashSet};

use crossbeam::queue::SegQueue;
use serde::{Deserialize, Serialize};

// 收集指定结构体切片中指定字段的值放入一个 `Vec` 返回，不去重。
//
// # 参数
// - `vec`：一个切片的引用，包含要进行收集的结构体。
// - `field_fn`：一个函数闭包，接受结构体的引用并返回要收集的字段的值。
//
// # 返回值
// 返回一个包含收集后的指定字段值的 `Vec`。
pub fn collect_field_values<T, F, V>(vec: &[T], field_fn: F) -> Vec<V>
where
    F: Fn(&T) -> V,
{
    let mut result = Vec::new();
    for item in vec {
        let value = field_fn(item);
        result.push(value);
    }
    result
}

// 收集指定结构体切片中指定字段的值放入一个 `HashSet` 并返回，实现去重。
//
// # 参数
// - `vec`：一个切片的引用，包含要进行收集的结构体。
// - `field_fn`：一个函数闭包，接受结构体的引用并返回要收集的字段的值。
//
// # 返回值
// 返回一个包含收集后的去重后的指定字段值的 `HashSet`。
pub fn collect_unique_field_values<T, F, V>(vec: &[T], field_fn: F) -> HashSet<V>
where
    T: Clone,
    F: Fn(&T) -> V,
    V: Clone + std::cmp::Eq + std::hash::Hash,
{
    let mut unique_values = HashSet::new();
    for item in vec {
        let value = field_fn(item);
        unique_values.insert(value);
    }
    unique_values
}

pub fn get_box_vec_len(boxed: &Box<dyn Any + Send + Sync>) -> i32 {
    if let Some(vec) = boxed.downcast_ref::<Vec<String>>() {
        return vec.len() as i32;
    }
    if let Some(vec) = boxed.downcast_ref::<Vec<&str>>() {
        return vec.len() as i32;
    }
    if let Some(vec) = boxed.downcast_ref::<Vec<char>>() {
        return vec.len() as i32;
    }
    if let Some(vec) = boxed.downcast_ref::<Vec<i32>>() {
        return vec.len() as i32;
    }
    if let Some(vec) = boxed.downcast_ref::<Vec<f32>>() {
        return vec.len() as i32;
    }
    if let Some(vec) = boxed.downcast_ref::<Vec<bool>>() {
        return vec.len() as i32;
    }
    if let Some(vec) = boxed.downcast_ref::<Vec<i8>>() {
        return vec.len() as i32;
    }
    if let Some(vec) = boxed.downcast_ref::<Vec<i16>>() {
        return vec.len() as i32;
    }
    if let Some(vec) = boxed.downcast_ref::<Vec<i64>>() {
        return vec.len() as i32;
    }
    if let Some(vec) = boxed.downcast_ref::<Vec<u8>>() {
        return vec.len() as i32;
    }
    if let Some(vec) = boxed.downcast_ref::<Vec<u16>>() {
        return vec.len() as i32;
    }
    if let Some(vec) = boxed.downcast_ref::<Vec<u32>>() {
        return vec.len() as i32;
    }
    if let Some(vec) = boxed.downcast_ref::<Vec<u64>>() {
        return vec.len() as i32;
    }
    panic!("没有实现处理的异常类型！,或者你传入了 Option，没有 unwrap?");
}

pub fn box_to_string(boxed: Box<dyn Any>) -> String {
    if let Some(value) = boxed.downcast_ref::<String>() {
        return value.clone();
    } else if let Some(value) = boxed.downcast_ref::<&str>() {
        return value.to_string();
    } else if let Some(value) = boxed.downcast_ref::<char>() {
        return value.to_string();
    } else if let Some(value) = boxed.downcast_ref::<i32>() {
        return value.to_string();
    } else if let Some(value) = boxed.downcast_ref::<f32>() {
        return value.to_string();
    } else if let Some(value) = boxed.downcast_ref::<bool>() {
        return value.to_string();
    } else if let Some(value) = boxed.downcast_ref::<i8>() {
        return value.to_string();
    } else if let Some(value) = boxed.downcast_ref::<i16>() {
        return value.to_string();
    } else if let Some(value) = boxed.downcast_ref::<i64>() {
        return value.to_string();
    } else if let Some(value) = boxed.downcast_ref::<u8>() {
        return value.to_string();
    } else if let Some(value) = boxed.downcast_ref::<u16>() {
        return value.to_string();
    } else if let Some(value) = boxed.downcast_ref::<u32>() {
        return value.to_string();
    } else if let Some(value) = boxed.downcast_ref::<u64>() {
        return value.to_string();
    } else {
        panic!("没有实现处理的异常类型！,或者你传入了 Option，没有 unwrap?");
    }
}

pub fn filter_data_by_function<T: Serialize + for<'de> Deserialize<'de>>(data: &Vec<T>, func: impl Fn(&T) -> bool) -> Vec<&T> {
    let mut result = Vec::new();
    for item in data {
        if func(item) {
            result.push(item);
        }
    }
    result
}

// 将vec集合转换成set集合
pub fn vec_to_set<T: std::hash::Hash + Eq>(vec: Vec<T>) -> HashSet<T> {
    vec.into_iter().collect()
}


// 将一个引用的向量按照指定大小进行分组，并返回包含对原始向量元素引用的向量的向量。
pub fn group_by_vec_size<T>(vec: &Vec<T>, group_size: usize) -> Vec<Vec<&T>> {
    // 用于存储分组结果的向量
    let mut groups: Vec<Vec<&T>> = Vec::new();
    // 如果传入的向量为空，则直接返回空的结果向量
    if vec.is_empty() {
        return groups;
    }
    // 遍历传入向量的分块
    for chunk in vec.chunks(group_size) {
        // 将当前分块中的元素引用收集到一个新的向量中，并添加到结果向量中
        groups.push(chunk.iter().collect());
    }
    // 返回分组结果
    groups
}

/**
 * 线程安全的队列
 */
pub fn remove_all_from_seg_queue <T>(sq: &SegQueue<T>) -> Vec<T> {
    let mut data = Vec::new();
    loop {
        let pop = sq.pop();
        if pop.is_none() {
            break;
        } else {
            let val = pop.unwrap();
            data.push(val);
        }
    }
    data
}