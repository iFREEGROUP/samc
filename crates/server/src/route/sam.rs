use std::vec;

use axum::{
    extract::{Multipart, State},
    response::IntoResponse,
    Json,
};
use hyper::StatusCode;
use segment::{load_image_from_base64, load_image_from_mem, Masker, Segment, SegmentInferable};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;

use crate::{config::Config, error::Error, render::Payload};

#[derive(Debug, Deserialize)]
pub(crate) struct SegmentParam {
    pub(crate) image: String,
    pub(crate) inputs: Vec<(f64, f64, i16)>,
    pub(crate) masks: Option<String>,
}

pub(crate) async fn sam_anything_base64(
    State(config): State<Config>,
    State(model): State<Segment>,
    Json(param): Json<SegmentParam>,
) -> Result<impl IntoResponse, Error> {
    let mut pos_points = vec![];
    let mut neg_points = vec![];
    let (image, h, w) = load_image_from_base64(
        &param.image,
        Some(segment::segment_anything::sam::IMAGE_SIZE),
    )?;
    info!("w:{},h:{}", w, h);
    for input in &param.inputs {
        if input.2 == 1 {
            pos_points.push((input.0, input.1));
        } else {
            neg_points.push((input.0, input.1));
        }
    }

    let mask = if let Some(api) = config.use_api {
        let neg_points = if !neg_points.is_empty() {
            Some(neg_points)
        } else {
            None
        };
        inference_api(&api, param.image, &pos_points, neg_points.as_deref()).await?
    } else {
        let pos_points: Vec<(f64, f64)> = pos_points
            .iter()
            .map(|(x, y)| (x / w as f64, y / h as f64))
            .collect();
        let neg_points: Vec<(f64, f64)> = neg_points
            .iter()
            .map(|(x, y)| (x / w as f64, y / h as f64))
            .collect();
        let neg_points = if !neg_points.is_empty() {
            Some(neg_points)
        } else {
            None
        };
        let mask = if let Some(m) = param.masks {
            let (m, _, _) =
                load_image_from_base64(&m, Some(segment::segment_anything::sam::IMAGE_SIZE))?;
            Some(m)
        } else {
            None
        };
        let mask = model.inference(image, &pos_points, neg_points.as_deref(), mask)?;

        mask.to_base64(w as u32, h as u32)?
    };

    Ok(Payload::success(json!({
        "mask": mask
    })))
}

async fn inference_api(
    api: &str,
    images: String,
    pos_points: &[(f64, f64)],
    neg_points: Option<&[(f64, f64)]>,
) -> Result<String, reqwest::Error> {
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Response {
        #[serde(rename = "inputs")]
        pub inputs: Vec<Vec<i32>>,
        #[serde(rename = "mask")]
        pub mask: String,
    }

    #[derive(Debug, Serialize)]
    pub struct Form {
        images: String,
        inputs: Vec<String>,
    }

    let mut inputs = vec![];
    for (x, y) in pos_points {
        inputs.push(format!("{},{},1", x, y));
    }
    if let Some(neg_points) = neg_points {
        for (x, y) in neg_points {
            inputs.push(format!("{},{},0", x, y));
        }
    }

    let form = Form { images, inputs };
    // info!("{:?}",&form);

    let client = reqwest::Client::new();
    let res: Response = client.post(api).json(&form).send().await?.json().await?;
    // info!("{:?}",&res);

    Ok(res.mask)
}

#[allow(clippy::single_match)]
pub(crate) async fn sam_anything(
    State(model): State<Segment>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, Error> {
    let mut files = vec![];
    let mut pos_points = vec![];
    let mut neg_points = vec![];

    while let Some(field) = multipart.next_field().await.unwrap() {
        match field.name().unwrap_or_default() {
            "inputs" => {
                let points: String = field.text().await.map_err(|_| Error::RequireParam)?;
                let points: Vec<usize> = points
                    .split(',')
                    .map(|x| x.parse::<usize>().unwrap_or_default())
                    .collect::<Vec<_>>();
                if points.len() != 3 {
                    return Err(Error::ErrorParam("IInputs format error,you should input 3 elements and the last one is 0 or 1,like: [x,y,0]".to_string()));
                }
                if points[2] == 1 {
                    pos_points.push((points[0], points[1]))
                }
                if points[2] == 0 {
                    neg_points.push((points[0], points[1]))
                }
            }

            "files" => {
                let file_name = field.file_name().unwrap_or_default().to_string();
                let _content_type = field.content_type().unwrap_or_default().to_string();
                let data = field.bytes().await.unwrap_or_default();
                files.push((file_name, data));
            }
            _ => (),
        }
    }

    if let Some((_name, bytes)) = files.first() {
        let (image, h, w) =
            load_image_from_mem(bytes, Some(segment::segment_anything::sam::IMAGE_SIZE))?;

        let pos_points: Vec<(f64, f64)> = pos_points
            .into_iter()
            .map(|x| (x.0 as f64 / w as f64, x.1 as f64 / h as f64))
            .collect();
        let neg_points: Vec<(f64, f64)> = neg_points
            .into_iter()
            .map(|x| (x.0 as f64 / w as f64, x.1 as f64 / h as f64))
            .collect();
        let neg_points = if !neg_points.is_empty() {
            Some(neg_points)
        } else {
            None
        };

        let img = image::load_from_memory(bytes)?;

        let mask = model.inference(image, &pos_points, neg_points.as_deref(),None)?;

        mask.save(img, "temp.png").unwrap();

        return Ok(Payload::success(json!({
            "mask": mask.to_base64(w as u32,h as u32).unwrap()
        })));
    }

    Ok(Payload::error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "unkown error",
    ))
}
