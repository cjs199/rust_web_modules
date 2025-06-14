use framework_macro::FieldNameConstants;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, FieldNameConstants, Debug)]
pub struct QueryDto<T> {
    pub page: i64,
    pub limit: i64,
    pub entity: T,
}
