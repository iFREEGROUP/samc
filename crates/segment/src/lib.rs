pub mod polygon;
pub mod segment_anything;
use self::segment_anything::sam::Sam;
use anyhow::anyhow;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use candle_core::{IndexOp, D};
pub use candle_core::{DType, Device, Error, Tensor};
use candle_nn::VarBuilder;
use image::DynamicImage;
use polygon::MicroConnecter;
use std::{
    io::{Cursor, Write},
    path::Path,
};
// pub type  Mask = Vec<u8>;
pub struct Mask {
    pub mask: Vec<u8>,
    pub h: usize,
    pub w: usize,
    pub points: Vec<(f64, f64, bool)>,
}

pub fn image_to_base64(data: &[u8]) -> Result<String, anyhow::Error> {
    let img = image::load_from_memory(data)?;
    let mut buf: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)?;
    let b64 = BASE64.encode(&buf);
    Ok(b64)
}

impl Mask {
    pub fn to_base64(&self, w: u32, h: u32) -> Result<String, anyhow::Error> {

        let mask_img: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> =
            image::ImageBuffer::from_raw(self.w as u32, self.h as u32, self.mask.clone()).unwrap();
        let mask_img = image::DynamicImage::from(mask_img).resize_to_fill(
            w,
            h,
            image::imageops::FilterType::CatmullRom,
        );

        let mask_img = mask_img.fill_polygon(128u8, 30).map_err(|e| anyhow!(e.to_string()))?;

        let mut buf: Vec<u8> = Vec::new();
        mask_img.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)?;

        let mut f = std::fs::File::create("path.png").unwrap();
        let _ = f.write_all(&buf);
        let b64 = BASE64.encode(&buf);
        Ok(b64)
    }
}

pub fn load_image<P: AsRef<std::path::Path>>(
    p: P,
    resize_longest: Option<usize>,
) -> candle_core::Result<(Tensor, usize, usize)> {
    let img = image::io::Reader::open(p)?
        .decode()
        .map_err(candle_core::Error::wrap)?;
    let (initial_h, initial_w) = (img.height() as usize, img.width() as usize);
    let img = match resize_longest {
        None => img,
        Some(resize_longest) => {
            let (height, width) = (img.height(), img.width());
            let resize_longest = resize_longest as u32;
            let (height, width) = if height < width {
                let h = (resize_longest * height) / width;
                (h, resize_longest)
            } else {
                let w = (resize_longest * width) / height;
                (resize_longest, w)
            };
            img.resize_exact(width, height, image::imageops::FilterType::CatmullRom)
        }
    };
    let (height, width) = (img.height() as usize, img.width() as usize);
    let img = img.to_rgb8();
    let data = img.into_raw();
    let data = Tensor::from_vec(data, (height, width, 3), &Device::Cpu)?.permute((2, 0, 1))?;
    Ok((data, initial_h, initial_w))
}

pub fn base64_to_image(data: &str) -> anyhow::Result<Vec<u8>> {
    let offset = data.find(',').unwrap_or(data.len()) + 1;

    let mut value = String::from(data);
    // let value = &data[..offset];
    value.drain(..offset);

    let res = BASE64
        .decode(value)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    Ok(res)
}

pub fn load_image_from_base64(
    data: &str,
    resize_longest: Option<usize>,
) -> anyhow::Result<(candle_core::Tensor, usize, usize)> {
    let res = base64_to_image(data)?;
    let loaded =
        load_image_from_mem(&res, resize_longest).map_err(|e| anyhow::anyhow!(e.to_string()))?;

    Ok(loaded)
}

