//! Recursive solution ranges compared before merging or creating public common parts.
use super::super::{
    edge_resolution::{curve_resolution, resolution_coefficient},
    edge_solutions::{SearchCurve, SolutionSearch},
};
use super::{attempt, fixtures, integer, real};
use rustycad_tkbo::int_tools::Range;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::{adaptor3d::Adaptor3dCurve, geom::GeomCurve};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
fn search_curve<'a>(
    geometry: &'a Handle<dyn GeomCurve>,
    adaptor: &'a BRepAdaptorCurve,
    range: Range,
    tolerance: f64,
) -> Result<SearchCurve<'a>, Failure> {
    let coefficient = resolution_coefficient(adaptor, range)?;
    let resolution = curve_resolution(
        &adaptor.transformed_curve().geom_curve(),
        adaptor.get_type()?,
        coefficient,
        tolerance,
    )?;
    Ok(SearchCurve {
        geometry,
        adaptor,
        tolerance,
        resolution,
        precision: 1e-7,
        coefficient,
    })
}
pub(super) fn case(values: &[f64]) -> Result<(), Failure> {
    let first = fixtures::geometry(values[0] as i32, 0.5)?;
    let second = fixtures::geometry(values[1] as i32, 0.5)?;
    let first_edge = BRepBuilderAPIMakeEdge::from_curve_params(&first, 0., 1.)?.edge()?;
    let second_edge = BRepBuilderAPIMakeEdge::from_curve_params(&second, 0., 1.)?.edge()?;
    let first_adaptor = BRepAdaptorCurve::with_edge(&first_edge)?;
    let second_adaptor = BRepAdaptorCurve::with_edge(&second_edge)?;
    let ranges = [
        Range::new(values[2], values[3]),
        Range::new(values[4], values[5]),
    ];
    let curves = [
        search_curve(&first, &first_adaptor, ranges[0], values[6])?,
        search_curve(&second, &second_adaptor, ranges[1], 2. * values[6])?,
    ];
    let search = SolutionSearch {
        curves,
        tolerance: values[6] + 2. * values[6],
    };
    let mut output = vec![[Range::new(17., 19.), Range::new(23., 29.)]];
    attempt(
        search
            .find(ranges, &mut output)
            .map(|split| integer(i32::from(split))),
    );
    integer(output.len() as i32);
    for pair in output {
        for range in pair {
            real(range.first());
            real(range.last());
        }
    }
    Ok(())
}
