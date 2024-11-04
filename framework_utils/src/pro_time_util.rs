use std::time::{SystemTime, UNIX_EPOCH};
use std::{thread, time};

pub const PATTERN_T_UTC: &str = "%Y-%m-%dT%H:%M:%SZ";
pub struct Second {}

impl Second {

    // 1秒
    pub const _1_SECOND: i64 = 1;

    // 1分钟
    pub const _1_MINUTE: i64 = Millisecond::_1_SECOND * 60;

    // 1分钟
    pub const _5_MINUTE: i64 = Millisecond::_1_MINUTE * 5;

    // 1小时
    pub const _1_HOUR: i64 = Millisecond::_1_MINUTE * 60;

    // 1天
    pub const _1_DAY: i64 = Millisecond::_1_HOUR * 24;

    // 2天
    pub const _2_DAY: i64 = Millisecond::_1_DAY * 2;

    // 3天
    pub const _3_DAY: i64 = Millisecond::_1_DAY * 3;

}

pub struct Millisecond {}
impl Millisecond {
    // 1毫秒
    pub const _1: i64 = 1;

    // 1毫秒
    pub const _200: i64 = 200;

    // 1秒
    pub const _1_SECOND: i64 = 1000;

    // 1分钟
    pub const _1_MINUTE: i64 = Millisecond::_1_SECOND * 60;

    // 3分钟
    pub const _3_MINUTE: i64 = Millisecond::_1_MINUTE * 3;

    // 5分钟
    pub const _5_MINUTE: i64 = Millisecond::_1_MINUTE * 5;

    // 1小时
    pub const _1_HOUR: i64 = Millisecond::_1_MINUTE * 60;

    // 1天
    pub const _1_DAY: i64 = Millisecond::_1_HOUR * 24;

    // 2天
    pub const _2_DAY: i64 = Millisecond::_1_DAY * 2;

    // 3天
    pub const _3_DAY: i64 = Millisecond::_1_DAY * 3;
}

/// 让当前线程睡眠指定的毫秒数。
pub fn sleep(millis: u64) {
    // 将传入的毫秒数转换为 Duration 类型，表示一段时间间隔。
    let ten_millis = time::Duration::from_millis(millis);
    // 使当前线程睡眠指定的时间间隔。
    thread::sleep(ten_millis);
}

/// 获取当前时间自 Unix 纪元（1970 年 1 月 1 日 00:00:00 UTC）以来的纳秒数。
pub fn get_current_nanos() -> u128 {
    // 获取当前系统时间。
    SystemTime::now()
        // 计算当前时间自 Unix 纪元以来的时间间隔。
        .duration_since(UNIX_EPOCH)
        // 如果获取时间间隔失败，触发此错误信息。通常不应该发生。
        .expect("获取时间间隔失败")
        // 将时间间隔转换为纳秒数并返回。
        .as_nanos() as u128
}

/// 获取当前时间自 Unix 纪元（1970 年 1 月 1 日 00:00:00 UTC）以来的毫秒数。
pub fn get_current_milliseconds() -> i64 {
    // 获取当前系统时间。
    SystemTime::now()
        // 计算当前时间自 Unix 纪元以来的时间间隔。
        .duration_since(UNIX_EPOCH)
        // 如果获取时间间隔失败，触发此错误信息。通常不应该发生。
        .expect("获取时间间隔失败")
        // 将时间间隔转换为毫秒数并返回。
        .as_millis() as i64
}

/// 获取当前时间自 Unix 纪元（1970 年 1 月 1 日 00:00:00 UTC）以来的秒数。
pub fn get_current_seconds() -> i64 {
    // 获取当前系统时间。
    SystemTime::now()
        // 计算当前时间自 Unix 纪元以来的时间间隔。
        .duration_since(UNIX_EPOCH)
        // 如果获取时间间隔失败，触发此错误信息。通常不应该发生。
        .expect("获取时间间隔失败")
        // 将时间间隔转换为秒数并返回。
        .as_secs() as i64
}
