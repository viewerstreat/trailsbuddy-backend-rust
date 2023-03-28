use axum::{extract::State, Json};
use mongodb::{bson::doc, options::FindOptions};
use serde::Serialize;
use std::sync::Arc;

use super::model::User;
use crate::{constants::*, handlers::wallet::model::Money, utils::AppError};

#[cfg(test)]
use mockall_double::double;

#[cfg_attr(test, double)]
use crate::database::AppDatabase;

#[derive(Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct LeaderboardData {
    id: u32,
    name: String,
    total_played: u32,
    contest_won: u32,
    total_earning: Money,
}

#[derive(Debug, Serialize)]
pub struct Response {
    success: bool,
    data: Vec<LeaderboardData>,
}

pub async fn get_leaderboard_handler(
    State(db): State<Arc<AppDatabase>>,
) -> Result<Json<Response>, AppError> {
    let filter = Some(doc! {"isActive": true, "totalPlayed": {"$gt": 0}});
    let sort = doc! {"totalEarning": -1, "contestWon": -1, "totalPlayed": -1, "id": 1};
    let mut options = FindOptions::default();
    options.sort = Some(sort);
    let options = Some(options);
    let result = db
        .find::<User>(DB_NAME, COLL_USERS, filter, options)
        .await?;
    let data = result
        .into_iter()
        .map(|user| LeaderboardData {
            id: user.id,
            name: user.name,
            total_played: user.total_played.unwrap_or_default(),
            contest_won: user.contest_won.unwrap_or_default(),
            total_earning: user.total_earning.unwrap_or_default(),
        })
        .collect::<Vec<_>>();
    let res = Response {
        success: true,
        data,
    };
    Ok(Json(res))
}