pub fn load_image_from_mem(
    b: &[u8],
    resize_longest: Option<usize>,
) -> candle_core::Result<(Tensor, usize, usize)> {
    let img = image::load_from_memory(b).map_err(candle_core::Error::wrap)?;
    let (initial_h, initial_w) = (img.height() as usize, img.width() as usize);
    let img = match resize_longest {
        None => img,
        Some(resize_longest) => {
            let (height, width) = (img.height(), img.width());
            let resize_longest = resize_longest as u32;
            let (height, width) = if height < width {
                let h = (resize_longest * height) / width;
                (h, resize_longest)
            } else {
                let w = (resize_longest * width) / height;
                (resize_longest, w)
            };
            img.resize_exact(width, height, image::imageops::FilterType::CatmullRom)
        }
    };
    let (height, width) = (img.height() as usize, img.width() as usize);
    let img = img.to_rgb8();
    let data = img.into_raw();
    let data = Tensor::from_vec(data, (height, width, 3), &Device::Cpu)?.permute((2, 0, 1))?;
    Ok((data, initial_h, initial_w))
}

pub trait SegmentInferable: Send + std::marker::Sync + 'static {
    fn inference(
        &self,
        image: Tensor,
        points: &[(f64, f64)],
        neg_points: Option<&[(f64, f64)]>,
    ) -> candle_core::Result<Mask>;
}

#[derive(Debug, Clone)]
pub struct Segment {
    device: Device,
    model: Sam,
}

impl Segment {
    pub fn new<P: AsRef<Path>>(
        model_file: P,
        device: Device,
        use_tiny: bool,
    ) -> candle_core::Result<Self> {
        let vb =
            unsafe { VarBuilder::from_mmaped_safetensors(&[model_file], DType::F32, &device)? };

        let model = if use_tiny {
            Sam::new_tiny(vb)?
        } else {
            Sam::new(768, 12, 12, &[2, 5, 8, 11], vb)? // sam_vit_b
        };

        Ok(Self { device, model })
    }
}

impl SegmentInferable for Segment {
    fn inference(
        &self,
        image: Tensor,
        pos_points: &[(f64, f64)],
        neg_points: Option<&[(f64, f64)]>,
    ) -> candle_core::Result<Mask> {
        // let (image, initial_h, initial_w) = load_image(image_path, Some(candle_transformers::models::segment_anything::sam::IMAGE_SIZE))?;
        let image = image.to_device(&self.device)?;
        let iter_points = pos_points.iter().map(|p| (p.0, p.1, true));
        let iter_neg_points = neg_points
            .unwrap_or_default()
            .iter()
            .map(|p| (p.0, p.1, false));
        let points = iter_points.chain(iter_neg_points).collect::<Vec<_>>();

        let start_time = std::time::Instant::now();
        let (mask, iou_predictions) = self.model.forward(&image, &points, true)?;
        println!(
            "mask generated in {:.2}s",
            start_time.elapsed().as_secs_f32()
        );
        let max_index = iou_predictions.argmax(D::Minus1)?.to_vec1::<u32>()?;
        // let min_index = iou_predictions.argmin(D::Minus1)?.to_vec1::<u32>()?;
        
        let mask = mask.i(max_index[0] as usize)?.unsqueeze(0)?;
        println!("iou_predictions: {iou_predictions}");

        let mask = (mask.ge(0.)? * 255.)?;

        let (_one, h, w) = mask.dims3()?;
        let mask = mask.expand((3, h, w))?;
        let mask_pixels = mask.permute((1, 2, 0))?.flatten_all()?.to_vec1::<u8>()?;
        println!("mask:\n{:?}", mask.shape());
        Ok(Mask {
            mask: mask_pixels,
            h,
            w,
            points,
        })
    }
}

pub trait Masker: Send + std::marker::Sync + 'static {
    fn save<P: AsRef<Path>>(&self, img: DynamicImage, p: P) -> anyhow::Result<()>;
}

