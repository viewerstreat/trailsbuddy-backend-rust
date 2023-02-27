use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[allow(non_camel_case_types)]
pub enum LoginScheme {
    #[default]
    OTP_BASED,
    GOOGLE,
    FACEBOOK,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: u32,
    pub name: String,
    pub phone: String,

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
    pub referred_by: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_played: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub contest_won: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_earning: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_ts: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_ts: Option<u64>,
}