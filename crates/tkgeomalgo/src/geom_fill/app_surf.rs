// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_AppSurf.hxx and
// GeomFill_AppSurf_0.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_AppSurf`: the instance of the template `AppBlend_AppSurf.gxx` on
//! `GeomFill_Line` and `GeomFill_SectionGenerator`.
use crate::app_blend::AppBlendAppSurf;

/// The approximation of a skinned surface (`GeomFill_AppSurf`). The template class
/// stores neither its line nor its generator, so the instance is the AppBlend type
/// itself (docs/geomalgo-design.md sections 7 and 13): `GeomFillLine` implements
/// `TheLine` and `GeomFillSectionGenerator` implements `TheSectionGenerator`, and
/// the `perform` methods take them.
///
/// `let mut app = GeomFillAppSurf::new(3, 8, 1e-7, 1e-9, 0, false);
/// app.perform(line.get(), &mut generator, false)?;`
pub type GeomFillAppSurf = AppBlendAppSurf;
