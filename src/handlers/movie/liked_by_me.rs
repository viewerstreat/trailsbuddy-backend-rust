use axum::{
    extract::{Query, State},
    Json,
};
use mongodb::bson::doc;
use std::sync::Arc;

use crate::{
    constants::*,
    database::AppDatabase,
    jwt::JwtClaims,
    models::*,
    utils::{parse_object_id, AppError},
};

/// Get movie liked by me
///
/// Get if the movie is liked by the logged in user
#[utoipa::path(
    get,
    path = "/api/v1/movie/isLikedByMe",
    params(MovieDetailParams, ("authorization" = String, Header, description = "JWT token")),
    security(("authorization" = [])),
    responses(
        (status = StatusCode::OK, description = "Liked by me", body = MovieLikedResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request", body = GenericResponse),
    ),
    tag = "App User API"
)]
pub async fn is_liked_by_me_handler(
    claims: JwtClaims,
    State(db): State<Arc<AppDatabase>>,
    Query(params): Query<MovieDetailParams>,
) -> Result<Json<MovieLikedResponse>, AppError> {
    let oid = parse_object_id(&params.movie_id, "invalid movieId")?;
    let filter = Some(doc! {"_id": oid});
    let movie = db
        .find_one::<Movie>(DB_NAME, COLL_MOVIES, filter, None)
        .await?
        .ok_or(AppError::NotFound("movie not found".into()))?;
    let is_liked_by_me = movie
        .props
        .likes
        .and_then(|likes| {
            let is_liked = likes
                .iter()
                .any(|like| like.user_id == claims.id && like.is_removed == false);
            Some(is_liked)
        })
        .unwrap_or(false);
    let res = MovieLikedResponse {
        success: true,
        is_liked_by_me,
    };
    Ok(Json(res))
}
