//! Exact curve-on-surface projection and cache/status digest.
use super::analysis_surface::analysis_surface_geometry;
use super::brep_lib_common::geom_curve;
use super::common::{Reader, attempt, end, flag, int, pnt2d, show};
use super::sh_common::curve2d;
use super::shape_construct::construct_spline_2d;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkshhealing::{
    shape_analysis::ShapeAnalysisSurface, shape_construct::ShapeConstructProjectCurveOnSurface,
    shape_extend::ShapeExtendStatus,
};

pub fn project_curve_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| project_curve_request(&mut reader));
    end();
}
fn project_curve_request(reader: &mut Reader<'_>) -> Result<(), Failure> {
    let operation = reader.integer();
    let initialization = reader.integer();
    let surface = analysis_surface_geometry(reader)?;
    let tolerance = reader.real();
    let mode = reader.integer();
    let projector = ShapeConstructProjectCurveOnSurface::new();
    int(projector.adjust_over_degen_mode());
    projector.set_adjust_over_degen_mode(mode);
    int(projector.adjust_over_degen_mode());
    match initialization {
        0 => projector.init(&surface, tolerance)?,
        1 => projector.init_analysis(
            &Handle::new(ShapeAnalysisSurface::new(&surface)?),
            tolerance,
        ),
        2 => {
            projector.set_surface(&surface)?;
            projector.set_precision(tolerance);
        }
        _ => (),
    }
    let curve = geom_curve(reader)?;
    project_curve_run(&projector, &curve, reader, operation)
}
fn project_curve_run(
    projector: &ShapeConstructProjectCurveOnSurface,
    curve: &Handle<dyn GeomCurve>,
    reader: &mut Reader<'_>,
    operation: i32,
) -> Result<(), Failure> {
    let range = (reader.real(), reader.real());
    let tolerances = (reader.real(), reader.real());
    let count = reader.integer();
    for _ in 0..count {
        let mut result = Handle::default();
        if operation == 0 {
            show(
                projector.perform(curve, range, &mut result, tolerances),
                flag,
            );
        } else {
            flag(projector.perform_by_proj_lib(curve, range, &mut result));
        }
        project_curve_status(projector);
        curve2d(&result);
        construct_spline_2d(&result);
        if result.is_null() {
            continue;
        }
        for parameter in [range.0, (range.0 + range.1) / 2., range.1] {
            show(result.value(parameter), pnt2d);
        }
    }
    Ok(())
}
fn project_curve_status(projector: &ShapeConstructProjectCurveOnSurface) {
    use ShapeExtendStatus::*;
    for status in [
        Ok, Done1, Done2, Done3, Done4, Done5, Done6, Done7, Done8, Done, Fail1, Fail2, Fail3,
        Fail4, Fail5, Fail6, Fail7, Fail8, Fail,
    ] {
        flag(projector.status(status));
    }
}
