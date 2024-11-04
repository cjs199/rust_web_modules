use std::sync::Mutex;

use super::pro_redis_util;
use framework_macro::interval_job;
use framework_macro::job;
use framework_utils::pro_job_util::TimerTask;
use framework_utils::pro_time_util;
use framework_utils::{pro_json_util, pro_map_util};
use idgenerator::*;
use log::info;
use serde::Serialize;

pub fn get_msg_pub_key(key: impl Into<String>) -> String {
    let key = key.into();
    let prefix = "msg:stream:pub:";
    if key.starts_with(prefix) {
        key.to_string()
    } else {
        format!("{}{}", prefix, key)
    }
}

pub fn get_msg_que_key(key: impl Into<String>) -> String {
    let key = key.into();
    let prefix = "msg:stream:queue:";
    if key.starts_with(prefix) {
        key.to_string()
    } else {
        format!("{}{}", prefix, key)
    }
}

pub fn put_msg_pub<V: Serialize + Send + 'static>(key: impl Into<String>, value: V) {
    let pub_ke = get_msg_pub_key(key);
    tokio::spawn(async move  {
        pro_redis_util::pubsub_send(pub_ke, pro_json_util::object_to_str(&value))
            .await;
    });
}

pub fn put_msg_que<V: Serialize>(key: impl Into<String>, value: V) {
    let key_str = get_msg_que_key(key);
    let item_str = pro_json_util::object_to_str(&value);
    MQ_QUE_VEC.lock().unwrap().push((key_str, item_str));
}

pub static MQ_QUE_VEC: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());

pub struct MqPushJob {}

#[job]
impl MqPushJob {
    #[interval_job(job_name = "mq_push_job", interval_millis = 1000)]
    pub async fn mq_push_job() {
        let mut mq_que_vec = MQ_QUE_VEC.lock().unwrap();
        if mq_que_vec.len() != 0 {
            let mq_que_vec_clone = mq_que_vec.clone();
            mq_que_vec.clear();
            drop(mq_que_vec);
            let mq_que_map = pro_map_util::group_by_key_field_get_val(
                &mq_que_vec_clone,
                |mq_que| mq_que.0.clone(),
                |mq_que| mq_que.1.clone(),
            );
            for (key, item_vec) in mq_que_map {
                pro_redis_util::streams_xadd_vec(key, item_vec).await;
            }
        }
    }
}
