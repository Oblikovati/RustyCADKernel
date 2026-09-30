// Rust translation of ModelingData/TKG3d/Geom/Geom_SweptSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_SweptSurface`: surfaces swept by a basis curve along or around a
//! direction (the native protected `basisCurve`, `direction` and `smooth`).
use super::curve::GeomCurve;
use super::geometry::GeomGeometry;
use super::surface::{GeomSurface, dump_surface};
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkg2d::geom2d::{dump_base, dump_class_name, dump_field};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Direction3;
use std::io;

/// A swept surface; `extrusion.basis_curve()`.
pub trait GeomSweptSurface: GeomSurface {
    /// The sweep direction (`Direction`); `surface.direction()`.
    fn direction(&self) -> Direction3;
    /// The swept curve (`BasisCurve`); `surface.basis_curve()`.
    fn basis_curve(&self) -> Handle<dyn GeomCurve>;
}
impl_transient_interface!(dyn GeomSweptSurface, "Geom_SweptSurface", dyn GeomSurface);

/// `Geom_SweptSurface::DumpJson`; `dump_swept_surface(&mut stream, -1, &basis, dir, smooth)?`.
pub fn dump_swept_surface(
    stream: &mut DumpStream,
    depth: i32,
    basis: &Handle<dyn GeomCurve>,
    direction: Direction3,
    smooth: Shape,
) -> io::Result<()> {
    dump_class_name(stream, "Geom_SweptSurface")?;
    dump_base(stream, depth, dump_surface)?;
    if let Some(curve) = basis.get() {
        dump_field(stream, depth, "basisCurve", |nested, level| {
            GeomGeometry::dump_json(curve, nested, level)
        })?;
    }
    dump_field(stream, depth, "direction", |nested, _| {
        direction.dump_json(nested)
    })?;
    dump::dump_integer(stream, "smooth", smooth as i32)
}
