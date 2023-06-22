use axum::{extract::State, Json};
use chrono::{prelude::*, serde::ts_seconds};
use futures::FutureExt;
use lazy_static::lazy_static;
use mongodb::{
    bson::doc,
    options::{FindOneAndUpdateOptions, ReturnDocument},
    ClientSession,
};
use regex::Regex;
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use validator::Validate;

use super::otp::get_user_by_id;
use crate::{
    constants::*,
    database::AppDatabase,
    jobs::finalize_contest::credit_prize::get_user_balance,
    jwt::JwtClaims,
    models::{
        user::{SpecialReferralCode, User},
        wallet::{Money, Wallet, WalletTransaction},
    },
    utils::{get_epoch_ts, AppError, ValidatedBody},
};

lazy_static! {
    static ref UPPER_ALPHA_NUM: Regex = Regex::new(r"^[A-Z0-9]+$").unwrap();
}

#[derive(Debug, Deserialize, Validate)]
pub struct ReqBody {
    #[serde(rename = "referralCode")]
    #[validate(length(equal = "REFERRAL_CODE_LEN"))]
    #[validate(regex = "UPPER_ALPHA_NUM")]
    referral_code: String,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SpecialCodeReqBody {
    #[validate(length(equal = "REFERRAL_CODE_LEN"))]
    #[validate(regex = "UPPER_ALPHA_NUM")]
    referral_code: String,
    #[validate(range(min = 1))]
    bonus: u64,
    #[serde(with = "ts_seconds")]
    valid_till: DateTime<Utc>,
}

/// Update fcmToken for an user
pub async fn use_referral_code_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<ReqBody>,
) -> Result<Json<JsonValue>, AppError> {
    let user = get_user_by_id(claims.id, &db)
        .await?
        .ok_or(AppError::NotFound("user not found".into()))?;
    // check if user has already used referral code
    if let Some(val) = user.has_used_referral_code {
        if val {
            let err = "User has already used referral";
            let err = AppError::BadRequestErr(err.into());
            return Err(err);
        }
    }
    // check for special referral code
    let curr_ts = get_epoch_ts() as i64;
    let filter = doc! {"referralCode": &body.referral_code, "isActive": true, "validTill": {"$gte": curr_ts}};
    let filter = Some(filter);
    let special_referral = db
        .find_one::<SpecialReferralCode>(DB_NAME, COLL_SPECIAL_REFERRAL_CODES, filter, None)
        .await?;
    if let Some(special_referral) = special_referral {
        let bonus = special_referral.bonus();
        add_special_referral_bonus(&db, claims.id, bonus, &body.referral_code).await?;
    } else {
        // check if valid referral code
        let filter = doc! {
            "referralCode": &body.referral_code,
            "isActive": true,
            "id": {"$ne": claims.id}
        };
        let filter = Some(filter);
        let referrer = db
            .find_one::<User>(DB_NAME, COLL_USERS, filter, None)
            .await?
            .ok_or(AppError::NotFound("Invalid referralCode".into()))?;
        add_referral_bonus(&db, claims.id, referrer.id, &body.referral_code).await?;
    }
    let res = json!({"success": true, "message": "referral code used successfully!!"});
    Ok(Json(res))
}

/// Create special referral codes
pub async fn create_special_code_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<SpecialCodeReqBody>,
) -> Result<Json<JsonValue>, AppError> {
    let curr_ts = get_epoch_ts() as i64;
    if body.valid_till.timestamp() <= curr_ts {
        let err = "validTill must be future date";
        let err = AppError::BadRequestErr(err.into());
        return Err(err);
    }
    let filter = Some(doc! {"referralCode": &body.referral_code});
    let data = db
        .find_one::<SpecialReferralCode>(DB_NAME, COLL_SPECIAL_REFERRAL_CODES, filter.clone(), None)
        .await?;
    if data.is_some() {
        let err = "Special referral code already exists";
        let err = AppError::BadRequestErr(err.into());
        return Err(err);
    }
    let data = db
        .find_one::<User>(DB_NAME, COLL_USERS, filter, None)
        .await?;
    if data.is_some() {
        let err = "Referral code already exists in users";
        let err = AppError::BadRequestErr(err.into());
        return Err(err);
    }
    let special_referral_code =
        SpecialReferralCode::new(&body.referral_code, body.bonus, &body.valid_till, claims.id);
    db.insert_one::<SpecialReferralCode>(
        DB_NAME,
        COLL_SPECIAL_REFERRAL_CODES,
        &special_referral_code,
        None,
    )
    .await?;
    let res = json!({"success": true, "message": "referral code saved"});
    Ok(Json(res))
}

