//! Native differential coverage for the 3D-rail `BRepOffset_Offset` constructor.
use rustycad_tkbrep::{brep::BRepTool, topexp::Explorer, topods};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::{
    geom_abs::Shape as Continuity,
    gp::{Axis2, Circle3, Point3, Transform3},
    toploc::Location,
};
use rustycad_tkoffset::brep_offset::{BRepOffsetOffset, BRepOffsetStatus};
use rustycad_tktopalgo::{
    brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeVertex},
    brep_check::BRepCheckAnalyzer,
    brep_gprop::BRepGProp,
};

#[test]
fn default_object_retains_uninitialized_native_state() {
    let offset = BRepOffsetOffset::new();
    assert!(offset.initial_shape().is_null());
    assert!(offset.face().is_null());
    assert_eq!(offset.status(), BRepOffsetStatus::Good);
}

#[test]
fn straight_path_and_two_3d_rails_build_the_native_face()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let path = edge((0.0, 0.0, 0.0), (0.0, 0.0, 10.0))?;
    let rail1 = edge((1.0, 0.0, 0.0), (1.0, 0.0, 10.0))?;
    let rail2 = edge((0.0, 1.0, 0.0), (0.0, 1.0, 10.0))?;
    let offset =
        BRepOffsetOffset::from_edges(&path, &rail1, &rail2, 0.5, false, 1.0e-4, Continuity::C1)?;
    let face = offset.face();
    assert_eq!(offset.status(), BRepOffsetStatus::Good);
    assert!(offset.initial_shape().is_same(&path.clone().into_shape()));
    let face_shape = face.clone().into_shape();
    assert_eq!(count(&face_shape, ShapeEnum::Edge), 4);
    assert_eq!(count(&face_shape, ShapeEnum::Vertex), 8);
    assert_all_face_edges_have_curves(&face_shape)?;
    assert!(BRepCheckAnalyzer::new(&face.clone().into_shape(), true, false, false)?.is_valid()?);
    assert!(BRepTool::curve_on_face(&rail1, face)?.is_stored);
    assert!(BRepTool::curve_on_face(&rail2, face)?.is_stored);
    assert!((BRepTool::edge_tolerance(&rail1)? - 0.525).abs() < 1.0e-12);
    assert!((BRepTool::edge_tolerance(&rail2)? - 0.525).abs() < 1.0e-12);
    let mut properties = GProps::default();
    BRepGProp::surface_properties(&face.clone().into_shape(), &mut properties, false, false)?;
    assert!((properties.mass() - 7.853_981_633_974_483).abs() < 1.0e-10);
    Ok(())
}

#[test]
fn generated_endpoint_images_are_connector_edges() -> Result<(), rustycad_tkernel::standard::Failure>
{
    let path = edge((0.0, 0.0, 0.0), (0.0, 0.0, 10.0))?;
    let rail1 = edge((1.0, 0.0, 0.0), (1.0, 0.0, 10.0))?;
    let rail2 = edge((0.0, 1.0, 0.0), (0.0, 1.0, 10.0))?;
    let offset =
        BRepOffsetOffset::from_edges(&path, &rail1, &rail2, 0.5, false, 1.0e-4, Continuity::C1)?;
    let (first, last) = rustycad_tkbrep::topexp::vertices(&path, false);
    let first_image = offset.generated(&first.into_shape());
    let last_image = offset.generated(&last.into_shape());
    assert!(!first_image.is_null());
    assert!(!last_image.is_null());
    assert!(!first_image.is_same(&last_image));
    let face_shape = offset.face().clone().into_shape();
    assert!(contains(&face_shape, &first_image)?);
    assert!(contains(&face_shape, &last_image)?);
    Ok(())
}

