//! BRepFill_PipeShell supplementary location, profile, and history tests.
#[path = "brep_fill_pipe_shell/support.rs"]
mod support;
use rustycad_tkbool::brep_fill::BRepFillPipeShell;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeomalgo::geom_fill::GeomFillPipeError;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use support::{make_square_profile, polyline};

/// The count of the sub-shapes of a kind.
fn count(shape: &Shape, kind: ShapeEnum) -> Result<usize, Failure> {
    Ok(Explorer::new(shape, kind, ShapeEnum::Shape)?.count())
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
