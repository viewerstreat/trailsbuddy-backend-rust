use axum::http::HeaderMap;
use rand::{thread_rng, Rng};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::jwt::JWT_KEYS;

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

pub fn get_user_id_from_token(headers: &HeaderMap) -> Option<u32> {
    let authorization = headers.get("Authorization")?.to_str().ok()?;
    let (_, token) = authorization.split_once(' ')?;
    let claims = JWT_KEYS.extract_claims(token)?;
    Some(claims.id)
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use super::*;

    #[test]
    fn test_get_epoch_ts() {
        let d = Duration::from_secs(1);
        let t1 = get_epoch_ts();
        thread::sleep(d);
        let t2 = get_epoch_ts();
        assert_eq!(t1 > 0, true);
        assert_eq!(t2 > 0, true);
        assert_eq!(t1 + 1 <= t2, true);
    }

    #[test]
    fn test_generate_otp_zero_len() {
        let otp = generate_otp(0);
        assert_eq!(otp, String::new());
    }

    #[test]
    fn test_generate_otp_six_len() {
        let otp = generate_otp(6);
        assert_eq!(otp.len(), 6);
        assert_eq!(otp.chars().all(|ch| ch.is_ascii_digit()), true);
    }

    #[test]
    fn test_generate_otp_random() {
        let otp1 = generate_otp(6);
        let otp2 = generate_otp(6);
        assert_ne!(otp1, otp2);
    }
}
