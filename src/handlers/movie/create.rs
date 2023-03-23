use axum::{extract::State, Json};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::{Validate, ValidationError};

use super::model::{Movie, MovieRespData};
use crate::{
    constants::*,
    jwt::JwtClaims,
    utils::{get_epoch_ts, AppError, ValidatedBody},
};

#[cfg(test)]
use mockall_double::double;

#[cfg_attr(test, double)]
use crate::database::AppDatabase;

fn validate_tags(tags: &Vec<String>) -> Result<(), ValidationError> {
    if tags.iter().any(|tag| tag.is_empty()) {
        let mut err = ValidationError::new("tags");
        err.message = Some("empty tags are not allowed".into());
        return Err(err);
    }
    Ok(())
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateMovieReqBody {
    #[validate(length(min = 1, max = 100))]
    name: String,
    #[validate(length(min = 1))]
    description: String,
    #[validate(custom(function = "validate_tags"))]
    tags: Option<Vec<String>>,
    #[validate(url)]
    banner_image_url: String,
    #[validate(url)]
    video_url: String,
    #[validate(length(min = 1))]
    sponsored_by: String,
    #[validate(url)]
    sponsored_by_logo: Option<String>,
    release_date: u64,
    release_outlets: Option<Vec<String>>,
    movie_promotion_expiry: u64,
}

#[derive(Debug, Serialize)]
pub struct Response {
    success: bool,
    data: MovieRespData,
}

pub async fn create_movie_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<CreateMovieReqBody>,
) -> Result<Json<Response>, AppError> {
    check_duplicate_name(&db, &body.name).await?;
    let ts = get_epoch_ts();
    let movie = Movie {
        name: body.name,
        description: body.description,
        tags: body.tags,
        video_url: body.video_url,
        banner_image_url: body.banner_image_url,
        sponsored_by: Some(body.sponsored_by),
        sponsored_by_logo: body.sponsored_by_logo,
        release_date: Some(body.release_date),
        release_outlets: body.release_outlets,
        movie_promotion_expiry: Some(body.movie_promotion_expiry),
        is_active: true,
        likes: Some(vec![]),
        views: Some(vec![]),
        created_by: Some(claims.id),
        created_ts: Some(ts),
        updated_by: None,
        updated_ts: None,
    };
    let result = db
        .insert_one::<Movie>(DB_NAME, COLL_MOVIES, &movie, None)
        .await?;
    let res = Response {
        success: true,
        data: movie.to_movie_resp_data(&result),
    };
    Ok(Json(res))
}

async fn check_duplicate_name(db: &Arc<AppDatabase>, name: &str) -> Result<(), AppError> {
    let filter = doc! {"name": name};
    let movie = db
        .find_one::<Movie>(DB_NAME, COLL_MOVIES, Some(filter), None)
        .await?;
    if movie.is_some() {
        let err = AppError::BadRequestErr("Movie alread exists with same name".into());
        return Err(err);
    }
    Ok(())
}
