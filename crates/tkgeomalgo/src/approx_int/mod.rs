// Rust translation of ModelingAlgorithms/TKGeomAlgo/ApproxInt at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package ApproxInt: the approximation of intersection lines. Its `.gxx`
//! templates are Rust generics (docs/geomalgo-design.md section 20), instantiated by
//! GeomInt (`crate::geom_int`) as natively:
//!
//! - `ApproxInt_SvSurfaces` is the trait [`ApproxIntSvSurfaces`];
//! - `ApproxInt_PrmPrmSvSurfaces.gxx` is [`ApproxIntPrmPrmSvSurfaces`];
//! - `ApproxInt_ImpPrmSvSurfaces.gxx` is [`ApproxIntImpPrmSvSurfaces`];
//! - `ApproxInt_MultiLine.gxx` is [`ApproxIntMultiLine`], and
//!   `ApproxInt_MultiLineTool.lxx` its `LineTool` implementation;
//! - `ApproxInt_Approx.gxx` is [`ApproxIntApprox`];
//! - `ApproxInt_KnotTools` is [`ApproxIntKnotTools`].
mod approx;
mod imp_prm_sv_surfaces;
mod knot_tools;
mod line;
mod multi_line;
mod prm_prm_sv_surfaces;
mod surface_tool;
mod sv_cache;
mod sv_surfaces;
mod test_points;

pub use approx::{ApproxIntApprox, ApproxIntApproxData, ApproxIntApproxTypes, ApproxIntRange};
pub use imp_prm_sv_surfaces::{ApproxIntImpPrmSvSurfaces, PatchBounds};
pub use knot_tools::{ApproxIntKnotPointSets, ApproxIntKnotTools};
pub use line::ApproxIntLine;
pub use multi_line::{ApproxIntMultiLine, ApproxIntMultiLineLayout, ApproxIntOffsets};
pub use prm_prm_sv_surfaces::ApproxIntPrmPrmSvSurfaces;
pub use surface_tool::ApproxIntSurfaceTool;
pub use sv_surfaces::{
    ApproxIntSvComputed, ApproxIntSvSurfaces, ApproxIntSvSurfacesBase, SvSurfacesRef,
};
