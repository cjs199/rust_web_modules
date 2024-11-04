use chrono::DateTime;
use chrono::Utc;
use framework_utils::pro_time_util;
use sqlx::mysql::MySqlRow;
use sqlx::Column;
use sqlx::Row;

use sqlx::types::BigDecimal;
use strum_macros::Display;
use strum_macros::EnumIter;

#[allow(warnings)]
#[derive(PartialEq)]
pub enum Condition {
    // 大于
    gt,
    // 小于
    lt,
    // 等于
    eq,
    // 不等于
    ne,
    like,
    In,
}

#[allow(warnings)]
#[derive(PartialEq , Display, EnumIter)]
pub enum Sort {
    Asc,

    Desc,
}

pub fn get_row_val_to_str(row: &MySqlRow, index: usize, sql_type: impl Into<String>) -> String {
    // VARCHAR
    // BIGINT
    // DECIMAL
    let sql_type: &str = &sql_type.into();
    match sql_type {
        "VARCHAR" => {
            let get: String = row.get(index);
            get
        }
        "JSON" => {
            let get: serde_json::Value = row.get(index);
            get.to_string()
        }
        "BIGINT" => {
            let get: i64 = row.get(index);
            get.to_string()
        }
        "INT" => {
            let get: i32 = row.get(index);
            get.to_string()
        }
        "DECIMAL" => {
            let get: BigDecimal = row.get(index);
            get.to_string()
        }
        "DATETIME" => {
            let get: DateTime<Utc> = row.get(index);
            get.format(pro_time_util::PATTERN_T_UTC).to_string()
        }
        _ => panic!("未知数据类型:{}", sql_type),
    }
}

pub fn get_row_column_name_to_str(row: &MySqlRow, index: usize) -> String {
    let column = row.column(index);
    let column_name = column.name();
    column_name.to_string()
}

pub fn get_row_column_type_name_to_str(row: &MySqlRow, index: usize) -> String {
    let column = row.column(index);
    let column_type_info = column.type_info();
    let column_type_name = column_type_info.to_string();
    column_type_name
}
