//! `TopOpeBRepBuild`: the builders of the topological operations (the areas of loops
//! of edges and faces, the face builder and the shape builder).
mod area_builder;
mod block_builder;
mod builder;
mod composite_classifier;
mod correct_face2d;
mod edge_builder;
mod face_builder;
mod gtool;
mod gtopo;
mod loops;
mod pave;
mod shape_set;
mod solid_builder;
mod vertex_info;
mod wire_edge_classifier;
mod wire_edge_set;
mod wire_to_face;

pub use area_builder::{
    LoopList, TopOpeBRepBuildArea2dBuilder, TopOpeBRepBuildAreaBuilder,
    TopOpeBRepBuildFaceAreaBuilder,
};
pub use block_builder::TopOpeBRepBuildBlockBuilder;
pub use builder::TopOpeBRepBuildBuilder;
pub use composite_classifier::TopOpeBRepBuildCompositeClassifier;
pub use correct_face2d::{MapOfTrans2dInfo, TopOpeBRepBuildCorrectFace2d};
pub use edge_builder::{
    TopOpeBRepBuildArea1dBuilder, TopOpeBRepBuildEdgeBuilder, TopOpeBRepBuildPaveClassifier,
};
pub use face_builder::{
    CLOSEDW, GCLOSEDW, ISUNKNOWN, ISVERTEX, ShapeToShapeMap, TopOpeBRepBuildFaceBuilder, UNCLOSEDW,
    fun_analyzemap_von1e, fun_detect_vertices_on1_edge,
};
pub use gtool::{TopOpeBRepBuildGIter, TopOpeBRepBuildGTool};
pub use gtopo::TopOpeBRepBuildGTopo;
pub use loops::{
    TopOpeBRepBuildBlockIterator, TopOpeBRepBuildLoop, TopOpeBRepBuildLoopBase,
    TopOpeBRepBuildLoopClassifier, TopOpeBRepBuildLoopEnum, TopOpeBRepBuildLoopSet,
    TopOpeBRepBuildLoopSetBase, TopOpeBRepBuildPlainLoopSet, TopOpeBRepBuildShapeListOfShape,
};
pub use pave::{TopOpeBRepBuildPave, TopOpeBRepBuildPaveSet};
pub use shape_set::{
    TopOpeBRepBuildPlainShapeSet, TopOpeBRepBuildShapeSet, TopOpeBRepBuildShapeSetBase,
};
pub use solid_builder::{
    TopOpeBRepBuildArea3dBuilder, TopOpeBRepBuildShellFaceClassifier, TopOpeBRepBuildShellFaceSet,
    TopOpeBRepBuildSolidAreaBuilder, TopOpeBRepBuildSolidBuilder,
};
pub use vertex_info::{MapOfShapeVertexInfo, TopOpeBRepBuildTools2d, TopOpeBRepBuildVertexInfo};
pub use wire_edge_classifier::TopOpeBRepBuildWireEdgeClassifier;
pub use wire_edge_set::TopOpeBRepBuildWireEdgeSet;
pub use wire_to_face::TopOpeBRepBuildWireToFace;
