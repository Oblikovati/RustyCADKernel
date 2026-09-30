//! IntTools root and geometric utility state compared with the pinned native class.
use super::edge_cases::fixtures;
use super::{integer, real, root_state};
use rustycad_tkbo::int_tools::{IntTools, Root};
use rustycad_tkbrep::{brep::BRepBuilder, brep_adaptor::BRepAdaptorCurve};
use rustycad_tkernel::{ncollection::Array1, standard::Failure};
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
fn attempt(result: Result<(), Failure>) {
    if let Err(error) = result {
        print!("{} ", error.exception_type());
    }
}
fn roots_snapshot(roots: &[Root]) {
    integer(roots.len() as i32);
    for root in roots {
        root_state(root);
    }
}
pub(super) fn roots(values: &[f64]) {
    let mut roots: Vec<Root> = (0..values[0] as i32)
        .map(|index| configured_root(index, values))
        .collect();
    IntTools::remove_identical_roots(&mut roots, values[3]);
    roots_snapshot(&roots);
    IntTools::sort_roots(&mut roots, values[3]);
    roots_snapshot(&roots);
    IntTools::find_root_states(&mut roots, values[4]);
    roots_snapshot(&roots);
}
fn configured_root(index: i32, values: &[f64]) -> Root {
    let seed = values[1] as i32;
    let mut root = Root::new(
        f64::from((index * 7 + seed) % 11 - 5) * values[2],
        (index + seed) % 5 - 1,
    );
    root.set_layer_height(f64::from(index));
    root.set_state_before(State::Out);
    root.set_state_after(State::Unknown);
    root.set_interval(
        -1.,
        1.,
        f64::from(index % 3 - 1) * values[2],
        f64::from(index % 5 - 2) * values[2],
    );
    root
}
pub(super) fn geometry(values: &[f64]) {
    attempt(geometry_case(values));
}
fn geometry_case(values: &[f64]) -> Result<(), Failure> {
    let source = fixtures::geometry(values[0] as i32, 0.5)?;
    let edge = BRepBuilderAPIMakeEdge::from_curve_params(&source, 0., 1.)?.edge()?;
    BRepBuilder::default().degenerated(&edge, values[6] != 0.)?;
    let curve = BRepAdaptorCurve::with_edge(&edge)?;
    attempt(IntTools::length(&edge).map(real));
    let mut radius = 17.;
    attempt(IntTools::get_radius(&curve, values[1], values[2], &mut radius).map(integer));
    real(radius);
    let mut parameter = 19.;
    let point = Point3::new(values[7], values[8], values[9]);
    attempt(IntTools::parameter(point, &source, &mut parameter).map(integer));
    real(parameter);
    let mut arguments =
        Array1::with_bounds(1, 3).map_err(|error| error.into_failure("probe arguments"))?;
    arguments.init(23.);
    attempt(
        IntTools::prepare_args(
            &curve,
            values[2],
            values[1],
            values[3] as i32,
            values[4],
            &mut arguments,
        )
        .map(integer),
    );
    integer(arguments.lower());
    integer(arguments.upper());
    for parameter in arguments.as_slice() {
        real(*parameter);
    }
    Ok(())
}
