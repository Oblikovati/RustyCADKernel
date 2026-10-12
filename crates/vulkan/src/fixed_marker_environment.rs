//! OCCT3d097a0 point sprites bind non-modulating encoded textures in the FFP path.
use crate::{PointStyle, RasterImage, RenderError};
use rustycad_tkernel::quantity::ColorRgba;

pub(crate) fn prepare_fixed_marker_image(style: &mut PointStyle) -> Result<(), RenderError> {
    let mut pixels: Vec<_> = style
        .image
        .pixels()
        .iter()
        .copied()
        .map(ColorRgba::linear_to_srgb_vec4)
        .collect();
    if style.native_image_top_down {
        reverse_fixed_marker_rows(&mut pixels, style.image.extent());
    }
    style.image = RasterImage::new(style.image.extent(), pixels)?;
    Ok(())
}
fn reverse_fixed_marker_rows(pixels: &mut [[f32; 4]], [width, height]: [u32; 2]) {
    // FFP uses LOWER_LEFT; native uploads raw memory rows, unlike the logical Row getter.
    for row in 0..height as usize / 2 {
        for column in 0..width as usize {
            pixels.swap(
                row * width as usize + column,
                (height as usize - 1 - row) * width as usize + column,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encoded_marker_storage_preserves_alpha_without_changing_dimensions() {
        let mut style = PointStyle::new(1., [1, 1], vec![[0.25, 0.5, 0.75, 0.125]]).unwrap();
        prepare_fixed_marker_image(&mut style).unwrap();
        assert_eq!(style.image.extent(), [1, 1]);
        assert!(style.image.pixels()[0][0] > 0.5);
        assert_eq!(style.image.pixels()[0][3], 0.125);
    }
    #[test]
    fn fixed_top_down_memory_rows_reverse_once_but_bottom_up_rows_stay_logical() {
        let rows = vec![[0., 0., 0., 0.25], [1., 1., 1., 0.75]];
        let mut style = PointStyle::new(2., [1, 2], rows.clone()).unwrap();
        prepare_fixed_marker_image(&mut style).unwrap();
        assert_eq!(style.image.pixels(), &[rows[1], rows[0]]);
        let mut bottom = PointStyle::new(2., [1, 2], rows.clone()).unwrap();
        bottom.native_image_top_down = false;
        prepare_fixed_marker_image(&mut bottom).unwrap();
        assert_eq!(bottom.image.pixels(), rows.as_slice());
    }
}
