use framework_macro::{interval_job, job};
use framework_redis::utils::pro_redis_util;
use framework_utils::pro_job_util::TimerTask;
use framework_utils::pro_time_util;
use idgenerator::*;
use log::info;

use crate::utils::pro_snowflake_util;

pub struct SnowflakeJob {}

static mut WORKER_ID_BIT_LEN: i32 = 0;

static mut WORKER_ID: i32 = 0;

#[job]
impl SnowflakeJob {
    fn get_snowflake_key(worker_id_bit_len: i32, worker_id: i32) -> String {
        format!("SnowflakeJob:{}:{}", worker_id_bit_len, worker_id)
    }

    // 缓存锁请求测试
    #[interval_job(job_name = "SnowflakeJob", interval_millis = 180000)]
    pub async fn snowflake_job() {
        if unsafe { WORKER_ID_BIT_LEN != 0 } {
            // 两个参数不为0 说明已经初始化了,为雪花算法续期
            let get_snowflake_key;
            unsafe {
                get_snowflake_key = SnowflakeJob::get_snowflake_key(WORKER_ID_BIT_LEN, WORKER_ID);
            }
            pro_redis_util::kv_set_expire(
                &get_snowflake_key,
                0,
                pro_time_util::Millisecond::_5_MINUTE,
            ).await;
        } else {
            pro_redis_util::lock_wraper("SnowflakeJobInit", pro_snowflake_util::next_id_str(), async{
                let worker_id_bit_len = 14;
                for worker_id in 0..(2 ^ worker_id_bit_len - 1) {
                    let get_snowflake_key =
                        SnowflakeJob::get_snowflake_key(worker_id_bit_len, worker_id);
                    let exists = pro_redis_util::exists(&get_snowflake_key).await;
                    if !exists {
                        // 这里报了错,因为静态变量修改,rust认为不安全,用 unsafe 包裹,这些数据不会在其他地方修改
                        unsafe {
                            WORKER_ID_BIT_LEN = worker_id_bit_len;
                            WORKER_ID = worker_id;
                        }
                        pro_redis_util::kv_set_expire(
                            &get_snowflake_key,
                            0,
                            pro_time_util::Millisecond::_5_MINUTE,
                        ).await;
                        let options: IdGeneratorOptions = IdGeneratorOptions::new()
                            .worker_id(worker_id.try_into().unwrap())
                            .worker_id_bit_len(worker_id_bit_len.try_into().unwrap());
                        IdInstance::init(options).unwrap();
                        return;
                    }
                }
            }).await;
        }
    }
}
