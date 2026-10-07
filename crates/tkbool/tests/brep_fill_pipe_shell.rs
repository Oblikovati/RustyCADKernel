//! BRepFill_PipeShell: a 1:1 translation of OCCT's GTests BRepFill_PipeShell_Test.cxx
//! (src/ModelingAlgorithms/TKBool/GTests), then tests of the location modes, the
//! profiles and the history.
use rustycad_tkbool::brep_fill::{BRepFillPipeShell, TransitionStyle};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Shape, Wire};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeomalgo::geom_fill::GeomFillPipeError;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakePolygon, BRepBuilderAPIMakeWire,
};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;

/// `MakeSquareProfile(theCenter, theHalfSide)`.
fn make_square_profile(the_center: Point3, the_half_side: f64) -> Result<Wire, Failure> {
    let mut a_polygon = BRepBuilderAPIMakePolygon::new();
    let (x, y, z) = (the_center.x(), the_center.y(), the_center.z());
    for (dx, dy) in [(-1., -1.), (1., -1.), (1., 1.), (-1., 1.)] {
        a_polygon.add_point(Point3::new(
            x + dx * the_half_side,
            y + dy * the_half_side,
            z,
        ))?;
    }
    a_polygon.close()?;
    a_polygon.wire()
}

/// The polyline spine through the points.
fn polyline(points: &[(f64, f64, f64)]) -> Result<Wire, Failure> {
    let mut a_spine_polygon = BRepBuilderAPIMakePolygon::new();
    for &(x, y, z) in points {
        a_spine_polygon.add_point(Point3::new(x, y, z))?;
    }
    a_spine_polygon.wire()
}

/// The count of the sub-shapes of a kind.
fn count(shape: &Shape, kind: ShapeEnum) -> Result<usize, Failure> {
    Ok(Explorer::new(shape, kind, ShapeEnum::Shape)?.count())
}

// TEST(BRepFill_PipeShellTest, MakeSolid_StraightSpineClosedProfile_ProducesClosedSolid)
#[test]
fn make_solid_straight_spine_closed_profile_produces_closed_solid() -> Result<(), Failure> {
    let a_spine_edge = BRepBuilderAPIMakeEdge::from_points(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, 0.0, 10.0),
    )?
    .edge()?;
    let a_spine = BRepBuilderAPIMakeWire::from_edge(&a_spine_edge)?.wire()?;
    let a_profile = make_square_profile(Point3::new(0.0, 0.0, 0.0), 1.0)?;
    let mut a_pipe = BRepFillPipeShell::new(&a_spine)?;
    a_pipe.add(&a_profile, false, false)?;
    assert!(a_pipe.is_ready());
    assert!(a_pipe.build()?);
    assert!(a_pipe.make_solid()?);
    let a_result: &Shape = a_pipe.shape();
    assert!(!a_result.is_null());
    assert!(a_result.closed()?);
    let an_analyzer = BRepCheckAnalyzer::new(a_result, true, false, false)?;
    assert!(an_analyzer.is_valid()?);
    Ok(())
}

// TEST(BRepFill_PipeShellTest, MakeSolid_PolylineSpineClosedProfile_ProducesClosedSolid)
#[test]
fn make_solid_polyline_spine_closed_profile_produces_closed_solid() -> Result<(), Failure> {
    let a_spine = polyline(&[(0.0, 0.0, 0.0), (0.0, 0.0, 5.0), (0.0, 5.0, 10.0)])?;
    let a_profile = make_square_profile(Point3::new(0.0, 0.0, 0.0), 0.5)?;
    let mut a_pipe = BRepFillPipeShell::new(&a_spine)?;
    a_pipe.set_transition(TransitionStyle::Right, 1.0e-2, 6.0);
    a_pipe.add(&a_profile, false, false)?;
    assert!(a_pipe.is_ready());
    assert!(a_pipe.build()?);
    assert!(a_pipe.make_solid()?);
    let a_result: &Shape = a_pipe.shape();
    assert!(!a_result.is_null());
    assert!(a_result.closed()?);
    Ok(())
}

#[test]
fn a_pipe_without_profile_is_not_ready_and_does_not_build() -> Result<(), Failure> {
    let mut pipe = BRepFillPipeShell::new(&polyline(&[(0., 0., 0.), (0., 0., 1.)])?)?;
    assert!(!pipe.is_ready());
    assert_eq!(pipe.get_status(), GeomFillPipeError::PipeOk);
    let error = pipe.build().err().map(|e| e.exception_type().to_string());
    assert_eq!(error, Some("StdFail_NotDone".to_string()));
    let error = pipe
        .make_solid()
        .err()
        .map(|e| e.exception_type().to_string());
    assert_eq!(error, Some("StdFail_NotDone".to_string()));
    Ok(())
}

#[test]
fn the_history_maps_profile_edges_to_their_tapes_and_spine_edges_to_faces() -> Result<(), Failure> {
    let spine = polyline(&[(0., 0., 0.), (0., 0., 4.)])?;
    let profile = make_square_profile(Point3::new(0., 0., 0.), 0.5)?;
    let mut pipe = BRepFillPipeShell::new(&spine)?;
    pipe.add(&profile, false, false)?;
    assert!(pipe.build()?);
    assert_eq!(count(pipe.shape(), ShapeEnum::Face)?, 4);
    for e in Explorer::new(&profile, ShapeEnum::Edge, ShapeEnum::Shape)? {
        assert_eq!(pipe.generated(&e).len(), 1);
    }
    for e in Explorer::new(&spine, ShapeEnum::Edge, ShapeEnum::Shape)? {
        assert_eq!(pipe.generated(&e).len(), 4);
    }
    // The first and last sections are the boundary wires of the sweep.
    assert_eq!(pipe.first_shape().shape_type()?, ShapeEnum::Wire);
    assert_eq!(pipe.last_shape().shape_type()?, ShapeEnum::Wire);
    assert_eq!(pipe.profiles().len(), 1);
    pipe.delete_profile(&profile)?;
    assert!(!pipe.is_ready());
    Ok(())
}

#[test]
fn the_location_modes_build_the_same_tube_on_a_straight_spine() -> Result<(), Failure> {
    let spine = polyline(&[(0., 0., 0.), (0., 0., 4.)])?;
    let profile = make_square_profile(Point3::new(0., 0., 0.), 0.5)?;
    let mut fixed = BRepFillPipeShell::new(&spine)?;
    fixed.set_axe(&Axis2::new(Point3::default(), Direction3::Z)?)?;
    let mut binormal = BRepFillPipeShell::new(&spine)?;
    binormal.set_bi_normal(Direction3::X)?;
    let mut frenet = BRepFillPipeShell::new(&spine)?;
    frenet.set_frenet(false)?;
    for pipe in [&mut fixed, &mut binormal, &mut frenet] {
        pipe.add(&profile, false, false)?;
        assert!(pipe.build()?);
        assert_eq!(count(pipe.shape(), ShapeEnum::Face)?, 4);
    }
    let sections = frenet.simulate(3)?;
    assert_eq!(sections.len(), 3);
    Ok(())
}
