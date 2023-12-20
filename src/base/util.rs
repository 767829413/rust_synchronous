use chrono::{DateTime, Local, Utc};
use std::time::SystemTime;

#[allow(unused)]
pub fn format_now_date() -> String {
    // 获取当前时间戳（UTC时间）
    let utc: DateTime<Utc> = Utc::now();

    // 将时间戳转换为本地时间
    let local: DateTime<Local> = utc.into();

    // 格式化为字符串
    let formatted = local.format("%Y-%m-%d %H:%M:%S").to_string();
    return formatted;
}

#[allow(unused)]
pub fn now_timestamp_nanos() -> u128 {
    // 获取当前时间戳
    let timestamp = SystemTime::now();

    // 获取时间戳的纳秒表示
    let nanos = timestamp
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    return nanos;
}

#[allow(unused)]
pub fn now_timestamp_millis() -> u128 {
    // 获取时间戳的纳秒表示
    let nanos = now_timestamp_nanos();

    (nanos / 1_000_000) % 1000
}

#[allow(unused)]
pub fn now_timestamp_micros() -> u128 {
    // 获取时间戳的纳秒表示
    let nanos = now_timestamp_nanos();

    (nanos / 1_000) % 1000
}

#[allow(unused)]
pub fn now_timestamp_seconds() -> u128 {
    // 获取时间戳的纳秒表示
    let nanos = now_timestamp_nanos();

    nanos / 1_000_000_000
}
