//! Bounded hardware sRGB conversion comparison; raw mismatches stay diagnostic.
#[path = "support/device_options.rs"]
mod device_options;
use martensite_vulkan::*;
use rustycad_tkernel::quantity::ColorRgba;
use rustycad_tkservice::{aspect::*, font::FontRect, graphic3d::*};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut renderer = VulkanRenderer::new(device_options::validation_options())?;
    println!("MARTENSITE_DEVICE={}", renderer.device_name());
    for row in include_str!("../tests/fixtures/blend_text/native-color-transfer.txt").lines() {
        probe_color_row(&mut renderer, row)?;
    }
    assert!(
        renderer.validation_errors().is_empty(),
        "{:?}",
        renderer.validation_errors()
    );
    Ok(())
}
fn probe_color_row(renderer: &mut VulkanRenderer, row: &str) -> Result<(), RenderError> {
    let fields: Vec<f32> = row.split_whitespace().map(|v| v.parse().unwrap()).collect();
    let rgba: [f32; 4] = fields[1..5].try_into().unwrap();
    let mut colors = Vec::new();
    for mode in [
        AspectTypeOfDisplayText::Blend,
        AspectTypeOfDisplayText::Normal,
    ] {
        let frame = color_probe_frame(rgba, mode);
        let pixel = renderer.render(&frame)?.pixel(8, 8).unwrap();
        colors.push(pixel);
    }
    colors[1][2] ^= 255;
    emit_probe_colors(fields[0] as u32, &fields[5..9], &colors);
    Ok(())
}
fn color_probe_frame(rgba: [f32; 4], mode: AspectTypeOfDisplayText) -> RenderFrame {
    let text = Graphic3dText::new(16.);
    let style = Graphic3dAspectText3d::default();
    style.set_text_display_type(mode);
    style.set_interior_color_rgba(ColorRgba::new(rgba[0], rgba[1], rgba[2], rgba[3]).unwrap());
    let draws = lower_pixel_text(
        &text,
        &style,
        &color_probe_layout(),
        &PixelTextView::default(),
    )
    .unwrap();
    RenderFrame::new(16, 16, [0., 0., 1., 1.], draws).unwrap()
}
fn color_probe_layout() -> GlyphLayout {
    // A solid atlas isolates format conversion from font coverage and raster-edge differences.
    GlyphLayout {
        positions: vec![[-1., -1.], [1., -1.], [1., 1.], [-1., 1.]],
        uv: vec![[0.; 2]; 4],
        indices: vec![0, 1, 2, 0, 2, 3],
        atlas: RasterImage::new([1, 1], vec![[1.; 4]]).unwrap(),
        formatter_bounds: FontRect::default(),
        point_size: 16,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn color_probe_retains_same_shader_geometry_and_linear_inputs_for_both_modes() {
        for mode in [
            AspectTypeOfDisplayText::Blend,
            AspectTypeOfDisplayText::Normal,
        ] {
            let frame = color_probe_frame([0.18, 0.4, 0.7, 1.], mode);
            assert_eq!(frame.draws()[0].vertices()[0].color, [0.18, 0.4, 0.7, 1.]);
            assert_eq!(frame.draws()[0].indices().len(), 6);
            assert!(frame.draws()[0].textured_style().is_some());
        }
    }
}

fn emit_probe_colors(index: u32, native: &[f32], colors: &[[u8; 4]]) {
    println!(
        "MARTENSITE_COLOR={{\"index\":{index},\"native\":{native:?},\"xor\":{:?},\"normal_srgb_xor\":{:?}}}",
        colors[0], colors[1]
    );
}
