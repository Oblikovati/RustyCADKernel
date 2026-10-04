use rustycad_tkbrep::{
    brep::{BRepCurveRepresentation, BRepGCurve, access::edge_tshape},
    topods::Edge,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::{geom2d::Geom2dCurve, geom2d_adaptor::Geom2dAdaptorCurve};
use rustycad_tkg3d::{
    adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface},
    geom::{GeomCurve, GeomSurface},
    geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface},
};
pub(super) struct ReferenceCurve {
    pub representation: Handle<dyn BRepCurveRepresentation>,
    pub adaptor: Handle<dyn Adaptor3dCurve>,
    pub range: (f64, f64),
}
pub(super) fn reference_curve(edge: &Edge) -> Result<Option<ReferenceCurve>, Failure> {
    let topology = edge_tshape(edge, "CorrectCurveOnSurface")?;
    if !topology.same_range() && topology.same_parameter() {
        return Ok(None);
    }
    let curves = topology.curves();
    let Some(representation) = select_reference(&curves, topology.degenerated())? else {
        return Ok(None);
    };
    let parametric: Handle<dyn BRepGCurve> = representation.cast();
    let range = parametric.range();
    if range.1 <= range.0 {
        return Ok(None);
    }
    let adaptor = adapt_reference(&representation, range)?;
    Ok(Some(ReferenceCurve {
        representation,
        adaptor,
        range,
    }))
}
fn select_reference(
    curves: &[Handle<dyn BRepCurveRepresentation>],
    degenerated: bool,
) -> Result<Option<Handle<dyn BRepCurveRepresentation>>, Failure> {
    let spatial: Vec<_> = curves.iter().filter(|curve| curve.is_curve3d()).collect();
    if spatial.len() != 1 {
        return Ok(None);
    }
    if !spatial[0].curve3d()?.is_null() {
        return Ok((!degenerated).then(|| spatial[0].clone()));
    }
    if degenerated {
        return Ok(None);
    }
    Ok(curves
        .iter()
        .find(|curve| curve.is_curve_on_surface())
        .cloned())
}
fn adapt_reference(
    representation: &Handle<dyn BRepCurveRepresentation>,
    range: (f64, f64),
) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
    let transform = representation.location().transformation();
    if representation.is_curve3d() {
        let curve: Handle<dyn GeomCurve> = representation.curve3d()?.transformed(transform)?.cast();
        return Ok(Handle::new(GeomAdaptorCurve::with_range(&curve, range.0, range.1)?).cast());
    }
    let surface: Handle<dyn GeomSurface> = representation.surface()?.transformed(transform)?.cast();
    Ok(curve_on_surface(&representation.pcurve()?, &surface, range)?.cast())
}
pub(super) fn curve_on_surface(
    curve: &Handle<dyn Geom2dCurve>,
    surface: &Handle<dyn GeomSurface>,
    range: (f64, f64),
) -> Result<Handle<Adaptor3dCurveOnSurface>, Failure> {
    let curve = Handle::new(Geom2dAdaptorCurve::with_range(curve, range.0, range.1)?).cast();
    let surface = Handle::new(GeomAdaptorSurface::with_surface(surface)?).cast();
    Ok(Handle::new(Adaptor3dCurveOnSurface::with_curve(
        &curve, &surface,
    )?))
}
