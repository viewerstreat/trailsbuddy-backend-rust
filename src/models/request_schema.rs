use chrono::{prelude::*, serde::ts_seconds};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::{
    constants::*,
    utils::{validate_future_timestamp, validate_phonenumber},
};

use super::LoginScheme;

/// request body schema for create user
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateUserReq {
    #[validate(length(min = 1, max = 50))]
    pub name: String,

    #[validate(custom(function = "validate_phonenumber"))]
    pub phone: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(email)]
    pub email: Option<String>,

    #[serde(rename = "profilePic")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(url)]
    pub profile_pic: Option<String>,
}

/// LoginScheme for Login with Facebook/Google
#[derive(Debug, Deserialize, ToSchema)]
pub enum SocialLoginScheme {
    GOOGLE,
    FACEBOOK,
}

impl From<SocialLoginScheme> for LoginScheme {
    fn from(value: SocialLoginScheme) -> Self {
        match value {
            SocialLoginScheme::GOOGLE => Self::GOOGLE,
            SocialLoginScheme::FACEBOOK => Self::FACEBOOK,
        }
    }
}

/// request body schema for login
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub login_scheme: SocialLoginScheme,
    pub id_token: Option<String>,
    pub fb_token: Option<String>,
}

/// request schema for verify user phone number
#[derive(Debug, Deserialize, Validate, IntoParams)]
pub struct VerifyUserReq {
    #[validate(custom(function = "validate_phonenumber"))]
    pub phone: String,
}

/// request schema for check otp with phone
#[derive(Debug, Deserialize, Validate, ToSchema, IntoParams)]
pub struct CheckOtpReq {
    #[validate(custom(function = "validate_phonenumber"))]
    pub phone: String,
    #[validate(length(equal = "OTP_LENGTH"))]
    pub otp: String,
}

/// request schema for update fcm token
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct FcmTokenReqBody {
    #[validate(length(min = 1))]
    pub token: String,
}

/// request schema for renew token
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RenewTokenReqBody {
    pub login_scheme: LoginScheme,
    pub id_token: Option<String>,
    pub fb_token: Option<String>,
    pub refresh_token: Option<String>,
}

lazy_static! {
    static ref UPPER_ALPHA_NUM: Regex = Regex::new(r"^[A-Z0-9]+$").unwrap();
}

/// request schema for referral code redeem request
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ReferralCodeReqBody {
    #[serde(rename = "referralCode")]
    #[validate(length(equal = "REFERRAL_CODE_LEN"))]
    #[validate(regex = "UPPER_ALPHA_NUM")]
    pub referral_code: String,
}

/// request schema to create special referral code
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SpecialCodeReqBody {
    #[validate(length(equal = "REFERRAL_CODE_LEN"))]
    #[validate(regex = "UPPER_ALPHA_NUM")]
    pub referral_code: String,
    #[validate(range(min = 1))]
    pub bonus: u64,
    #[serde(with = "ts_seconds")]
    #[validate(custom = "validate_future_timestamp")]
    pub valid_till: DateTime<Utc>,
}

type ToSchemaRetType<'__s> = (
    &'__s str,
    utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
);
impl<'__s> utoipa::ToSchema<'__s> for SpecialCodeReqBody {
    fn schema() -> ToSchemaRetType<'__s> {
        let string = utoipa::openapi::SchemaType::String;
        let integer = utoipa::openapi::SchemaType::Integer;
        let int64_format =
            utoipa::openapi::SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::Int64);
        let referral_code = utoipa::openapi::ObjectBuilder::new().schema_type(string);
        let bonus = utoipa::openapi::ObjectBuilder::new()
            .schema_type(integer.clone())
            .format(Some(int64_format.clone()));
        let valid_till = utoipa::openapi::ObjectBuilder::new()
            .schema_type(integer)
            .format(Some(int64_format));
        let example = json!({"referralCode":"DHAMAKA7","bonus":100, "validTill": 1689046909});
        (
            "SpecialCodeReqBody",
            utoipa::openapi::ObjectBuilder::new()
                .property("referralCode", referral_code)
                .required("referralCode")
                .property("bonus", bonus)
                .required("bonus")
                .property("validTill", valid_till)
                .required("validTill")
                .example(Some(example))
                .into(),
        )
    }
}

/// request schema to update user
#[derive(Debug, Default, Clone, Deserialize, Validate, ToSchema)]
pub struct UpdateUserReq {
    #[validate(length(min = 1, max = 50))]
    pub name: Option<String>,

    #[validate(custom(function = "validate_phonenumber"))]
    pub phone: Option<String>,

    #[validate(email)]
    pub email: Option<String>,

    #[serde(rename = "profilePic")]
    #[validate(url)]
    pub profile_pic: Option<String>,
}

/// signup request for admin user
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AdminSignupRequest {
    #[validate(custom(function = "validate_phonenumber"))]
    pub phone: String,
    #[validate(length(min = 1, max = 50))]
    pub name: String,
}

/// request schema for get clip
#[derive(Debug, Serialize, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct GetClipParams {
    #[serde(rename = "_id")]
    pub id: Option<String>,
    pub page_index: Option<u64>,
    pub page_size: Option<u64>,
}

/// request schema for clip create
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateClipReqBody {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1))]
    pub description: String,
    #[validate(url)]
    pub banner_image_url: String,
    #[validate(url)]
    pub video_url: String,
}

/// request schema to add view for clip
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClipAddViewReqBody {
    pub clip_id: String,
}
