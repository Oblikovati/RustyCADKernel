//! The frame seam rejects incomplete stencil state before GPU work.
use martensite_vulkan::*;
fn triangle(pass: RenderCappingPass) -> PrimitiveDraw {
    PrimitiveDraw::new(
        vec![RenderVertex::new([0.; 3], [1.; 4]); 3],
        vec![0, 1, 2],
        identity_clip(),
    )
    .unwrap()
    .with_capping_pass(pass)
    .unwrap()
}
#[test]
fn complete_masks_and_ordinary_draws_preserve_their_order() {
    use RenderCappingPass::*;
    let phases = [
        None,
        MaskStart,
        MaskContinue,
        Surface,
        None,
        MaskStart,
        Surface,
    ];
    let frame = RenderFrame::new(4, 4, [0.; 4], phases.map(triangle).to_vec()).unwrap();
    assert_eq!(
        frame
            .draws()
            .iter()
            .map(PrimitiveDraw::capping_pass)
            .collect::<Vec<_>>(),
        phases
    );
}
#[test]
fn invalid_capping_order_fails_before_backend_submission() {
    use RenderCappingPass::*;
    for phases in [
        vec![Surface],
        vec![MaskContinue],
        vec![MaskStart],
        vec![MaskStart, None],
        vec![MaskStart, MaskStart, Surface],
    ] {
        assert!(
            RenderFrame::new(4, 4, [0.; 4], phases.into_iter().map(triangle).collect()).is_err()
        );
    }
    let layers = vec![
        RenderLayer::new(vec![triangle(MaskStart)], true, true, false),
        RenderLayer::new(vec![triangle(Surface)], true, true, false),
    ];
    assert!(RenderFrame::with_layers(4, 4, [0.; 4], layers).is_err());
}
#[test]
fn capping_cannot_accidentally_mask_lines() {
    let line = PrimitiveDraw::segments(
        vec![RenderVertex::new([0.; 3], [1.; 4]); 2],
        vec![0, 1],
        identity_clip(),
        LineStyle::new(1., 0xffff, 1).unwrap(),
    )
    .unwrap();
    assert!(
        line.clone()
            .with_capping_pass(RenderCappingPass::MaskStart)
            .is_err()
    );
    assert!(line.with_capping_pass(RenderCappingPass::None).is_ok());
}
