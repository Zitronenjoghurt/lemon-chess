use std::time::{SystemTime, UNIX_EPOCH};

pub fn timestamp_now_nanos() -> u64 {
    let start_time = SystemTime::now();
    let since_unix = start_time
        .duration_since(UNIX_EPOCH)
        .expect("Somehow the time went backwards...");
    since_unix.as_nanos() as u64
}
