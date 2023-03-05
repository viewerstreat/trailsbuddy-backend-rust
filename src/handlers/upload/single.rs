use axum::{extract::Multipart, Json};
use serde_json::{json, Value as JsonValue};

use crate::{
    constants::*,
    utils::{get_epoch_ts, get_object_url, get_random_num, AppError},
};

fn uniq_file_name(file_name: &str) -> String {
    let ts = get_epoch_ts();
    let random = get_random_num(101, 999);
    let (name, ext) = file_name.rsplit_once('.').unwrap_or((file_name, "unknown"));
    let name = name.split_whitespace().collect::<Vec<_>>().join("_");
    format!("{name}_{ts}_{random}.{ext}")
}

pub async fn upload_handler(mut files: Multipart) -> Result<Json<JsonValue>, AppError> {
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
    let key = uniq_file_name(&file_name);
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
    let res = json!({"success": true, "data": {"ETag": &e_tag, "url": &url}});
    Ok(Json(res))
}
