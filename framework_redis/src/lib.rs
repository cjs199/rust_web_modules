use utils::{pro_redis_mq_msg_util::MqPushJob, pro_redis_util::REDIS_POOL};


pub mod utils;

// redis 系统相关初始化
pub fn init_redis() {

    // 启动时就激活redis
    let _ = REDIS_POOL.get();

    // redis mq 支持
    MqPushJob::init_job();

}
