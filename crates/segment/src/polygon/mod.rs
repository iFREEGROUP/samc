use image::{DynamicImage, ImageBuffer, ImageError, Rgb, RgbImage};
use imageproc::contours::Contour;
use simplify_polyline::*;


/// MicroConnecter trait 用于清理分割的掩码无用的连通区
pub trait MicroConnecter {
    fn fill_polygon(&self, threshold: u8, min_points: usize) -> Result<RgbImage, ImageError>;
}

impl MicroConnecter for DynamicImage {
    fn fill_polygon(&self, threshold: u8, min_points: usize) -> Result<RgbImage, ImageError> {
        let img_proc = self.to_luma8();
        let img_proc = imageproc::contrast::threshold(&img_proc, threshold);

        let contours: Vec<Contour<i32>> = imageproc::contours::find_contours(&img_proc);

        let mut img: RgbImage = ImageBuffer::new(self.width(), self.height());

        for contour in &contours {
            let points: Vec<Point<2, f64>> = contour
                .points
                .iter()
                .map(|p| point!(p.x as f64, p.y as f64))
                .collect();

            let new_points = simplify_polyline::simplify(&points, 1.0, true);
            if new_points.len() < min_points {
                continue;
            }

            let poly: Vec<imageproc::point::Point<i32>> = new_points
                .iter()
                .map(|x| imageproc::point::Point::new(x.vec[0] as i32, x.vec[1] as i32))
                .collect();

            let fill_color = Rgb([0, 255u8, 0]);

            // println!("points::　{:?}",points);
            println!("new_points:: {:?}", new_points.len());
            imageproc::drawing::draw_antialiased_polygon_mut(
                &mut img,
                &poly,
                fill_color,
                imageproc::pixelops::interpolate,
            )
        }


        Ok(img)
    }
}


#[cfg(test)]
mod test {
    use image::{ImageBuffer, Rgb, RgbImage};
    use imageproc::contours::Contour;
    use simplify_polyline::point;
    use simplify_polyline::*;

    use super::MicroConnecter;


    #[test]
    fn test_fill_polygon()->anyhow::Result<()>{
        let img_data = include_bytes!("../../../../testdata/1711426394_6_9759_mask.png");
        let img = image::load_from_memory_with_format(img_data, image::ImageFormat::Png)?;
        let threshold = 128u8;
        let r = img.fill_polygon(threshold, 20);
        assert!(r.is_ok());
        Ok(())
        
    }

    #[test]
    fn test() -> anyhow::Result<()> {
        let img_data = include_bytes!("../../../../testdata/1711426394_6_9759_mask.png");
        let img = image::load_from_memory_with_format(img_data, image::ImageFormat::Png)?;
        let img_proc = img.to_luma8();
        let threshold = 128u8;
        let img_proc = imageproc::contrast::threshold(&img_proc, threshold);

        let contours: Vec<Contour<i32>> = imageproc::contours::find_contours(&img_proc);

        // println!("contours::　{:?}",&contours);

        let width = img.width();
        let height = img.height();
        let mut img: RgbImage = ImageBuffer::new(width, height);

        for contour in &contours {
            let points: Vec<Point<2, f64>> = contour
                .points
                .iter()
                .map(|p| point!(p.x as f64, p.y as f64))
                .collect();

            let new_points = simplify_polyline::simplify(&points, 1.0, true);
            if new_points.len() < 20 {
                continue;
            }

            let poly: Vec<imageproc::point::Point<i32>> = new_points
                .iter()
                .map(|x| imageproc::point::Point::new(x.vec[0] as i32, x.vec[1] as i32))
                .collect();

            let fill_color = Rgb([255, 0, 0]);

            // println!("points::　{:?}",points);
            println!("new_points::　{:?}", new_points.len());
            imageproc::drawing::draw_antialiased_polygon_mut(
                &mut img,
                &poly,
                fill_color,
                imageproc::pixelops::interpolate,
            )
        }
        img.save("filled_polygon.png").unwrap();
        Ok(())
    }
}
