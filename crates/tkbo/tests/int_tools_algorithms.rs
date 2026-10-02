//! IntTools utility regressions retain source-level quirks and caller-owned outputs.
use rustycad_tkbo::int_tools::{IntTools, Root};
use rustycad_tkbrep::{brep::BRepBuilder, brep_adaptor::BRepAdaptorCurve, topods::Edge};
use rustycad_tkernel::{
    ncollection::Array1,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::{
    geom::{GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomCurve, GeomLine, GeomOffsetCurve},
    top_abs::State,
};
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
fn line() -> Handle<dyn GeomCurve> {
    Handle::new(GeomLine::from_point(Point3::default(), Direction3::X)).into_cast()
}
fn edge(curve: &Handle<dyn GeomCurve>) -> Result<Edge, Failure> {
    BRepBuilderAPIMakeEdge::from_curve_params(curve, 0., 1.)?.edge()
}
#[test]
fn identical_root_removal_skips_the_shifted_successor() {
    let mut roots = vec![Root::new(1., 0); 3];
    IntTools::remove_identical_roots(&mut roots, 1.);
    assert_eq!(roots.len(), 2);
    IntTools::remove_identical_roots(&mut roots, 0.);
    assert_eq!(roots.len(), 2);
}
#[test]
fn sorting_ignores_tolerance_and_retains_full_root_records() {
    let mut roots = vec![Root::new(3., 3), Root::new(1., 1), Root::new(2., 2)];
    IntTools::sort_roots(&mut roots, 100.);
    assert_eq!(
        roots.iter().map(Root::root_type).collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    IntTools::sort_roots(&mut [], 0.);
}
#[test]
fn root_states_keep_unknown_kinds_and_the_strict_smart_threshold() {
    let mut roots = vec![
        Root::new(0., 0),
        Root::new(0., 1),
        Root::new(0., 2),
        Root::new(0., 9),
    ];
    for root in &mut roots {
        root.set_interval(0., 1., 1., -0.5);
    }
    IntTools::find_root_states(&mut roots, 0.5);
    let states: Vec<_> = roots
        .iter()
        .map(|root| (root.state_before(), root.state_after()))
        .collect();
    assert_eq!(
        states,
        vec![
            (State::Out, State::In),
            (State::On, State::On),
            (State::On, State::In),
            (State::Unknown, State::Unknown)
        ]
    );
    IntTools::find_root_states(&mut roots, 1.);
    assert_eq!(roots[2].state_before(), State::Out);
    assert_eq!(roots[2].state_after(), State::On);
}
#[test]
fn length_ignores_degenerate_and_nongeometric_edges() -> Result<(), Failure> {
    let geometric = edge(&line())?;
    assert_eq!(IntTools::length(&geometric)?, 1.);
    BRepBuilder::default().degenerated(&geometric, true)?;
    assert_eq!(IntTools::length(&geometric)?, 0.);
    let mut empty = Edge::default();
    BRepBuilder::default().make_edge(&mut empty)?;
    assert_eq!(IntTools::length(&empty)?, 0.);
    Ok(())
}
#[test]
fn line_radius_leaves_the_output_untouched_while_circle_writes_it() -> Result<(), Failure> {
    let mut radius = 17.;
    let linear = BRepAdaptorCurve::with_edge(&edge(&line())?)?;
    assert_eq!(IntTools::get_radius(&linear, 0., 1., &mut radius)?, 1);
    assert_eq!(radius, 17.);
    let circle: Handle<dyn GeomCurve> =
        Handle::new(GeomCircle::new(Axis2::default(), 3.)?).into_cast();
    let circular = BRepAdaptorCurve::with_edge(&edge(&circle)?)?;
    assert_eq!(IntTools::get_radius(&circular, 0., 0., &mut radius)?, 0);
    assert_eq!(radius, 3.);
    Ok(())
}

#[test]
fn spline_offset_radius_failure_keeps_the_native_sentinel() -> Result<(), Failure> {
    assert_spline_offset_radius_failure(spline_basis()?)?;
    assert_spline_offset_radius_failure(bezier_basis()?)
}

fn assert_spline_offset_radius_failure(basis: Handle<dyn GeomCurve>) -> Result<(), Failure> {
    let offset = Handle::new(GeomOffsetCurve::new(&basis, 0.5, Direction3::Z, false)?).into_cast();
    let adaptor = BRepAdaptorCurve::with_edge(&edge(&offset)?)?;
    let mut radius = 17.0;
    assert_eq!(IntTools::get_radius(&adaptor, 0.0, 0.0, &mut radius)?, -1);
    assert_eq!(radius, 17.0);
    Ok(())
}

fn spline_basis() -> Result<Handle<dyn GeomCurve>, Failure> {
    let poles = [
        Point3::default(),
        Point3::new(1.0, 2.0, 0.0),
        Point3::new(3.0, 0.0, 0.0),
    ];
    Ok(Handle::new(GeomBSplineCurve::new(
        &Array1::from_shared(&poles, 1),
        &Array1::from_shared(&[0.0, 1.0], 1),
        &Array1::from_shared(&[3, 3], 1),
        2,
        false,
    )?)
    .into_cast())
}

fn bezier_basis() -> Result<Handle<dyn GeomCurve>, Failure> {
    let poles = [
        Point3::default(),
        Point3::new(1.0, 2.0, 0.0),
        Point3::new(-1.0, 2.0, 0.0),
        Point3::default(),
    ];
    Ok(Handle::new(GeomBezierCurve::new(&Array1::from_shared(&poles, 1))?).into_cast())
}

#[test]
fn unsupported_offset_parameter_keeps_the_previous_parameter() -> Result<(), Failure> {
    let curve = line();
    let mut parameter = 19.;
    assert_eq!(
        IntTools::parameter(Point3::new(3., 4., 5.), &curve, &mut parameter)?,
        0
    );
    assert_eq!(parameter, 3.);
    let offset: Handle<dyn GeomCurve> =
        Handle::new(GeomOffsetCurve::new(&curve, 0.5, Direction3::Z, false)?).into_cast();
    assert_eq!(
        IntTools::parameter(Point3::default(), &offset, &mut parameter)?,
        1
    );
    assert_eq!(parameter, 3.);
    Ok(())
}
#[test]
fn subdivision_rebases_arrays_and_retains_native_nonpositive_counts() -> Result<(), Failure> {
    let curve = BRepAdaptorCurve::with_edge(&edge(&line())?)?;
    let mut arguments = Array1::from_shared(&[17., 19.], 5);
    assert_eq!(
        IntTools::prepare_args(&curve, 1., 0., 3, 0.01, &mut arguments)?,
        0
    );
    assert_eq!(arguments.lower(), 0);
    assert_eq!(arguments.as_slice(), &[0., 1. / 3., 2. / 3., 1.]);
    for count in [-2, 0] {
        assert_eq!(
            IntTools::prepare_args(&curve, 2., 0., count, 0.01, &mut arguments)?,
            0
        );
        assert_eq!(arguments.as_slice(), &[2.]);
    }
    Ok(())
}
