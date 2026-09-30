// Port of Contap_HContTool at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Native contour sampling and arc queries.
/// Contap_HContTool: static queries. The arc and vertex members are the ones of
/// `IntPatch_HInterTool` (the two native classes have the same bodies); the surface
/// sampling is Contap's own and keeps its range in process-wide statics
/// (docs/hlr-design.md section 6).
///
/// ```
/// use rustycad_tkhlr::contap::ContapHContTool;
/// assert!(!ContapHContTool::has_been_seen(&Default::default()));
/// assert!(ContapHContTool::value(&Default::default(), 1).is_err());
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct ContapHContTool;

mod arcs;
mod sampling;
mod traits;
