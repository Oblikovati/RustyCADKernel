//! The BRepMesh face triangulation algorithms (OCCT 3d097a03): the
//! `BRepMesh_BaseMeshAlgo` hierarchy with its node-insertion and deflection-control
//! templates, and `BRepMesh_MeshAlgoFactory`.
//!
//! The C++ virtual protected members become the hook traits
//! [`BaseMeshAlgoHooks`]/[`ConstrainedBaseMeshAlgoHooks`]/[`NodeInsertionHooks`] whose
//! default bodies are the native ones; the `BaseAlgo` template argument is a
//! [`MeshGenerator`]. Every instance is reached through the
//! [`BRepMeshDelaunayBaseMeshAlgo`] handle, the dynamic type the templates inherit.
mod algo_object;
mod base;
mod collect;
mod deflection_control;
mod deflection_split;
mod delaunay_insertion;
mod factory;
mod generator;
mod node_insertion;

pub use algo_object::{
    BRepMeshDelaunayBaseMeshAlgo, BaseMeshAlgo, ConstrainedBaseMeshAlgo, CustomBaseMeshAlgo,
    FaceMeshingCore,
};
pub use base::{
    BaseMeshAlgoData, BaseMeshAlgoHooks, ConstrainedBaseMeshAlgoHooks, base_init_data_structure,
    base_perform, base_register_node,
};
pub use deflection_control::DelaunayDeflectionControlMeshAlgo;
pub use delaunay_insertion::{
    DelaunayInsertionHooks, DelaunayNodeInsertionMeshAlgo, delaunay_insertion_cells_count,
    delaunay_insertion_init_data_structure, delaunay_insertion_post_process, insert_nodes,
};
pub use factory::BRepMeshMeshAlgoFactory;
pub use generator::{
    CustomBaseTriangulation, CustomDelaunayGenerator, CustomGenerator, DelaunayBaseMeshAlgoCore,
    DelaunayGenerator, MeshGenerator,
};
pub use node_insertion::{
    NodeInsertionData, NodeInsertionHooks, NodeInsertionMeshAlgo,
    node_insertion_add_node_to_structure, node_insertion_init_data_structure,
    node_insertion_perform, node_insertion_point2d,
};
