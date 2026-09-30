//! Restriction internal vertices with empty, coincident and repeated vertex inputs.
use super::super::{
    common::{Reader, attempt, end, int},
    contap_surface, restriction_internal,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dTopolTool;
use rustycad_tkhlr::contap::{ContapHCurve2dTool, ContapLine, ContapPoint};
use rustycad_tkmath::gp::Point3;
fn seed(line: &ContapLine, first: f64, last: f64, scenario: i32) {
    let parameters = match scenario {
        0 => vec![],
        1 => vec![(first + last) / 2.],
        2 => vec![first, last],
        _ => vec![first, (first + last) / 2., (first + last) / 2., last],
    };
    for parameter in parameters {
        let mut point = ContapPoint::with_value(Point3::new(1., 2., 3.), 4., 5.);
        point.set_parameter(parameter);
        line.add_vertex(&point);
    }
}
pub fn internal_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| {
        let surface = contap_surface::surface_adaptor(&mut reader)?;
        let domain = Adaptor3dTopolTool::with_surface(&surface)?;
        let (_, function) = super::attachment::functions(&surface, &mut reader)?;
        let scenario = reader.integer();
        domain.init();
        while domain.more() {
            let arc = domain.value()?;
            let (first, last) = (
                ContapHCurve2dTool::first_parameter(&arc)?,
                ContapHCurve2dTool::last_parameter(&arc)?,
            );
            let mut line = ContapLine::new();
            line.set_arc(&arc);
            seed(&line, first, last, scenario);
            attempt(|| {
                restriction_internal::compute_restriction_internal(&line, first, last, &function)
            });
            super::sections::report(&line)?;
            domain.next();
        }
        int(-1);
        Ok::<(), Failure>(())
    });
    end();
}
