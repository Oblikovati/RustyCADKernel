// SPDX-License-Identifier: GPL-2.0-only
//! `MeshVS_CommonSensitiveEntity` at OCCT 3d097a0: the whole-mesh sensitive set, whose
//! elements are the selectable nodes (`MeshVS_MSM_NODES`) or the selectable faces
//! (`MeshVS_MSM_PRECISE`), read from the data source on each test.
use crate::MeshVSDrawerAttribute as Key;
use crate::data_source::MeshVSDataSource;
use crate::data_source::normals::{coord, coords_buffer};
use crate::flags::{MESHVS_ET_FACE, MESHVS_ET_NONE};
use crate::mesh::{MeshVSMeshMembers, MeshVSMeshSource};
use crate::selection_policy::MeshVSMeshSelectionMethod;
use rustycad_tkernel::ncollection::{Array1, Vec3};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::gp::{Point3, Xyz};
use rustycad_tkv3d::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use rustycad_tkv3d::select_mgr::SelectMgrEntityOwnerSource;
use rustycad_tkv3d::select3d::{
    Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveEntityBase,
    Select3DSensitiveEntityMembers, Select3DSensitiveSet, Select3DSensitiveSetBase,
    Select3DTypeOfSensitivity, select3d_set_build_bvh, select3d_set_dump_json,
    select3d_set_matches,
};
use std::io;

/// The whole-mesh sensitive set; `MeshVSCommonSensitiveEntity::new(&owner, &mesh, MeshVSMeshSelectionMethod::Nodes)?`.
pub struct MeshVSCommonSensitiveEntity {
    transient: StandardTransient,
    entity: Select3DSensitiveEntityBase,
    set: Select3DSensitiveSetBase,
    data_source: Handle<dyn MeshVSDataSource>,
    item_indexes: SharedState<Vec<i32>>,
    sel_method: MeshVSMeshSelectionMethod,
    max_face_nodes: i32,
    cog: Point3,
    bnd_box: Select3DBndBox3d,
}
rustycad_tkernel::impl_transient!(MeshVSCommonSensitiveEntity, "MeshVS_CommonSensitiveEntity", dyn Select3DSensitiveSet, transient,
    interfaces: [dyn Select3DSensitiveEntity, dyn Select3DSensitiveSet]);

