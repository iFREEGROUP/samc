use crate::error::Error;
use crate::{config::Config, render::Payload};
use anyhow::anyhow;
use axum::{extract::State, response::IntoResponse, Json};
use imageinfo::ImageInfo;
use segment::base64_to_image;
use serde_json::json;
use std::path::{Path, PathBuf};
use tokio::{
    fs::OpenOptions,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
};

async fn is_image(path: &Path) -> anyhow::Result<bool> {
    match ImageInfo::from_file_path(path) {
        Ok(info) => Ok(matches!(info.ext, "png" | "jpg" | "jpeg")),
        Err(e) => Err(anyhow!(e.to_string())),
    }
}

async fn find_rotate_from_file(base_dir: &Path, file: &str) -> anyhow::Result<f32> {
    let mut rotate = 0.0;
    let rotate_label_file_path = base_dir.join("rotate.txt");
    let rotate_label_file = tokio::fs::File::open(rotate_label_file_path).await?;

    let reader = BufReader::new(rotate_label_file);

    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        if line.is_empty() {
            continue;
        }
        let sp: Vec<&str> = line.split(',').collect();
        if sp[0] == file {
            rotate = sp[1].parse::<f32>().unwrap_or_default();
            break;
        }
    }

    Ok(rotate)
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct FileInfo {
    pub(crate) image_path: String,
    pub(crate) mask_path: Option<String>,
    pub(crate) rotate: f32,
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
                if let Ok(r) = is_image(entity.path().as_path()).await {
                    if !r {
                        continue;
                    }
                }

                let name: Vec<&str> = file_name.split('.').collect();
                let name: Vec<String> = name[..name.len() - 1]
                    .iter()
                    .map(|x| x.to_string())
                    .collect();

                let name = name.join("");
                let mask_path = format!("{}_mask.png", name);
                // info!("name:{:?}",&mask_path);
                let mut mask_full_path = PathBuf::from(&config.base_dir);
                mask_full_path.push(&mask_path);
                let mask_path = if std::path::Path::new(&mask_full_path).exists() {
                    Some(mask_path)
                } else {
                    None
                };
                let base_dir = Path::new(config.base_dir.as_str());
                let rotate =
                    if let Ok(rotate) = find_rotate_from_file(base_dir, file_name.as_str()).await {
                        rotate
                    } else {
                        0f32
                    };

                files.push(FileInfo {
                    image_path: file_name,
                    mask_path,
                    rotate,
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

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_find_rotate_from_file() -> anyhow::Result<()> {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let base_dir = PathBuf::from(manifest_dir);
        let base_dir = base_dir.join("testdata");
        println!("CARGO_MANIFEST_DIR: {:?}", &base_dir);
        let file = "1711426394_6_9759.jpg";
        let rotate = find_rotate_from_file(base_dir.as_path(), file).await?;

        assert_eq!(15f32, rotate);

        Ok(())
    }
}
