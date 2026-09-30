// Rust translation of ModelingAlgorithms/TKGeomAlgo/AppBlend at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package AppBlend: the B-spline approximation of a surface swept by
//! sections (fillets, sweeps, skinned surfaces).
//!
//! `AppBlend_Approx` is the abstract result ([`AppBlendApprox`]).
//! `AppBlend_AppSurf.gxx` is a template over `TheLine` and `TheSectionGenerator`
//! (docs/geomalgo-design.md, "AppBlend template"): since the class stores neither,
//! [`AppBlendAppSurf`] is one type whose `perform` methods are generic over the two
//! traits of `tools`, and each native instance (`GeomFill_AppSurf`,
//! `GeomFill_AppSweep`, `BRepBlend_AppSurf`) is an alias of it.
//! `AppBlend_Debug.cxx` holds the two process-wide switches of `context`.
mod app_surf;
mod approx;
mod context;
mod internal_perform;
mod sections;
mod solution;
mod tools;
mod tronc_perform;

pub use app_surf::AppBlendAppSurf;
pub use approx::{AppBlendApprox, AppBlendCurve2d, AppBlendSurface};
pub use context::{
    get_context_approx_with_no_tgt, get_context_spline_approx, set_context_approx_with_no_tgt,
    set_context_spline_approx,
};
pub use tools::{AppBlendSection, TheLine, TheSectionGenerator};
