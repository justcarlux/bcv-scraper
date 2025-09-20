use std::time::{SystemTime, UNIX_EPOCH};

pub fn current_time_millis() -> u128 {
    return SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
}
