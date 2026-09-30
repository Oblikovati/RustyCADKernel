//! Observable partial outputs from curve/box clipping searches.
use super::super::{edge_parameters::ParameterSearch, edge_resolution::resolution_coefficient};
use super::{fixtures, integer, real};
use rustycad_tkbo::int_tools::Range;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::{bnd::BoundingBox3, gp::Point3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
pub(super) fn case(values: &[f64]) -> Result<(), Failure> {
    let source = fixtures::geometry(values[0] as i32, 0.5)?;
    let edge = BRepBuilderAPIMakeEdge::from_curve_params(&source, 0., 1.)?.edge()?;
    let curve = BRepAdaptorCurve::with_edge(&edge)?;
    let range = Range::new(values[1], values[2]);
    let mut bounds = BoundingBox3::default();
    bounds.add_point(Point3::new(values[3] - values[4], -values[4], -values[4]));
    bounds.add_point(Point3::new(values[3] + values[4], values[4], values[4]));
    bounds.set_gap(values[5]);
    let search = ParameterSearch {
        curve: &curve,
        range,
        tolerance: values[6],
        resolution: values[7],
        precision: values[8],
        coefficient: resolution_coefficient(&curve, range)?,
        bounds: &bounds,
    };
    let (mut first, mut last) = (17., 19.);
    let result = search.find(&mut first, &mut last);
    match result {
        Ok(found) => integer(i32::from(found)),
        Err(error) => print!("{} ", error.exception_type()),
    }
    real(first);
    real(last);
    Ok(())
}
