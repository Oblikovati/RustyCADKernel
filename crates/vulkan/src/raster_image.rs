//! Immutable image pixels shared by native point sprites and glyph atlases.
use crate::{RenderError, error::rejected};
use std::sync::Arc;
/// Top-down linear RGBA texels; `RasterImage::new([1,1], vec![[1.;4]])?`.
#[derive(Clone, Debug, PartialEq)]
pub struct RasterImage {
    pub(crate) extent: [u32; 2],
    pub(crate) pixels: Arc<Vec<[f32; 4]>>,
}
impl RasterImage {
    /// Validate dimensions and texel components; `RasterImage::new([1,1], vec![[1.;4]])?`.
    pub fn new(extent: [u32; 2], pixels: Vec<[f32; 4]>) -> Result<Self, RenderError> {
        let count = u64::from(extent[0]) * u64::from(extent[1]);
        if extent.contains(&0) || count != pixels.len() as u64 {
            return Err(rejected(
                (extent, pixels.len()),
                "positive dimensions and width*height RGBA texels",
            ));
        }
        validate_pixels(&pixels)?;
        Ok(Self {
            extent,
            pixels: Arc::new(pixels),
        })
    }
    /// Read image dimensions; `image.extent()`.
    pub fn extent(&self) -> [u32; 2] {
        self.extent
    }
    /// Read immutable texels; `image.pixels()`.
    pub fn pixels(&self) -> &[[f32; 4]] {
        &self.pixels
    }
}
fn validate_pixels(pixels: &[[f32; 4]]) -> Result<(), RenderError> {
    if pixels
        .iter()
        .flatten()
        .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
    {
        return Err(rejected(pixels, "finite sprite components in 0..=1"));
    }
    Ok(())
}

// FreeType monochrome glyphs are Gray, while antialiased glyphs are Alpha.
pub(crate) fn coverage_pixel(
    image: &rustycad_tkservice::image::ImagePixMap,
    x: u32,
    y: u32,
) -> Result<f32, RenderError> {
    let color = image
        .pixel_color(x as i32, y as i32, false)
        .map_err(|e| rejected(e, "a valid coverage pixel"))?;
    Ok(
        if image.format() == rustycad_tkservice::image::ImageFormat::Gray {
            color.rgb().red() as f32
        } else {
            color.alpha()
        },
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkservice::image::{ImageFormat, ImagePixMap};
    #[test]
    fn grayscale_and_alpha_glyphs_have_matching_coverage() {
        for format in [ImageFormat::Gray, ImageFormat::Alpha] {
            let image = ImagePixMap::default();
            assert!(image.init_wrapper(format, Some(vec![0, 128, 255]), 3, 1, 3));
            assert_eq!(coverage_pixel(&image, 0, 0).unwrap(), 0.);
            assert!((coverage_pixel(&image, 1, 0).unwrap() - 128. / 255.).abs() < 1e-6);
            assert_eq!(coverage_pixel(&image, 2, 0).unwrap(), 1.);
        }
    }
    #[test]
    fn immutable_images_reject_bad_shape_and_nonfinite_components() {
        assert!(RasterImage::new([0, 1], vec![]).is_err());
        assert!(RasterImage::new([2, 1], vec![[1.; 4]]).is_err());
        assert!(RasterImage::new([1, 1], vec![[f32::NAN; 4]]).is_err());
        assert!(RasterImage::new([1, 1], vec![[2.; 4]]).is_err());
        let image = RasterImage::new([1, 1], vec![[0.5; 4]]).unwrap();
        assert_eq!(image.extent(), [1, 1]);
        assert_eq!(image.pixels(), &[[0.5; 4]]);
    }
}
