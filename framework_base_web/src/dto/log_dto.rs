use framework_macro::pro_json_ser_der;
use framework_utils::pro_json_util;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};

#[derive(Clone)]
#[pro_json_ser_der]
pub struct LogDto {
    pub service_name: String,
    pub log_msg: String,
    pub time: DateTime<Utc>,
    pub level: String,
    pub trace_id: String,
}