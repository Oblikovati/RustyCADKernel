//! `TopOpeBRepDS`: the data structure of the topological operations (points, curves,
//! surfaces and shapes, and the interferences between them).
mod build_tool;
mod check;
mod connex;
mod curve;
mod data;
mod data_structure;
mod dumper;
mod edge3d_interference_tool;
mod edge_interference_tool;
mod eir;
pub mod export;
mod face_interference_tool;
mod filter;
mod fir;
mod funk;
mod hdata_structure;
mod interference;
mod interference_tool;
mod iterators;
mod kind;
mod package;
mod point;
pub mod process_edge_interferences;
pub mod process_face_interferences;
pub mod process_interferences_tool;
mod redu;
mod repvg;
mod tki;
mod tool;
mod transition;

pub use build_tool::{
    EdgeCurveTol, OrientedEdgeVertices, TopOpeBRepDSBuildTool, fun_get_uv,
    fun_make_uiso_line_on_sphe, fun_reverse_pc, fun_uiso_line_on_sphe, topopebrepds_set_the_pcurve,
};
pub use check::TopOpeBRepDSCheck;
pub use connex::{
    fdscnx_close, fdscnx_dump, fdscnx_dump_index, fdscnx_dump_shape,
    fdscnx_edge_connexity_same_shape, fdscnx_edge_connexity_same_shape_ds,
    fdscnx_edge_connexity_shape_index, fdscnx_edge_connexity_shape_index_ds,
    fdscnx_face_edge_connex_faces, fdscnx_face_edge_connex_faces_ds, fdscnx_has_connex_face,
    fdscnx_has_connex_face_ds, fdscnx_prepare, fdscnx_prepare_ds,
};
pub use curve::TopOpeBRepDSCurve;
pub use data::{
    ListOfInterference, TopOpeBRepDSCurveData, TopOpeBRepDSGeometryData,
    TopOpeBRepDSListOfShapeOn1State, TopOpeBRepDSPointData, TopOpeBRepDSShapeData,
    TopOpeBRepDSShapeWithState, TopOpeBRepDSSurfaceData,
};
pub use data_structure::{
    MapOfShapeWithState, TopOpeBRepDSCurveExplorer, TopOpeBRepDSDataStructure,
    TopOpeBRepDSPointExplorer,
};
pub use dumper::TopOpeBRepDSDumper;
pub use edge_interference_tool::TopOpeBRepDSEdgeInterferenceTool;
pub use edge3d_interference_tool::TopOpeBRepDSEdge3dInterferenceTool;
pub use eir::{TopOpeBRepDSEIR, fun_recl_se};
pub use face_interference_tool::{
    FaceGeomData, TopOpeBRepDSFaceInterferenceTool, fun_compute_geom_data,
    fun_compute_geom_data_norm, fun_parameters,
};
pub use filter::{TopOpeBRepDSFilter, TopOpeBRepDSReducer};
pub use fir::{
    TopOpeBRepDSFIR, fun_find_p_on_f, fun_gmap_s, fun_is_p_on_f, fun_reduce_edge_geometry,
    fun_reduce_edge_geometry1, fun_stateedgeface,
};
pub use funk::fun_unkf_stasta;
pub(crate) use hdata_structure::store_interference_ds;
pub use hdata_structure::{
    TopOpeBRepDSHDataStructure, fun_hds_facesinterfer, fun_topopebrepds_sort_on_parameter,
};
pub use interference::{
    TopOpeBRepDSCurvePointInterference, TopOpeBRepDSEdgeVertexInterference,
    TopOpeBRepDSFaceEdgeInterference, TopOpeBRepDSInterference, TopOpeBRepDSShapeShapeInterference,
    TopOpeBRepDSSolidSurfaceInterference, TopOpeBRepDSSurfaceCurveInterference,
};
pub use interference_tool::TopOpeBRepDSInterferenceTool;
pub use iterators::{
    TopOpeBRepDSCurveIterator, TopOpeBRepDSInterferenceIterator, TopOpeBRepDSPointIterator,
    TopOpeBRepDSSurfaceIterator,
};
pub use kind::{TopOpeBRepDSCheckStatus, TopOpeBRepDSConfig, TopOpeBRepDSKind};
pub use package::{
    is_geometry, is_topology, kind_to_shape, print_config, print_kind, print_kind_index,
    print_shape_type_index, print_state, shape_to_kind, sprint_config, sprint_kind,
    sprint_kind_index, sprint_orientation, sprint_shape_type, sprint_shape_type_index,
    sprint_state,
};
pub use point::{TopOpeBRepDSPoint, TopOpeBRepDSSurface};
pub use redu::{SideState, fun_ds_get_tr, fun_ds_redu2d1d, fun_scanloi};
pub use repvg::{fds_repvg, fds_repvg2};
pub use tki::TopOpeBRepDSTKI;
pub use tool::{MapOfShapeOn1State, TopOpeBRepDSTOOL};
pub use transition::TopOpeBRepDSTransition;
