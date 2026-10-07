//! The static image operations of `Image_PixMap`: `SwapRgbaBgra`, `ToBlackWhite`, `FlipY`.
use super::pix_map_data::ImagePixMapData;
use super::pixel_convert::{image_color_from_raw_pixel, image_color_to_raw_pixel};
use super::{ImageFormat, ImagePixMap};
use rustycad_tkernel::quantity::ColorRgba;
use rustycad_tkernel::standard::Failure;

/// Calls `visit` with the bytes of every pixel, slice by slice, row by row.
fn for_each_pixel(data: &mut ImagePixMapData, mut visit: impl FnMut(&mut [u8])) {
    let (size_x, size_y, size_z) = (data.size_x, data.size_y, data.size_z);
    for slice in 0..size_z {
        for row in 0..size_y {
            for col in 0..size_x {
                if let Some(pixel) = data.change_value_xyz(col, row, slice) {
                    visit(pixel);
                }
            }
        }
    }
}

/// Like [`for_each_pixel`], stopping at the first error.
fn try_for_each_pixel(
    data: &mut ImagePixMapData,
    mut visit: impl FnMut(&mut [u8]) -> Result<(), Failure>,
) -> Result<(), Failure> {
    let mut result = Ok(());
    for_each_pixel(data, |pixel| {
        if result.is_ok() {
            result = visit(pixel);
        }
    });
    result
}

/// Swaps the first and third components of `size`-byte components.
fn swap_first_and_third(pixel: &mut [u8], size: usize) {
    let (first, rest) = pixel.split_at_mut(2 * size);
    first[..size].swap_with_slice(&mut rest[..size]);
}

impl ImagePixMap {
    /// Swaps the red and blue channels in place (8-bit and float RGB/RGBA formats); the
    /// 32-bit RGB formats get an opaque padding byte. False for other formats.
    ///
    /// Example: `ImagePixMap::swap_rgba_bgra(&pixmap)`.
    pub fn swap_rgba_bgra(image: &ImagePixMap) -> bool {
        let mut state = image.change_data();
        let format = state.format;
        use ImageFormat as F;
        let (component, reset_alpha) = match format {
            F::Bgr32 | F::Rgb32 => (1, true),
            F::Bgra | F::Rgba | F::Bgr | F::Rgb => (1, false),
            F::Bgrf | F::Rgbf | F::Bgraf | F::Rgbaf => (4, false),
            _ => return false,
        };
        for_each_pixel(&mut state.data, |pixel| {
            swap_first_and_third(pixel, component);
            if reset_alpha {
                pixel[3] = 255;
            }
        });
        true
    }
    /// Turns every non-black pixel white (alpha kept for the generic formats).
    ///
    /// Example: `ImagePixMap::to_black_white(&pixmap)?`.
    pub fn to_black_white(image: &ImagePixMap) -> Result<(), Failure> {
        let mut state = image.change_data();
        let format = state.format;
        use ImageFormat as F;
        match format {
            F::Gray | F::Alpha => for_each_pixel(&mut state.data, |p| {
                if p[0] != 0 {
                    p[0] = 255
                }
            }),
            F::Gray16 => for_each_pixel(&mut state.data, |p| {
                if p[..2] != [0, 0] {
                    p[..2].copy_from_slice(&u16::MAX.to_ne_bytes())
                }
            }),
            F::Rgb | F::Bgr | F::Rgb32 | F::Bgr32 | F::Rgba | F::Bgra => {
                for_each_pixel(&mut state.data, |p| {
                    if p[..3] != [0, 0, 0] {
                        p[..3].fill(255)
                    }
                })
            }
            _ => return generic_to_black_white(&mut state.data, format),
        }
        Ok(())
    }
    /// Mirrors every slice vertically by swapping rows; false for an empty image.
    ///
    /// Example: `ImagePixMap::flip_y(&pixmap)`.
    pub fn flip_y(image: &ImagePixMap) -> bool {
        let mut state = image.change_data();
        let data = &mut state.data;
        if data.is_empty() || data.size_x == 0 || data.size_y == 0 {
            return false;
        }
        for slice in 0..data.size_z {
            for top in 0..data.size_y / 2 {
                let bottom = data.size_y - 1 - top;
                let Some(row) = data.slice_row(slice, top).map(<[u8]>::to_vec) else {
                    continue;
                };
                let Some(other) = data.slice_row(slice, bottom).map(<[u8]>::to_vec) else {
                    continue;
                };
                if let Some(target) = data.change_slice_row(slice, top) {
                    target.copy_from_slice(&other);
                }
                if let Some(target) = data.change_slice_row(slice, bottom) {
                    target.copy_from_slice(&row);
                }
            }
        }
        true
    }
}

/// The `default` branch of `ToBlackWhite`: through the pixel colors.
fn generic_to_black_white(data: &mut ImagePixMapData, format: ImageFormat) -> Result<(), Failure> {
    let white = ColorRgba::new(1.0, 1.0, 1.0, 1.0)?;
    try_for_each_pixel(data, |pixel| {
        let color = image_color_from_raw_pixel(pixel, format, false)?.to_vec4();
        if color.x != 0.0 || color.y != 0.0 || color.z != 0.0 {
            image_color_to_raw_pixel(pixel, format, &white, false);
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swap_flip_and_black_white() {
        let pixmap = ImagePixMap::default();
        pixmap.init_wrapper(
            ImageFormat::Rgb32,
            Some(vec![1, 2, 3, 4, 5, 6, 7, 8]),
            1,
            2,
            0,
        );
        assert!(ImagePixMap::swap_rgba_bgra(&pixmap));
        assert_eq!(pixmap.data().data.data(), &[3, 2, 1, 255, 7, 6, 5, 255]);
        assert!(ImagePixMap::flip_y(&pixmap));
        assert_eq!(pixmap.data().data.data(), &[7, 6, 5, 255, 3, 2, 1, 255]);
        ImagePixMap::to_black_white(&pixmap).unwrap();
        assert_eq!(
            pixmap.data().data.data(),
            &[255, 255, 255, 255, 255, 255, 255, 255]
        );
        let floats = ImagePixMap::default();
        floats.init_zero(ImageFormat::GrayF, 1, 1, 0, 0);
        floats
            .change_data()
            .data
            .change_data()
            .copy_from_slice(&0.5f32.to_ne_bytes());
        ImagePixMap::to_black_white(&floats).unwrap();
        assert_eq!(floats.data().data.data(), &1.0f32.to_ne_bytes());
        assert!(!ImagePixMap::swap_rgba_bgra(&floats));
    }
}
