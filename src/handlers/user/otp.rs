use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    constants::*,
    models::user::User,
    utils::{generate_otp, get_epoch_ts},
};

use crate::database::AppDatabase;

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

// Generate and send otp
pub async fn generate_send_otp(user_id: u32, db: &Arc<AppDatabase>) -> anyhow::Result<()> {
    let f = Some(doc! {"id": user_id});
    let user = db
        .find_one::<User>(DB_NAME, COLL_USERS, f, None)
        .await?
        .ok_or(anyhow::anyhow!("User not found with id: {user_id}"))?;
    let Some(phone) = &user.phone else {
            let err = anyhow::anyhow!("User phone not found");
            return Err(err);
        };
    let otp = generate_otp(OTP_LENGTH);
    let otp = Otp::new(user_id, otp.as_str());
    db.insert_one::<Otp>(DB_NAME, COLL_OTP, &otp, None).await?;
    send_otp(phone, &otp.otp);
    Ok(())
}

// send otp to a given phone. SMS gateway API or SMS queue API to be called from here
pub fn send_otp(phone: &str, otp: &str) {
    tracing::debug!("Send otp {otp} to phone {phone}");
}