async fn add_special_referral_bonus(
    db: &Arc<AppDatabase>,
    user_id: u32,
    bonus: u64,
    referral_code: &str,
) -> anyhow::Result<()> {
    tracing::debug!("adding special referral bonus for user: {}", user_id);
    db.execute_transaction(None, None, |db, session| {
        let referral_code = referral_code.to_owned();
        async move {
            update_users(db, session, user_id, &referral_code, 0).await?;
            credit_referral_bonus(db, session, user_id, bonus).await?;
            Ok(())
        }
        .boxed()
    })
    .await?;
    Ok(())
}

async fn add_referral_bonus(
    db: &Arc<AppDatabase>,
    user_id: u32,
    referred_id: u32,
    referral_code: &str,
) -> anyhow::Result<()> {
    tracing::debug!(
        "adding referral bonus for user: {}, referrer: {}",
        user_id,
        referred_id
    );
    db.execute_transaction(None, None, |db, session| {
        let referral_code = referral_code.to_owned();
        async move {
            update_users(db, session, user_id, &referral_code, referred_id).await?;
            credit_referral_bonus(db, session, user_id, REFERRAL_BONUS).await?;
            credit_referrer_bonus(db, session, referred_id, user_id).await?;
            Ok(())
        }
        .boxed()
    })
    .await?;
    Ok(())
}

async fn update_users(
    db: &AppDatabase,
    session: &mut ClientSession,
    user_id: u32,
    referral_code: &str,
    referred_id: u32,
) -> anyhow::Result<()> {
    let ts = get_epoch_ts() as i64;
    let filter = doc! {"id": user_id};
    let update = doc! {"$set": {"hasUsedReferralCode": true, "usedReferralCode": referral_code, "referred_by": referred_id, "updatedTs": ts}};
    db.update_one_with_session(session, DB_NAME, COLL_USERS, filter, update, None)
        .await?;
    Ok(())
}

pub async fn credit_referral_bonus(
    db: &AppDatabase,
    session: &mut ClientSession,
    user_id: u32,
    bonus: u64,
) -> anyhow::Result<()> {
    let (balance_before, balance_after) = update_wallet(db, session, user_id, bonus).await?;
    let transaction =
        WalletTransaction::referral_bonus_trans(user_id, bonus, balance_before, balance_after);
    db.insert_one_with_session(
        session,
        DB_NAME,
        COLL_WALLET_TRANSACTIONS,
        &transaction,
        None,
    )
    .await?;
    Ok(())
}

pub async fn credit_referrer_bonus(
    db: &AppDatabase,
    session: &mut ClientSession,
    referrer_id: u32,
    user_id: u32,
) -> anyhow::Result<()> {
    let (balance_before, balance_after) =
        update_wallet(db, session, referrer_id, REFERRER_BONUS).await?;
    let transaction = WalletTransaction::referrer_bonus_trans(
        referrer_id,
        balance_before,
        balance_after,
        user_id,
    );
    db.insert_one_with_session(
        session,
        DB_NAME,
        COLL_WALLET_TRANSACTIONS,
        &transaction,
        None,
    )
    .await?;
    Ok(())
}

async fn update_wallet(
    db: &AppDatabase,
    session: &mut ClientSession,
    user_id: u32,
    bonus: u64,
) -> anyhow::Result<(Money, Money)> {
    let balance_before = get_user_balance(db, session, user_id).await?;
    let filter = doc! {"userId": user_id};
    let ts = get_epoch_ts() as i64;
    let update = doc! {
        "$inc": { "balance.bonus": bonus as i64},
        "$set": {"updatedTs": ts}
    };
    let options = FindOneAndUpdateOptions::builder()
        .upsert(Some(true))
        .return_document(Some(ReturnDocument::After))
        .build();
    let wallet = db
        .find_one_and_update_with_session::<Wallet>(
            session,
            DB_NAME,
            COLL_WALLETS,
            filter,
            update,
            Some(options),
        )
        .await?
        .ok_or(anyhow::anyhow!("not able to update wallet"))?;
    let balance_after = wallet.balance();
    if balance_after != balance_before + Money::new(0, bonus) {
        let err = anyhow::anyhow!(
            "balance_before {:?} and balance_after {:?} not matching",
            balance_before,
            balance_after
        );
        return Err(err);
    }
    Ok((balance_before, balance_after))
}
