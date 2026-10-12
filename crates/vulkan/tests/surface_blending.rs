//! Transparent geometry retains submission order and cannot enter opaque-only paths.
use martensite_vulkan::*;
fn colored_surface(rgb: [f32; 3], alpha: f32) -> PrimitiveDraw {
    let color = [rgb[0], rgb[1], rgb[2], alpha];
    PrimitiveDraw::new(
        vec![RenderVertex::new([0.; 3], color); 3],
        vec![0, 1, 2],
        identity_clip(),
    )
    .unwrap()
}
#[test]
fn transparent_surfaces_follow_opaque_geometry_without_distance_sorting() {
    let green = colored_surface([0., 1., 0.], 0.5);
    let red = colored_surface([1., 0., 0.], 1.);
    let blue = colored_surface([0., 0., 1.], 0.2);
    let layer = RenderLayer::new(
        vec![green.clone(), red.clone(), blue.clone()],
        true,
        true,
        true,
    );
    let frame = RenderFrame::with_layers(16, 16, [0.; 4], vec![layer]).unwrap();
    assert_eq!(frame.draws(), &[red, green, blue]);
}
#[test]
fn transparent_cap_batches_stay_together_and_lines_keep_their_own_policy() {
    let draw = colored_surface([1., 0., 0.], 0.5);
    let mask = draw
        .clone()
        .with_capping_pass(RenderCappingPass::MaskStart)
        .unwrap();
    let surface = draw
        .clone()
        .with_capping_pass(RenderCappingPass::Surface)
        .unwrap();
    let frame = RenderFrame::new(16, 16, [0.; 4], vec![mask, surface]).unwrap();
    assert_eq!(
        frame
            .draws()
            .iter()
            .map(PrimitiveDraw::capping_pass)
            .collect::<Vec<_>>(),
        [RenderCappingPass::MaskStart, RenderCappingPass::Surface]
    );
    assert!(frame.draws().iter().all(PrimitiveDraw::is_surface_blended));
    let line = PrimitiveDraw::segments(
        draw.vertices()[..2].to_vec(),
        vec![0, 1],
        identity_clip(),
        LineStyle::new(1., 0xffff, 1).unwrap(),
    )
    .unwrap();
    assert!(!line.is_surface_blended());
    assert!(!line.line_style().unwrap().alpha_blending());
}

#[test]
fn transparent_layers_flush_before_depth_clear_and_after_opaque_layers() {
    let green = colored_surface([0., 1., 0.], 0.5);
    let red = colored_surface([1., 0., 0.], 1.);
    let blue = colored_surface([0., 0., 1.], 1.);
    let layers = vec![
        RenderLayer::new(vec![green.clone()], true, true, true),
        RenderLayer::new(vec![red.clone()], true, true, false),
        RenderLayer::new(vec![blue.clone()], true, true, true),
    ];
    let frame = RenderFrame::with_layers(16, 16, [0.; 4], layers).unwrap();
    assert_eq!(frame.draws(), &[red, green, blue]);
    assert_eq!(frame.layers()[2].depth_policy(), (true, false, false));
    assert_eq!(frame.layers()[3].depth_policy(), (true, true, true));
}

#[test]
fn translucent_points_and_text_do_not_silently_enter_surface_pipelines() {
    let vertices = colored_surface([1., 0., 0.], 0.5).vertices().to_vec();
    let point = PointStyle::new(1., [1, 1], vec![[1.; 4]]).unwrap();
    let draw =
        PrimitiveDraw::points(vertices.clone(), vec![0, 1, 2], identity_clip(), point).unwrap();
    assert!(!draw.is_surface_blended());
    assert!(draw.point_style().unwrap().alpha_blending());
    let image = RasterImage::new([1, 1], vec![[1.; 4]]).unwrap();
    let style = TexturedStyle::new(image, vec![[0.; 2]; 3], 0.1).unwrap();
    let text = PrimitiveDraw::textured(vertices, vec![0, 1, 2], identity_clip(), style).unwrap();
    assert!(!text.is_surface_blended());
}