#[test]
fn supplied_endpoint_edges_are_reused_and_populated()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let path = edge((0.0, 0.0, 0.0), (0.0, 0.0, 10.0))?;
    let rail1 = edge((1.0, 0.0, 0.0), (1.0, 0.0, 10.0))?;
    let rail2 = edge((0.0, 1.0, 0.0), (0.0, 1.0, 10.0))?;
    let (start1, _) = rustycad_tkbrep::topexp::vertices(&rail1, false);
    let (start2, _) = rustycad_tkbrep::topexp::vertices(&rail2, false);
    let (_, end1) = rustycad_tkbrep::topexp::vertices(&rail1, false);
    let (_, end2) = rustycad_tkbrep::topexp::vertices(&rail2, false);
    let first = BRepBuilderAPIMakeEdge::from_vertices(&start2, &start1)?.edge()?;
    let last = BRepBuilderAPIMakeEdge::from_vertices(&end1, &end2)?.edge()?;
    let offset = BRepOffsetOffset::from_edges_with_connectors(
        &path,
        &rail1,
        &rail2,
        0.5,
        &first,
        &last,
        false,
        1.0e-4,
        Continuity::C1,
    )?;
    let (path_start, path_end) = rustycad_tkbrep::topexp::vertices(&path, false);
    assert!(
        offset
            .generated(&path_start.into_shape())
            .is_same(&first.clone().into_shape())
    );
    assert!(
        offset
            .generated(&path_end.into_shape())
            .is_same(&last.clone().into_shape())
    );
    let face = offset.face();
    for connector in [&first, &last] {
        assert!(contains(
            &face.clone().into_shape(),
            &connector.clone().into_shape()
        )?);
        assert!(BRepTool::curve_on_face(connector, face)?.is_stored);
        assert!(BRepTool::curve_with_location(connector)?.curve.is_some());
    }
    let face_shape = face.clone().into_shape();
    assert_eq!(count(&face_shape, ShapeEnum::Edge), 4);
    assert!(BRepCheckAnalyzer::new(&face_shape, true, false, false)?.is_valid()?);
    let mut properties = GProps::default();
    BRepGProp::surface_properties(&face_shape, &mut properties, false, false)?;
    assert!((properties.mass() - 7.853_981_633_974_483).abs() < 1.0e-10);
    Ok(())
}

#[test]
fn unmatched_supplied_endpoint_is_replaced_without_mutating_it()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let path = edge((0.0, 0.0, 0.0), (0.0, 0.0, 10.0))?;
    let rail1 = edge((1.0, 0.0, 0.0), (1.0, 0.0, 10.0))?;
    let rail2 = edge((0.0, 1.0, 0.0), (0.0, 1.0, 10.0))?;
    let (_, end1) = rustycad_tkbrep::topexp::vertices(&rail1, false);
    let (_, end2) = rustycad_tkbrep::topexp::vertices(&rail2, false);
    let unmatched = edge((50.0, 50.0, 0.0), (60.0, 60.0, 0.0))?;
    let last = BRepBuilderAPIMakeEdge::from_vertices(&end1, &end2)?.edge()?;
    let offset = BRepOffsetOffset::from_edges_with_connectors(
        &path,
        &rail1,
        &rail2,
        0.5,
        &unmatched,
        &last,
        false,
        1.0e-4,
        Continuity::C1,
    )?;
    let (path_start, path_end) = rustycad_tkbrep::topexp::vertices(&path, false);
    let first_image = offset.generated(&path_start.into_shape());
    let last_image = offset.generated(&path_end.into_shape());
    assert!(!first_image.is_same(&unmatched.clone().into_shape()));
    assert!(last_image.is_same(&last.clone().into_shape()));
    assert!(!first_image.is_null() && !last_image.is_null());
    let face_shape = offset.face().clone().into_shape();
    assert!(contains(&face_shape, &first_image)? && contains(&face_shape, &last_image)?);
    assert_eq!(count(&face_shape, ShapeEnum::Edge), 4);
    assert!(!BRepTool::curve_on_face(&unmatched, offset.face())?.is_stored);
    assert!(BRepTool::curve_with_location(&unmatched)?.curve.is_some());
    assert!(
        BRepCheckAnalyzer::new(&offset.face().clone().into_shape(), true, false, false,)?
            .is_valid()?
    );
    Ok(())
}