impl MeshVSCommonSensitiveEntity {
    /// `MeshVS_CommonSensitiveEntity(theOwner, theParentMesh, theSelMethod)`: the elements, the
    /// center (a NaN center without element, as the native division by zero) and the box;
    /// the node set detects with sensitivity 8. A drawer without positive
    /// `MeshVS_DA_MaxFaceNodes` raises `Standard_ProgramError` (Standard_ASSERT_RAISE).
    pub fn new<P: MeshVSMeshSource + ?Sized>(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        parent: &P,
        sel_method: MeshVSMeshSelectionMethod,
    ) -> Result<Self, Failure> {
        let max_face_nodes = parent_max_face_nodes(parent)?;
        let mut entity = Self {
            transient: StandardTransient::default(),
            entity: Select3DSensitiveEntityBase::new(owner),
            set: Select3DSensitiveSetBase::default(),
            data_source: parent.get_data_source(),
            item_indexes: SharedState::new(Vec::new()),
            sel_method,
            max_face_nodes,
            cog: Point3::default(),
            bnd_box: Select3DBndBox3d::default(),
        };
        match sel_method {
            MeshVSMeshSelectionMethod::Nodes => entity.init_nodes(parent)?,
            MeshVSMeshSelectionMethod::Precise => entity.init_faces(parent),
            _ => {}
        }
        Ok(entity)
    }
    fn source(&self) -> &dyn MeshVSDataSource {
        // The native constructor dereferences the data source before any use.
        self.data_source
            .get()
            .expect("MeshVS_CommonSensitiveEntity data source")
    }
    fn add_vertex(&mut self, center: &mut Xyz, vertex: Point3) {
        *center += vertex.xyz();
        self.bnd_box
            .add(&Vec3::new(vertex.x(), vertex.y(), vertex.z()));
    }
    fn init_nodes<P: MeshVSMeshSource + ?Sized>(&mut self, parent: &P) -> Result<(), Failure> {
        let (mut center, mut count) = (Xyz::new(0.0, 0.0, 0.0), 0);
        let mut items = Vec::new();
        for node in self.source().get_all_nodes().iter() {
            if parent.is_selectable_node(node) {
                let vertex = self.get_vertex_by_index(node);
                self.add_vertex(&mut center, vertex);
                count += 1;
                items.push(node);
            }
        }
        *self.item_indexes.write() = items;
        self.set_sensitivity_factor(8)?;
        self.cog = Point3::from(center / f64::from(count));
        Ok(())
    }
    fn init_faces<P: MeshVSMeshSource + ?Sized>(&mut self, parent: &P) {
        let mut center = Xyz::new(0.0, 0.0, 0.0);
        let nodes = self.source().get_all_nodes();
        for node in nodes.iter() {
            let vertex = self.get_vertex_by_index(node);
            self.add_vertex(&mut center, vertex);
        }
        self.cog = Point3::from(center / f64::from(nodes.extent()));
        let mut items = Vec::new();
        for element in self.source().get_all_elements().iter() {
            let mut kind = MESHVS_ET_NONE;
            if parent.is_selectable_elem(element)
                && self.source().get_geom_type(element, true, &mut kind)
                && kind == MESHVS_ET_FACE
            {
                items.push(element);
            }
        }
        *self.item_indexes.write() = items;
    }
    /// The private `getVertexByIndex`: the node coordinates, the origin when unknown.
    fn get_vertex_by_index(&self, node: i32) -> Point3 {
        let mut coords = Array1::<f64>::with_size(3);
        coords.update_lower_bound(1);
        let (mut nb_nodes, mut kind) = (0, MESHVS_ET_NONE);
        if !self
            .source()
            .get_geom(node, false, &mut coords, &mut nb_nodes, &mut kind)
        {
            return Point3::default();
        }
        Point3::new(coords[1], coords[2], coords[3])
    }
    fn item(&self, idx: i32) -> i32 {
        self.item_indexes.read()[idx as usize]
    }
    /// The coordinates and node count of a face, `None` when the data source cannot give
    /// them (the native false returns).
    fn face_coords(&self, item: i32) -> Option<(Array1<'static, f64>, i32)> {
        let mut coords = coords_buffer(self.max_face_nodes).ok()?;
        let (mut nb_nodes, mut kind) = (0, MESHVS_ET_NONE);
        if !self
            .source()
            .get_geom(item, true, &mut coords, &mut nb_nodes, &mut kind)
            || nb_nodes == 0
        {
            return None;
        }
        let mut nodes = Array1::<i32>::with_bounds(1, nb_nodes).ok()?;
        if !self
            .source()
            .get_nodes_by_element(item, &mut nodes, &mut nb_nodes)
        {
            return None;
        }
        Some((coords, nb_nodes))
    }
    fn face_points(coords: &Array1<'_, f64>, nb_nodes: i32) -> Result<Vec<Point3>, Failure> {
        (1..=nb_nodes)
            .map(|i| {
                Ok(Point3::new(
                    coord(coords, 3 * i - 2)?,
                    coord(coords, 3 * i - 1)?,
                    coord(coords, 3 * i)?,
                ))
            })
            .collect()
    }
}

fn parent_max_face_nodes<P: MeshVSMeshSource + ?Sized>(parent: &P) -> Result<i32, Failure> {
    let mut max_face_nodes = 0;
    let drawer = parent.get_drawer();
    if let Some(drawer) = drawer.get() {
        drawer.get_integer(Key::MaxFaceNodes as i32, &mut max_face_nodes);
    }
    if max_face_nodes <= 0 {
        return Err(Failure::with_type(
            "Standard_ProgramError",
            &format!(
                "MeshVS_CommonSensitiveEntity MaxFaceNodes={max_face_nodes}: The maximal amount of nodes in a face must be greater than zero to create sensitive entity"
            ),
            "",
        ));
    }
    Ok(max_face_nodes)
}

impl Select3DSensitiveEntity for MeshVSCommonSensitiveEntity {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        &self.entity
    }
    fn matches(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        select3d_set_matches(self, manager, pick, false)
    }
    /// `NbSubElements`: the number of nodes or faces.
    fn nb_sub_elements(&self) -> i32 {
        self.item_indexes.read().len() as i32
    }
    fn bounding_box(&self) -> Select3DBndBox3d {
        self.bnd_box
    }
    fn center_of_geometry(&self) -> Point3 {
        self.cog
    }
    /// `GetConnected`: a copy sharing the owner, data source, elements, center and box.
    fn get_connected(&self) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
        Ok(Handle::new(Self {
            transient: StandardTransient::default(),
            entity: Select3DSensitiveEntityBase::new(&self.owner_id()),
            set: Select3DSensitiveSetBase::default(),
            data_source: self.data_source.clone(),
            item_indexes: SharedState::new(self.item_indexes.read().clone()),
            sel_method: self.sel_method,
            max_face_nodes: self.max_face_nodes,
            cog: self.cog,
            bnd_box: self.bnd_box,
        })
        .cast())
    }
    fn bvh(&self) -> Result<(), Failure> {
        select3d_set_build_bvh(self)
    }
    fn to_build_bvh(&self) -> bool {
        self.set.is_dirty()
    }
    fn clear(&self) {}
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        select3d_set_dump_json(self, stream, depth)
    }
}

