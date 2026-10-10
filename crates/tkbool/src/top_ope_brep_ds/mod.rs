//! `TopOpeBRepDS`: the data structure of the topological operations (points, curves,
//! surfaces and shapes, and the interferences between them).
mod build_tool;
mod curve;
mod data;
mod data_structure;
mod hdata_structure;
mod interference;
mod interference_tool;
mod iterators;
mod kind;
mod package;
mod point;
mod transition;

pub use build_tool::{
    EdgeCurveTol, OrientedEdgeVertices, TopOpeBRepDSBuildTool, fun_get_uv,
    fun_make_uiso_line_on_sphe, fun_reverse_pc, fun_uiso_line_on_sphe, topopebrepds_set_the_pcurve,
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
pub use transition::TopOpeBRepDSTransition;
