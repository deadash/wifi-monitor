use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Timelike};

#[derive(Debug, Serialize, Deserialize)]
pub enum TimeCondition {
    TimeRange(String, String),
    After(String),
    Before(String),
}


pub fn is_time_condition_satisfied(time_condition: &Option<TimeCondition>, now: &DateTime<Tz>) -> bool {
    match time_condition {
        Some(TimeCondition::TimeRange(start, end)) => {
            // 解析开始时间和结束时间为小时和分钟
            let (start_hour, start_minute) = parse_time(start);
            let (end_hour, end_minute) = parse_time(end);
            
            // 将当前时间的小时和分钟转换为分钟总数进行比较
            let current_minutes = now.hour() * 60 + now.minute();
            let start_minutes = start_hour * 60 + start_minute;
            let end_minutes = end_hour * 60 + end_minute;

            current_minutes >= start_minutes && current_minutes <= end_minutes
        },
        Some(TimeCondition::After(time)) => {
            // 解析指定的时间为小时和分钟
            let (after_hour, after_minute) = parse_time(time);
            
            // 将当前时间和指定时间转换为分钟总数进行比较
            let current_minutes = now.hour() * 60 + now.minute();
            let after_minutes = after_hour * 60 + after_minute;

            current_minutes >= after_minutes
        },
        Some(TimeCondition::Before(time)) => {
            // 解析指定的时间为小时和分钟
            let (before_hour, before_minute) = parse_time(time);
            
            // 将当前时间和指定时间转换为分钟总数进行比较
            let current_minutes = now.hour() * 60 + now.minute();
            let before_minutes = before_hour * 60 + before_minute;

            current_minutes < before_minutes
        },
        None => true, // 如果没有设置时间条件，视为总是满足
    }
}

// 解析时间字符串为小时和分钟
fn parse_time(time_str: &str) -> (u32, u32) {
    let parts: Vec<&str> = time_str.split(':').collect();
    let hour = parts.get(0).unwrap_or(&"00").parse::<u32>().unwrap_or(0);
    let minute = parts.get(1).unwrap_or(&"00").parse::<u32>().unwrap_or(0);
    (hour, minute)
}