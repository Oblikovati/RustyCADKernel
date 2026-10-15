// SPDX-License-Identifier: GPL-2.0-only
//! Martensite mesh appearance and immutable visualization connectivity.
//! The mesh interactive object, its data sources, presentation builders, owners and sensitive
//! entities render through the TKV3d presentations (the Vulkan 1.3 driver), never OpenGL.
//!
//! ```
//! use martensite_tkmeshvs::{MeshVSDrawer, MeshVSDrawerSource};
//! use rustycad_tkernel::standard::Handle;
//! let source = Handle::new(MeshVSDrawer::default());
//! source.set_integer(7, 42);
//! let destination = MeshVSDrawer::default();
//! destination.assign(&source);
//! let mut output = 9;
//! assert!(!destination.get_integer(8, &mut output));
//! assert_eq!(output, 9);
//! assert!(destination.get_integer(7, &mut output));
//! assert_eq!(output, 42);
//! ```
pub mod drawer;
pub mod flags;
pub mod node_pairs;
pub mod selection_policy;
pub mod two_colors;
pub use drawer::{MeshVSDrawer, MeshVSDrawerSource, meshvs_drawer_assign};
pub use flags::{MeshVSDisplayModeFlags, MeshVSDrawerAttribute, MeshVSEntityType};
pub use node_pairs::{MeshVSNodePair, MeshVSSymmetricPairHasher, MeshVSTwoNodes};
pub use two_colors::{
    MeshVSColorOutputs, MeshVSTwoColors, bind_two_colors, extract_color, extract_colors,
    extract_colors_into,
};

pub mod tool;
pub use tool::MeshVSTool;

pub mod volume_topology;
pub use volume_topology::{
    MeshVSTopologyError, MeshVSTopologyOperation, MeshVSVolumeTopology, create_prism_topology,
    create_pyramid_topology,
};

pub mod buffer;
pub use buffer::{MESHVS_BUF_SIZE, MeshVSBuffer};

pub mod data_source;
pub use data_source::{
    MeshVSDataSource, MeshVSDetectedMaps, MeshVSPackedMap, MeshVSTopologyHandle,
};

pub mod data_source_3d;
pub use data_source_3d::{MeshVSDataSource3D, MeshVSDataSource3DBase};

pub mod deformed_data_source;
pub use deformed_data_source::MeshVSDeformedDataSource;

pub mod prs_builder;
pub use prs_builder::{MeshVSPrsBuilder, MeshVSPrsBuilderBase, MeshVSPrsBuilderMembers};

pub mod mesh;
pub use mesh::{MeshVSMesh, MeshVSMeshBase, MeshVSMeshMembers, MeshVSMeshSource};

pub mod mesh_entity_owner;
pub use mesh_entity_owner::MeshVSMeshEntityOwner;

pub mod mesh_owner;
pub use mesh_owner::{MeshVSMeshOwner, MeshVSMeshOwnerBase, MeshVSMeshOwnerSource};

pub mod common_sensitive_entity;
pub use common_sensitive_entity::MeshVSCommonSensitiveEntity;

pub mod sensitive_simple;
pub use sensitive_simple::{
    MeshVSDummySensitiveEntity, MeshVSSensitiveFace, MeshVSSensitiveSegment,
};

pub mod sensitive_quad;
pub use sensitive_quad::MeshVSSensitiveQuad;

pub mod sensitive_polyhedron;
pub use sensitive_polyhedron::MeshVSSensitivePolyhedron;

pub mod sensitive_mesh;
pub use sensitive_mesh::MeshVSSensitiveMesh;

pub mod mesh_prs_builder;
pub use mesh_prs_builder::{MeshVSMeshPrsBuilder, MeshVSMeshPrsBuilderSource};

pub mod text_prs_builder;
pub use text_prs_builder::{MeshVSTextMaps, MeshVSTextPrsBuilder};

pub mod vector_prs_builder;
pub use vector_prs_builder::{MeshVSVectorFields, MeshVSVectorPrsBuilder};

pub mod nodal_color_prs_builder;
pub use nodal_color_prs_builder::{
    MeshVSImageTexture2D, MeshVSNodalColorFields, MeshVSNodalColorPrsBuilder,
};

pub mod elemental_color_prs_builder;
pub use elemental_color_prs_builder::{MeshVSElementalColorMaps, MeshVSElementalColorPrsBuilder};
