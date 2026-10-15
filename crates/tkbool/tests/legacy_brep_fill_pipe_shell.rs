//! Exact translations of BRepFill_PipeShell_Test.cxx at OCCT 3d097a0.
#[path = "brep_fill_pipe_shell/support.rs"]
mod support;
use rustycad_tkbool::brep_fill::{BRepFillPipeShell, TransitionStyle};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeWire};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use support::{make_square_profile, polyline};

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
