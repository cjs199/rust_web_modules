use framework_macro::pro_json_ser_der;
use framework_utils::pro_json_util;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use framework_utils::json::serde_customize::num_deser;
use framework_utils::json::serde_customize::num_ser;

#[pro_json_ser_der]
pub struct RequestRecordDto {
    pub time: DateTime<Utc>,
    pub user_id: i64,
    pub request_uri: String,
    pub ip: String,
    pub exec_time: i32,
    pub app_name: String,
    pub state: i32,
}
