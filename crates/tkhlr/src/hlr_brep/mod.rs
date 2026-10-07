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
mod edge_data;
pub use edge_data::EdgeData;
mod edge_face_tool;
pub use edge_face_tool::{EdgeFaceTool, TypeOfResultingEdge};
mod face_data;
pub use face_data::FaceData;
mod face_iterator;
pub use face_iterator::FaceIterator;
mod inter_c_surf;
pub use inter_c_surf::{
    InterCSurf, InterCSurfTypes, TheCSFunctionOfInterCSurf, TheExactInterCSurf,
    TheInterferenceOfInterCSurf, ThePolygonOfInterCSurf, ThePolygonToolOfInterCSurf,
    ThePolyhedronOfInterCSurf, ThePolyhedronToolOfInterCSurf, TheQuadCurvExactInterCSurf,
    TheQuadCurvFuncOfTheQuadCurvExactInterCSurf,
};
mod line_tool;
mod make_edge;
pub use line_tool::LineTool;
pub use make_edge::{make_edge, make_edge_3d, poly_hlr_angle_and_deflection};
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
