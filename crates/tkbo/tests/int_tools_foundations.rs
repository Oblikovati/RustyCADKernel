//! Focused tests of every IntTools foundation operation; differential cases live in the probe.
use rustycad_tkbo::int_tools::*;
use rustycad_tkg3d::top_abs::State;
#[test]
fn range_bounds_preserve_order_and_signed_zero() {
    assert_eq!(Range::default().range(), (0.0, 0.0));
    let mut range = Range::new(2.0, -1.0);
    assert_eq!(range.range(), (2.0, -1.0));
    range.set_first(-0.0);
    range.set_last(7.0);
    assert_eq!(range.first().to_bits(), (-0.0f64).to_bits());
    assert_eq!(range.last(), 7.0);
}
#[test]
fn root_stores_interval_and_classifies_all_state_pairs() {
    let mut root = Root::default();
    assert_eq!(root.root_type(), -1);
    assert!(!root.is_valid());
    assert_eq!(root.interval(), (0.0, 0.0, 0.0, 0.0));
    root = Root::new(0.5, 2);
    assert_eq!(root.root(), 0.5);
    assert_eq!(root.root_type(), 2);
    root.set_root(-0.5);
    root.set_type(1);
    root.set_layer_height(0.125);
    assert_eq!(root.root(), -0.5);
    assert_eq!(root.root_type(), 1);
    assert_eq!(root.layer_height(), 0.125);
    root.set_interval(-1.0, 2.0, -3.0, 4.0);
    assert_eq!(root.interval(), (-1.0, 2.0, -3.0, 4.0));
}
#[test]
fn root_classifies_all_state_pairs() {
    let mut root = Root::default();
    for before in State::ALL {
        for after in State::ALL {
            root.set_state_before(before);
            root.set_state_after(after);
            assert_eq!((root.state_before(), root.state_after()), (before, after));
            assert_eq!(
                root.is_valid(),
                matches!(
                    (before, after),
                    (State::Out, State::In | State::On) | (State::In | State::On, State::Out)
                )
            );
        }
    }
}
#[test]
fn curve_subdivision_and_equality_include_depth() {
    let mut base = BaseRangeSample::default();
    assert_eq!(base.get_depth(), 0);
    base = BaseRangeSample::new(-2);
    base.set_depth(3);
    assert_eq!(base.get_depth(), 3);
}
#[test]
fn curve_subdivision_preserves_native_bounds() {
    let mut sample = CurveRangeSample::new(1);
    assert_eq!(sample.get_range_index(), 1);
    sample.set_depth(2);
    assert_eq!(sample.get_range(0.0, 16.0, 2).range(), (4.0, 8.0));
    assert_eq!(sample.get_range_index_deeper(3), 3);
    assert!(sample.is_equal(&sample));
    assert_ne!(sample, CurveRangeSample::default());
    sample.set_range_index(0);
    assert_eq!(sample.get_range(0.0, 16.0, 2).range(), (0.0, 4.0));
    sample.set_depth(-1);
    assert_eq!(sample.get_range(7.0, -1.0, 0).range(), (7.0, -1.0));
}
#[test]
fn curve_hash_uses_native_depth_and_index_bytes() {
    let mut sample = CurveRangeSample::default();
    sample.set_depth(-1);
    let bytes: Vec<u8> = [-1i32, 0].into_iter().flat_map(i32::to_ne_bytes).collect();
    assert_eq!(
        sample.hash_code(),
        rustycad_tkernel::standard::hash_bytes(&bytes)
    );
}
#[test]
fn surface_samples_copy_and_assign_independently() {
    let sample = SurfaceRangeSample::new(1, 2, 3, 4);
    assert_eq!(sample.get_indexes(), (1, 3));
    assert_eq!(sample.get_depths(), (2, 4));
    let (u, v) = sample.get_ranges();
    assert_eq!(sample.get_sample_range_u(), &u);
    assert_eq!(sample.get_sample_range_v(), &v);
    let mut copy = SurfaceRangeSample::from_ranges(v, u);
    assert!(!copy.is_equal(&sample));
    copy.assign(&sample);
    assert_eq!(copy, sample);
    copy.set_ranges(v, u);
    copy.set_sample_range_u(u);
    copy.set_sample_range_v(v);
    assert_eq!(copy, sample);
}
#[test]
fn surface_indexes_and_depths_are_independent() {
    let mut sample = SurfaceRangeSample::default();
    sample.set_indexes(2, 1);
    sample.set_index_u(3);
    sample.set_index_v(2);
    sample.set_depth_u(2);
    sample.set_depth_v(1);
    assert_eq!((sample.get_index_u(), sample.get_index_v()), (3, 2));
    assert_eq!((sample.get_depth_u(), sample.get_depth_v()), (2, 1));
    assert_eq!(
        (
            sample.get_range_index_u_deeper(2),
            sample.get_range_index_v_deeper(3)
        ),
        (6, 6)
    );
}
#[test]
fn surface_ranges_use_each_axis_sample() {
    let sample = SurfaceRangeSample::new(3, 2, 2, 1);
    assert_eq!(sample.get_range_u(0.0, 16.0, 2).range(), (12.0, 16.0));
    assert_eq!(sample.get_range_v(0.0, 9.0, 3).range(), (6.0, 9.0));
    let (u, v) = sample.get_ranges();
    let bytes: Vec<u8> = [u.hash_code(), v.hash_code()]
        .into_iter()
        .flat_map(usize::to_ne_bytes)
        .collect();
    assert_eq!(
        sample.hash_code(),
        rustycad_tkernel::standard::hash_bytes(&bytes)
    );
}
