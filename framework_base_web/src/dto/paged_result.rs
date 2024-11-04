use serde::{Deserialize, Serialize};

#[allow(warnings)]
#[derive(Debug, Serialize, Deserialize)]
pub struct PageResult<T> {
    pub content: Vec<T>,
    pub totalElements: i64,
    pub page: i64,
}