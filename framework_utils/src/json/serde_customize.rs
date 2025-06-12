use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

// 如果反解析时,发现字段不存在,则返回None
pub fn option_default_none<T>() -> Option<T> {
    None
}

// 数字序列化为字符串
pub fn option_num_ser<T: std::fmt::Debug, S>(
    option_num: &Option<T>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    T: Serialize + std::fmt::Display,
    S: Serializer,
{
    match option_num {
        Some(num) => serializer.serialize_str(&num.to_string()),
        None => serializer.serialize_none(),
    }
}

// 数字序列化为字符串
pub fn num_ser<T: std::fmt::Debug, S>(num: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Serialize + std::fmt::Display,
    S: Serializer,
{
    serializer.serialize_str(&num.to_string())
}

// 反序列化,兼容数字和字符串,将之解析为指定的类型
pub fn option_num_deser<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Default + Deserialize<'de> + std::str::FromStr,
    D: Deserializer<'de>,
{
    let deserialize = Value::deserialize(deserializer);
    match deserialize {
        Ok(value) => {
            let mut val_str = value.to_string();
            if val_str.is_empty() {
                Ok(None)
            } else {
                // 如果字符串包含 " 那么说明是字符串数字,将"号去掉
                if val_str.contains("\"") {
                    val_str = val_str.replace("\"", "");
                }
                let s_parse_result = val_str.parse::<T>();
                match s_parse_result {
                    Ok(s_parse) => Ok(Some(s_parse)),
                    Err(_) => Ok(None),
                }
            }
        }
        Err(_) => Ok(None),
    }
}

// 反序列化,兼容数字和字符串,将之解析为指定的类型
pub fn num_deser<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de> + std::str::FromStr,
    D: Deserializer<'de>,
{
    // 使用Value作为泛型进行转换,兼容数字和字符串
    let deserialize = Value::deserialize(deserializer);
    let value = deserialize.ok().unwrap();
    // T可能是各种数字,比如int,float,通过Value作为媒介,转换为指定的需要的类型
    let mut val_str = value.to_string();
    // 如果字符串包含 " 那么说明是字符串数字,将"号去掉
    if val_str.contains("\"") {
        val_str = val_str.replace("\"", "");
    }
    let s_parse_result = val_str.parse::<T>().ok().unwrap();
    Ok(s_parse_result)
}

// 先尝试解析,如果解析失败返回None
pub fn option_empty_ignore_deser<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let ret_option = Option::deserialize(deserializer)?;
    Ok(ret_option)
}
