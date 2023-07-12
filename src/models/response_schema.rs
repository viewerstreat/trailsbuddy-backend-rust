use mongodb::bson::Document;
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

use super::{AdminUser, ClipRespData, LeaderboardData, User};

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
