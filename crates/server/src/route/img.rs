use axum::{extract::State, response::IntoResponse};
use tracing::info;

use crate::error::Error;
use crate::{config::Config, render::Payload};

pub(crate) async fn image_files(State(config): State<Config>) -> Result<impl IntoResponse, Error> {
    let meta = tokio::fs::metadata(config.base_dir.as_str())
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    if !meta.is_dir() {
        return Err(Error::NotDir);
    }

    let mut entities = tokio::fs::read_dir(config.base_dir.as_str())
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let mut files = Vec::new();
    while let Ok(entity) = entities.next_entry().await {
        if let Some(entity) = entity {
            if entity.file_type().await.unwrap().is_file() {
                files.push(entity.file_name().into_string().unwrap());
            }
        } else {
            break;
        }
    }

    Ok(Payload::success(files))
}
