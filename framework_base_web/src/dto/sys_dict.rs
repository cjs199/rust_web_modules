use framework_utils::pro_json_util;
use redis::{from_redis_value, FromRedisValue, RedisResult, Value};
use serde::{Deserialize, Serialize};
use framework_utils::json::serde_customize::num_ser;
use framework_utils::json::serde_customize::num_deser;

#[allow(non_snake_case)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct RedisSysDict {
    pub dictCode: String,
    pub dictName: String,
    pub sysDictItems: Option<(String, Vec<RedisSysDictItems>)>,
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct RedisSysDictItems {
    #[serde(serialize_with = "num_ser", deserialize_with = "num_deser")]
    pub dictId: i64,
    pub dictItemCode: String,
    pub dictItemName: String,
    pub dictItemValue: String,
}

impl FromRedisValue for RedisSysDict {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        let json_str: String = from_redis_value(v)?;
        let obj: RedisSysDict = pro_json_util::str_to_object(&json_str).unwrap();
        Ok(obj)
    }
}
