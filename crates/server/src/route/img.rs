use crate::error::Error;
use crate::{config::Config, render::Payload};
use anyhow::anyhow;
use axum::{extract::State, response::IntoResponse, Json};
use segment::base64_to_image;
use serde_json::json;
use std::path::PathBuf;
use tokio::{
    fs::OpenOptions,
    io::{AsyncWriteExt, BufWriter},
};
use tracing::info;

#[derive(Debug, serde::Serialize)]
pub(crate) struct FileInfo {
    pub(crate) image_path: String,
    pub(crate) mask_path: Option<String>,
}

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
                let file_name = entity.file_name().into_string().unwrap();
                if file_name.starts_with('.') {
                    continue;
                }
                if file_name.ends_with("_mask.png") {
                    continue;
                }

                let name: Vec<&str> = file_name.split('.').collect();
                let name: Vec<String> = name[..name.len() - 1]
                    .iter()
                    .map(|x| x.to_string())
                    .collect();

                let name = name.join("");
                let mask_path = format!("{}_mask.png", name);
                // info!("name:{:?}",&mask_path);
                let mask_full_path = format!("{}/{}", &config.base_dir, &mask_path);
                let mask_path = if std::path::Path::new(&mask_full_path).exists() {
                    Some(mask_path)
                } else {
                    None
                };

                files.push(FileInfo {
                    image_path: file_name,
                    mask_path,
                });
            }
        } else {
            break;
        }
    }

    Ok(Payload::success(files))
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct MaskParam {
    pub(crate) ori_image_name: String,
    pub(crate) mask: String,
    pub(crate) rotate: f32,
}

pub(crate) async fn save_mask(
    State(config): State<Config>,
    Json(param): Json<MaskParam>,
) -> Result<impl IntoResponse, Error> {
    // 获取原图片的路径
    let ori_image_path = PathBuf::from(config.base_dir.as_str()).join(&param.ori_image_name);
    let then = ori_image_path
        .as_path()
        .file_stem()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();
    let file_name = format!("{}_mask.png", then);
    let mask_image_path = PathBuf::from(config.base_dir.as_str()).join(&file_name);
    let data = param.mask.as_str();
    let res = base64_to_image(data)?;

    let mut f = tokio::fs::File::create(&mask_image_path)
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    let _ = f.write_all(&res).await;

    //保存旋转角度
    let mut rotate_file_path = PathBuf::from(&config.base_dir);
    rotate_file_path.push("rotate.txt");
    let mut rotate_file = OpenOptions::new()
        .append(true) // 设置为追加模式
        .create(true) // 如果文件不存在，则创建
        .open(rotate_file_path)
        .await
        .map_err(|e| anyhow!(e.to_string()))?;

    let content = format!("{},{}\n", &param.ori_image_name, param.rotate);
    rotate_file
        .write_all(content.as_bytes())
        .await
        .map_err(|e| anyhow!(e.to_string()))?;

    Ok(Payload::success(json!({
        "full_path":mask_image_path,
        "file_name":file_name,
    })))
}
