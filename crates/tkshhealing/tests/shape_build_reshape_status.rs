//! Supplemental ShapeBuild_ReShape status and type-change behavior tests.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::Wire;
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkshhealing::shape_build::ShapeBuildReShape;
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

#[path = "support/reshape.rs"]
mod reshape_support;
use reshape_support::{compound, vertex};

#[test]
fn apply_records_the_status_flags_of_the_last_call() {
    let (v1, v2, v3) = (vertex(0.0), vertex(1.0), vertex(2.0));
    let parent = compound(&[&v1, &v2]);
    let reshape = ShapeBuildReShape::new();
    reshape.remove(&v2).unwrap();
    let result = reshape.apply(&parent, ShapeEnum::Shape).unwrap();
    assert_eq!(result.nb_children(), 1);
    assert!(reshape.status_flag(ShapeExtendStatus::Done3));
    assert!(reshape.status_flag(ShapeExtendStatus::Done4));
    reshape.replace(&v1, &v3).unwrap();
    reshape.apply(&v1, ShapeEnum::Shape).unwrap();
    assert!(reshape.status_flag(ShapeExtendStatus::Done1));
    assert!(Handle::new(ShapeBuildReShape::new()).is_kind_name("BRepTools_ReShape"));
    assert_eq!(
        ShapeBuildReShape::static_type().name(),
        "ShapeBuild_ReShape"
    );
}

#[test]
fn wrong_type_replacement_is_ignored_with_fail1() {
    let (v1, v2) = (vertex(0.0), vertex(1.0));
    let edge = BRepBuilderAPIMakeEdge::from_vertices(&v1, &v2)
        .unwrap()
        .edge()
        .unwrap();
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    builder.add(&mut wire, &edge).unwrap();
    let reshape = ShapeBuildReShape::new();
    // The edge is replaced by a compound of a vertex, which is not an edge.
    reshape.replace(&edge, &compound(&[&vertex(3.0)])).unwrap();
    let result = reshape.apply(&wire, ShapeEnum::Shape).unwrap();
    assert_eq!(result.nb_children(), 0);
    assert!(reshape.status_flag(ShapeExtendStatus::Fail1));
    // The build-mode Apply returns the recorded replacement of a compound item as is.
    let holder = compound(&[&edge]);
    let rebuilt = reshape
        .apply_with_build_mode(&holder, ShapeEnum::Face, 0)
        .unwrap();
    assert_eq!(rebuilt.nb_children(), 1);
}
