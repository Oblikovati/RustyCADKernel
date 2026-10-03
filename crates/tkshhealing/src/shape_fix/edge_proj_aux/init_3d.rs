#![allow(dead_code)] // OCCT exposes Init3d only to subclasses; public Compute uses Init2d.

use super::EdgeProjAuxState;
use super::projection;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::Geom2dBoundedCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::extrema::ExtPC;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Point3;

pub(super) fn initialize(state: &mut EdgeProjAuxState, tolerance: f64) -> Result<(), Failure> {
    let surface = BRepTool::surface(&state.face)?;
    let pcurve = BRepTool::curve_on_face(&state.edge, &state.face)?.pcurve;
    if pcurve.is_null() {
        return Ok(());
    }
    let vertices = (
        topexp::first_vertex(&state.edge, true),
        topexp::last_vertex(&state.edge, true),
    );
    let points = (BRepTool::pnt(&vertices.0)?, BRepTool::pnt(&vertices.1)?);
    let range = (
        pcurve.get().map_or(0.0, |curve| curve.first_parameter()),
        pcurve.get().map_or(0.0, |curve| curve.last_parameter()),
    );
    let adaptor = projection::adapt_geometry(&pcurve, &surface, range)?;
    if bounded_endpoints_match(&pcurve, &adaptor, points, tolerance)? {
        state.first_param = range.0;
        state.last_param = range.1;
        finish_both(state);
        return Ok(());
    }
    project_extrema(state, &pcurve, &adaptor, points, range, tolerance)
}

fn bounded_endpoints_match(
    curve: &rustycad_tkernel::standard::Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
    adaptor: &impl Adaptor3dCurve,
    points: (Point3, Point3),
    tolerance: f64,
) -> Result<bool, Failure> {
    if curve.cast::<dyn Geom2dBoundedCurve>().is_null() {
        return Ok(false);
    }
    let range = (adaptor.first_parameter()?, adaptor.last_parameter()?);
    Ok(adaptor.value(range.0)?.distance(&points.0) <= tolerance
        && adaptor.value(range.1)?.distance(&points.1) <= tolerance)
}

fn project_extrema(
    state: &mut EdgeProjAuxState,
    pcurve: &rustycad_tkernel::standard::Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
    adaptor: &impl Adaptor3dCurve,
    points: (Point3, Point3),
    range: (f64, f64),
    tolerance: f64,
) -> Result<(), Failure> {
    let continuity = pcurve.get().ok_or_else(null_pcurve)?.continuity();
    let finite = !precision::is_infinite(range.0) && !precision::is_infinite(range.1);
    if finite && continuity != Shape::C0 {
        let Some(first) = extrema_parameter(points.0, adaptor, range, tolerance)? else {
            return Ok(());
        };
        let Some(last) = extrema_parameter(points.1, adaptor, range, tolerance)? else {
            return Ok(());
        };
        state.first_param = first;
        state.last_param = last;
        super::parameter_update::update_parameters(state, pcurve)?;
    }
    finish_both(state);
    Ok(())
}

fn extrema_parameter(
    point: Point3,
    adaptor: &impl Adaptor3dCurve,
    range: (f64, f64),
    tolerance: f64,
) -> Result<Option<f64>, Failure> {
    let extrema = match ExtPC::with_range(point, adaptor, range.0, range.1, tolerance) {
        Ok(result) => result,
        Err(_) => return Ok(None),
    };
    if !extrema.is_done() {
        return Ok(None);
    }
    let mut parameter = range.0;
    for index in 1..=extrema.nb_ext()? {
        if extrema.is_min(index)? {
            parameter = extrema.point(index)?.parameter();
        }
    }
    Ok(Some(parameter))
}

fn finish_both(state: &mut EdgeProjAuxState) {
    state.first_done = true;
    state.last_done = true;
}

fn null_pcurve() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_EdgeProjAux: null p-curve; expected a live Geom2d_Curve",
    )
}
