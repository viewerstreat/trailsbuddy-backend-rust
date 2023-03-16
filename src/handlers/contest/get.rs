use axum::{
    extract::{Query, State},
    Json,
};
use mongodb::{
    bson::{doc, Document},
    options::FindOptions,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::create::{Contest, ContestCategory, ContestStatus};
use crate::{
    constants::*,
    utils::{parse_object_id, AppError},
};

#[cfg(test)]
use mockall_double::double;

#[cfg_attr(test, double)]
use crate::database::AppDatabase;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Params {
    #[serde(rename = "_id")]
    _id: Option<String>,
    movie_id: Option<String>,
    category: Option<ContestCategory>,
    page_size: Option<u64>,
    page_index: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct Response {
    success: bool,
    data: Vec<Contest>,
}

pub async fn get_contest_handler(
    State(db): State<Arc<AppDatabase>>,
    params: Query<Params>,
) -> Result<Json<Response>, AppError> {
    let find_by = get_query(&params)?;
    let options = get_find_options(&params);
    let data = get_result(&db, find_by, options).await?;
    let res = Response {
        success: true,
        data,
    };
    Ok(Json(res))
}

fn get_query(params: &Params) -> Result<Document, AppError> {
    let mut find_by = doc! {};
    if let Some(id) = &params._id {
        let id = parse_object_id(id, "Not able to parse _id")?;
        find_by.insert("_id", id);
    }
    if let Some(movie_id) = &params.movie_id {
        find_by.insert("movieId", movie_id);
        find_by.insert("status", ContestStatus::ACTIVE.to_bson()?);
    }
    if let Some(category) = &params.category {
        find_by.insert("category", category.to_bson()?);
        find_by.insert("status", ContestStatus::ACTIVE.to_bson()?);
    }

    Ok(find_by)
}

fn get_find_options(params: &Params) -> FindOptions {
    let page_index = params.page_index.unwrap_or(0);
    let page_size = params.page_size.unwrap_or(DEFAULT_QUERY_LIMIT);
    let skip = if params._id.is_none() {
        page_index * page_size
    } else {
        0
    };
    let sort = doc! {"_id": -1};
    FindOptions::builder()
        .sort(Some(sort))
        .skip(Some(skip))
        .limit(Some(page_size as i64))
        .build()
}

async fn get_result(
    db: &Arc<AppDatabase>,
    find_by: Document,
    options: FindOptions,
) -> Result<Vec<Contest>, AppError> {
    let data = db
        .find::<Contest>(DB_NAME, COLL_CONTESTS, Some(find_by), Some(options))
        .await
        .map_err(|err| {
            tracing::debug!("{:?}", err);
            anyhow::anyhow!("Not able to query from database")
        })?;
    Ok(data)
}
