use serde::{Deserialize, Deserializer, Serialize, Serializer};

// 如果反解析时,发现字段不存在,则返回None
pub fn option_default_none<T>() -> Option<T> {
    None
}

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

pub fn num_ser<T: std::fmt::Debug, S>(
    num: &T,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    T: Serialize + std::fmt::Display,
    S: Serializer,
{
    serializer.serialize_str(&num.to_string())
}

pub fn option_num_deser<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Default + Deserialize<'de> + std::str::FromStr,
    D: Deserializer<'de>,
{
    let deserialize = String::deserialize(deserializer);
    match deserialize {
        Ok(s) => {
            if s.is_empty() {
                Ok(None)
            } else {
                let s_parse_result = s.parse::<T>();
                match s_parse_result {
                    Ok(s_parse) => Ok(Some(s_parse)),
                    Err(_) => Ok(None),
                }
            }
        }
        Err(_) => Ok(None),
    }
}

pub fn num_deser<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de> + std::str::FromStr,
    D: Deserializer<'de>,
{
    let deserialize = String::deserialize(deserializer);
    let s = deserialize.ok().unwrap();
    let s_parse_result = s.parse::<T>().ok().unwrap();
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