impl Masker for Mask {
    fn save<P: AsRef<Path>>(&self, mut img: DynamicImage, p: P) -> anyhow::Result<()> {
        let mask_img: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> =
            match image::ImageBuffer::from_raw(self.w as u32, self.h as u32, self.mask.clone()) {
                Some(image) => image,
                None => anyhow::bail!("error saving merged image"),
            };
        let mask_img = image::DynamicImage::from(mask_img).resize_to_fill(
            img.width(),
            img.height(),
            image::imageops::FilterType::CatmullRom,
        );
        for x in 0..img.width() {
            for y in 0..img.height() {
                let mask_p = imageproc::drawing::Canvas::get_pixel(&mask_img, x, y);
                if mask_p.0[0] > 100 {
                    let mut img_p = imageproc::drawing::Canvas::get_pixel(&img, x, y);
                    img_p.0[2] = 255 - (255 - img_p.0[2]) / 2;
                    img_p.0[1] /= 2;
                    img_p.0[0] /= 2;
                    imageproc::drawing::Canvas::draw_pixel(&mut img, x, y, img_p)
                }
            }
        }
        for (x, y, b) in &self.points {
            let x = (x * img.width() as f64) as i32;
            let y = (y * img.height() as f64) as i32;
            let color = if *b {
                image::Rgba([255, 0, 0, 200])
            } else {
                image::Rgba([0, 255, 0, 200])
            };
            imageproc::drawing::draw_filled_circle_mut(&mut img, (x, y), 3, color);
        }

        img.save(p)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use candle_core::DType;
    use candle_nn::VarBuilder;
    use candle_transformers::models::segment_anything::sam;
    use candle_transformers::models::segment_anything::sam::Sam;

    use crate::load_image;

    #[test]
    fn test_sam() -> anyhow::Result<()> {
        let device = candle_core::Device::Cpu;

        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(
                &["mobile_sam-tiny-vitt.safetensors"],
                DType::F32,
                &device,
            )?
        };
        let image_path = "/Users/kingzcheung/rust/itemsam/screenshot-20240424-113925.jpg";

        let (image, _initial_h, _initial_w) = load_image(image_path, Some(sam::IMAGE_SIZE))?;
        let image = image.to_device(&device)?;

        let sam = Sam::new_tiny(vb)?;
        let iter_points = [[0.5, 0.5]].iter().map(|p| (p[0], p[1], true));
        let iter_neg_points = [[0.6, 0.5]].iter().map(|p| (p[0], p[1], false));
        let points = iter_points.chain(iter_neg_points).collect::<Vec<_>>();
        let start_time = std::time::Instant::now();
        let (mask, iou_predictions) = sam.forward(&image, &points, false)?;
        println!(
            "mask generated in {:.2}s",
            start_time.elapsed().as_secs_f32()
        );
        // println!("mask:\n{mask}");
        println!("iou_predictions: {iou_predictions}");

        let mask = (mask.ge(-1.2)? * 255.)?;
        let (_one, h, w) = mask.dims3()?;
        let mask = mask.expand((3, h, w))?;

        let mut img = image::io::Reader::open(image_path)?
            .decode()
            .map_err(candle_core::Error::wrap)?;
        let mask_pixels = mask.permute((1, 2, 0))?.flatten_all()?.to_vec1::<u8>()?;
        let mask_img: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> =
            match image::ImageBuffer::from_raw(w as u32, h as u32, mask_pixels) {
                Some(image) => image,
                None => anyhow::bail!("error saving merged image"),
            };
        let mask_img = image::DynamicImage::from(mask_img).resize_to_fill(
            img.width(),
            img.height(),
            image::imageops::FilterType::CatmullRom,
        );
        for x in 0..img.width() {
            for y in 0..img.height() {
                let mask_p = imageproc::drawing::Canvas::get_pixel(&mask_img, x, y);
                if mask_p.0[0] > 100 {
                    let mut img_p = imageproc::drawing::Canvas::get_pixel(&img, x, y);
                    img_p.0[2] = 255 - (255 - img_p.0[2]) / 2;
                    img_p.0[1] /= 2;
                    img_p.0[0] /= 2;
                    imageproc::drawing::Canvas::draw_pixel(&mut img, x, y, img_p)
                }
            }
        }
        for (x, y, b) in points {
            let x = (x * img.width() as f64) as i32;
            let y = (y * img.height() as f64) as i32;
            let color = if b {
                image::Rgba([255, 0, 0, 200])
            } else {
                image::Rgba([0, 255, 0, 200])
            };
            imageproc::drawing::draw_filled_circle_mut(&mut img, (x, y), 3, color);
        }
        img.save("sam_merged.jpg")?;
        Ok(())
    }
}
