// Rust translation of the IntPatch instances of the IntImp, IntStart and IntWalk templates:
// IntPatch_TheSurfFunction, IntPatch_CurvIntSurf, IntPatch_TheSOnBounds,
// IntPatch_TheSearchInside and IntPatch_TheIWalking (.hxx/_0.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The IntPatch instances of the generic templates (docs/geomalgo-design.md sections
//! 6, 10, 12 and 16): each one is an alias of the generic over a bundle that names the
//! IntPatch classes, as the `_0.cxx` files name them in their macros.
use super::arc_function::IntPatchArcFunction;
use super::cs_function::IntPatchCSFunction;
use super::h_inter_tool::IntPatchHInterTool;
use super::iwline::IntPatchTheIWLineOfTheIWalking;
use super::path_point::{IntPatchThePathPointOfTheSOnBounds, IntPatchTheSegmentOfTheSOnBounds};
use crate::int_imp::{HSurfaceTool, IntImpIntCS, IntImpZerImpFunc};
use crate::int_start::{
    SearchInside, SearchInsideTypes, SearchOnBoundaries, SearchOnBoundariesTypes, TheTopolTool,
};
use crate::int_surf::IntSurfQuadricTool;
use crate::int_walk::{IWalkingTypes, IntWalkIWalking};

/// `IntPatch_TheSurfFunction`: `IntImp_ZerImpFunc.gxx` on `Adaptor3d_HSurfaceTool` and
/// `IntSurf_QuadricTool`.
///
/// `IntPatchTheSurfFunction::with_surfaces(&surface, &quadric)`.
pub type IntPatchTheSurfFunction = IntImpZerImpFunc<HSurfaceTool, IntSurfQuadricTool>;

/// `IntPatch_CurvIntSurf`: `IntImp_IntCS.gxx` on `IntPatch_CSFunction`.
///
/// `IntPatchCurvIntSurf::new([u, v, w], &function, 1e-7, 0.0)?`.
pub type IntPatchCurvIntSurf = IntImpIntCS<IntPatchCSFunction>;

/// The macro types of `IntPatch_TheSOnBounds`: the domain is the virtual
/// `Adaptor3d_TopolTool` (any [`TheTopolTool`]), the tool `IntPatch_HInterTool`, the
/// function `IntPatch_ArcFunction`.
pub struct IntPatchTheSOnBoundsTypes;

impl SearchOnBoundariesTypes for IntPatchTheSOnBoundsTypes {
    type TopolTool = dyn TheTopolTool;
    type SOBTool = IntPatchHInterTool;
    type Function = IntPatchArcFunction;
    type PathPoint = IntPatchThePathPointOfTheSOnBounds;
    type Segment = IntPatchTheSegmentOfTheSOnBounds;
}

/// `IntPatch_TheSOnBounds`: `IntStart_SearchOnBoundaries.gxx` on the IntPatch classes.
///
/// `IntPatchTheSOnBounds::new().perform(&mut function, &domain, 1e-7, 1e-6, false)?`.
pub type IntPatchTheSOnBounds = SearchOnBoundaries<IntPatchTheSOnBoundsTypes>;

/// The macro types of `IntPatch_TheSearchInside`.
pub struct IntPatchTheSearchInsideTypes;

impl SearchInsideTypes for IntPatchTheSearchInsideTypes {
    type SurfaceTool = HSurfaceTool;
    type TopolTool = dyn TheTopolTool;
    type Function = IntPatchTheSurfFunction;
}

/// `IntPatch_TheSearchInside`: `IntStart_SearchInside.gxx` on `IntPatch_TheSurfFunction`.
///
/// `IntPatchTheSearchInside::with_domain(&mut function, &surface, &domain, 1e-6)?`.
pub type IntPatchTheSearchInside = SearchInside<IntPatchTheSearchInsideTypes>;

/// The macro types of `IntPatch_TheIWalking`.
pub struct IntPatchTheIWalkingTypes;

impl IWalkingTypes for IntPatchTheIWalkingTypes {
    type SurfaceTool = HSurfaceTool;
    type Function = IntPatchTheSurfFunction;
    type Line = IntPatchTheIWLineOfTheIWalking;
}

/// `IntPatch_TheIWalking`: `IntWalk_IWalking.gxx` on `IntPatch_TheSurfFunction` and
/// `IntPatch_TheIWLineOfTheIWalking`.
///
/// `IntPatchTheIWalking::new(1e-6, 0.01, 0.1, false)?`.
pub type IntPatchTheIWalking = IntWalkIWalking<IntPatchTheIWalkingTypes>;
