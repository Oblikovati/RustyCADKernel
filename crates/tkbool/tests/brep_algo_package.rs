//! The BRepAlgo package: wire concatenation, conversion into arcs and segments, and
//! validity of operation results.
use rustycad_tkbool::brep_algo;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Shape, Wire};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakePolygon;
use rustycad_tktopalgo::brep_lib::BRepLibMakeFace;

/// The polyline wire through the points (closed when asked).
fn polyline(points: &[(f64, f64, f64)], closed: bool) -> Result<Wire, Failure> {
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    for &(x, y, z) in points {
        polygon.add_point(Point3::new(x, y, z))?;
    }
    if closed {
        polygon.close()?;
    }
    polygon.wire()
}

/// The count of the sub-shapes of a kind.
fn count(shape: &Shape, kind: ShapeEnum) -> Result<usize, Failure> {
    Ok(Explorer::new(shape, kind, ShapeEnum::Shape)?.count())
}

#[test]
fn collinear_segments_concatenate_into_one_c0_edge() -> Result<(), Failure> {
    let wire = polyline(&[(0., 0., 0.), (1., 0., 0.), (2., 0., 0.)], false)?;
    let edge = brep_algo::concatenate_wire_c0(&wire)?;
    assert!(!edge.is_null());
    let (curve, first, last) = BRepTool::curve(&edge)?;
    let c = curve.get().ok_or_else(|| Failure::new("null curve"))?;
    assert!(c.value(first)?.distance(&Point3::new(0., 0., 0.)) < 1e-9);
    assert!(c.value(last)?.distance(&Point3::new(2., 0., 0.)) < 1e-9);
    Ok(())
}

#[test]
fn a_cornered_wire_keeps_its_corners_in_g1() -> Result<(), Failure> {
    let wire = polyline(&[(0., 0., 0.), (1., 0., 0.), (1., 1., 0.)], false)?;
    let result = brep_algo::concatenate_wire(&wire, Continuity::G1, 1e-4)?;
    assert_eq!(count(result.as_ref(), ShapeEnum::Edge)?, 2);
    Ok(())
}

#[test]
fn a_square_face_converts_into_segments() -> Result<(), Failure> {
    let wire = polyline(
        &[(0., 0., 0.), (2., 0., 0.), (2., 2., 0.), (0., 2., 0.)],
        true,
    )?;
    let face = BRepLibMakeFace::from_wire(&wire, true)?.face()?;
    let converted = brep_algo::convert_face(&face, 0.1)?;
    assert!(!converted.is_null());
    assert_eq!(count(converted.as_ref(), ShapeEnum::Edge)?, 4);
    Ok(())
}

#[test]
fn a_face_is_valid_alone_and_as_a_result() -> Result<(), Failure> {
    let wire = polyline(
        &[(0., 0., 0.), (2., 0., 0.), (2., 2., 0.), (0., 2., 0.)],
        true,
    )?;
    let face: Shape = BRepLibMakeFace::from_wire(&wire, true)?
        .face()?
        .into_shape();
    assert!(brep_algo::is_valid(&face)?);
    assert!(brep_algo::is_topologically_valid(&face)?);
    let mut args: List<Shape> = List::default();
    args.append(face.clone());
    // The result is an argument: valid without check.
    assert!(brep_algo::is_valid_result(&args, &face, true, true)?);
    // A null result is valid.
    assert!(brep_algo::is_valid_result(
        &args,
        &Shape::default(),
        true,
        true
    )?);
    Ok(())
}
