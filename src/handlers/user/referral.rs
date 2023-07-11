use axum::{extract::State, Json};
use futures::FutureExt;
use mongodb::{bson::doc, ClientSession};
use std::sync::Arc;

use super::otp::get_user_by_id;
use crate::{
    constants::*,
    database::AppDatabase,
    handlers::*,
    jwt::{JwtClaims, JwtClaimsAdmin},
    models::*,
    utils::{get_epoch_ts, AppError, ValidatedBody},
};

/// Redeem a referral code
///
/// User reedem a referral code after signup
#[utoipa::path(
    post,
    path = "/api/v1/user/useReferralCode",
    params(("authorization" = String, Header, description = "JWT token")),
    security(("authorization" = [])),
    request_body = ReferralCodeReqBody,
    responses(
        (status = StatusCode::OK, description = "User successfully redeems referral code", body = GenericResponse),
        (status = StatusCode::NOT_FOUND, description = "User/referral code not found", body = GenericResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request", body = GenericResponse),
        (status = StatusCode::UNAUTHORIZED, description = "Invalid token", body = GenericResponse)
    ),
    tag = "App User API"
)]
pub async fn use_referral_code_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<ReferralCodeReqBody>,
) -> Result<Json<GenericResponse>, AppError> {
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
    let res = GenericResponse {
        success: true,
        message: "referral code used successfully!!".to_owned(),
    };
    Ok(Json(res))
}

/// Create special referral code
///
/// Admin user creates a special referral code
#[utoipa::path(
    post,
    path = "/api/v1/admin/createSpecialReferralCode",
    params(("authorization" = String, Header, description = "Admin JWT token")),
    request_body = SpecialCodeReqBody,
    responses(
        (status = StatusCode::OK, description = "Referral code saved", body = GenericResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request", body = GenericResponse),
        (status = StatusCode::UNAUTHORIZED, description = "Invalid token", body = GenericResponse)
    ),
    security(
        ("authorization" = [])
    ),
    tag = "Admin API"
)]
pub async fn create_special_code_handler(
    claims: JwtClaimsAdmin,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<SpecialCodeReqBody>,
) -> Result<Json<GenericResponse>, AppError> {
    let claims = claims.data;
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
    let res = GenericResponse {
        success: true,
        message: "referral code saved".to_owned(),
    };
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
    let (balance_before, balance_after) =
        update_wallet_with_session(db, session, user_id, 0, bonus, false, false).await?;
    let transaction =
        WalletTransaction::referral_bonus_trans(user_id, bonus, balance_before, balance_after);
    insert_wallet_transaction_session(db, session, &transaction).await?;
    Ok(())
}

pub async fn credit_referrer_bonus(
    db: &AppDatabase,
    session: &mut ClientSession,
    referrer_id: u32,
    user_id: u32,
) -> anyhow::Result<()> {
    let (balance_before, balance_after) =
        update_wallet_with_session(db, session, referrer_id, 0, REFERRER_BONUS, false, false)
            .await?;
    let transaction = WalletTransaction::referrer_bonus_trans(
        referrer_id,
        balance_before,
        balance_after,
        user_id,
    );
    insert_wallet_transaction_session(db, session, &transaction).await?;
    Ok(())
}
