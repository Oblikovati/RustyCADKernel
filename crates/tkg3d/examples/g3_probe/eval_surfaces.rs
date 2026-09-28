//! The GeomEval elementary surfaces of the TKG3d probe; the counterpart of tools/oracle/tkg3d_probe_eval_surfaces.cxx.
use super::print::Reader;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;

/// One eval surface specification of kind `kind`; the kinds are filled in by the port.
pub fn spec(kind: i32, _r: &mut Reader) -> Result<Handle<dyn GeomSurface>, Failure> {
    panic!("eval surface kind {kind} has no Rust builder in the probe")
}
