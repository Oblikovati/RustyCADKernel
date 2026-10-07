//! `TopOpeBRepDS`: the data structure of the topological operations (points, curves,
//! surfaces and shapes, and the interferences between them).
mod interference;
mod kind;
mod package;
mod transition;

pub use interference::{
    TopOpeBRepDSCurvePointInterference, TopOpeBRepDSEdgeVertexInterference,
    TopOpeBRepDSFaceEdgeInterference, TopOpeBRepDSInterference, TopOpeBRepDSShapeShapeInterference,
    TopOpeBRepDSSolidSurfaceInterference, TopOpeBRepDSSurfaceCurveInterference,
};
pub use kind::{TopOpeBRepDSCheckStatus, TopOpeBRepDSConfig, TopOpeBRepDSKind};
pub use package::{
    is_geometry, is_topology, kind_to_shape, print_config, print_kind, print_kind_index,
    print_shape_type_index, print_state, shape_to_kind, sprint_config, sprint_kind,
    sprint_kind_index, sprint_orientation, sprint_shape_type, sprint_shape_type_index,
    sprint_state,
};
pub use transition::TopOpeBRepDSTransition;
