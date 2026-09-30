//! Native working-range state, including reused bounding boxes and failed interval writes.
use super::{flag, real, tools_geometry};
use rustycad_tkbo::int_tools::{Context, ShrunkRange};
use rustycad_tkbrep::{brep::BRepBuilder, topexp};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
fn outcome(result: Result<(), Failure>) {
    if let Err(error) = result {
        print!("{} ", error.exception_type());
    }
}
fn report(range: &ShrunkRange) {
    flag(range.is_done());
    flag(range.is_splittable());
    real(range.length());
    let (first, last) = range.shrunk_range();
    real(first);
    real(last);
    flag(range.edge().is_null());
    let bounds = range.bounding_box();
    flag(bounds.is_void());
    real(bounds.gap());
    outcome(bounds.get().map(|values| {
        for value in values {
            real(value);
        }
    }));
}
fn run(values: &[f64]) -> Result<(), Failure> {
    let mut range = ShrunkRange::new();
    report(&range);
    range.perform()?;
    report(&range);
    let context = Handle::new(Context::default());
    range.set_context(&context);
    flag(range.context() == &context);
    range.set_context(&Handle::default());
    flag(range.context().is_null());
    let geometry = tools_geometry::geometry(values[0] as i32, values[1], values[2])?;
    let edge =
        BRepBuilderAPIMakeEdge::from_curve_params(&geometry, values[1], values[2])?.edge()?;
    let (first, last) = topexp::vertices(&edge, false);
    let builder = BRepBuilder::default();
    builder.update_edge_tolerance(&edge, values[3])?;
    builder.update_vertex_tolerance(&first, values[4])?;
    builder.update_vertex_tolerance(&last, values[5])?;
    range.set_data(&edge, values[1], values[2], &first, &last);
    flag(range.edge().is_same(&edge));
    for _ in 0..2 {
        outcome(range.perform());
        report(&range);
    }
    outcome(range.set_shrunk_range(values[1], values[2]));
    report(&range);
    range.set_data(&edge, values[6], values[7], &last, &first);
    report(&range);
    outcome(range.perform());
    report(&range);
    Ok(())
}
pub fn case(values: &[f64]) {
    outcome(run(values));
}