impl Select3DSensitiveSet for MeshVSCommonSensitiveEntity {
    fn set_base(&self) -> &Select3DSensitiveSetBase {
        &self.set
    }
    fn size(&self) -> i32 {
        self.item_indexes.read().len() as i32
    }
    /// `Box`: the box of the face nodes or of the node.
    fn element_box(&self, idx: i32) -> Select3DBndBox3d {
        let item = self.item(idx);
        let mut bounds = Select3DBndBox3d::default();
        match self.sel_method {
            MeshVSMeshSelectionMethod::Precise => {
                let Some((coords, nb_nodes)) = self.face_coords(item) else {
                    return bounds;
                };
                for point in Self::face_points(&coords, nb_nodes).unwrap_or_default() {
                    bounds.add(&Vec3::new(point.x(), point.y(), point.z()));
                }
            }
            MeshVSMeshSelectionMethod::Nodes => {
                let vertex = self.get_vertex_by_index(item);
                bounds.add(&Vec3::new(vertex.x(), vertex.y(), vertex.z()));
            }
            _ => {}
        }
        bounds
    }
    /// `Center`: the coordinate of the element box center (Z for every axis above 1).
    fn center(&self, idx: i32, axis: i32) -> Result<f64, Failure> {
        let bounds = self.element_box(idx);
        let center = (*bounds.corner_min() + *bounds.corner_max()) * 0.5;
        Ok(match axis {
            0 => center.x,
            1 => center.y,
            _ => center.z,
        })
    }
    fn swap(&self, idx1: i32, idx2: i32) {
        self.item_indexes.write().swap(idx1 as usize, idx2 as usize);
    }
    fn overlaps_element(
        &self,
        pick: &mut SelectBasicsPickResult,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        elem_idx: i32,
        is_full_inside: bool,
    ) -> Result<bool, Failure> {
        if is_full_inside {
            return Ok(true);
        }
        let item = self.item(elem_idx);
        match self.sel_method {
            MeshVSMeshSelectionMethod::Precise => self.overlaps_face(pick, manager, item),
            MeshVSMeshSelectionMethod::Nodes => {
                manager.overlaps_point(&self.get_vertex_by_index(item), pick)
            }
            _ => Ok(false),
        }
    }
    fn element_is_inside(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        elem_idx: i32,
        is_full_inside: bool,
    ) -> Result<bool, Failure> {
        if is_full_inside {
            return Ok(true);
        }
        let item = self.item(elem_idx);
        match self.sel_method {
            MeshVSMeshSelectionMethod::Precise => self.face_is_inside(manager, item),
            MeshVSMeshSelectionMethod::Nodes => {
                manager.overlaps_point_only(&self.get_vertex_by_index(item))
            }
            _ => Ok(false),
        }
    }
    fn distance_to_cog(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
    ) -> Result<f64, Failure> {
        manager.dist_to_geometry_center(&self.cog)
    }
}

impl MeshVSCommonSensitiveEntity {
    /// A triangle, else the polygon of the face nodes.
    fn overlaps_face(
        &self,
        pick: &mut SelectBasicsPickResult,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        item: i32,
    ) -> Result<bool, Failure> {
        let Some((coords, nb_nodes)) = self.face_coords(item) else {
            return Ok(false);
        };
        let interior = Select3DTypeOfSensitivity::Interior as i32;
        if nb_nodes == 3 {
            let p = Self::face_points(&coords, 3)?;
            return manager.overlaps_triangle(&p[0], &p[1], &p[2], interior, pick);
        }
        let points = Self::face_points(&coords, nb_nodes)?;
        manager.overlaps_polygon(&Array1::from_shared(&points, 1), interior, pick)
    }
    /// Every face node lies in the volume.
    fn face_is_inside(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        item: i32,
    ) -> Result<bool, Failure> {
        let Some((coords, nb_nodes)) = self.face_coords(item) else {
            return Ok(false);
        };
        for point in Self::face_points(&coords, nb_nodes)? {
            if !manager.overlaps_point_only(&point)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
