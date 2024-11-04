use framework_base_web::utils::pro_snowflake_util;
use framework_macro::redis_mq_pub;
use framework_macro::{redis_mq, redis_mq_que};
use framework_redis::utils::pro_redis_mq_msg_util;
use framework_redis::utils::pro_redis_util;
use framework_utils::pro_json_util;
use log::info;
use r2d2_redis::redis::Value::Data;
use r2d2_redis::redis;

pub struct TestMqQue {}

#[redis_mq]
impl TestMqQue {

    // redis 队列 ,接收key的消息
    #[redis_mq_que(que = "key", group = "sys_group")]
    pub async fn test_job_que(data: i64) {
        println!("test_job_que:{}", data);
    }

    // redis 广播 ,函数test_job_pub1,接收test_job_pub 的消息
    #[redis_mq_pub("test_job_pub")]
    pub async fn test_job_pub1(data: i64) {
        println!("test_job_pub1:{}", data);
    }
    
    // redis 广播 ,函数test_job_pub2,接收test_job_pub 的消息
    #[redis_mq_pub("test_job_pub")]
    pub async fn test_job_pub2(data: i64) {
        println!("test_job_pub2:{}", data);
    }

}

