//! Behaviour tests of the Sweep package: the indexation of a directing edge, its
//! iteration and the native quirks docs/tkprim-progress.md lists.
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkprim::sweep::{SweepNumShape, SweepNumShapeIterator, SweepNumShapeTool};

#[test]
fn a_finite_edge_indexes_itself_and_both_vertices() {
    let edge = SweepNumShape::new(2, ShapeEnum::Edge);
    let tool = SweepNumShapeTool::new(&edge);
    assert_eq!(tool.nb_shapes(), 3);
    assert_eq!(tool.index(&edge), 1);
    assert_eq!(tool.shape(1), edge);
    assert_eq!(tool.first_vertex().unwrap().index(), 1);
    assert_eq!(tool.last_vertex().unwrap().index(), 2);
    assert_eq!(tool.index(&tool.last_vertex().unwrap()), 3);
}

#[test]
fn a_closed_edge_shares_its_vertex() {
    let edge = SweepNumShape::with_flags(2, ShapeEnum::Edge, true, false, false);
    let tool = SweepNumShapeTool::new(&edge);
    assert_eq!(tool.nb_shapes(), 2);
    let last = tool.last_vertex().unwrap();
    assert_eq!(last.index(), 1);
    assert_eq!(tool.index(&last), 2);
}

#[test]
fn an_infinite_end_has_no_vertex() {
    let edge = SweepNumShape::with_flags(1, ShapeEnum::Edge, false, false, true);
    let tool = SweepNumShapeTool::new(&edge);
    assert!(tool.has_first_vertex());
    assert!(!tool.has_last_vertex());
    let error = tool.last_vertex().unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
}

#[test]
fn vertices_are_forward_then_reversed() {
    let mut it = SweepNumShapeIterator::new();
    it.init(&SweepNumShape::new(2, ShapeEnum::Edge));
    let mut orientations = Vec::new();
    while it.more() {
        orientations.push(it.orientation());
        it.next();
    }
    assert_eq!(orientations, [Orientation::Forward, Orientation::Reversed]);
}

#[test]
fn the_lone_vertex_of_an_edge_infinite_at_its_beginning_is_reversed() {
    let mut it = SweepNumShapeIterator::new();
    it.init(&SweepNumShape::with_flags(
        1,
        ShapeEnum::Edge,
        false,
        true,
        false,
    ));
    assert!(it.more());
    assert_eq!(it.orientation(), Orientation::Reversed);
}

#[test]
fn init_on_a_non_edge_keeps_the_current_iteration() {
    let mut it = SweepNumShapeIterator::new();
    it.init(&SweepNumShape::new(2, ShapeEnum::Edge));
    it.init(&SweepNumShape::new(1, ShapeEnum::Vertex));
    assert!(it.more());
    assert_eq!(it.value().index(), 1);
    it.next();
    assert!(!it.more());
}

#[test]
fn vertex_orientation_depends_on_its_index() {
    assert_eq!(
        SweepNumShape::new(2, ShapeEnum::Vertex).orientation(),
        Orientation::Forward
    );
    assert_eq!(
        SweepNumShape::new(1, ShapeEnum::Vertex).orientation(),
        Orientation::Reversed
    );
    assert_eq!(
        SweepNumShape::default().orientation(),
        Orientation::Reversed
    );
}
