use axum::http::HeaderMap;
use mongodb::bson::oid::ObjectId;
use rand::{distributions::uniform::SampleUniform, thread_rng, Rng};
use regex::Regex;
use serde::{Deserialize, Deserializer};
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use super::AppError;
use crate::{constants::*, jwt::JWT_KEYS};

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

/// Generate referral_code for an user
pub fn generate_referral_code(id: u32, name: &str) -> String {
    let mut code = String::new();
    // put first 3 chars from the name into the code
    // if name does not contain 3 alphabetic char then random char is added
    let mut chars = name.chars();
    loop {
        let ch = chars.next().unwrap_or_else(|| get_random_num('A', 'Z'));
        if ch.is_ascii_alphabetic() {
            code.push(ch.to_ascii_uppercase());
        }
        if code.len() >= 3 {
            break;
        }
    }
    // put  last 3 chars from id into the code
    // avoid putting '0' into the code since it is confusing with 'O'
    let id = id % 1000;
    let id = id.to_string();
    for ch in id.chars() {
        if code.len() >= REFERRAL_CODE_LEN {
            break;
        }
        if ch.is_ascii_digit() && ch != '0' {
            code.push(ch);
        }
    }
    // fill rest all characters with random digits
    for _ in code.len()..REFERRAL_CODE_LEN {
        code.push(char::from_digit(get_random_num(1, 10), 10).unwrap_or('0'));
    }

    code
}

/// Generate a random number in a given range
/// panics if the lower bound is greater than the higher bound
pub fn get_random_num<T>(low: T, high: T) -> T
where
    T: PartialEq + PartialOrd + SampleUniform,
{
    assert!(low < high);
    let mut rng = thread_rng();
    rng.gen_range(low..high)
}

/// Extracts user_id value from authorization header
pub fn get_user_id_from_token(headers: &HeaderMap) -> Option<u32> {
    let authorization = headers.get("Authorization")?.to_str().ok()?;
    let (_, token) = authorization.split_once(' ')?;
    let claims = JWT_KEYS.extract_claims(token)?;
    Some(claims.id)
}

/// Returns S3 object url for a given key
pub fn get_object_url(key: &str) -> String {
    let region = std::env::var("AWS_REGION").unwrap_or(AWS_REGION.to_owned());
    format!("https://{}.s3.{}.amazonaws.com/{}", AWS_BUCKET, region, key)
}

/// Parse the given value as ObjectId
pub fn parse_object_id(id: &str, error_message: &str) -> Result<ObjectId, AppError> {
    let oid = ObjectId::parse_str(id).map_err(|err| {
        tracing::debug!("{:?}", err);
        AppError::BadRequestErr(error_message.into())
    })?;
    Ok(oid)
}

/// Deserialize helper for ObjectId field
pub fn deserialize_helper<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let val = Option::<ObjectId>::deserialize(deserializer)?;
    match val {
        None => Ok(None),
        Some(val) => Ok(Some(val.to_hex())),
    }
}

/// replace placeholder variables from the template text
/// placeholders are of patters {{variable}}
pub fn replace_placeholders(s: &str, options: HashMap<String, String>) -> anyhow::Result<String> {
    let re = Regex::new(r"\{\{(\w+)\}\}")?;
    let mut options = options;
    let mut replaced = String::from(s);
    for cap in re.captures_iter(s) {
        let var = &cap[1];
        if let Some(val) = options.get(var) {
            let find = &cap[0];
            let find = find.replace("{", r"\{");
            let find = find.replace("}", r"\}");
            if let Ok(re) = Regex::new(&find) {
                let rs = re.replace_all(s, val);
                replaced = rs.to_string();
                options.remove(var);
            }
        }
    }

    Ok(replaced)
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

    #[test]
    fn test_generate_referral_code() {
        let code = generate_referral_code(1, "");
        assert_eq!(code.len(), REFERRAL_CODE_LEN);
        let code = generate_referral_code(0, "");
        assert_eq!(code.len(), REFERRAL_CODE_LEN);
        let code = generate_referral_code(0, "Siba");
        assert_eq!(code.len(), REFERRAL_CODE_LEN);
        assert!(code.chars().all(|ch| ch != '0'));
        let mut chars = code.chars();
        assert_eq!(chars.next(), Some('S'));
        assert_eq!(chars.next(), Some('I'));
        assert_eq!(chars.next(), Some('B'));
        let code = generate_referral_code(1, "Mr. Bachchan");
        let mut chars = code.chars();
        assert_eq!(code.len(), REFERRAL_CODE_LEN);
        assert_eq!(chars.next(), Some('M'));
        assert_eq!(chars.next(), Some('R'));
        assert_eq!(chars.next(), Some('B'));
        assert_eq!(chars.next(), Some('1'));
        let code = generate_referral_code(14563, "Sibaprasad Maiti");
        let mut chars = code.chars();
        assert_eq!(code.len(), REFERRAL_CODE_LEN);
        assert_eq!(chars.next(), Some('S'));
        assert_eq!(chars.next(), Some('I'));
        assert_eq!(chars.next(), Some('B'));
        assert_eq!(chars.next(), Some('5'));
        assert_eq!(chars.next(), Some('6'));
        assert_eq!(chars.next(), Some('3'));
    }
}
