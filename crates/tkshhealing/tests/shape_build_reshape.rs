//! Behavior tests of ShapeBuild_ReShape. The first three run the ShapeBuild_ReShapeTest
//! cases of the upstream GTests/ShapeBuild_ReShape_Test.cxx on the same shapes (the suite
//! is not mapped whole: its fourth case needs BRepPrimAPI_MakeBox and ShapeFix_Shape).
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Compound, Shape, ShapeIterator, Vertex};
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tkshhealing::shape_build::ShapeBuildReShape;
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeVertex};

fn vertex(x: f64) -> Vertex {
    BRepBuilderAPIMakeVertex::new(Point3::new(x, 0.0, 0.0))
        .unwrap()
        .vertex()
        .unwrap()
}

fn compound(items: &[&Shape]) -> Shape {
    let builder = BRepBuilder::default();
    let mut c = Compound::default();
    builder.make_compound(&mut c);
    for item in items {
        builder.add(&mut c, item).unwrap();
    }
    c.into_shape()
}

#[test]
fn reshape_test_apply_performs_vertex_replacement_on_compound() {
    let (v1, v2, v2_repl) = (vertex(0.0), vertex(1.0), vertex(1.5));
    let parent = compound(&[&v1, &v2]);
    let reshape = Handle::new(ShapeBuildReShape::new());
    reshape.replace(&v2, &v2_repl).unwrap();
    let result = reshape.apply(&parent, ShapeEnum::Shape).unwrap();
    assert!(!result.is_null());
    let mut found = false;
    let mut it = ShapeIterator::new(&result, true, true);
    while it.more() {
        let item = it.value().unwrap();
        found |= item.tshape_address() == v2_repl.tshape_address();
        assert!(item.tshape_address() != v2.tshape_address());
        it.next();
    }
    assert!(found);
}

#[test]
fn reshape_test_apply_structural_containment_without_crash() {
    let (v1, v2, v3) = (vertex(0.0), vertex(1.0), vertex(2.0));
    let edge = BRepBuilderAPIMakeEdge::from_vertices(&v1, &v2)
        .unwrap()
        .edge()
        .unwrap();
    let container = compound(&[&edge, &v3]);
    let reshape = Handle::new(ShapeBuildReShape::new());
    reshape.replace(&edge, &container).unwrap();
    let result = reshape.apply(&edge, ShapeEnum::Shape).unwrap();
    assert!(!result.is_null());
}

#[test]
fn reshape_test_apply_diamond_shared_vertex_in_two_compounds() {
    let (shared, shared_new) = (vertex(0.0), vertex(0.1));
    let (other1, other2) = (vertex(1.0), vertex(2.0));
    let parent_a = compound(&[&shared, &other1]);
    let parent_b = compound(&[&shared, &other2]);
    let grand = compound(&[&parent_a, &parent_b]);
    let reshape = Handle::new(ShapeBuildReShape::new());
    reshape.replace(&shared, &shared_new).unwrap();
    let result = reshape.apply(&grand, ShapeEnum::Shape).unwrap();
    assert!(!result.is_null());
    let (mut replaced, mut originals) = (0, 0);
    let mut explorer = Explorer::new(&result, ShapeEnum::Vertex, ShapeEnum::Shape).unwrap();
    while explorer.more() {
        let address = explorer.current().tshape_address();
        replaced += i32::from(address == shared_new.tshape_address());
        originals += i32::from(address == shared.tshape_address());
        explorer.next().unwrap();
    }
    assert_eq!(
        originals, 0,
        "Original vertex must be substituted everywhere"
    );
    assert_eq!(
        replaced, 2,
        "Replacement must appear in both parentA and parentB"
    );
}

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
fn a_sub_shape_replaced_by_a_shape_of_another_type_is_ignored_with_fail1() {
    let (v1, v2) = (vertex(0.0), vertex(1.0));
    let edge = BRepBuilderAPIMakeEdge::from_vertices(&v1, &v2)
        .unwrap()
        .edge()
        .unwrap();
    let builder = BRepBuilder::default();
    let mut wire = rustycad_tkbrep::topods::Wire::default();
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
