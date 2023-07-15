use axum::{extract::State, Json};
use mongodb::{bson::doc, options::FindOptions};
use std::sync::Arc;

use crate::{constants::*, database::AppDatabase, models::*, utils::AppError};

/// Get Leaderboard data
#[utoipa::path(
    get,
    path = "/api/v1/user/getLeaderboard",
    responses(
        (status = StatusCode::OK, description = "Leaderboard data list", body = LeaderboardResponse)
    ),
    tag = "App User API"
)]
pub async fn get_leaderboard_handler(
    State(db): State<Arc<AppDatabase>>,
) -> Result<Json<LeaderboardResponse>, AppError> {
    let filter = Some(doc! {"isActive": true, "totalPlayed": {"$gt": 0}});
    let sort = doc! {
        "totalEarning.real": -1,
        "totalEarning.bonus": -1,
        "contestWon": -1,
        "totalPlayed": -1,
        "id": 1
    };
    let mut options = FindOptions::default();
    options.sort = Some(sort);
    let options = Some(options);
    let result = db
        .find::<User>(DB_NAME, COLL_USERS, filter, options)
        .await?;
    let data = result
        .into_iter()
        .map(|user| {
            LeaderboardData::new(
                user.id,
                user.name,
                user.total_played.unwrap_or_default(),
                user.contest_won.unwrap_or_default(),
                user.total_earning.unwrap_or_default(),
            )
        })
        .collect::<Vec<_>>();
    let res = LeaderboardResponse {
        success: true,
        data,
    };
    Ok(Json(res))
}
