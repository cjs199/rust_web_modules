use framework_base_web::base_service::DB_ONCE_LOCK;
use framework_base_web::dto::paged_result::PageResult;
use framework_base_web::utils::pro_base_security_util;
use framework_base_web::utils::pro_snowflake_util;
use framework_base_web::utils::pro_sql_query_util;
use framework_base_web::utils::pro_sql_query_util::Condition;
use framework_base_web::utils::pro_sql_query_util::Sort;
use framework_macro::pro_json_ser_der;
use framework_macro::table;
use framework_macro::SqlEnum;
use framework_utils::exception_enum::ProException;
use framework_utils::json::serde_customize::option_empty_ignore_deser;
use framework_utils::json::serde_customize::{
    option_default_none, option_num_deser, option_num_ser,
};
use framework_utils::pro_collection_util;
use framework_utils::pro_json_util;
use framework_utils::pro_str_util;
use log::error;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::mysql::MySqlRow;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::types::Json;
use sqlx::Executor;
use sqlx::MySql;
use sqlx::Row;
use sqlx::Transaction;
use sqlx::{mysql::MySqlQueryResult, FromRow};
use std::any::Any;
use std::collections::HashMap;
use std::future::Future;

#[table(test)]
#[pro_json_ser_der]
pub struct Test {
    #[id]
    pub id: Option<i64>,
    pub create_by: Option<i64>,
    pub create_time: Option<DateTime<Utc>>,
    pub update_by: Option<i64>,
    pub update_time: Option<DateTime<Utc>>,
    pub version: Option<i32>,
    pub str_column: Option<String>,
    pub f32_column: Option<f32>,
    // 测试json枚举字段
    pub json_column: Option<Json<HashMap<String, Value>>>,
    // 测试枚举字段
    pub enum_column: Option<TestEnum>,
}


// 枚举列类型demo
#[derive(Debug, sqlx::Decode, sqlx::Encode, Serialize, Deserialize, Default, SqlEnum)]
#[sqlx]
pub enum TestEnum {
    #[default]
    Small,
    Medium,
    Large,
}
