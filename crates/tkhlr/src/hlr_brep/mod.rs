// HLRBRep at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Exact hidden-line curve and topology tools.
mod area_limit;
pub use area_limit::AreaLimit;
mod b_curve_tool;
pub use b_curve_tool::BCurveTool;

mod b_surface_tool;
pub use b_surface_tool::BSurfaceTool;

mod cl_props;
pub use cl_props::{CLProps, CLPropsATool};
mod curve;
mod curve_2d;
pub use curve::{Curve, MinMaxBox};
mod curve_tool;
mod inter_c_surf;
pub use inter_c_surf::{
    InterCSurf, InterCSurfTypes, TheCSFunctionOfInterCSurf, TheExactInterCSurf,
    TheInterferenceOfInterCSurf, ThePolygonOfInterCSurf, ThePolygonToolOfInterCSurf,
    ThePolyhedronOfInterCSurf, ThePolyhedronToolOfInterCSurf, TheQuadCurvExactInterCSurf,
    TheQuadCurvFuncOfTheQuadCurvExactInterCSurf,
};
mod line_tool;
pub use line_tool::LineTool;
mod sl_props;
pub use sl_props::{SLProps, SLPropsATool};
mod surface;
pub use surface::Surface;
mod surface_tool;
pub use curve_tool::CurveTool;
pub use surface_tool::SurfaceTool;
mod c_inter;
pub use c_inter::{
    CInter, CInterTypes, ExactIntersectionPointOfTheIntPCurvePCurveOfCInter, IntConicCurveOfCInter,
    MyImpParToolOfTheIntersectorOfTheIntConicCurveOfCInter,
    PCLocFOfTheLocateExtPCOfTheProjPCurOfCInter, TheCurveLocatorOfTheProjPCurOfCInter,
    TheDistBetweenPCurvesOfTheIntPCurvePCurveOfCInter, TheIntConicCurveOfCInter,
    TheIntPCurvePCurveOfCInter, TheIntersectorOfTheIntConicCurveOfCInter,
    TheLocateExtPCOfTheProjPCurOfCInter, ThePolygon2dOfTheIntPCurvePCurveOfCInter,
    TheProjPCurOfCInter,
};

// Consumed by the remaining HLRBRep_Data port; not a public OCCT declaration.
#[cfg_attr(not(test), expect(dead_code))]
mod rejection_cache;
