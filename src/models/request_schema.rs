use chrono::{prelude::*, serde::ts_seconds};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::{
    constants::*,
    utils::{validate_future_timestamp, validate_phonenumber, validate_tags},
};

use super::{ContestCategory, ContestProps, LoginScheme, MediaType, QuestionReqBody};

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
#[derive(Debug, Serialize, Deserialize, ToSchema)]
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
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub login_scheme: SocialLoginScheme,
    pub id_token: Option<String>,
    pub fb_token: Option<String>,
}

/// request schema for verify user phone number
#[derive(Debug, Serialize, Deserialize, Validate, IntoParams)]
pub struct VerifyUserReq {
    #[validate(custom(function = "validate_phonenumber"))]
    pub phone: String,
}

/// request schema for check otp with phone
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema, IntoParams)]
pub struct CheckOtpReq {
    #[validate(custom(function = "validate_phonenumber"))]
    pub phone: String,
    #[validate(length(equal = "OTP_LENGTH"))]
    pub otp: String,
}

/// request schema for update fcm token
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct FcmTokenReqBody {
    #[validate(length(min = 1))]
    pub token: String,
}

/// request schema for renew token
#[derive(Debug, Serialize, Deserialize, ToSchema)]
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
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct ReferralCodeReqBody {
    #[serde(rename = "referralCode")]
    #[validate(length(equal = "REFERRAL_CODE_LEN"))]
    #[validate(regex = "UPPER_ALPHA_NUM")]
    pub referral_code: String,
}

/// request schema to create special referral code
#[derive(Debug, Serialize, Deserialize, Validate)]
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
#[derive(Debug, Default, Clone, Serialize, Deserialize, Validate, ToSchema)]
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
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
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
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
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
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClipAddViewReqBody {
    pub clip_id: String,
}

/// request schema for create movie
#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateMovieReqBody {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1))]
    pub description: String,
    #[validate(custom(function = "validate_tags"))]
    pub tags: Option<Vec<String>>,
    #[validate(url)]
    pub banner_image_url: String,
    #[validate(url)]
    pub video_url: String,
    #[validate(length(min = 1))]
    pub sponsored_by: String,
    #[validate(url)]
    pub sponsored_by_logo: Option<String>,
    #[serde(with = "ts_seconds")]
    pub release_date: DateTime<Utc>,
    pub release_outlets: Option<Vec<String>>,
    #[serde(with = "ts_seconds")]
    #[validate(custom = "validate_future_timestamp")]
    pub movie_promotion_expiry: DateTime<Utc>,
}

impl<'__s> utoipa::ToSchema<'__s> for CreateMovieReqBody {
    fn schema() -> ToSchemaRetType<'__s> {
        use utoipa::openapi::ObjectBuilder;
        let string = utoipa::openapi::SchemaType::String;
        let name = ObjectBuilder::new().schema_type(string.clone());
        let example = json!({
            "name":"string",
            "description": "string",
            "tags": ["string"],
            "bannerImageUrl": "string",
            "videoUrl": "string",
            "sponsoredBy": "string",
            "sponsoredByLogo": "string",
            "releaseDate": 1689219392,
            "releaseOutlets": ["string"],
            "moviePromotionExpiry": 1689219392
        });
        (
            "CreateMovieReqBody",
            utoipa::openapi::ObjectBuilder::new()
                .property("name", name)
                .required("name")
                .example(Some(example))
                .into(),
        )
    }
}

/// request schema for movie details
#[derive(Debug, Serialize, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct MovieDetailParams {
    pub movie_id: String,
}

/// request schema to add view for movie
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MovieAddViewReqBody {
    pub movie_id: String,
}

/// request schema to add favourite
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddFavReqBody {
    pub media_type: MediaType,
    pub media_id: String,
}

/// request params for get favourite list
#[derive(Debug, Serialize, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct GetFavParams {
    pub media_type: MediaType,
    pub page_index: Option<u64>,
    pub page_size: Option<u64>,
}

