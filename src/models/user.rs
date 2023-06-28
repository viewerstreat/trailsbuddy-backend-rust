use chrono::prelude::*;
use mongodb::bson::Bson;
use serde::{Deserialize, Serialize};

use super::wallet::Money;
use crate::utils::get_epoch_ts;

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[allow(non_camel_case_types)]
pub enum LoginScheme {
    #[default]
    OTP_BASED,
    GOOGLE,
    FACEBOOK,
}

impl From<LoginScheme> for Bson {
    fn from(value: LoginScheme) -> Self {
        match value {
            LoginScheme::OTP_BASED => Self::String("OTP_BASED".to_owned()),
            LoginScheme::GOOGLE => Self::String("GOOGLE".to_owned()),
            LoginScheme::FACEBOOK => Self::String("FACEBOOK".to_owned()),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: u32,
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_pic: Option<String>,

    pub login_scheme: LoginScheme,
    pub is_active: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_login_time: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_used_referral_code: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub referral_code: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub referred_by: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_played: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub contest_won: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_earning: Option<Money>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_ts: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_ts: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fcm_tokens: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecialReferralCode {
    referral_code: String,
    bonus: u64,
    is_active: bool,
    valid_till: u64,
    created_ts: u64,
    updated_ts: Option<u64>,
    created_by: u32,
    updated_by: Option<u32>,
}

impl SpecialReferralCode {
    pub fn new(
        referral_code: &str,
        bonus: u64,
        valid_till: &DateTime<Utc>,
        created_by: u32,
    ) -> Self {
        Self {
            referral_code: referral_code.to_owned(),
            bonus,
            is_active: true,
            valid_till: valid_till.timestamp() as u64,
            created_ts: get_epoch_ts(),
            created_by,
            updated_ts: None,
            updated_by: None,
        }
    }
    pub fn bonus(&self) -> u64 {
        self.bonus
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct LeaderboardData {
    id: u32,
    name: String,
    total_played: u32,
    contest_won: u32,
    total_earning: Money,
}

impl LeaderboardData {
    pub fn new(
        id: u32,
        name: String,
        total_played: u32,
        contest_won: u32,
        total_earning: Money,
    ) -> Self {
        Self {
            id,
            name,
            total_played,
            contest_won,
            total_earning,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminUser {
    pub id: u32,
    pub name: String,
    pub phone: String,
    pub is_active: bool,
}
