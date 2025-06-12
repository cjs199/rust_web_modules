use serde::{Deserialize, Serialize};
use serde_json::{self, Error};
use std::collections::HashMap;

// 将任意可序列化的类型 T 转换为字符串表示的 JSON 格式
pub fn object_to_str<T: ?Sized + Serialize>(value: &T) -> String {
    // 将 value 序列化为 JSON 字符串，如果出现错误则会导致程序 panic，实际应用中应进行错误处理
    serde_json::to_string(value).unwrap()
}

// 将任意可序列化的类型 T 转换为美观打印的字符串表示的 JSON 格式
pub fn object_to_str_pretty<T: ?Sized + Serialize>(value: &T) -> String {
    // 将 value 序列化为美观打印的 JSON 字符串，如果出现错误则会导致程序 panic，实际应用中应进行错误处理
    serde_json::to_string_pretty(value).unwrap()
}

// 从 JSON 字符串反序列化为指定类型 T
pub fn str_to_object<'a, T: Deserialize<'a>>(json: &'a str) -> Result<T, Error> {
    // 从字符串反序列化为指定类型，如果出现错误则会导致程序 panic，实际应用中应进行错误处理
    serde_json::from_str(json)
}

// 从 JSON 字符串反序列化为 HashMap<String, serde_json::Value>
pub fn str_to_map<'a, T: Deserialize<'a>>(
    json: &'a str,
) -> Result<HashMap<String, serde_json::Value>, Error> {
    // 实际上调用了 str_to_object 函数进行反序列化，期望输入的 JSON 字符串能被反序列化为一个 HashMap
    str_to_object(json)
}

// 将任意可序列化的类型 T 转换为字节向量表示的 JSON 格式
pub fn vec_to_vec<T: ?Sized + Serialize>(value: &T) -> Vec<u8> {
    // 将 value 序列化为字节向量，如果出现错误则会导致程序 panic，实际应用中应进行错误处理
    let value = serde_json::to_vec(value).unwrap();
    return value;
}

// 从字节向量反序列化为指定类型 T
pub fn vec_to_object<T: for<'a> Deserialize<'a>>(json: Vec<u8>) -> Result<T, Error> {
    // 将字节向量转换为字符串
    let data_str: String = String::from_utf8(json).unwrap();
    // 再从字符串反序列化为指定类型，如果出现错误则会导致程序 panic，实际应用中应进行错误处理
    str_to_object(data_str.as_str())
}

// 深拷贝
pub fn clone<T: Serialize, R: for<'de> Deserialize<'de>>(original: &T) -> R {
    // 将传入的对象 original 序列化为 JSON 字符串。
    // 如果序列化过程中出现错误，会导致程序 panic，实际应用中应进行错误处理。
    let json = serde_json::to_string(original).unwrap();

    // 从 JSON 字符串反序列化为类型 R 的对象。
    // 如果反序列化过程中出现错误，会导致程序 panic，实际应用中应进行错误处理。
    serde_json::from_str(&json).unwrap()
}
