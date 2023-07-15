use mongodb::bson::Document;
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

use super::{
    AdminUser, ClipRespData, Contest, LeaderboardData, Money, MovieDetails, MovieRespData,
    Notifications, PlayTracker, Question, QuestionWithoutCorrectFlag, User,
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenericResponse {
    pub success: bool,
    pub message: String,
}

/// response schema for user login
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub success: bool,
    pub data: User,
    pub token: String,
    pub refresh_token: Option<String>,
}

/// response schema for get leaderboard data
#[derive(Debug, Serialize, ToSchema)]
pub struct LeaderboardResponse {
    pub success: bool,
    pub data: Vec<LeaderboardData>,
}

/// response schema for update user
#[derive(Debug, Default, Serialize, ToSchema)]
pub struct UpdateUserResponse {
    pub success: bool,
    pub data: User,
}

/// response schema for Admin Login
#[derive(Debug, Serialize, ToSchema)]
pub struct AdminLoginResponse {
    pub success: bool,
    pub data: AdminUser,
    pub token: String,
}

/// response schema for get clip
#[derive(Debug, Serialize, ToSchema)]
pub struct GetClipResponse {
    pub success: bool,
    pub data: Vec<WrapDocument>,
}

#[derive(Debug, Serialize)]
pub struct WrapDocument(Document);

type ToSchemaRetType<'__s> = (
    &'__s str,
    utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
);
impl<'__s> utoipa::ToSchema<'__s> for WrapDocument {
    fn schema() -> ToSchemaRetType<'__s> {
        let string = utoipa::openapi::SchemaType::String;
        let id = utoipa::openapi::ObjectBuilder::new().schema_type(string);
        let example = json!({"_id":"6498ffd4779369ff1bec4d5c", "...": "..."});
        (
            "WrapDocument",
            utoipa::openapi::ObjectBuilder::new()
                .property("_id", id)
                .example(Some(example))
                .into(),
        )
    }
}

impl From<Document> for WrapDocument {
    fn from(value: Document) -> Self {
        Self(value)
    }
}

/// response schema for create clip
#[derive(Debug, Serialize, ToSchema)]
pub struct ClipResponse {
    pub success: bool,
    pub data: ClipRespData,
}

/// response schema for add view
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddViewResponse {
    pub success: bool,
    pub message: String,
    pub view_count: u32,
}

/// response schema for create movie
#[derive(Debug, Serialize, ToSchema)]
pub struct MovieResponse {
    pub success: bool,
    pub data: MovieRespData,
}

/// response schema for movie details response
#[derive(Debug, Serialize, ToSchema)]
pub struct MovieDetailResponse {
    pub success: bool,
    pub data: MovieDetails,
}

/// response schema for movie liked by me
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MovieLikedResponse {
    pub success: bool,
    pub is_liked_by_me: bool,
}

/// response schema for create contest
#[derive(Debug, Serialize, ToSchema)]
pub struct ContestResponse {
    pub success: bool,
    pub data: Contest,
}

/// response schema for get contest
#[derive(Debug, Serialize, ToSchema)]
pub struct GetContestResponse {
    pub success: bool,
    pub data: Vec<Contest>,
}

/// response schema for get question
#[derive(Debug, Serialize, ToSchema)]
pub struct GetQuestionResponse {
    pub success: bool,
    pub data: Option<Vec<Question>>,
}

/// response schema for get notification
#[derive(Debug, Serialize, ToSchema)]
pub struct GetNotiResp {
    pub success: bool,
    pub data: Vec<Notifications>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GetBalResponse {
    success: bool,
    balance: Money,
}
impl GetBalResponse {
    pub fn new(balance: Money) -> Self {
        Self {
            success: true,
            balance,
        }
    }
}

/// response schema for Add Balance Init
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddBalInitRes {
    pub success: bool,
    pub transaction_id: String,
    pub app_upi_id: String,
}

/// response schema for withdraw init
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawInitRes {
    pub success: bool,
    pub transaction_id: String,
}

/// response schema for PlayTracker response
#[derive(Debug, Serialize, ToSchema)]
pub struct PlayTrackerResponse {
    pub success: bool,
    pub data: PlayTracker,
}

/// response schema for play tracker start
#[derive(Debug, Serialize, ToSchema)]
pub struct PlayTrackerQuesRes {
    pub success: bool,
    pub data: PlayTracker,
    pub question: Option<QuestionWithoutCorrectFlag>,
}