#[test]
fn nonidentity_unmatched_supplied_endpoint_is_ignored()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let path = edge((0.0, 0.0, 0.0), (0.0, 0.0, 10.0))?;
    let rail1 = edge((1.0, 0.0, 0.0), (1.0, 0.0, 10.0))?;
    let rail2 = edge((0.0, 1.0, 0.0), (0.0, 1.0, 10.0))?;
    let (end1, end2) = end_vertices(&rail1, &rail2);
    let remote = edge((0.0, 0.0, 0.0), (1.0, 1.0, 0.0))?;
    let moved = remote.moved(
        &Location::new(Transform3::translation([50.0, 50.0, 0.0])),
        false,
    )?;
    let unmatched = topods::Edge::unchecked(moved);
    let last = BRepBuilderAPIMakeEdge::from_vertices(&end1, &end2)?.edge()?;
    let offset = BRepOffsetOffset::from_edges_with_connectors(
        &path,
        &rail1,
        &rail2,
        0.5,
        &unmatched,
        &last,
        false,
        1.0e-4,
        Continuity::C1,
    )?;
    let (path_start, path_end) = rustycad_tkbrep::topexp::vertices(&path, false);
    let first_image = offset.generated(&path_start.into_shape());
    assert!(!first_image.is_same(&unmatched.clone().into_shape()));
    assert!(
        offset
            .generated(&path_end.into_shape())
            .is_same(&last.clone().into_shape())
    );
    assert!(!BRepTool::curve_on_face(&unmatched, offset.face())?.is_stored);
    assert!(
        BRepCheckAnalyzer::new(&offset.face().clone().into_shape(), true, false, false,)?
            .is_valid()?
    );
    Ok(())
}

#[test]
fn partially_matching_supplied_endpoint_is_reused_as_native_does()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let path = edge((0.0, 0.0, 0.0), (0.0, 0.0, 10.0))?;
    let rail1 = edge((1.0, 0.0, 0.0), (1.0, 0.0, 10.0))?;
    let rail2 = edge((0.0, 1.0, 0.0), (0.0, 1.0, 10.0))?;
    let (start1, _) = rustycad_tkbrep::topexp::vertices(&rail1, false);
    let (_, end1) = rustycad_tkbrep::topexp::vertices(&rail1, false);
    let (_, end2) = rustycad_tkbrep::topexp::vertices(&rail2, false);
    let remote = BRepBuilderAPIMakeVertex::new(Point3::new(50.0, 50.0, 0.0))?.vertex()?;
    let malformed_first = BRepBuilderAPIMakeEdge::from_vertices(&start1, &remote)?.edge()?;
    let last = BRepBuilderAPIMakeEdge::from_vertices(&end1, &end2)?.edge()?;
    let offset = BRepOffsetOffset::from_edges_with_connectors(
        &path,
        &rail1,
        &rail2,
        0.5,
        &malformed_first,
        &last,
        false,
        1.0e-4,
        Continuity::C1,
    )?;
    let (path_start, _) = rustycad_tkbrep::topexp::vertices(&path, false);
    let image = offset.generated(&path_start.into_shape());
    assert!(image.is_same(&malformed_first.clone().into_shape()));
    let (_, path_end) = rustycad_tkbrep::topexp::vertices(&path, false);
    assert!(
        offset
            .generated(&path_end.into_shape())
            .is_same(&last.clone().into_shape())
    );
    assert!(BRepTool::curve_on_face(&malformed_first, offset.face())?.is_stored);
    let face_shape = offset.face().clone().into_shape();
    assert_eq!(count(&face_shape, ShapeEnum::Edge), 4);
    assert!(!BRepCheckAnalyzer::new(&face_shape, true, false, false)?.is_valid()?);
    Ok(())
}

