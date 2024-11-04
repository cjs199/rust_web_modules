use std::collections::HashMap;

use crate::entities::test::{Test, TestSqlQuery};
use axum::{response::IntoResponse, Json};
use framework_base_web::utils::{pro_snowflake_util, pro_sql_query_util::Condition};
use framework_macro::{control, get, pro_anonymous};
use framework_redis::utils::pro_redis_util;
use framework_utils::{pro_json_util, pro_time_util};
use serde_json::Value;
use sqlx::Row;

pub struct TestControl {}

#[control]
impl TestControl {

    // pro_anonymous 匿名访问注解
    #[pro_anonymous]
    #[get("/get_demo")]
    pub async fn get_demo() -> impl IntoResponse {
        println!("get_demo");
        Json("OK")
    }

    // pro_anonymous 匿名访问注解 
    #[pro_anonymous]
    #[get("/post_demo")]
    pub async fn post_demo() -> impl IntoResponse {
        println!("post_demo");
        Json("OK")
    }

    // 通过id查询
    #[pro_anonymous]
    #[get("/test_direct_insert")]
    pub async fn test_direct_insert() -> impl IntoResponse {
        let direct_find_all = TestSqlQuery::direct_find_all().await.unwrap();
        for mut item in direct_find_all {
            item.id = None;
            let mut hash_map: HashMap<String, Value> = HashMap::new();
            hash_map.insert("k".to_string(), Value::String("v".to_string()));
            let json: sqlx::types::Json<HashMap<String, Value>> = sqlx::types::Json(hash_map);
            item.json_column = Some(json);
            let _direct_find_all = TestSqlQuery::direct_insert(item).await;
        }
        Json("OK")
    }

    // 查询一条打印
    #[pro_anonymous]
    #[get("/test_direct_find_by_id")]
    pub async fn test_direct_find_by_id() -> impl IntoResponse {
        let id: i64 = 9732915109888006;
        let direct_find_by_id = TestSqlQuery::direct_find_by_id(Box::new(id)).await.unwrap();
        println!(
            "test_direct_find_by_id:{}",
            pro_json_util::object_to_str_pretty(&direct_find_by_id)
        );
        Json("OK")
    }

    // 个性化查询
    #[pro_anonymous]
    #[get("/test_diy_find")]
    pub async fn test_diy_find() -> impl IntoResponse {
        let find_all = TestSqlQuery::new()
        .select((&[Test::FIELD_ID,Test::FIELD_VERSION ]).to_vec())
        .where_(Test::FIELD_ID, Condition::gt, Box::new(2))
        .limit(10)
        .find_all(|row|{
            let id:i64 = row.get(0);
            let version:i32 = row.get(1);
            return (id,version);
        }).await.unwrap();
        println!(
            "test_diy_find:{}",
            pro_json_util::object_to_str_pretty(&find_all)
        );
        Json("OK")
    }

    // 查询所有打印
    #[pro_anonymous]
    #[get("/test_direct_find_all")]
    pub async fn test_direct_find_all() -> impl IntoResponse {
        let direct_find_all = TestSqlQuery::direct_find_all().await.unwrap();
        println!(
            "test_direct_find_all:{}",
            pro_json_util::object_to_str_pretty(&direct_find_all)
        );
        Json("OK")
    }

    // 查询所有打印
    #[pro_anonymous]
    #[get("/test_lock_wraper")]
    pub async fn test_lock_wraper() -> impl IntoResponse {
        pro_redis_util::lock_wraper("test_lock_wraper", pro_snowflake_util::next_id_str(), async{
            println!("加锁");
            pro_time_util::sleep(10000);
            println!("解锁");
        }).await;
        Json("OK")
    }
}
