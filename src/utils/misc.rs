use rand::{thread_rng, Rng};
use std::time::{SystemTime, UNIX_EPOCH};

/// Get EPOCH timestamp in seconds
pub fn get_epoch_ts() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

/// Generate OTP of a given length
pub fn generate_otp(len: u32) -> String {
    let mut rng = thread_rng();
    (0..len)
        .map(|_| {
            let n = rng.gen_range(0..10);
            char::from_digit(n, 10).unwrap_or('0')
        })
        .collect()
}
