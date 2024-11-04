use jsonmap::JsonMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ElementDataDto {
    pub code: i32,
    pub data: JsonMap<String>,
}