impl<'__s> utoipa::ToSchema<'__s> for ContestProps {
    fn schema() -> ToSchemaRetType<'__s> {
        use utoipa::openapi::ObjectBuilder;
        let string = utoipa::openapi::SchemaType::String;
        let title = ObjectBuilder::new().schema_type(string.clone());
        let example = json!({
            "title":"string",
            "category": "movie",
            "movieId": "string",
            "sponsoredBy": "string",
            "sponsoredByLogo": "string",
            "bannerImageUrl": "string",
            "videoUrl": "string",
            "entryFee": 0,
            "entryFeeMaxBonusMoney": 0,
            "prizeSelection": "TOP_WINNERS",
            "topWinnersCount": 0,
            "prizeRatioNumerator": 0,
            "prizeRatioDenominator": 0,
            "prizeValueRealMoney": 0,
            "prizeValueBonusMoney": 0,
            "startTime": 1689219392,
            "endTime": 1689219392,
            "minRequiredPlayers": 0
        });
        (
            "ContestProps",
            utoipa::openapi::ObjectBuilder::new()
                .property("title", title)
                .required("title")
                .example(Some(example))
                .into(),
        )
    }
}

/// request params for get contest
#[derive(Debug, Serialize, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct GetContestParams {
    #[serde(rename = "_id")]
    pub _id: Option<String>,
    pub movie_id: Option<String>,
    pub category: Option<ContestCategory>,
    pub page_size: Option<u64>,
    pub page_index: Option<u64>,
}

/// request schema for contest activate
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct ContestIdRequest {
    #[validate(length(min = 1))]
    pub contest_id: String,
}

/// request schema for create question
#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateQuesReqBody {
    #[validate(length(min = 1))]
    pub contest_id: String,
    #[serde(flatten)]
    #[validate]
    pub question: QuestionReqBody,
}

/// request schema for question delete
#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuesDelReqBody {
    #[validate(length(min = 1))]
    pub contest_id: String,
    #[validate(range(min = 1))]
    pub question_no: u32,
}

/// request params for get notifications
#[derive(Debug, Serialize, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct GetNotiReq {
    pub page_index: Option<u64>,
    pub page_size: Option<u64>,
}

/// request schema for clear/mark read notification
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct ClearNotiReq {
    #[validate(length(equal = 24))]
    pub _id: String,
}

/// request schema for Add Balanace Init request
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct AddBalInitReq {
    #[validate(range(min = 1))]
    pub amount: u64,
}

/// request schema for Add Balance Init request
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddBalEndReq {
    #[validate(range(min = 1))]
    pub amount: u64,
    pub transaction_id: String,
    pub is_successful: bool,
    pub error_reason: Option<String>,
    pub tracking_id: Option<String>,
}

/// request schema for Withdraw balance init request
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawBalInitReq {
    #[validate(range(min = "WITHDRAW_BAL_MIN_AMOUNT"))]
    pub amount: u64,
    #[validate(email)]
    pub receiver_upi_id: String,
}

/// request schema for Withdraw balance finalize
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawBalEndReq {
    #[validate(range(min = "WITHDRAW_BAL_MIN_AMOUNT"))]
    pub amount: u64,
    pub transaction_id: String,
    pub is_successful: bool,
    pub error_reason: Option<String>,
    pub tracking_id: Option<String>,
}

/// request schema for pay contest request
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PayContestReqBody {
    pub contest_id: String,
    pub bonus_money_amount: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnswerPlayTrackerReqBody {
    #[validate(length(min = 1))]
    pub contest_id: String,
    #[validate(range(min = 1))]
    pub question_no: u32,
    #[validate(range(min = 1, max = 4))]
    pub selected_option_id: u32,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateBroadcastReq {
    #[validate(length(min = 1))]
    pub message: String,
}

/// request schema for multipart upload initiate
#[derive(Debug, Deserialize, Validate, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct MultipartUploadInitiateReq {
    #[validate(length(min = 1))]
    pub file_name: String,
}

/// request schema for multipart upload part
#[derive(Debug, Deserialize, Validate, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct MultipartUploadPartReq {
    #[validate(length(min = 1))]
    pub key: String,
    #[validate(length(min = 1))]
    pub upload_id: String,
    #[validate(range(min = 1))]
    pub part_number: i32,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UploadPartCompleted {
    #[validate(length(min = 1))]
    pub e_tag: String,
    #[validate(range(min = 1))]
    pub part_number: i32,
    #[validate(range(min = 1))]
    pub size: usize,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CompleteMultipartUploadReq {
    #[validate(length(min = 1))]
    pub upload_id: String,
    #[validate(length(min = 1))]
    pub key: String,
    #[validate]
    pub completed_parts: Vec<UploadPartCompleted>,
}
