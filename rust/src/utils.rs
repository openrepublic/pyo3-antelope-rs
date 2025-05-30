// use chrono::{NaiveDateTime, TimeZone, Utc};
// 
// pub fn str_to_timestamp(ts: &str) -> u32 {
//     let naive_dt =
//         NaiveDateTime::parse_from_str(ts, "%Y-%m-%dT%H:%M:%S").expect("Failed to parse datetime");
// 
//     naive_dt.and_utc().timestamp() as u32
// }
// 
// pub fn str_to_timestamp_ms(ts: &str) -> u64 {
//     let naive_dt =
//         NaiveDateTime::parse_from_str(ts, "%Y-%m-%dT%H:%M:%S").expect("Failed to parse datetime");
// 
//     naive_dt.and_utc().timestamp_millis() as u64
// }
// 
// pub fn timestamp_to_str(ts: u32) -> Option<String> {
//     Utc.timestamp_opt(ts as i64, 0)
//         .single() // Handle LocalResult by taking the single valid option
//         .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S").to_string()) // Format the datetime if valid
// }
// 
// pub fn timestamp_ms_to_str(ts: u64) -> Option<String> {
//     let milliseconds = (ts % 1000) as u16;
// 
//     Utc.timestamp_millis_opt(ts as i64)
//         .single() // Handle LocalResult correctly
//         .map(|dt| format!("{}.{}", dt.format("%Y-%m-%dT%H:%M:%S"), milliseconds))
// }
