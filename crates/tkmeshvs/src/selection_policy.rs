// SPDX-License-Identifier: GPL-2.0-only
//! Martensite mesh selection policies retain OCCT 3d097a0 values and mask combinations.
//!
//! ```
//! use martensite_tkmeshvs::selection_policy::*;
//! let grouped_faces = MeshVSSelectionModeFlags(MESHVS_SMF_GROUP.0 | MESHVS_SMF_FACE.0);
//! assert_eq!(grouped_faces.0, 264);
//! assert_eq!(MESHVS_BP_DEFAULT, MESHVS_BP_USER);
//! ```

/// Original signed priority; `let priority: MeshVSBuilderPriority = -7` remains representable.
pub type MeshVSBuilderPriority = i32;

/// Native bit mask, including unnamed combinations; `MeshVSSelectionModeFlags(0x109)`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct MeshVSSelectionModeFlags(pub i32);

/// Native whole-mesh sensitive-entity policy; `MeshVSMeshSelectionMethod::Nodes as i32 == 1`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum MeshVSMeshSelectionMethod {
    /// Native `MeshVS_MSM_PRECISE`.
    Precise = 0,
    /// Native `MeshVS_MSM_NODES`.
    Nodes = 1,
    /// Native `MeshVS_MSM_BOX`.
    Box = 2,
    /// C++ permits unnamed value 3 within this enumeration's native range.
    Unnamed3 = 3,
}

/// Preserve the complete defined C++ enum range; `mesh_selection_method_from_native(3).is_some()`.
pub fn mesh_selection_method_from_native(value: i32) -> Option<MeshVSMeshSelectionMethod> {
    match value {
        0 => Some(MeshVSMeshSelectionMethod::Precise),
        1 => Some(MeshVSMeshSelectionMethod::Nodes),
        2 => Some(MeshVSMeshSelectionMethod::Box),
        3 => Some(MeshVSMeshSelectionMethod::Unnamed3),
        _ => None,
    }
}

/// Native `MeshVS_BP_Mesh` priority.
pub const MESHVS_BP_MESH: MeshVSBuilderPriority = 5;
/// Native `MeshVS_BP_NodalColor` priority.
pub const MESHVS_BP_NODAL_COLOR: MeshVSBuilderPriority = 10;
/// Native `MeshVS_BP_ElemColor` priority.
pub const MESHVS_BP_ELEM_COLOR: MeshVSBuilderPriority = 15;
/// Native `MeshVS_BP_Text` priority.
pub const MESHVS_BP_TEXT: MeshVSBuilderPriority = 20;
/// Native `MeshVS_BP_Vector` priority.
pub const MESHVS_BP_VECTOR: MeshVSBuilderPriority = 25;
/// Native `MeshVS_BP_User` extension priority.
pub const MESHVS_BP_USER: MeshVSBuilderPriority = 30;
/// Native `MeshVS_BP_Default` aliases the user priority.
pub const MESHVS_BP_DEFAULT: MeshVSBuilderPriority = MESHVS_BP_USER;

/// Native `MeshVS_SMF_Mesh` selects the whole mesh.
pub const MESHVS_SMF_MESH: MeshVSSelectionModeFlags = MeshVSSelectionModeFlags(0);
/// Native `MeshVS_SMF_Node` selects nodes.
pub const MESHVS_SMF_NODE: MeshVSSelectionModeFlags = MeshVSSelectionModeFlags(1);
/// Native `MeshVS_SMF_0D` selects zero-dimensional elements.
pub const MESHVS_SMF_0D: MeshVSSelectionModeFlags = MeshVSSelectionModeFlags(2);
/// Native `MeshVS_SMF_Link` selects links.
pub const MESHVS_SMF_LINK: MeshVSSelectionModeFlags = MeshVSSelectionModeFlags(4);
/// Native `MeshVS_SMF_Face` selects faces.
pub const MESHVS_SMF_FACE: MeshVSSelectionModeFlags = MeshVSSelectionModeFlags(8);
/// Native `MeshVS_SMF_Volume` selects volumes.
pub const MESHVS_SMF_VOLUME: MeshVSSelectionModeFlags = MeshVSSelectionModeFlags(16);
/// Native `MeshVS_SMF_Element` combines all element kinds.
pub const MESHVS_SMF_ELEMENT: MeshVSSelectionModeFlags = MeshVSSelectionModeFlags(2 | 4 | 8 | 16);
/// Native `MeshVS_SMF_All` combines nodes and elements.
pub const MESHVS_SMF_ALL: MeshVSSelectionModeFlags =
    MeshVSSelectionModeFlags(MESHVS_SMF_ELEMENT.0 | MESHVS_SMF_NODE.0);
/// Native `MeshVS_SMF_Group` can qualify entity-kind masks.
pub const MESHVS_SMF_GROUP: MeshVSSelectionModeFlags = MeshVSSelectionModeFlags(0x100);
