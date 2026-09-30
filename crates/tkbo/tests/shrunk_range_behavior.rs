//! ShrunkRange sentinel state, shared shape mutation and retained native history.
use rustycad_tkbo::int_tools::{Context, ShrunkRange};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topexp,
    topods::{Edge, Vertex},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
fn edge(length: f64) -> (Edge, Vertex, Vertex) {
    let edge = BRepBuilderAPIMakeEdge::from_points(Point3::default(), Point3::new(length, 0., 0.))
        .unwrap()
        .edge()
        .unwrap();
    let (first, last) = topexp::vertices(&edge, false);
    (edge, first, last)
}
fn selected(length: f64) -> (ShrunkRange, Vertex, Vertex) {
    let (edge, first, last) = edge(length);
    let mut range = ShrunkRange::new();
    range.set_data(&edge, 0., length, &first, &last);
    (range, first, last)
}
#[test]
fn default_range_has_native_sentinels_and_accepts_null_context() {
    let mut range = ShrunkRange::default();
    assert_eq!(range.shrunk_range(), (-99., -99.));
    assert!(range.bounding_box().is_void());
    assert!(range.edge().is_null());
    range.perform().unwrap();
    assert!(!range.is_done());
    assert!(!range.is_splittable());
    assert_eq!(range.length(), 0.);
    let context = Handle::new(Context::default());
    range.set_context(&context);
    assert!(range.context() == &context);
    range.set_context(&Handle::default());
    assert!(range.context().is_null());
}
#[test]
fn computed_interval_excludes_endpoint_tolerance_spheres() {
    let (mut range, _, _) = selected(10.);
    range.perform().unwrap();
    assert!(range.is_done());
    assert!(range.is_splittable());
    let (first, last) = range.shrunk_range();
    assert!(first > 0. && first < 1e-6);
    assert!(last < 10. && last > 10. - 1e-6);
    assert!((range.length() - (last - first)).abs() < 1e-14);
    let saved = range.shrunk_range();
    range.perform().unwrap();
    assert_eq!(range.shrunk_range(), saved);
}
#[test]
fn set_data_resets_flags_and_length_but_retains_range_and_box() {
    let (mut range, _, _) = selected(10.);
    range.perform().unwrap();
    let saved = range.shrunk_range();
    let bounds = range.bounding_box().get().unwrap();
    let (edge, first, last) = edge(2.);
    range.set_data(&edge, 0., 0., &first, &last);
    assert!(!range.is_done());
    assert!(!range.is_splittable());
    assert_eq!(range.length(), 0.);
    assert_eq!(range.shrunk_range(), saved);
    assert_eq!(range.bounding_box().get().unwrap(), bounds);
    assert!(range.edge().is_same(&edge));
    range.perform().unwrap();
    assert_eq!(range.shrunk_range(), saved);
}
#[test]
fn manual_range_accumulates_box_without_changing_completion_flags() {
    let (mut range, _, _) = selected(10.);
    range.set_shrunk_range(-2., 12.).unwrap();
    assert!(!range.is_done());
    assert_eq!(range.shrunk_range(), (-2., 12.));
    let bounds = range.bounding_box().get().unwrap();
    range.set_shrunk_range(2., 3.).unwrap();
    assert_eq!(range.bounding_box().get().unwrap(), bounds);
    assert_eq!(range.shrunk_range(), (2., 3.));
}
#[test]
fn failed_repeat_after_shared_vertex_growth_retains_previous_length() {
    let (mut range, first, _) = selected(10.);
    range.perform().unwrap();
    let length = range.length();
    BRepBuilder::default()
        .update_vertex_tolerance(&first, 100.)
        .unwrap();
    range.perform().unwrap();
    assert!(!range.is_done());
    assert!(!range.is_splittable());
    assert_eq!(range.length(), length);
}
#[test]
fn a_completed_micro_interval_can_remain_unsplittable() {
    let (mut range, _, _) = selected(7e-7);
    range.perform().unwrap();
    assert!(range.is_done());
    assert!(!range.is_splittable());
    assert!(range.length() > 1e-7);
}
