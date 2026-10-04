//! Native differential coverage for the 3D-rail `BRepOffset_Offset` constructor.
use rustycad_tkbrep::{brep::BRepTool, topexp::Explorer, topods};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::{geom_abs::Shape as Continuity, gp::Point3};
use rustycad_tkoffset::brep_offset::{BRepOffsetOffset, BRepOffsetStatus};
use rustycad_tktopalgo::{
    brep_builder_api::BRepBuilderAPIMakeEdge, brep_check::BRepCheckAnalyzer, brep_gprop::BRepGProp,
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
