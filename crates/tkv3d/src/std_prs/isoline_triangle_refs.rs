//! Borrowed native inputs remain live through user surface callbacks instead of copied handles/parameters.
use rustycad_tkernel::{
    ncollection::Sequence,
    standard::{Handle, SharedState},
};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::{poly::Triangulation, toploc::Location};
/// Native const references with safe callback-visible aliasing; `StdPrsIsolineTriangulationInputs { mesh, surface, location, parameters }`.
#[derive(Clone, Copy)]
pub struct StdPrsIsolineTriangulationInputs<'a> {
    /// Actual caller triangulation handle, which callbacks may replace.
    pub mesh: &'a SharedState<Handle<Triangulation>>,
    /// Actual caller surface handle, including the valid null-surface interpolation path.
    pub surface: &'a SharedState<Handle<dyn GeomSurface>>,
    /// Actual caller location, observed when each computed segment is transformed.
    pub location: &'a SharedState<Location>,
    /// U and V parameter references may be aliases; lengths follow native axis sampling policy.
    pub parameters: [&'a SharedState<Sequence<f64>>; 2],
}
