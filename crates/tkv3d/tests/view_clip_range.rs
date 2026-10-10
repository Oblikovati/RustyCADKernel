//! Exact clipping-depth behavior, including output values on an unsuccessful lookup.
use rustycad_tkmath::bnd::Range1;
use rustycad_tkv3d::select_mgr::SelectMgrViewClipRange;

#[test]
fn void_query_preserves_the_callers_depth_like_native_get_min() {
    let mut clip = SelectMgrViewClipRange::default();
    let mut depth = 123.0;
    assert!(!clip.get_nearest_depth_into(&Range1::default(), &mut depth));
    *clip.change_unclip_range() = Range1::default();
    assert!(clip.get_nearest_depth_into(&Range1::default(), &mut depth));
    assert_eq!(depth, 123.0);
}

#[test]
fn clipped_lookup_reports_its_boundary_even_when_outside_query() {
    let mut clip = SelectMgrViewClipRange::default();
    clip.add_clip_sub_range(Range1::new(0.0, 5.0).unwrap());
    let query = Range1::new(1.0, 2.0).unwrap();
    let mut depth = 123.0;
    assert!(!clip.get_nearest_depth_into(&query, &mut depth));
    assert_eq!(depth, 5.0);
    assert_eq!(clip.get_nearest_depth(&query), None);
}

#[test]
fn clipping_subranges_merge_in_native_single_pass_order() {
    let mut clip = SelectMgrViewClipRange::default();
    for (first, last) in [(0., 2.), (4., 6.), (1., 5.)] {
        clip.add_clip_sub_range(Range1::new(first, last).unwrap());
    }
    assert_eq!(
        clip.get_nearest_depth(&Range1::new(1., 10.).unwrap()),
        Some(5.)
    );
    assert!(clip.is_clipped(5.0));
    clip.set_void();
    assert!(!clip.is_clipped(5.0));
    clip.change_unclip_range().trim_from(7.0);
    assert_eq!(
        clip.get_nearest_depth(&Range1::new(1., 10.).unwrap()),
        Some(7.)
    );
}
