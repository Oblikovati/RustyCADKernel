// SPDX-License-Identifier: GPL-2.0-only
//! `MeshVS_MeshPrsBuilder` at OCCT 3d097a0: the builder of the nodes, links, faces and
//! volumes of a mesh in the wireframe, shading and shrink modes, and of the highlighted and
//! selected entities. The overridable members are [`MeshVSMeshPrsBuilderSource`]; their
//! native bodies are the `meshvs_mesh_prs_*` functions.
mod draw_arrays;
mod elements;
mod elements_faces;
mod hilight;
mod nodes;
mod primitives;
mod solid;

pub(crate) use draw_arrays::copy_fill as draw_arrays_copy_fill;
pub use draw_arrays::{DrawArraysInput, meshvs_mesh_prs_draw_arrays};
pub use elements::meshvs_mesh_prs_build_elements;
pub use hilight::meshvs_mesh_prs_build_hilight_prs;
pub use nodes::meshvs_mesh_prs_build_nodes;
pub(crate) use primitives::{fan_index, node_xyz, volume_edge_map};
pub use solid::{MeshVSFaceSolidOptions, meshvs_mesh_prs_add_face_solid_prs};

use crate::data_source::{MeshVSDataSource, MeshVSPackedMap};
use crate::flags::{MESHVS_DMF_HILIGHT_PRS, MESHVS_DMF_OCC_MASK};
use crate::mesh::MeshVSMeshSource;
use crate::prs_builder::{MeshVSPrsBuilder, MeshVSPrsBuilderBase};
use crate::selection_policy::MESHVS_BP_MESH;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;

/// The overridable members of `MeshVS_MeshPrsBuilder`; the default bodies are the native ones.
pub trait MeshVSMeshPrsBuilderSource: MeshVSPrsBuilder {
    /// `BuildNodes`: the markers of the visible nodes.
    fn build_nodes(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        ids: &MeshVSPackedMap,
        ids_to_exclude: &mut MeshVSPackedMap,
        display_mode: i32,
    ) -> Result<(), Failure> {
        meshvs_mesh_prs_build_nodes(self, prs, ids, ids_to_exclude, display_mode)
    }
    /// `BuildElements`: the links, faces and volumes of the visible elements.
    fn build_elements(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        ids: &MeshVSPackedMap,
        ids_to_exclude: &mut MeshVSPackedMap,
        display_mode: i32,
    ) -> Result<(), Failure> {
        meshvs_mesh_prs_build_elements(self, prs, ids, ids_to_exclude, display_mode)
    }
    /// `BuildHilightPrs`: the first entity of `ids`, highlighted.
    fn build_hilight_prs(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        ids: &MeshVSPackedMap,
        is_element: bool,
    ) -> Result<(), Failure> {
        meshvs_mesh_prs_build_hilight_prs(self, prs, ids, is_element)
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn MeshVSMeshPrsBuilderSource,
    "MeshVS_MeshPrsBuilder",
    dyn MeshVSPrsBuilder
);

/// The native `MeshVS_MeshPrsBuilder::Build` body: the highlight of one entity, else the
/// elements or the nodes. Example: `meshvs_mesh_prs_build(&*builder, &prs, &ids, &mut excluded, true, 1)?`.
pub fn meshvs_mesh_prs_build<B: MeshVSMeshPrsBuilderSource + ?Sized>(
    builder: &B,
    prs: &Handle<dyn Graphic3dStructureSource>,
    ids: &MeshVSPackedMap,
    ids_to_exclude: &mut MeshVSPackedMap,
    is_element: bool,
    display_mode: i32,
) -> Result<(), Failure> {
    if display_mode <= 0 {
        return Ok(());
    }
    let has_hilight = (display_mode & MESHVS_DMF_HILIGHT_PRS) != 0;
    if has_hilight && ids.extent() == 1 {
        return builder.build_hilight_prs(prs, ids, is_element);
    }
    if is_element {
        return builder.build_elements(prs, ids, ids_to_exclude, display_mode);
    }
    builder.build_nodes(prs, ids, ids_to_exclude, display_mode)
}

/// The concrete `MeshVS_MeshPrsBuilder`; `MeshVSMeshPrsBuilder::new(&mesh, MESHVS_DMF_OCC_MASK, &Handle::default(), -1, MESHVS_BP_MESH)`.
pub struct MeshVSMeshPrsBuilder {
    transient: StandardTransient,
    base: MeshVSPrsBuilderBase,
}
rustycad_tkernel::impl_transient!(MeshVSMeshPrsBuilder, "MeshVS_MeshPrsBuilder", dyn MeshVSPrsBuilder, transient,
    interfaces: [dyn MeshVSMeshPrsBuilderSource, dyn MeshVSPrsBuilder]);

impl MeshVSMeshPrsBuilder {
    /// `MeshVS_MeshPrsBuilder(Parent, Flags, DS, Id, Priority)`.
    pub fn new(
        parent: &Handle<dyn MeshVSMeshSource>,
        flags: i32,
        data_source: &Handle<dyn MeshVSDataSource>,
        id: i32,
        priority: i32,
    ) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: MeshVSPrsBuilderBase::new(parent, flags, data_source, id, priority),
        }
    }
    /// The native default arguments: `MeshVS_DMF_OCCMask`, no own data source, a free id and
    /// `MeshVS_BP_Mesh`. Example: `MeshVSMeshPrsBuilder::with_defaults(&mesh)`.
    pub fn with_defaults(parent: &Handle<dyn MeshVSMeshSource>) -> Self {
        Self::new(
            parent,
            MESHVS_DMF_OCC_MASK,
            &Handle::default(),
            -1,
            MESHVS_BP_MESH,
        )
    }
}

impl MeshVSPrsBuilder for MeshVSMeshPrsBuilder {
    fn prs_builder_base(&self) -> &MeshVSPrsBuilderBase {
        &self.base
    }
    fn build(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        ids: &MeshVSPackedMap,
        ids_to_exclude: &mut MeshVSPackedMap,
        is_element: bool,
        display_mode: i32,
    ) -> Result<(), Failure> {
        meshvs_mesh_prs_build(self, prs, ids, ids_to_exclude, is_element, display_mode)
    }
}

impl MeshVSMeshPrsBuilderSource for MeshVSMeshPrsBuilder {}
