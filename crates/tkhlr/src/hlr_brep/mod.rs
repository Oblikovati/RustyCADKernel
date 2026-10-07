// HLRBRep at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Exact hidden-line curve and topology tools.
mod algo;
pub use algo::Algo;
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
mod data;
pub use data::{Data, EdgeDataArray, FaceDataArray, LocalGeometry2D};
mod edge_builder;
pub use edge_builder::EdgeBuilder;
mod edge_data;
pub use edge_data::{EdgeData, EdgeDataMask};
mod edge_face_tool;
mod edge_i_list;
pub use edge_i_list::EdgeIList;
mod edge_interference_tool;
pub use edge_face_tool::{EdgeFaceTool, TypeOfResultingEdge};
pub use edge_interference_tool::EdgeInterferenceTool;
mod face_data;
pub use face_data::{FaceData, FaceDataMask};
mod face_iterator;
pub use face_iterator::FaceIterator;
mod hider;
pub use hider::Hider;
mod hlr_to_shape;
pub use hlr_to_shape::HLRToShape;
mod inter_c_surf;
pub use inter_c_surf::{
    InterCSurf, InterCSurfTypes, TheCSFunctionOfInterCSurf, TheExactInterCSurf,
    TheInterferenceOfInterCSurf, ThePolygonOfInterCSurf, ThePolygonToolOfInterCSurf,
    ThePolyhedronOfInterCSurf, ThePolyhedronToolOfInterCSurf, TheQuadCurvExactInterCSurf,
    TheQuadCurvFuncOfTheQuadCurvExactInterCSurf,
};
mod internal_algo;
pub use internal_algo::InternalAlgo;
mod intersector;
pub use intersector::Intersector;
mod line_tool;
mod make_edge;
pub use line_tool::LineTool;
pub use make_edge::{make_edge, make_edge_3d, poly_hlr_angle_and_deflection};
mod shape_bounds;
pub use shape_bounds::{ShapeBounds, ShapeRanges};
mod shape_to_hlr;
pub use shape_to_hlr::ShapeToHLR;
mod sl_props;
pub use sl_props::{SLProps, SLPropsATool};
mod surface;
pub use surface::Surface;
mod surface_tool;
mod vertex_list;
pub use curve_tool::CurveTool;
pub use surface_tool::SurfaceTool;
pub use vertex_list::VertexList;
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

// The private HLRBRep_Data::RejectionCache.
mod rejection_cache;
