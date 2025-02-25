use chrono::NaiveDateTime;

pub fn str_to_timestamp(ts: &str) -> u32 {
    let naive_dt = NaiveDateTime::parse_from_str(ts, "%Y-%m-%dT%H:%M:%S")
        .expect("Failed to parse datetime");

    naive_dt.and_utc().timestamp() as u32
}