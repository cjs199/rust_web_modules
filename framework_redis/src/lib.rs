use utils::pro_redis_mq_msg_util::MqPushJob;

pub mod utils;

// redis 系统相关初始化
pub fn init_redis() {

    // redis mq 支持
    MqPushJob::init_job();

}
