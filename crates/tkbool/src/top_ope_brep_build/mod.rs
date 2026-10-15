//! `TopOpeBRepBuild`: the builders of the topological operations (the areas of loops
//! of edges and faces, the face builder and the shape builder).
mod area_builder;
mod block_builder;
mod builder;
mod builder1;
mod builder_on;
mod composite_classifier;
mod correct_face2d;
mod edge_builder;
mod face_builder;
mod fuse_face;
mod globals;
mod gtool;
mod gtopo;
mod hbuilder;
mod loops;
mod pave;
mod shape_set;
mod shell_to_solid;
mod solid_builder;
mod tools;
mod vertex_info;
mod wire_edge_classifier;
mod wire_edge_set;
mod wire_to_face;

pub use area_builder::{
    LoopList, TopOpeBRepBuildArea2dBuilder, TopOpeBRepBuildAreaBuilder,
    TopOpeBRepBuildFaceAreaBuilder,
};
pub use block_builder::TopOpeBRepBuildBlockBuilder;
pub use builder::{
    TopOpeBRepBuildBuilder, fun_ismotheropedef, fun_motherope, fun_setmotherope,
    fun_unsetmotherope, kpresu,
};
pub use builder_on::{TopOpeBRepBuildBuilderON, fun_build_tb, fun_keep_eon};
pub use builder1::{TopOpeBRepBuildBuilder1, TwoPiecesSeq};
pub use composite_classifier::TopOpeBRepBuildCompositeClassifier;
pub use correct_face2d::{MapOfTrans2dInfo, TopOpeBRepBuildCorrectFace2d};
pub use edge_builder::{
    TopOpeBRepBuildArea1dBuilder, TopOpeBRepBuildEdgeBuilder, TopOpeBRepBuildPaveClassifier,
};
pub use face_builder::{
    CLOSEDW, GCLOSEDW, ISUNKNOWN, ISVERTEX, ShapeToShapeMap, TopOpeBRepBuildFaceBuilder, UNCLOSEDW,
    fun_analyzemap_von1e, fun_detect_vertices_on1_edge,
};
pub use fuse_face::TopOpeBRepBuildFuseFace;
pub use gtool::{TopOpeBRepBuildGIter, TopOpeBRepBuildGTool};
pub use gtopo::TopOpeBRepBuildGTopo;
pub use hbuilder::{HBuilderState, TopOpeBRepBuildHBuilder};
pub use loops::{
    TopOpeBRepBuildBlockIterator, TopOpeBRepBuildLoop, TopOpeBRepBuildLoopBase,
    TopOpeBRepBuildLoopClassifier, TopOpeBRepBuildLoopEnum, TopOpeBRepBuildLoopSet,
    TopOpeBRepBuildLoopSetBase, TopOpeBRepBuildPlainLoopSet, TopOpeBRepBuildShapeListOfShape,
};
pub use pave::{TopOpeBRepBuildPave, TopOpeBRepBuildPaveSet};
pub use shape_set::{
    TopOpeBRepBuildPlainShapeSet, TopOpeBRepBuildShapeSet, TopOpeBRepBuildShapeSetBase,
};
pub use shell_to_solid::TopOpeBRepBuildShellToSolid;
pub use solid_builder::{
    TopOpeBRepBuildArea3dBuilder, TopOpeBRepBuildShellFaceClassifier, TopOpeBRepBuildShellFaceSet,
    TopOpeBRepBuildSolidAreaBuilder, TopOpeBRepBuildSolidBuilder,
};
pub use tools::{DataMapOfShapeState, TopOpeBRepBuildTools};
pub use vertex_info::{MapOfShapeVertexInfo, TopOpeBRepBuildTools2d, TopOpeBRepBuildVertexInfo};
pub use wire_edge_classifier::TopOpeBRepBuildWireEdgeClassifier;
pub use wire_edge_set::TopOpeBRepBuildWireEdgeSet;
pub use wire_to_face::TopOpeBRepBuildWireToFace;
