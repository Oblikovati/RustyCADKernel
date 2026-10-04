//! Differential coverage for the native `GeomFill_Pipe` option overload.
use rustycad_tkbrep::{brep::BRepTool, topexp::Explorer, topods};
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkg3d::{
    geom::{GeomBSplineSurface, GeomBezierCurve, GeomCurve},
    top_abs::ShapeEnum,
};
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::{geom_abs::Shape as Continuity, gp::Point3};
use rustycad_tkoffset::brep_offset::{BRepOffsetOffset, BRepOffsetStatus};
use rustycad_tktopalgo::{
    brep_check::BRepCheckAnalyzer, brep_gprop::BRepGProp, brep_lib::BRepLibMakeEdge,
};

#[test]
fn nondefault_pipe_options_match_native_surface_results()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let configurations = option_configurations();
    for config in configurations {
        assert_native_option_result(config)?;
    }
    Ok(())
}

fn option_configurations() -> [(bool, f64, Continuity, i32, i32, f64); 6] {
    [
        (false, 1.0e-4, Continuity::C1, 3, 2, 14.988_343_128_797_375),
        (true, 1.0e-4, Continuity::C1, 8, 7, 12.641_642_204_954_515),
        (false, 1.0, Continuity::C1, 3, 2, 14.988_343_128_797_375),
        (false, 1.0e-12, Continuity::C1, 3, 2, 14.988_343_128_797_375),
        (false, 1.0e-4, Continuity::C0, 3, 2, 14.988_343_128_797_375),
        (false, 1.0e-4, Continuity::C2, 3, 2, 14.988_343_128_797_375),
    ]
}

fn assert_native_option_result(
    config: (bool, f64, Continuity, i32, i32, f64),
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let (polynomial, tolerance, continuity, poles, degree, area) = config;
    let (path, rail1, rail2) = bezier_rails()?;
    let offset = BRepOffsetOffset::from_edges(
        &path, &rail1, &rail2, 0.5, polynomial, tolerance, continuity,
    )?;
    assert_option_face(&offset, &rail1, &rail2, (poles, degree), area)
}

fn bezier_rails()
-> Result<(topods::Edge, topods::Edge, topods::Edge), rustycad_tkernel::standard::Failure> {
    Ok((
        bezier_edge(0.0, 0.0)?,
        bezier_edge(1.0, 0.0)?,
        bezier_edge(0.0, 1.0)?,
    ))
}

fn bezier_edge(dx: f64, dy: f64) -> Result<topods::Edge, rustycad_tkernel::standard::Failure> {
    let mut poles = Array1::<Point3>::with_bounds(1, 3).expect("valid Bezier pole bounds");
    poles[1] = Point3::new(dx, dy, 0.0);
    poles[2] = Point3::new(dx, dy + 3.0, 5.0);
    poles[3] = Point3::new(dx, dy, 10.0);
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomBezierCurve::new(&poles)?).cast();
    let mut maker = BRepLibMakeEdge::new();
    maker.init_curve(&curve)?;
    maker.edge()
}

fn assert_option_face(
    offset: &BRepOffsetOffset,
    rail1: &topods::Edge,
    rail2: &topods::Edge,
    spline_shape: (i32, i32),
    expected_area: f64,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let face = offset.face();
    let shape = face.clone().into_shape();
    assert_eq!(offset.status(), BRepOffsetStatus::Good);
    assert_eq!(count(&shape, ShapeEnum::Edge)?, 4);
    assert_eq!(count(&shape, ShapeEnum::Vertex)?, 8);
    assert_eq!(count_curves(&shape)?, 2);
    assert_option_surface(face, spline_shape)?;
    assert!(!BRepCheckAnalyzer::new(&shape, true, false, false)?.is_valid()?);
    assert_source_tolerances(rail1, rail2)?;
    assert_face_area(&shape, expected_area)
}

fn assert_option_surface(
    face: &topods::Face,
    expected: (i32, i32),
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let spline = BRepTool::surface(face)?.cast::<GeomBSplineSurface>();
    let surface = spline
        .get()
        .expect("generic native pipe surface is B-spline");
    assert_eq!((surface.nb_u_poles(), surface.u_degree()), expected);
    assert_eq!((surface.nb_v_poles(), surface.v_degree()), (3, 2));
    Ok(())
}

fn assert_source_tolerances(
    rail1: &topods::Edge,
    rail2: &topods::Edge,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    assert!((BRepTool::edge_tolerance(rail1)? - 1.0e-7).abs() < 1.0e-12);
    assert!((BRepTool::edge_tolerance(rail2)? - 1.0e-7).abs() < 1.0e-12);
    Ok(())
}

fn assert_face_area(
    shape: &topods::Shape,
    expected_area: f64,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let mut properties = GProps::default();
    BRepGProp::surface_properties(shape, &mut properties, false, false)?;
    assert!((properties.mass() - expected_area).abs() < 1.0e-8);
    Ok(())
}

fn count(
    shape: &topods::Shape,
    kind: ShapeEnum,
) -> Result<usize, rustycad_tkernel::standard::Failure> {
    Ok(Explorer::new(shape, kind, ShapeEnum::Shape)?.count())
}

fn count_curves(shape: &topods::Shape) -> Result<usize, rustycad_tkernel::standard::Failure> {
    Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)?.try_fold(0, |count, edge| {
        let edge = topods::Edge::unchecked(edge);
        Ok(count + usize::from(BRepTool::curve_with_location(&edge)?.curve.is_some()))
    })
}
