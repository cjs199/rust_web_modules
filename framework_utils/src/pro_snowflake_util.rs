use idgenerator::*;
use log::warn;

pub fn init(base_time: i64, worker_id: i32, worker_id_bit_len: i32) {
    let mut snowflake_new = IdGeneratorOptions::new();
    if base_time > 0 {
        snowflake_new =snowflake_new.base_time(base_time);
    } else {
        warn!("系统没有设置雪花算法的偏移时间,使用默认的起始时间");
    }
    let options: IdGeneratorOptions = snowflake_new
        .worker_id(worker_id.try_into().unwrap())
        .worker_id_bit_len(worker_id_bit_len.try_into().unwrap());
    IdInstance::init(options).unwrap();
}

pub fn next_id() -> i64 {
    let id = IdInstance::next_id();
    return id;
}

pub fn next_id_str() -> String {
    let id = IdInstance::next_id();
    return id.to_string();
}
