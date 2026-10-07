//! `Image_Diff::SaveDiffImage`: paints the different pixels white on black.
use super::diff::{ImageDiff, image_diff_unpack_x, image_diff_unpack_y};
use super::pixel_convert::image_color_to_raw_pixel;
use super::{ImageAlienPixMap, ImageFormat, ImagePixMap, ImagePixMapState};
use rustycad_tkernel::quantity::ColorRgba;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::tcollection::AsciiString;

/// Paints one packed pixel white in the image's own format (`SetPixelColor` ignores
/// pixels outside the image; the raw branches write the first channel bytes).
fn paint_white(image: &mut ImagePixMapState, pixel: i32, white: &ColorRgba) {
    let (x, y) = (
        usize::from(image_diff_unpack_x(pixel)),
        usize::from(image_diff_unpack_y(pixel)),
    );
    let format = image.format;
    let Some(raw) = image.data.change_value(y, x) else {
        return;
    };
    use ImageFormat as F;
    match format {
        F::Gray | F::Alpha => raw[0] = 255,
        F::Rgb | F::Bgr | F::Rgb32 | F::Bgr32 | F::Rgba | F::Bgra => raw[..3].fill(255),
        _ => image_color_to_raw_pixel(raw, format, white, false),
    }
}

impl ImageDiff {
    /// `SaveDiffImage(Image_PixMap&)`: (re)allocates a gray image of the reference size when
    /// needed, clears it and paints the different pixels, or the non-linear groups when the
    /// border filter grouped them. Example: `diff.save_diff_image(&target)?`.
    pub fn save_diff_image(&self, target: &ImagePixMap) -> Result<bool, Failure> {
        let fields = self.fields();
        let (Some(reference), Some(_)) = (fields.image_ref.get(), fields.image_new.get()) else {
            return Ok(false);
        };
        let (width, height) = (reference.width(), reference.height());
        let needs_init = target.is_empty() || target.width() != width || target.height() != height;
        if needs_init && !target.init_trash(ImageFormat::Gray, width, height, 0) {
            return Ok(false);
        }
        let white = ColorRgba::new(1.0, 1.0, 1.0, 1.0)?;
        let mut image = target.change_data();
        image.data.change_data().fill(0);
        if fields.groups_of_diff_pixels.is_empty() {
            if !fields.is_border_filter_on {
                fields
                    .diff_pixels
                    .iter()
                    .for_each(|pixel| paint_white(&mut image, *pixel, &white));
            }
            return Ok(true);
        }
        for (index, group) in fields.groups_of_diff_pixels.iter().enumerate() {
            if !fields.linear_groups.contains(index as i32 + 1) {
                group
                    .iter()
                    .for_each(|pixel| paint_white(&mut image, pixel, &white));
            }
        }
        Ok(true)
    }
    /// `SaveDiffImage(path)`: the difference image saved through Image_AlienPixMap (PPM
    /// without an image library); false without images or path.
    ///
    /// Example: `diff.save_diff_image_file(&AsciiString::from("diff.ppm"))?`.
    pub fn save_diff_image_file(&self, path: &AsciiString) -> Result<bool, Failure> {
        let (width, height) = {
            let fields = self.fields();
            let (Some(reference), Some(_)) = (fields.image_ref.get(), fields.image_new.get())
            else {
                return Ok(false);
            };
            (reference.width(), reference.height())
        };
        if path.as_bytes().is_empty() {
            return Ok(false);
        }
        let image = ImageAlienPixMap::default();
        if !image.init_trash(ImageFormat::Gray, width, height, 0)
            || !self.save_diff_image(&image)?
        {
            return Ok(false);
        }
        image.save(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkernel::standard::Handle;

    fn gray(values: &[u8]) -> Handle<ImagePixMap> {
        let image = ImagePixMap::default();
        image.init_wrapper(ImageFormat::Gray, Some(values.to_vec()), 3, 3, 0);
        Handle::new(image)
    }

    #[test]
    fn compare_counts_and_paints_differences() {
        let diff = ImageDiff::default();
        assert_eq!(diff.compare().unwrap(), -1);
        let reference = gray(&[0, 0, 0, 0, 10, 0, 0, 0, 0]);
        let new = gray(&[0, 0, 0, 0, 40, 0, 0, 0, 9]);
        assert!(diff.init(&reference, &new, false).unwrap());
        assert_eq!(diff.compare().unwrap(), 2);
        diff.set_color_tolerance(0.1);
        assert_eq!(diff.compare().unwrap(), 1);
        let target = ImagePixMap::default();
        assert!(diff.save_diff_image(&target).unwrap());
        assert_eq!(
            target
                .data()
                .data
                .data()
                .iter()
                .filter(|b| **b == 255)
                .count(),
            1
        );
    }

    #[test]
    fn border_filter_drops_thin_lines_next_to_drawn_pixels() {
        let reference = gray(&[255, 255, 255, 0, 0, 0, 0, 0, 0]);
        let new = gray(&[255, 255, 255, 50, 50, 50, 0, 0, 0]);
        let diff = ImageDiff::default();
        diff.init(&reference, &new, false).unwrap();
        diff.set_border_filter_on(true);
        assert_eq!(diff.compare().unwrap(), 0);
        assert_eq!(diff.fields().groups_of_diff_pixels.len(), 1);
    }
}
