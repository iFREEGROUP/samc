use axum::{
    extract::{Multipart, State},
    response::IntoResponse,
    Json,
};
use hyper::StatusCode;
use segment::{load_image_from_base64, load_image_from_mem, Masker, Segment, SegmentInferable};
use serde::Deserialize;
use serde_json::json;

use crate::{error::Error, render::Payload};

#[derive(Debug, Deserialize)]
pub(crate) struct SegmentParam {
    pub(crate) image: String,
    pub(crate) inputs: Vec<(f64, f64, bool)>,
}

pub(crate) async fn sam_anything_base64(
    State(model): State<Segment>,
    Json(param): Json<SegmentParam>,
) -> Result<impl IntoResponse, Error> {
    let mut pos_points = vec![];
    let mut neg_points = vec![];
    let (image, h, w) = load_image_from_base64(
        &param.image,
        Some(segment::segment_anything::sam::IMAGE_SIZE),
    )?;

    for input in &param.inputs {
        if !input.2 {
            pos_points.push((input.0, input.1));
        } else {
            neg_points.push((input.0, input.1));
        }
    }

    let neg_points = if !neg_points.is_empty() {
        Some(neg_points)
    } else {
        None
    };

    let mask = model.inference(image, &pos_points, neg_points.as_deref())?;

    Ok(Payload::success(json!({
        "mask": mask.to_base64(w as u32,h as u32).unwrap()
    })))
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
                if points[2] == 0 {
                    pos_points.push((points[0], points[1]))
                }
                if points[2] == 1 {
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

        let mask = model.inference(image, &pos_points, neg_points.as_deref())?;

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
