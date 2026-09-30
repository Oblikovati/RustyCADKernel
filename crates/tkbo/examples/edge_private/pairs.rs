//! Pairwise native decisions and selected parameters, without a public EdgeEdge facade.
use super::super::{
    edge_best::best_solution, edge_coincidence::CurvePair, edge_resolution::resolution_coefficient,
};
use super::{attempt, fixtures, integer, real};
use rustycad_tkbo::int_tools::Range;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkernel::standard::Failure;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
pub(super) fn case(values: &[f64]) -> Result<(), Failure> {
    let source = fixtures::geometry(values[0] as i32, 0.5)?;
    let target = fixtures::geometry(values[1] as i32, 0.5)?;
    let pair = CurvePair {
        first: &source,
        second: &target,
        first_range: Range::new(values[2], values[3]),
        second_range: Range::new(values[4], values[5]),
    };
    let edge = BRepBuilderAPIMakeEdge::from_curve_params(&source, 0., 1.)?.edge()?;
    let curve = BRepAdaptorCurve::with_edge(&edge)?;
    let coefficient = resolution_coefficient(&curve, pair.first_range)?;
    attempt(pair.is_coincident(values[6]).map(|v| integer(i32::from(v))));
    attempt(pair.check_coincidence(values[6], values[7]).map(integer));
    attempt(
        super::super::edge_intersection::is_intersection(&pair, values[6], (values[7], values[7]))
            .map(|v| integer(i32::from(v))),
    );
    let mut output = (17., 19.);
    attempt(best_solution(
        &pair,
        &curve,
        coefficient,
        values[6],
        1e-7,
        &mut output,
    ));
    real(output.0);
    real(output.1);
    Ok(())
}