#[test]
fn closed_circle_path_matches_native_single_connector_topology()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let path = BRepBuilderAPIMakeEdge::from_circ(Circle3::new(Axis2::default(), 1.0)?)?.edge()?;
    let rail1 = edge((2.0, 0.0, 0.0), (2.0, 0.0, 1.0))?;
    let rail2 = edge((3.0, 0.0, 0.0), (3.0, 0.0, 1.0))?;
    let offset =
        BRepOffsetOffset::from_edges(&path, &rail1, &rail2, 0.5, false, 1.0e-4, Continuity::C1)?;
    let face = offset.face();
    let face_shape = face.clone().into_shape();
    assert_eq!(count(&face_shape, ShapeEnum::Edge), 4);
    assert_eq!(count(&face_shape, ShapeEnum::Vertex), 8);
    assert!(!BRepCheckAnalyzer::new(&face_shape, true, false, false)?.is_valid()?);
    let (path_start, path_end) = rustycad_tkbrep::topexp::vertices(&path, false);
    assert!(path_start.is_same(&path_end));
    let image = offset.generated(&path_start.into_shape());
    assert_eq!(count_same_edges(&face_shape, &image)?, 2);
    assert_eq!(count_curves(&face_shape)?, 2);
    let mut properties = GProps::default();
    BRepGProp::surface_properties(&face_shape, &mut properties, false, false)?;
    assert!((properties.mass() - 6.284_108_096_297_737).abs() < 1.0e-10);
    Ok(())
}

#[test]
fn sub_confusion_circle_rail_is_marked_degenerated()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let path = edge((0.0, 0.0, 0.0), (0.0, 0.0, 10.0))?;
    let axis = Axis2::default();
    let rail1 = BRepBuilderAPIMakeEdge::from_circ(Circle3::new(axis, 0.5e-7)?)?.edge()?;
    let rail2 = BRepBuilderAPIMakeEdge::from_circ(Circle3::new(axis, 1.0)?)?.edge()?;
    let offset =
        BRepOffsetOffset::from_edges(&path, &rail1, &rail2, 0.5, false, 1.0e-4, Continuity::C1)?;
    let face_shape = offset.face().clone().into_shape();
    assert_eq!(count(&face_shape, ShapeEnum::Edge), 4);
    assert_eq!(count_curves(&face_shape)?, 1);
    assert_eq!(count_degenerated_edges(&face_shape)?, 1);
    Ok(())
}

fn edge(
    first: (f64, f64, f64),
    last: (f64, f64, f64),
) -> Result<topods::Edge, rustycad_tkernel::standard::Failure> {
    let point = |(x, y, z)| Point3::new(x, y, z);
    BRepBuilderAPIMakeEdge::from_points(point(first), point(last))?.edge()
}

fn count(shape: &topods::Shape, kind: ShapeEnum) -> usize {
    Explorer::new(shape, kind, ShapeEnum::Shape)
        .unwrap()
        .count()
}

fn end_vertices(rail1: &topods::Edge, rail2: &topods::Edge) -> (topods::Vertex, topods::Vertex) {
    let (_, end1) = rustycad_tkbrep::topexp::vertices(rail1, false);
    let (_, end2) = rustycad_tkbrep::topexp::vertices(rail2, false);
    (end1, end2)
}

fn count_same_edges(
    shape: &topods::Shape,
    expected: &topods::Shape,
) -> Result<usize, rustycad_tkernel::standard::Failure> {
    Ok(Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)?
        .filter(|edge| edge.is_same(expected))
        .count())
}

fn count_curves(shape: &topods::Shape) -> Result<usize, rustycad_tkernel::standard::Failure> {
    Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)?.try_fold(0, |count, edge| {
        let edge = topods::Edge::unchecked(edge);
        Ok(count + usize::from(BRepTool::curve_with_location(&edge)?.curve.is_some()))
    })
}

fn count_degenerated_edges(
    shape: &topods::Shape,
) -> Result<usize, rustycad_tkernel::standard::Failure> {
    Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)?.try_fold(0, |count, edge| {
        let edge = topods::Edge::unchecked(edge);
        Ok(count + usize::from(BRepTool::degenerated(&edge)?))
    })
}

fn assert_all_face_edges_have_curves(
    face: &topods::Shape,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    for edge in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edge = topods::Edge::unchecked(edge);
        assert!(BRepTool::curve_with_location(&edge)?.curve.is_some());
    }
    Ok(())
}

fn contains(
    shape: &topods::Shape,
    expected: &topods::Shape,
) -> Result<bool, rustycad_tkernel::standard::Failure> {
    for candidate in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if candidate.is_same(expected) {
            return Ok(true);
        }
    }
    Ok(false)
}
