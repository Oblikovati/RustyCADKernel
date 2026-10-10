//! Immutable draw clipping preserves geometry and can be explicitly cleared.
use martensite_vulkan::*;
#[test]
fn attaching_clipping_preserves_geometry_and_empty_volume_removes_it() {
    let vertices = vec![RenderVertex::new([0.; 3], [1.; 4]); 3];
    let draw = PrimitiveDraw::new(vertices.clone(), vec![0, 1, 2], identity_clip()).unwrap();
    assert!(draw.clipping().is_none());
    let clipping = RenderClipping::new(vec![vec![[1., 0., 0., 0.]]], identity_clip()).unwrap();
    let draw = draw.with_clipping(clipping.clone());
    assert_eq!(draw.clipping(), Some(&clipping));
    assert_eq!(draw.vertices(), vertices);
    assert!(
        draw.with_clipping(RenderClipping::new(vec![], identity_clip()).unwrap())
            .clipping()
            .is_none()
    );
}
