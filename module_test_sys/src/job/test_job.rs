use framework_macro::{interval_job, job, redis_lock_job};
use framework_redis::utils::{pro_redis_mq_msg_util, pro_redis_util};
use framework_utils::pro_job_util::TimerTask;
use framework_utils::pro_time_util;
use idgenerator::*;
use log::info;

// 本地缓存清理定时任务
pub struct TestJobQue {}

#[job]
impl TestJobQue {
    
    // 间隔执行,向redis队列key放入一个雪花算法生成的id
    #[interval_job(job_name = "test_job_que", interval_millis = 5000)]
    pub async fn test_job_que() {
        pro_redis_mq_msg_util::put_msg_que("key", IdInstance::next_id());
    }
    
    // 分布式锁定时任务,向redis广播test_job_pub 放入一个雪花算法生成的id
    #[redis_lock_job(job_name = "test_job_pub", interval_millis = 5000)]
    pub async fn test_job_pub() {
        pro_redis_mq_msg_util::put_msg_pub("test_job_pub", IdInstance::next_id());
    }

}
