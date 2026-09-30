// Rust translation of ModelingData/TKG3d/Geom/Geom_BoundedSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_BoundedSurface`: surfaces limited by finite parameters.
use super::surface::GeomSurface;
use rustycad_tkernel::impl_transient_interface;

/// A surface with finite bounds; the native class adds no member;
/// `surface.cast::<dyn GeomBoundedSurface>()`.
pub trait GeomBoundedSurface: GeomSurface {}
impl_transient_interface!(
    dyn GeomBoundedSurface,
    "Geom_BoundedSurface",
    dyn GeomSurface
);
