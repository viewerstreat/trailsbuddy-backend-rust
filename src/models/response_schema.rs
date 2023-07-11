use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{AdminUser, LeaderboardData, User};

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
