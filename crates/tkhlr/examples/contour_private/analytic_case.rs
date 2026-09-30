//! Elementary contour orchestration compared through native public Perform.
use super::super::{
    analytic::AnalyticContour,
    common::{Reader, attempt, end, flag, int},
    contap_surface,
};
use rustycad_tkg3d::adaptor3d::Adaptor3dTopolTool;
use rustycad_tkhlr::contap::{ContapArcFunction, ContapSurfFunction, ContapTheSearch};
use rustycad_tkmath::gp::{Direction3, Point3};
pub fn analytic_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| {
        let surface = contap_surface::surface_adaptor(&mut reader)?;
        let domain = Adaptor3dTopolTool::with_surface(&surface)?;
        let mode = reader.integer();
        let input = reader.dir();
        // Native Init accepts gp_Vec, implicitly normalizing it again for each Set.
        let direction = Direction3::new(input.x(), input.y(), input.z())?;
        let (mut function, mut arc) = (ContapSurfFunction::new(), ContapArcFunction::new());
        match mode {
            0 => {
                function.set_direction(direction);
                arc.set_direction(direction);
            }
            1 => {
                let eye = Point3::new(3., 4., 5.);
                function.set_eye(eye);
                arc.set_eye(eye);
            }
            _ => {
                function.set_direction_angle(direction, 0.2);
                arc.set_direction_angle(direction, 0.2);
            }
        }
        let mut contour = AnalyticContour {
            done: false,
            lines: Vec::new(),
            search: ContapTheSearch::new(),
            function,
            arc_function: arc,
        };
        for _ in 0..2 {
            attempt(|| {
                contour.function.set_surface(&surface)?;
                contour.arc_function.set_surface(&surface)?;
                contour.perform(&domain)
            });
            flag(contour.done);
            if contour.done {
                int(contour.lines.len() as i32);
                for line in &contour.lines {
                    report(line, &domain)?;
                }
            }
        }
        Ok(())
    });
    end();
}

pub(super) fn report(
    line: &rustycad_tkhlr::contap::ContapLine,
    domain: &Adaptor3dTopolTool,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    super::super::contour_records::line_geometry(line);
    super::sections::report(line)?;
    if let Ok(arc) = line.arc() {
        int(super::super::contour_boundary::arc_rank(domain, &arc));
    }
    for index in 1..=line.nb_vertex() {
        let point = line.vertex(index)?;
        super::super::contour_records::point_snapshot(&point);
        if let Ok(arc) = point.arc() {
            int(super::super::contour_boundary::arc_rank(domain, &arc));
        }
    }
    Ok(())
}
