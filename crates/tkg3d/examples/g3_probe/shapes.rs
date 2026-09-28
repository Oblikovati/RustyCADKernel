//! The curve and surface specifications of tools/oracle/tkg3d_probe_common.cxx
//! (g3_curve and g3_surface), built with the Rust classes.
use super::print::Reader;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomLine, GeomSurface};

/// One curve specification, as g3_curve reads it.
pub fn curve(r: &mut Reader) -> Result<Handle<dyn GeomCurve>, Failure> {
    let kind = r.integer();
    match kind {
        0 => {
            let p = r.pnt();
            Ok(Handle::new(GeomLine::from_point(p, r.dir())).cast())
        }
        1 => {
            let frame = r.ax2();
            Ok(Handle::new(GeomCircle::new(frame, r.real())?).cast())
        }
        _ => panic!("curve kind {kind} has no Rust builder in the probe"),
    }
}

/// One surface specification, as g3_surface reads it.
pub fn surface(r: &mut Reader) -> Result<Handle<dyn GeomSurface>, Failure> {
    let kind = r.integer();
    panic!("surface kind {kind} has no Rust builder in the probe")
}
