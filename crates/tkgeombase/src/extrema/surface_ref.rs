// The `const Adaptor3d_Surface*` members of the Extrema surface classes at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The surface an Extrema object keeps. Natively a raw pointer to an adaptor that
//! the caller owns, or that a handle held by an enclosing object keeps alive
//! (Extrema_ExtPExtS and Extrema_ExtPRevS keep the handle of the adaptor that their
//! Extrema_GenExtPS points to); here a borrow or a shared handle.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;

/// A borrowed or a shared surface adaptor; `SurfaceRef::Borrowed(&surface).get()`.
#[derive(Clone)]
pub(crate) enum SurfaceRef<'a> {
    /// An adaptor the caller keeps alive.
    Borrowed(&'a dyn Adaptor3dSurface),
    /// An adaptor kept alive by the handle.
    Shared(Handle<dyn Adaptor3dSurface>),
}

impl SurfaceRef<'_> {
    /// The adaptor; `surface.get().value(0.0, 0.0)?`.
    pub fn get(&self) -> &dyn Adaptor3dSurface {
        match self {
            SurfaceRef::Borrowed(surface) => *surface,
            SurfaceRef::Shared(handle) => &**handle,
        }
    }
}
