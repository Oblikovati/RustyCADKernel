//! Behavior tests of ShapeBuild_ReShape and ShapeFix_Shape from the upstream
//! GTests/ShapeBuild_ReShape_Test.cxx.
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tkshhealing::shape_build::ShapeBuildReShape;
use rustycad_tkshhealing::shape_fix::ShapeFixShape;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape};

#[path = "support/reshape.rs"]
mod reshape_support;
use reshape_support::{compound, vertex};

fn subshape_count(shape: &Shape, kind: ShapeEnum) -> usize {
    let mut mapped = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(shape, kind, &mut mapped).unwrap();
    mapped.extent()
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
#[allow(non_snake_case)]
fn ShapeFix_ShapeStabilityTest__RepeatedPerformDoesNotMultiplyEdges() {
    let mut box_maker = BRepPrimAPIMakeBox::from_sizes(10.0, 10.0, 10.0).unwrap();
    let initial = box_maker.shape().unwrap().clone();
    let expected = [
        subshape_count(&initial, ShapeEnum::Edge),
        subshape_count(&initial, ShapeEnum::Vertex),
        subshape_count(&initial, ShapeEnum::Face),
    ];
    assert_eq!(expected[0], 12);

    let mut current = initial;
    for iteration in 0..5 {
        let fixer = ShapeFixShape::new(current);
        fixer.perform().unwrap();
        current = fixer.shape();
        assert!(!current.is_null(), "iteration {iteration}");
        assert_eq!(subshape_count(&current, ShapeEnum::Face), expected[2]);
        assert_eq!(subshape_count(&current, ShapeEnum::Edge), expected[0]);
        assert_eq!(subshape_count(&current, ShapeEnum::Vertex), expected[1]);
    }
}
