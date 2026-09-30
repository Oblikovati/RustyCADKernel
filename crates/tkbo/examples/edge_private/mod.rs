//! Helper state is compared without exposing an unfinished intersection API.
use super::{edge_math, edge_resolution};
mod fixtures;
use rustycad_tkbo::int_tools::Range;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::{bnd::BoundingBox3, gp::Point3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
fn real(value: f64) {
    print!("{} ", value.to_bits());
}
fn integer(value: i32) {
    print!("{value} ");
}
fn attempt(result: Result<(), Failure>) {
    if let Err(error) = result {
        print!("{} ", error.exception_type());
    }
}
fn math(values: &[f64]) -> Result<(), Failure> {
    let range = Range::new(values[0], values[1]);
    let mut output = vec![Range::new(17., 19.)];
    integer(edge_math::split_range(
        range.first(),
        range.last(),
        values[2],
        values[3] as i32,
        &mut output,
    ));
    integer(output.len() as i32);
    for item in output {
        real(item.first());
        real(item.last());
    }
    let mut bounds = BoundingBox3::default();
    bounds.add_point(Point3::new(-1., -2., -3.));
    bounds.add_point(Point3::new(4., 5., 6.));
    bounds.set_gap(values[4]);
    real(edge_math::point_box_distance(
        &bounds,
        Point3::new(values[5], values[6], values[7]),
    )?);
    Ok(())
}
fn geometry(values: &[f64]) -> Result<(), Failure> {
    let source = fixtures::geometry(values[0] as i32, values[5])?;
    let edge = BRepBuilderAPIMakeEdge::from_curve_params(&source, 0., 1.)?.edge()?;
    let curve = BRepAdaptorCurve::with_edge(&edge)?;
    let kind = curve.get_type()?;
    integer(edge_math::curve_priority(kind));
    let range = Range::new(values[1], values[2]);
    let coefficient = edge_resolution::resolution_coefficient(&curve, range)?;
    real(coefficient);
    let local = curve.transformed_curve().geom_curve();
    attempt(edge_resolution::curve_resolution(&local, kind, coefficient, values[3]).map(real));
    attempt(edge_math::curve_deflection(&curve, range).map(real));
    attempt(
        edge_math::is_closed(&source, values[1], values[2], values[3], values[4])
            .map(|v| integer(i32::from(v))),
    );
    Ok(())
}
pub fn run(operation: &str, values: &[f64]) {
    match operation {
        "edge_math" => attempt(math(values)),
        "edge_geometry" => attempt(geometry(values)),
        _ => {
            panic!("Unknown edge helper operation {operation}; expected edge_math or edge_geometry")
        }
    }
}
