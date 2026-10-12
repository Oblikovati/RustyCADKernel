//! Real GPU LOAD/color/depth contracts across XOR and ordinary pipeline transitions.
use martensite_vulkan::*;
use rustycad_tkernel::quantity::ColorRgba;
use rustycad_tkservice::{aspect::*, font::FontRect, graphic3d::*};
pub fn check_blend_boundaries(renderer: &mut VulkanRenderer) {
    let xor = fullscreen_xor();
    for kind in ["ordinary", "lit", "atlas"] {
        let later = boundary_surface(kind);
        let actual =
            RenderFrame::new(64, 64, [0., 0., 1., 1.], vec![xor.clone(), later.clone()]).unwrap();
        let control = RenderFrame::new(64, 64, [1., 1., 0., 1.], vec![later]).unwrap();
        assert_capture_identity(renderer, &actual, &control, kind);
    }
    let double = RenderFrame::new(64, 64, [0., 0., 1., 1.], vec![xor.clone(), xor]).unwrap();
    check_layered_blend(renderer);
    check_capped_blend(renderer);
    let empty = RenderFrame::new(64, 64, [0., 0., 1., 1.], vec![]).unwrap();
    assert_capture_identity(renderer, &double, &empty, "repeated XOR cancellation");
}
/// Verify ordered attachment transitions preserve both buffers; `assert_capture_identity(renderer, actual, control, "surface")`.
pub fn assert_capture_identity(
    renderer: &mut VulkanRenderer,
    actual: &RenderFrame,
    control: &RenderFrame,
    label: &str,
) {
    let capture = renderer.render(actual).unwrap();
    let expected = renderer.render(control).unwrap();
    assert_eq!(
        capture.rgba, expected.rgba,
        "{label}: XOR↔ordinary color LOAD"
    );
    assert_eq!(
        capture.depth, expected.depth,
        "{label}: XOR↔ordinary depth LOAD"
    );
}
/// Lower genuine full-viewport pixel text for attachment boundaries; `let draw = fullscreen_xor()`.
pub fn fullscreen_xor() -> PrimitiveDraw {
    let text = Graphic3dText::new(16.);
    let aspect = Graphic3dAspectText3d::default();
    aspect.set_text_display_type(AspectTypeOfDisplayText::Blend);
    aspect.set_interior_color_rgba(ColorRgba::new(1., 1., 1., 1.).unwrap());
    let layout = GlyphLayout {
        positions: vec![[-1., -1.], [1., -1.], [1., 1.], [-1., 1.]],
        uv: vec![[0.; 2]; 4],
        indices: vec![0, 1, 2, 0, 2, 3],
        atlas: RasterImage::new([1, 1], vec![[1.; 4]]).unwrap(),
        formatter_bounds: FontRect::default(),
        point_size: 16,
    };
    lower_pixel_text(&text, &aspect, &layout, &PixelTextView::default())
        .unwrap()
        .remove(0)
}
fn boundary_surface(kind: &str) -> PrimitiveDraw {
    let vertices = [[-0.8, -0.8, -0.5], [0.8, -0.8, -0.5], [0., 0.8, -0.5]]
        .map(|p| RenderVertex::new(p, [0.18, 0.4, 0.7, 1.]));
    let draw = PrimitiveDraw::new(vertices.to_vec(), vec![0, 1, 2], identity_clip())
        .unwrap()
        .with_polygon_offset(RenderPolygonOffset::default());
    if kind == "atlas" {
        return boundary_atlas(vertices.to_vec());
    }
    if kind == "lit" {
        return boundary_lit(draw);
    }
    draw
}
fn boundary_lit(draw: PrimitiveDraw) -> PrimitiveDraw {
    let mut material = RenderMaterial::default();
    for face in &mut material.common {
        face.emission = [0.18, 0.4, 0.7, 0.];
    }
    let lighting = RenderLighting::new(
        vec![[0., 0., 1.]; 3],
        material,
        identity_clip(),
        identity_clip(),
        false,
        RenderShading::Phong,
    )
    .unwrap();
    draw.with_lighting(lighting).unwrap()
}
fn check_layered_blend(renderer: &mut VulkanRenderer) {
    for depth_write in [false, true] {
        let layers = vec![
            RenderLayer::new(vec![fullscreen_xor()], true, depth_write, true),
            RenderLayer::new(vec![boundary_surface("ordinary")], true, depth_write, false),
        ];
        let actual = RenderFrame::with_layers(64, 64, [0., 0., 1., 1.], layers).unwrap();
        let layer = RenderLayer::new(vec![boundary_surface("ordinary")], true, depth_write, true);
        let control = RenderFrame::with_layers(64, 64, [1., 1., 0., 1.], vec![layer]).unwrap();
        assert_capture_identity(renderer, &actual, &control, "layer depth write override");
    }
}

fn check_capped_blend(renderer: &mut VulkanRenderer) {
    let mut actual = vec![fullscreen_xor()];
    actual.extend(boundary_cap([1., 0., 0., 1.]));
    actual.push(fullscreen_xor());
    let actual = RenderFrame::new(64, 64, [0., 0., 1., 1.], actual).unwrap();
    let control =
        RenderFrame::new(64, 64, [0., 0., 1., 1.], boundary_cap([0., 1., 1., 1.])).unwrap();
    assert_capture_identity(
        renderer,
        &actual,
        &control,
        "XOR↔capping stencil/depth LOAD",
    );
}
fn boundary_cap(color: [f32; 4]) -> Vec<PrimitiveDraw> {
    super::capping_runtime::batch(1, color)
        .into_iter()
        .map(|draw| draw.with_polygon_offset(RenderPolygonOffset::default()))
        .collect()
}

fn boundary_atlas(vertices: Vec<RenderVertex>) -> PrimitiveDraw {
    let image = RasterImage::new([1, 1], vec![[1.; 4]]).unwrap();
    let style = TexturedStyle::new(image, vec![[0.; 2]; 3], 0.285).unwrap();
    PrimitiveDraw::textured(vertices, vec![0, 1, 2], identity_clip(), style)
        .unwrap()
        .with_polygon_offset(RenderPolygonOffset::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn boundary_geometry_preserves_expected_native_depth_and_distinct_pipelines() {
        let xor = fullscreen_xor();
        assert_eq!(xor.indices().len(), 6);
        assert!(boundary_surface("atlas").textured_style().is_some());
        assert!(boundary_surface("lit").lighting().is_some());
        assert!(boundary_surface("ordinary").textured_style().is_none());
        assert_eq!(
            boundary_cap([1.; 4])[0].capping_pass(),
            RenderCappingPass::MaskStart
        );
    }
}
