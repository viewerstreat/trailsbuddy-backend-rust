use serde::{Deserialize, Serialize};

use crate::{constants::*, utils::get_epoch_ts};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Otp {
    pub user_id: u32,
    pub otp: String,
    pub valid_till: u64,
    pub is_used: bool,
    pub update_ts: u64,
}

impl Otp {
    pub fn new(user_id: u32, otp: &str) -> Self {
        let ts = get_epoch_ts();
        Self {
            user_id,
            otp: otp.to_string(),
            valid_till: ts + OTP_VALIDITY_MINS * 60,
            is_used: false,
            update_ts: ts,
        }
    }
}
