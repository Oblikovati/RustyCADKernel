//! Rebuilds a section curve and both p-curves over a clipped parameter interval.
use rustycad_tkbo::int_tools::Curve;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::geom::{GeomCurve, GeomTrimmedCurve};

pub(super) fn trim(curve: &Curve, (first, last): (f64, f64)) -> Result<Curve, Failure> {
    let geometry = trim_geometry(curve.curve(), first, last)?;
    let first_pcurve = trim_pcurve(curve.first_curve2d(), first, last)?;
    let second_pcurve = trim_pcurve(curve.second_curve2d(), first, last)?;
    Ok(Curve::new(
        &geometry,
        &first_pcurve,
        &second_pcurve,
        curve.tolerance(),
        curve.tangential_tolerance(),
    ))
}

fn trim_geometry(
    curve: &Handle<dyn GeomCurve>,
    first: f64,
    last: f64,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    Ok(Handle::new(GeomTrimmedCurve::new(curve, first, last, true, true)?).cast())
}

fn trim_pcurve(
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    if curve.is_null() {
        return Ok(Handle::default());
    }
    Ok(Handle::new(Geom2dTrimmedCurve::new(curve, first, last, true, true)?).cast())
}
