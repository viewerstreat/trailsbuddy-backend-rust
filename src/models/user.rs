use mongodb::bson::Bson;
use serde::{Deserialize, Serialize};

use super::wallet::Money;

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[allow(non_camel_case_types)]
pub enum LoginScheme {
    #[default]
    OTP_BASED,
    GOOGLE,
    FACEBOOK,
}
impl LoginScheme {
    pub fn to_bson(&self) -> anyhow::Result<Bson> {
        let bson = mongodb::bson::to_bson(self)?;
        Ok(bson)
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
    pub referred_by: Option<String>,

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
