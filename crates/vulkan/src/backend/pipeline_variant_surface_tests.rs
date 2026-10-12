use super::*;
#[test]
fn surface_mask_blend_keeps_ordered_depth_and_independent_capping_identity() {
    let mut draw = PrimitiveDraw::new(vec![], vec![], crate::identity_clip()).unwrap();
    draw.surface_cutoff = Some(0.5);
    draw.surface_forced_blend = true;
    let ordinary = special_variant_key(&draw);
    let kind = special_variant(&draw, true);
    assert!(kind.surface_alpha && kind.surface_blend && !draw.surface_blend);
    draw.capping_pass = RenderCappingPass::Surface;
    assert_ne!(ordinary, special_variant_key(&draw));
    assert_eq!(
        special_variant(&draw, true).capping,
        RenderCappingPass::Surface
    );
}
#[test]
fn unsupported_special_line_and_clamp_features_fail_before_pipeline_creation() {
    let mut kind = PipelineVariant::from_index(32, false);
    assert!(validate_special_features(kind, true, false).is_err());
    assert!(validate_special_features(kind, true, true).is_ok());
    kind.clamp = true;
    assert!(validate_special_features(kind, false, true).is_err());
}
