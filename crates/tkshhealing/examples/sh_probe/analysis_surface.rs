//! Native surface-analysis state, singularity ordering and Newton projection.
use super::brep_lib_common::geom_curve;
use super::common::{Reader, attempt, end, flag, int, pnt, real, show};
use super::face::surface;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomOffsetSurface, GeomSurface, GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution,
};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisSurface;

pub fn analysis_surface_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| analysis_surface_request(&mut reader));
    end();
}

fn analysis_surface_request(reader: &mut Reader<'_>) -> Result<(), Failure> {
    let operation = reader.integer();
    let analysis = ShapeAnalysisSurface::new(&analysis_surface_geometry(reader)?)?;
    let precision = reader.real();
    match operation {
        0 => singularity_digest(&analysis, precision)?,
        1 => closure_digest(&analysis, precision)?,
        4 => degenerate_digest(&analysis, &mut *reader, precision)?,
        5 => degenerate_points_digest(&analysis, &mut *reader, precision)?,
        _ => {
            let (hint, point, _initial) = (reader.pnt2d(), reader.pnt(), reader.pnt2d());
            let solution = if operation == 2 {
                analysis.value_of_uv(point, precision)?
            } else {
                analysis.next_value_of_uv(hint, point, precision, 1e-7)?
            };
            real(solution.x());
            real(solution.y());
        }
    }
    real(analysis.gap());
    Ok(())
}

fn singularity_digest(analysis: &ShapeAnalysisSurface, precision: f64) -> Result<(), Failure> {
    int(analysis.nb_singularities(precision)? as i32);
    flag(analysis.has_singularities(precision)?);
    for index in 0..6 {
        let entry = analysis.singularity(index)?;
        flag(entry.is_some());
        let Some(entry) = entry else {
            continue;
        };
        real(entry.precision);
        pnt(entry.point);
        for value in [
            entry.first_uv.x(),
            entry.first_uv.y(),
            entry.last_uv.x(),
            entry.last_uv.y(),
            entry.first_parameter,
            entry.last_parameter,
        ] {
            real(value);
        }
        flag(entry.u_iso);
        flag(analysis.is_degenerated(entry.point, precision)?);
        real(analysis.gap());
    }
    Ok(())
}

fn closure_digest(analysis: &ShapeAnalysisSurface, precision: f64) -> Result<(), Failure> {
    for _ in 0..2 {
        flag(analysis.is_u_closed(precision)?);
        real(analysis.u_close_val());
        real(analysis.gap());
        flag(analysis.is_v_closed(precision)?);
        real(analysis.v_close_val());
        real(analysis.gap());
    }
    Ok(())
}

fn degenerate_digest(
    analysis: &ShapeAnalysisSurface,
    reader: &mut Reader<'_>,
    tolerance: f64,
) -> Result<(), Failure> {
    let point = reader.pnt();
    let neighbour = reader.pnt2d();
    let mut uv = reader.pnt2d();
    flag(analysis.project_degenerated(point, tolerance, neighbour, &mut uv)?);
    real(uv.x());
    real(uv.y());
    real(analysis.gap());
    let entry = analysis.degenerated_values(point, tolerance, true)?;
    flag(entry.is_some());
    if let Some(entry) = entry {
        real(entry.first_uv.x());
        real(entry.first_uv.y());
        real(entry.last_uv.x());
        real(entry.last_uv.y());
        real(entry.first_parameter);
        real(entry.last_parameter);
    }
    flag(analysis.is_degenerated_segment(neighbour, uv, tolerance, 10.)?);
    Ok(())
}

fn degenerate_points_digest(
    analysis: &ShapeAnalysisSurface,
    reader: &mut Reader<'_>,
    tolerance: f64,
) -> Result<(), Failure> {
    let direct = reader.flag();
    let count = reader.integer();
    let stored = reader.integer();
    let mut points = Vec::new();
    let mut uv = Vec::new();
    for _ in 0..stored {
        points.push(reader.pnt());
        uv.push(reader.pnt2d());
    }
    show(
        analysis.project_degenerated_sequence(count, &points, &mut uv, tolerance, direct),
        flag,
    );
    for parameter in uv {
        real(parameter.x());
        real(parameter.y());
    }
    Ok(())
}

pub(super) fn analysis_surface_geometry(
    reader: &mut Reader<'_>,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    match reader.integer() {
        0 => surface(reader),
        1 => {
            let offset = reader.real();
            Ok(Handle::new(GeomOffsetSurface::new(&surface(reader)?, offset, false)?).cast())
        }
        2 => Ok(Handle::new(GeomSurfaceOfLinearExtrusion::new(
            &geom_curve(reader)?,
            Direction3::Z,
        )?)
        .cast()),
        _ => Ok(Handle::new(GeomSurfaceOfRevolution::new(
            &geom_curve(reader)?,
            Axis1::new(Point3::default(), Direction3::Z),
        )?)
        .cast()),
    }
}
