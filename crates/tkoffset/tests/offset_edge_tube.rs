//! Native differential for the free-boundary edge tube initializer.
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::{Explorer, vertices},
    topods::{Edge, Shape},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::Point3;
use rustycad_tkoffset::brep_offset::{BRepOffsetOffset, BRepOffsetStatus};
use rustycad_tktopalgo::{
    brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape},
    brep_check::BRepCheckAnalyzer,
    brep_gprop::BRepGProp,
};

#[test]
fn edge_tube_initializer_matches_native_positive_and_negative_modes() -> Result<(), Failure> {
    let path = straight_edge()?;
    assert_tube(&path, 1.0, Orientation::Forward)?;
    assert_tube(&path, -1.0, Orientation::Reversed)
}

fn assert_tube(path: &Edge, radius: f64, orientation: Orientation) -> Result<(), Failure> {
    let mut offset = BRepOffsetOffset::new();
    offset.init_edge_tube(path, radius)?;
    let face = offset.face();
    let shape = face.clone().into_shape();
    assert_eq!(offset.status(), BRepOffsetStatus::Good);
    assert!(offset.initial_shape().is_same(&path.clone().into_shape()));
    assert_eq!(face.orientation(), orientation);
    assert_eq!(count(&shape, ShapeEnum::Edge)?, 4);
    assert_eq!(count(&shape, ShapeEnum::Vertex)?, 8);
    assert!(BRepCheckAnalyzer::new(&shape, true, false, false)?.is_valid()?);
    assert_generated_edge_indices(&offset, path)?;
    assert_tube_area(&shape)?;
    assert!((BRepTool::face_tolerance(face)? - 1.0e-7).abs() < 1.0e-15);
    Ok(())
}

fn assert_generated_edge_indices(offset: &BRepOffsetOffset, path: &Edge) -> Result<(), Failure> {
    let (first, last) = vertices(path, false);
    assert_generated_index(offset, &first.into_shape(), Some(3))?;
    assert_generated_index(offset, &last.into_shape(), Some(0))?;
    assert_generated_index(offset, &path.clone().into_shape(), Some(3))
}

fn assert_generated_index(
    offset: &BRepOffsetOffset,
    source: &Shape,
    expected: Option<usize>,
) -> Result<(), Failure> {
    let generated = offset.generated(source);
    assert!(!generated.is_null());
    assert_eq!(edge_index(offset, &generated)?, expected);
    Ok(())
}

fn edge_index(offset: &BRepOffsetOffset, expected: &Shape) -> Result<Option<usize>, Failure> {
    let mut edges = Explorer::new(
        &offset.face().clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )?;
    let mut index = 0;
    while edges.more() {
        if edges.current().is_same(expected) {
            return Ok(Some(index));
        }
        edges.next()?;
        index += 1;
    }
    Ok(None)
}

fn assert_tube_area(shape: &Shape) -> Result<(), Failure> {
    let mut properties = GProps::default();
    BRepGProp::surface_properties(shape, &mut properties, false, false)?;
    assert!((properties.mass() - 20.0 * std::f64::consts::PI).abs() < 1.0e-10);
    Ok(())
}

fn count(shape: &Shape, kind: ShapeEnum) -> Result<usize, Failure> {
    Ok(Explorer::new(shape, kind, ShapeEnum::Shape)?.count())
}

fn straight_edge() -> Result<Edge, Failure> {
    let mut maker = BRepBuilderAPIMakeEdge::from_points(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, 0.0, 10.0),
    )?;
    Ok(Edge::unchecked(maker.shape()?.clone()))
}
