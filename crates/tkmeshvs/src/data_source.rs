// SPDX-License-Identifier: GPL-2.0-only
//! `MeshVS_DataSource` at OCCT 3d097a0: the abstract mesh an application lends to the
//! presentation. The class is abstract with virtual members, so it is the
//! [`MeshVSDataSource`] trait; the default bodies are the native ones and stay callable as
//! the `meshvs_data_source_*` functions from an override.
//!
//! Native out-parameters stay `&mut` arguments: an implementation writes the coordinates of
//! `GetGeom` into the caller's one-based array, as natively. The native `void*` entity
//! addresses are opaque `usize` values here.
pub(crate) mod normals;
pub use normals::{
    meshvs_data_source_get_bounding_box, meshvs_data_source_get_normal,
    meshvs_data_source_get_normals_by_element,
};

use crate::MeshVSEntityType;
use crate::flags::MESHVS_ET_NONE;
use crate::mesh::MeshVSMeshSource;
use rustycad_tkernel::ncollection::{Array1, HArray1, PackedMap, Sequence};
use rustycad_tkernel::standard::{Handle, StandardTransient, Transient};
use rustycad_tkernel::tcol_std::HPackedMapOfInteger;
use rustycad_tkmath::bnd::{BoundingBox2, BoundingBox3};
use rustycad_tkmath::gp::Point2;

/// The native `occ::handle<NCollection_HArray1<NCollection_Sequence<int>>>` of a volume:
/// the zero-based node indices of each face.
pub type MeshVSTopologyHandle = Handle<HArray1<Sequence<i32>>>;

/// The native `TColStd_PackedMapOfInteger`.
pub type MeshVSPackedMap = PackedMap<i32>;

/// The abstract mesh of a presentation; the default bodies are the native ones.
///
/// ```ignore
/// impl MeshVSDataSource for MyMesh { /* the pure virtual members */ }
/// ```
pub trait MeshVSDataSource: Transient {
    /// `GetGeom` (pure virtual): the coordinates of a node or the nodes of an element.
    fn get_geom(
        &self,
        id: i32,
        is_element: bool,
        coords: &mut Array1<'_, f64>,
        nb_nodes: &mut i32,
        kind: &mut MeshVSEntityType,
    ) -> bool;
    /// `GetGeomType` (pure virtual): the entity type of a node or an element.
    fn get_geom_type(&self, id: i32, is_element: bool, kind: &mut MeshVSEntityType) -> bool;
    /// `Get3DGeom`: the face topology of a volume; false by default.
    fn get_3d_geom(&self, id: i32, nb_nodes: &mut i32, data: &mut MeshVSTopologyHandle) -> bool {
        let _ = (id, nb_nodes, data);
        false
    }
    /// `GetAddr` (pure virtual): the opaque address of a node or an element.
    fn get_addr(&self, id: i32, is_element: bool) -> usize;
    /// `GetNodesByElement` (pure virtual): the node ids of an element.
    fn get_nodes_by_element(
        &self,
        id: i32,
        node_ids: &mut Array1<'_, i32>,
        nb_nodes: &mut i32,
    ) -> bool;
    /// `GetAllNodes` (pure virtual): the ids of every node, by value.
    fn get_all_nodes(&self) -> MeshVSPackedMap;
    /// `GetAllElements` (pure virtual): the ids of every element, by value.
    fn get_all_elements(&self) -> MeshVSPackedMap;
    /// `GetNormal`: the normal of a face from its first, second and last nodes.
    fn get_normal(&self, id: i32, max: i32, nx: &mut f64, ny: &mut f64, nz: &mut f64) -> bool {
        meshvs_data_source_get_normal(self, id, max, [nx, ny, nz])
    }
    /// `GetNodeNormal`: no nodal normal by default.
    fn get_node_normal(
        &self,
        rank_node: i32,
        element_id: i32,
        nx: &mut f64,
        ny: &mut f64,
        nz: &mut f64,
    ) -> bool {
        let _ = (rank_node, element_id, nx, ny, nz);
        false
    }
    /// `GetNormalsByElement`: the nodal normals of a face or the face normals of a volume.
    fn get_normals_by_element(
        &self,
        id: i32,
        is_nodal: bool,
        max_nodes: i32,
        normals: &mut Handle<HArray1<f64>>,
    ) -> bool {
        meshvs_data_source_get_normals_by_element(self, id, is_nodal, max_nodes, normals)
    }
    /// `GetAllGroups`: no group by default.
    fn get_all_groups(&self, ids: &mut MeshVSPackedMap) {
        let _ = ids;
    }
    /// `GetGroup`: sets the type to `MeshVS_ET_NONE` and finds no group by default.
    fn get_group(&self, id: i32, kind: &mut MeshVSEntityType, ids: &mut MeshVSPackedMap) -> bool {
        let _ = (id, ids);
        *kind = MESHVS_ET_NONE;
        false
    }
    /// `GetGroupAddr`: the null address by default.
    fn get_group_addr(&self, id: i32) -> usize {
        let _ = id;
        0
    }
    /// `IsAdvancedSelectionEnabled`: false by default.
    fn is_advanced_selection_enabled(&self) -> bool {
        false
    }
    /// `GetBoundingBox`: the box of every node.
    fn get_bounding_box(&self) -> BoundingBox3 {
        meshvs_data_source_get_bounding_box(self)
    }
    /// `GetDetectedEntities(Prs, X, Y, aTol, Nodes, Elements, DMin)`: false by default.
    fn get_detected_entities_at_point(
        &self,
        mesh: &Handle<dyn MeshVSMeshSource>,
        point: [f64; 2],
        tolerance: f64,
        detected: MeshVSDetectedMaps<'_>,
        d_min: &mut f64,
    ) -> bool {
        let _ = (mesh, point, tolerance, detected, d_min);
        false
    }
    /// `GetDetectedEntities(Prs, XMin, YMin, XMax, YMax, aTol, Nodes, Elements)`: false by default.
    fn get_detected_entities_in_box(
        &self,
        mesh: &Handle<dyn MeshVSMeshSource>,
        bounds: [f64; 4],
        tolerance: f64,
        detected: MeshVSDetectedMaps<'_>,
    ) -> bool {
        let _ = (mesh, bounds, tolerance, detected);
        false
    }
    /// `GetDetectedEntities(Prs, Polyline, aBox, aTol, Nodes, Elements)`: false by default.
    fn get_detected_entities_in_polyline(
        &self,
        mesh: &Handle<dyn MeshVSMeshSource>,
        polyline: &Array1<'_, Point2>,
        bounds: &BoundingBox2,
        tolerance: f64,
        detected: MeshVSDetectedMaps<'_>,
    ) -> bool {
        let _ = (mesh, polyline, bounds, tolerance, detected);
        false
    }
    /// `GetDetectedEntities(Prs, Nodes, Elements)`: false by default.
    fn get_detected_entities(
        &self,
        mesh: &Handle<dyn MeshVSMeshSource>,
        detected: MeshVSDetectedMaps<'_>,
    ) -> bool {
        let _ = (mesh, detected);
        false
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn MeshVSDataSource,
    "MeshVS_DataSource",
    StandardTransient
);

/// The native `Nodes` and `Elements` handle out-parameters of `GetDetectedEntities`.
pub struct MeshVSDetectedMaps<'a> {
    /// The detected node ids.
    pub nodes: &'a mut Handle<HPackedMapOfInteger>,
    /// The detected element ids.
    pub elements: &'a mut Handle<HPackedMapOfInteger>,
}
