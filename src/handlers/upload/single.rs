use axum::{extract::Multipart, Json};

use crate::{
    constants::*,
    models::*,
    utils::{get_object_url, AppError},
};

/// upload a file
#[utoipa::path(
    post,
    path = "/api/v1/upload/single",
    responses(
        (status = StatusCode::OK, description = "upload successful", body = FileUploadRes),
        (status = StatusCode::BAD_REQUEST, description = "Bad request", body = GenericResponse),
    ),
    tag = "App User API"
)]
pub async fn upload_handler(mut files: Multipart) -> Result<Json<FileUploadRes>, AppError> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_s3::Client::new(&config);
    let file = files
        .next_field()
        .await?
        .ok_or(AppError::BadRequestErr("no file".into()))?;
    let file_name = file
        .file_name()
        .ok_or(AppError::BadRequestErr("unable to read file name".into()))?
        .to_string();
    let data = file.bytes().await.map_err(|err| {
        tracing::debug!("{:?}", err);
        AppError::BadRequestErr("unable to read file content".into())
    })?;
    let key = super::uniq_file_name(&file_name);
    let resp = client
        .put_object()
        .bucket(AWS_BUCKET)
        .key(&key)
        .body(data.into())
        .send()
        .await?;
    tracing::debug!("{:?}", resp);
    let e_tag = resp
        .e_tag
        .ok_or(anyhow::anyhow!("unable to get ETag value"))?;
    let url = get_object_url(&key);
    let res = FileUploadRes {
        success: true,
        e_tag,
        url,
    };
    Ok(Json(res))
}
