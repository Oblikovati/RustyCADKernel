//! Adaptor preparation for the edge-rail `GeomFill_Pipe` branch.
use super::offset_rails_geometry::construction_error;
use super::offset_rails_geometry::unsupported_edge;
use rustycad_tkbrep::{
    brep::{BRepTool, EdgeCurveOnSurface},
    topods::Edge,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::{geom2d::Geom2dTrimmedCurve, geom2d_adaptor::Geom2dAdaptorCurve};
use rustycad_tkg3d::{
    adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface},
    geom::{GeomCurve, GeomSurface, GeomTrimmedCurve},
    geom_adaptor::GeomAdaptorCurve,
};
use rustycad_tkgeomalgo::geom_fill::GeomFillPipe;
use rustycad_tkmath::geom_abs::Shape as Continuity;

fn build_rail_pipe(
    path: &Edge,
    edge1: &Edge,
    edge2: &Edge,
    offset: f64,
    tolerance: f64,
    polynomial: bool,
    continuity: Continuity,
) -> Result<GeomFillPipe, Failure> {
    let curves = rail_curves(path, edge1, edge2)?;
    let adaptors = rail_adaptors(edge1, edge2, &curves)?;
    perform_rail_pipe(&adaptors, offset.abs(), tolerance, polynomial, continuity)
}

pub(super) fn build_ready_rail_pipe(
    path: &Edge,
    edge1: &Edge,
    edge2: &Edge,
    offset: f64,
    tolerance: f64,
    polynomial: bool,
    continuity: Continuity,
) -> Result<GeomFillPipe, Failure> {
    let pipe = build_rail_pipe(
        path, edge1, edge2, offset, tolerance, polynomial, continuity,
    )?;
    if pipe.is_done() {
        return Ok(pipe);
    }
    Err(construction_error("GeomFill_Pipe: Cannot make a surface"))
}

fn rail_curves(
    path: &Edge,
    edge1: &Edge,
    edge2: &Edge,
) -> Result<[Handle<dyn GeomCurve>; 3], Failure> {
    Ok([
        trimmed_curve(path)?,
        trimmed_curve_if_present(edge1)?,
        trimmed_curve_if_present(edge2)?,
    ])
}

fn rail_adaptors(
    edge1: &Edge,
    edge2: &Edge,
    curves: &[Handle<dyn GeomCurve>; 3],
) -> Result<[Handle<dyn Adaptor3dCurve>; 3], Failure> {
    Ok([
        adaptor(&curves[0])?,
        edge_adaptor(edge1, &curves[1])?,
        edge_adaptor(edge2, &curves[2])?,
    ])
}

fn perform_rail_pipe(
    adaptors: &[Handle<dyn Adaptor3dCurve>; 3],
    radius: f64,
    tolerance: f64,
    polynomial: bool,
    continuity: Continuity,
) -> Result<GeomFillPipe, Failure> {
    let mut pipe =
        GeomFillPipe::with_adaptor_rails(&adaptors[0], &adaptors[1], &adaptors[2], radius);
    pipe.perform(tolerance, polynomial, continuity, 11, 30)?;
    Ok(pipe)
}

pub(super) fn trimmed_curve(edge: &Edge) -> Result<Handle<dyn GeomCurve>, Failure> {
    let (curve, first, last) = BRepTool::curve(edge)?;
    let trimmed = GeomTrimmedCurve::new(&curve, first, last, true, true)?;
    Ok(Handle::new(trimmed).cast())
}

fn trimmed_curve_if_present(edge: &Edge) -> Result<Handle<dyn GeomCurve>, Failure> {
    if BRepTool::curve_with_location(edge)?.curve.is_null() {
        return Ok(Handle::default());
    }
    trimmed_curve(edge)
}

fn adaptor(curve: &Handle<dyn GeomCurve>) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
    Ok(Handle::new(GeomAdaptorCurve::with_curve(curve)?).cast())
}

fn edge_adaptor(
    edge: &Edge,
    curve: &Handle<dyn GeomCurve>,
) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
    if curve.is_some() {
        return adaptor(curve);
    }
    pcurve_adaptor(edge)
}

fn pcurve_adaptor(edge: &Edge) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
    let representation = BRepTool::first_curve_on_surface(edge)?;
    let curve = adapted_pcurve(&representation)?;
    let surface = adapted_pcurve_surface(&representation)?;
    Ok(Handle::new(Adaptor3dCurveOnSurface::with_curve(&curve, &surface)?).cast())
}

fn adapted_pcurve(
    representation: &EdgeCurveOnSurface,
) -> Result<Handle<dyn rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d>, Failure> {
    let trimmed = Geom2dTrimmedCurve::new(
        &representation.pcurve,
        representation.first,
        representation.last,
        true,
        true,
    )?;
    Ok(Handle::new(Geom2dAdaptorCurve::with_curve(
        &Handle::new(trimmed).cast(),
    )?)
    .cast())
}

fn adapted_pcurve_surface(
    representation: &EdgeCurveOnSurface,
) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
    let surface = representation
        .surface
        .get()
        .ok_or_else(|| unsupported_edge("p-curve surface"))?;
    let surface = surface
        .transformed(representation.location.transformation())?
        .cast::<dyn GeomSurface>();
    Ok(
        Handle::new(rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface::with_surface(&surface)?)
            .cast(),
    )
}
