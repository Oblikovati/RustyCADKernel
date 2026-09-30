//! Complete contour orchestration with repeated searches and diagnostic parity.
use super::super::{
    common::{Reader, attempt, end, flag, int, show, text},
    contap_surface,
};
use rustycad_tkg3d::adaptor3d::Adaptor3dTopolTool;
use rustycad_tkhlr::contap::ContapContour;
use rustycad_tkmath::gp::{Point3, Vector3};
pub fn full_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| {
        let surface = contap_surface::surface_adaptor(&mut reader)?;
        let domain = Adaptor3dTopolTool::with_surface(&surface)?;
        let mode = reader.integer();
        let direction = Vector3::from(reader.dir());
        let mut contour = ContapContour::new();
        match mode {
            0 => contour.init_direction(direction)?,
            1 => contour.init_eye(Point3::new(3., 4., 5.)),
            _ => contour.init_direction_angle(direction, 0.2)?,
        }
        for _ in 0..2 {
            let mut console = Vec::new();
            let result = contour.perform_with_diagnostics(&surface, &domain, &mut console);
            text(&console);
            show(result, |()| {});
            flag(contour.is_done());
            if contour.is_done() {
                int(contour.nb_lines()?);
                for index in 1..=contour.nb_lines()? {
                    super::analytic_case::report(contour.line(index)?, &domain)?;
                }
            }
        }
        Ok(())
    });
    end();
}
