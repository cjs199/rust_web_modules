use crate::config::init_config::{LOG_CACHE, LOG_FILE_WRITER};
use framework_macro::{interval_job, job};
use framework_redis::utils::pro_redis_util;
use framework_utils::pro_job_util::TimerTask;
use framework_utils::pro_time_util;
use idgenerator::*;
use log::info;
use std::io::prelude::*;

pub struct LogJob {}

#[job]
impl LogJob {
    #[interval_job(job_name = "log_writer_job", interval_millis = 3000)]
    pub async fn log_writer_job() {
        let mut log_cache = LOG_CACHE.lock().unwrap();
        if log_cache.len() != 0 {
            let new_log_cache = log_cache.clone();
            log_cache.clear();
            drop(log_cache);
            let mut writer = LOG_FILE_WRITER.lock().unwrap();
            for log_dto in &new_log_cache {
                writeln!(writer, "{}", log_dto.log_msg).unwrap();
            }
            writer.flush().unwrap();
        }
    }
}
