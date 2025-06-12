use std::collections::HashMap;

use framework_macro::pro_json_ser_der;
use framework_utils::pro_json_util;
use framework_utils::json::serde_customize::num_ser;
use framework_utils::json::serde_customize::num_deser;
use serde::{Deserialize, Serialize};
use serde_json::Value;


// 登录信息数据传输对象结构体
#[pro_json_ser_der]
pub struct LoginInfoDto {
    // 应用名称
    pub app: String,
    // 用户 ID，使用 Option 表示可为空
    pub uid: i64,
    // TOKEN 对应的权限数据传输对象
    pub aid: String,
    // 到期时间(单位秒),缩短时间长度
    pub exp: i64,
    // 除去基本信息外，其他信息存放位置，使用字符串作为键，Value 可以表示多种 JSON 值类型
    pub ui: HashMap<String, Value>,
}
