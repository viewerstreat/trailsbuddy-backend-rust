use mockall_double::double;
use mongodb::bson::doc;
use serde::Serialize;
use std::sync::Arc;

use crate::{
    constants::*,
    utils::{generate_otp, get_epoch_ts},
};
use otp_inner::generate_send_otp;

#[double]
use crate::database::AppDatabase;

use super::model::User;

#[derive(Debug, Serialize)]
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

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub mod otp_inner {
    use super::*;

    // Generate and send otp
    pub async fn generate_send_otp(user_id: u32, db: &Arc<AppDatabase>) -> anyhow::Result<()> {
        let f = Some(doc! {"id": user_id});
        let user = db
            .find_one::<User>(DB_NAME, COLL_USERS, f, None)
            .await?
            .ok_or(anyhow::anyhow!("User not found with id: {user_id}"))?;
        let otp = generate_otp(OTP_LENGTH);
        let otp = Otp::new(user_id, otp.as_str());
        db.insert_one::<Otp>(DB_NAME, COLL_OTP, &otp, None).await?;
        send_otp(&user.phone, &otp.otp);
        Ok(())
    }

    // send otp to a given phone. SMS gateway API or SMS queue API to be called from here
    pub fn send_otp(phone: &str, otp: &str) {
        tracing::debug!("Send otp {otp} to phone {phone}");
    }
}

#[cfg(test)]
mod otp_tests {

    use mockall::predicate::{eq, function};
    use mongodb::{
        bson::doc,
        options::{FindOneOptions, InsertOneOptions},
    };

    use crate::handlers::user::model::User;

    use super::*;

    #[test]
    fn test_new() {
        let user_id = 1;
        let otp_val = "887797";
        let otp = Otp::new(user_id, otp_val);
        assert_eq!(otp.user_id, user_id);
        assert_eq!(otp.otp, otp_val);
        assert_eq!(otp.valid_till, otp.update_ts + OTP_VALIDITY_MINS * 60);
        assert_eq!(otp.is_used, false);
    }

    #[tokio::test]
    async fn test_generate_send_otp() {
        let user_id = 1;
        let f = Some(doc! {"id": user_id});
        let check_none = function(|options: &Option<FindOneOptions>| options.is_none());
        let check_none_ins = function(|options: &Option<InsertOneOptions>| options.is_none());
        let check_otp = function(move |otp: &Otp| {
            otp.user_id == user_id
                && otp.is_used == false
                && otp.otp.len() == OTP_LENGTH as usize
                && otp.valid_till > get_epoch_ts()
        });
        let mut mock_db = AppDatabase::default();
        mock_db
            .expect_find_one::<User>()
            .with(eq(DB_NAME), eq(COLL_USERS), eq(f), check_none)
            .times(1)
            .returning(|_, _, _, _| Ok(Some(User::default())));

        mock_db
            .expect_insert_one::<Otp>()
            .with(eq(DB_NAME), eq(COLL_OTP), check_otp, check_none_ins)
            .times(1)
            .returning(|_, _, _, _| Ok(String::new()));
        let db = Arc::new(mock_db);
        let result = generate_send_otp(user_id, &db).await;
        assert_eq!(result.is_ok(), true);
    }
}
