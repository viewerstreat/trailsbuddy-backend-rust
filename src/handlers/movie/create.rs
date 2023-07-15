use axum::{extract::State, Json};
use mongodb::bson::doc;
use std::sync::Arc;

use crate::{
    constants::*,
    database::AppDatabase,
    jwt::JwtClaimsAdmin,
    models::*,
    utils::{get_epoch_ts, AppError, ValidatedBody},
};

/// Movie create
///
/// Create a new movie
#[utoipa::path(
    post,
    path = "/api/v1/movie",
    params(("authorization" = String, Header, description = "Admin JWT token")),
    security(("authorization" = [])),
    request_body = CreateMovieReqBody,
    responses(
        (status = StatusCode::OK, description = "Movie created", body = MovieResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request", body = GenericResponse)
    ),
    tag = "Admin API"
)]
pub async fn create_movie_handler(
    claims: JwtClaimsAdmin,
    State(db): State<Arc<AppDatabase>>,
    ValidatedBody(body): ValidatedBody<CreateMovieReqBody>,
) -> Result<Json<MovieResponse>, AppError> {
    let claims = claims.data;
    check_duplicate_name(&db, &body.name).await?;
    let ts = get_epoch_ts();
    let movie_props = MovieProps {
        name: body.name,
        description: body.description,
        tags: body.tags,
        video_url: body.video_url,
        banner_image_url: body.banner_image_url,
        sponsored_by: Some(body.sponsored_by),
        sponsored_by_logo: body.sponsored_by_logo,
        release_date: Some(body.release_date.timestamp() as u64),
        release_outlets: body.release_outlets,
        movie_promotion_expiry: Some(body.movie_promotion_expiry.timestamp() as u64),
        is_active: true,
        likes: Some(vec![]),
        views: Some(vec![]),
    };
    let movie = Movie {
        props: movie_props,
        created_by: Some(claims.id),
        created_ts: Some(ts),
        updated_by: None,
        updated_ts: None,
    };
    let result = db
        .insert_one::<Movie>(DB_NAME, COLL_MOVIES, &movie, None)
        .await?;
    let res = MovieResponse {
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
