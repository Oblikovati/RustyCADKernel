//! The curve and surface specifications of tools/oracle/tkg3d_probe_common.cxx
//! (g3_curve and g3_surface), built with the Rust classes.
use super::print::Reader;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomCircle, GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomEllipse, GeomHyperbola,
    GeomLine, GeomParabola, GeomPlane, GeomSphericalSurface, GeomSurface,
    GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution, GeomToroidalSurface, GeomTrimmedCurve,
};

/// One curve specification, as g3_curve reads it.
pub fn curve(r: &mut Reader) -> Result<Handle<dyn GeomCurve>, Failure> {
    let kind = r.integer();
    match kind {
        0 => {
            let p = r.pnt();
            Ok(Handle::new(GeomLine::from_point(p, r.dir())).cast())
        }
        1..=4 => conic(kind, r),
        7 => trimmed(r),
        _ => panic!("curve kind {kind} has no Rust builder in the probe"),
    }
}

// Kinds 1 to 4: circle F R, ellipse F A B, hyperbola F A B, parabola F focal.
fn conic(kind: i32, r: &mut Reader) -> Result<Handle<dyn GeomCurve>, Failure> {
    let (frame, x) = (r.ax2(), r.real());
    match kind {
        1 => Ok(Handle::new(GeomCircle::new(frame, x)?).cast()),
        4 => Ok(Handle::new(GeomParabola::new(frame, x)?).cast()),
        2 => Ok(Handle::new(GeomEllipse::new(frame, x, r.real())?).cast()),
        _ => Ok(Handle::new(GeomHyperbola::new(frame, x, r.real())?).cast()),
    }
}

// Kind 7: trimmed u1 u2 sense adjust <curve>.
fn trimmed(r: &mut Reader) -> Result<Handle<dyn GeomCurve>, Failure> {
    let (u1, u2) = (r.real(), r.real());
    let (sense, adjust) = (r.flag(), r.flag());
    let basis = curve(r)?;
    Ok(Handle::new(GeomTrimmedCurve::new(&basis, u1, u2, sense, adjust)?).cast())
}

/// One surface specification, as g3_surface reads it.
pub fn surface(r: &mut Reader) -> Result<Handle<dyn GeomSurface>, Failure> {
    let kind = r.integer();
    match kind {
        0..=4 => elementary(kind, r),
        5 => {
            let d = r.dir();
            let basis = curve(r)?;
            Ok(Handle::new(GeomSurfaceOfLinearExtrusion::new(&basis, d)?).cast())
        }
        6 => {
            let a = r.ax1();
            let basis = curve(r)?;
            Ok(Handle::new(GeomSurfaceOfRevolution::new(&basis, a)?).cast())
        }
        _ => panic!("surface kind {kind} has no Rust builder in the probe"),
    }
}

// Kinds 0 to 4: plane F, cylinder F R, cone F angle R, sphere F R, torus F R r.
fn elementary(kind: i32, r: &mut Reader) -> Result<Handle<dyn GeomSurface>, Failure> {
    let frame = r.ax3();
    if kind == 0 {
        return Ok(Handle::new(GeomPlane::new(frame)).cast());
    }
    let x = r.real();
    match kind {
        1 => Ok(Handle::new(GeomCylindricalSurface::new(frame, x)?).cast()),
        3 => Ok(Handle::new(GeomSphericalSurface::new(frame, x)?).cast()),
        2 => Ok(Handle::new(GeomConicalSurface::new(frame, x, r.real())?).cast()),
        _ => Ok(Handle::new(GeomToroidalSurface::new(frame, x, r.real())?).cast()),
    }
}
