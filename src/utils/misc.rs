use std::time::{SystemTime, UNIX_EPOCH};

/// Get EPOCH timestamp in seconds
pub fn get_epoch_ts() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}
