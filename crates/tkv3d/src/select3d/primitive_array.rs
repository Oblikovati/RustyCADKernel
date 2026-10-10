//! `Select3D_SensitivePrimitiveArray`: the triangles or points of a primitive array (vertex and
//! index buffers) in a BVH tree, optionally grouped in patches of neighbour elements or split
//! into sub-arrays. The detection members are in primitive_array_match.rs.
use super::bvh_index_buffer::Select3DBvhIndexBuffer;
use super::{Select3DBndBox3d, Select3DSensitiveEntityBase, Select3DSensitiveSetBase};
use crate::select_mgr::SelectMgrEntityOwnerSource;
use rustycad_tkernel::ncollection::{Vec2, Vec3};
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkernel::standard::short_real::short_real_last;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcol_std::HPackedMapOfInteger;
use rustycad_tkmath::gp::{GeneralTransform3, Point3, Transform3};
use rustycad_tkmath::toploc::Location;
use rustycad_tkservice::graphic3d::{
    Graphic3dBufferBase, Graphic3dBufferMembers as _, Graphic3dBufferSource,
    Graphic3dIndexBufferMembers as _, Graphic3dIndexBufferSource, Graphic3dTypeOfAttribute,
    Graphic3dTypeOfData, Graphic3dTypeOfPrimitiveArray,
};

/// The fields of a primitive array.
#[derive(Clone)]
pub(crate) struct PrimArrayState {
    pub(crate) groups: Option<Vec<Handle<Select3DSensitivePrimitiveArray>>>,
    pub(crate) verts: Handle<dyn Graphic3dBufferSource>,
    pub(crate) indices: Handle<dyn Graphic3dIndexBufferSource>,
    /// `myPosData` as a byte offset in the vertex buffer (None for the null pointer).
    pub(crate) pos_offset: Option<usize>,
    pub(crate) pos_stride: usize,
    pub(crate) prim_type: Graphic3dTypeOfPrimitiveArray,
    pub(crate) index_lower: i32,
    pub(crate) index_upper: i32,
    pub(crate) patch_size_max: i32,
    pub(crate) patch_distance: f32,
    pub(crate) is_3d: bool,
    pub(crate) init_location: Location,
    pub(crate) cdg3d: Point3,
    pub(crate) bnd_box: Select3DBndBox3d,
    pub(crate) inv_init_location: GeneralTransform3,
    pub(crate) detected_elem_map: Handle<HPackedMapOfInteger>,
    pub(crate) detected_node_map: Handle<HPackedMapOfInteger>,
    pub(crate) min_depth_elem: f64,
    pub(crate) min_depth_node: f64,
    pub(crate) min_depth_edge: f64,
    pub(crate) detected_elem: i32,
    pub(crate) detected_node: i32,
    pub(crate) detected_edge_node1: i32,
    pub(crate) detected_edge_node2: i32,
    pub(crate) to_detect_elem: bool,
    pub(crate) to_detect_node: bool,
    pub(crate) to_detect_edge: bool,
}

/// A sensitive primitive array.
///
/// Example: `let sens = Select3DSensitivePrimitiveArray::new(&owner)?; sens.init_triangulation_all(&verts, &indices, &Location::default(), true, 1)?;`.
pub struct Select3DSensitivePrimitiveArray {
    transient: StandardTransient,
    pub(crate) entity: Select3DSensitiveEntityBase,
    pub(crate) set: Select3DSensitiveSetBase,
    pub(crate) bvh_indices: Select3DBvhIndexBuffer,
    pub(crate) state: SharedState<PrimArrayState>,
}
rustycad_tkernel::impl_transient!(Select3DSensitivePrimitiveArray, "Select3D_SensitivePrimitiveArray", dyn super::Select3DSensitiveSet, transient,
    interfaces: [dyn super::Select3DSensitiveEntity, dyn super::Select3DSensitiveSet]);

/// A point of single-precision coordinates.
pub(crate) fn vec3_to_pnt(v: Vec3<f32>) -> Point3 {
    Point3::new(f64::from(v.x), f64::from(v.y), f64::from(v.z))
}

/// A point of single-precision plane coordinates (`z = 0`).
pub(crate) fn vec2_to_pnt(v: Vec2<f32>) -> Point3 {
    Point3::new(f64::from(v.x), f64::from(v.y), 0.0)
}

/// The arguments of `InitTriangulation` and `InitPoints`.
#[derive(Clone)]
pub struct Select3DPrimArrayInit {
    /// The vertex buffer.
    pub verts: Handle<dyn Graphic3dBufferSource>,
    /// The index buffer (null to use the vertices in order).
    pub indices: Handle<dyn Graphic3dIndexBufferSource>,
    /// The initial location.
    pub init_loc: Location,
    /// The first index (inclusive).
    pub index_lower: i32,
    /// The last index (inclusive).
    pub index_upper: i32,
    /// Whether to compute the bounding box.
    pub to_eval_min_max: bool,
    /// The number of sub-arrays (sub-groups).
    pub nb_groups: i32,
}

impl Select3DPrimArrayInit {
    /// The whole index (or vertex) range of the buffers, as the overloads without range.
    /// Example: `Select3DPrimArrayInit::whole(&verts, &indices, &Location::default(), true, 1)`.
    pub fn whole(
        verts: &Handle<dyn Graphic3dBufferSource>,
        indices: &Handle<dyn Graphic3dIndexBufferSource>,
        init_loc: &Location,
        to_eval_min_max: bool,
        nb_groups: i32,
    ) -> Self {
        let upper = match (indices.get(), verts.get()) {
            (Some(i), _) => i.nb_elements() - 1,
            (None, Some(v)) => v.nb_elements() - 1,
            (None, None) => 0,
        };
        Self {
            verts: verts.clone(),
            indices: indices.clone(),
            init_loc: init_loc.clone(),
            index_lower: 0,
            index_upper: upper,
            to_eval_min_max,
            nb_groups,
        }
    }
}

impl Select3DSensitivePrimitiveArray {
    /// An empty array of an owner: patches of 1, element detection only.
    /// Example: `Select3DSensitivePrimitiveArray::new(&owner)?`.
    pub fn new(owner: &Handle<dyn SelectMgrEntityOwnerSource>) -> Result<Self, Failure> {
        let state = PrimArrayState {
            groups: None,
            verts: Handle::default(),
            indices: Handle::default(),
            pos_offset: None,
            pos_stride: usize::MAX,
            prim_type: Graphic3dTypeOfPrimitiveArray::Undefined,
            index_lower: 0,
            index_upper: 0,
            patch_size_max: 1,
            patch_distance: short_real_last(),
            is_3d: false,
            init_location: Location::default(),
            cdg3d: Point3::default(),
            bnd_box: Select3DBndBox3d::default(),
            inv_init_location: GeneralTransform3::from(Transform3::default()),
            detected_elem_map: Handle::default(),
            detected_node_map: Handle::default(),
            min_depth_elem: real_last(),
            min_depth_node: real_last(),
            min_depth_edge: real_last(),
            detected_elem: -1,
            detected_node: -1,
            detected_edge_node1: -1,
            detected_edge_node2: -1,
            to_detect_elem: true,
            to_detect_node: false,
            to_detect_edge: false,
        };
        Ok(Self {
            transient: StandardTransient::default(),
            entity: Select3DSensitiveEntityBase::new(owner),
            set: Select3DSensitiveSetBase::default(),
            bvh_indices: Select3DBvhIndexBuffer::new(Graphic3dBufferBase::default_allocator())?,
            state: SharedState::new(state),
        })
    }
    /// A copy of the fields.
    pub(crate) fn read(&self) -> PrimArrayState {
        self.state.read().clone()
    }
    /// `PatchSizeMax`; `sens.patch_size_max()`.
    pub fn patch_size_max(&self) -> i32 {
        self.state.read().patch_size_max
    }
    /// `SetPatchSizeMax`; `sens.set_patch_size_max(8)`.
    pub fn set_patch_size_max(&self, patch_size_max: i32) {
        self.state.write().patch_size_max = patch_size_max;
    }
    /// `PatchDistance`; `sens.patch_distance()`.
    pub fn patch_distance(&self) -> f32 {
        self.state.read().patch_distance
    }
    /// `SetPatchDistance`; `sens.set_patch_distance(0.5)`.
    pub fn set_patch_distance(&self, patch_dist_max: f32) {
        self.state.write().patch_distance = patch_dist_max;
    }
    /// `SetMinMax`: the box of the array and of its sub-arrays.
    /// Example: `sens.set_min_max(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)`.
    pub fn set_min_max(
        &self,
        min_x: f64,
        min_y: f64,
        min_z: f64,
        max_x: f64,
        max_y: f64,
        max_z: f64,
    ) {
        let bounds = Select3DBndBox3d::from_corners(
            Vec3::new(min_x, min_y, min_z),
            Vec3::new(max_x, max_y, max_z),
        );
        let groups = {
            let mut state = self.state.write();
            state.bnd_box = bounds;
            state.groups.clone()
        };
        for group in groups.unwrap_or_default().iter().filter_map(Handle::get) {
            group.state.write().bnd_box = bounds;
        }
    }
    /// `ToDetectElements`; `sens.to_detect_elements()`.
    pub fn to_detect_elements(&self) -> bool {
        self.state.read().to_detect_elem
    }
    /// `SetDetectElements`; `sens.set_detect_elements(false)`.
    pub fn set_detect_elements(&self, to_detect: bool) {
        self.state.write().to_detect_elem = to_detect;
    }
    /// `ToDetectElementMap`; `sens.to_detect_element_map()`.
    pub fn to_detect_element_map(&self) -> bool {
        !self.state.read().detected_elem_map.is_null()
    }
    /// `SetDetectElementMap`: a new (or cleared) map, or none; `sens.set_detect_element_map(true)`.
    pub fn set_detect_element_map(&self, to_detect: bool) {
        let mut state = self.state.write();
        state.detected_elem_map = detection_map(&state.detected_elem_map, to_detect);
    }
    /// `ToDetectNodes`; `sens.to_detect_nodes()`.
    pub fn to_detect_nodes(&self) -> bool {
        self.state.read().to_detect_node
    }
    /// `SetDetectNodes`; `sens.set_detect_nodes(true)`.
    pub fn set_detect_nodes(&self, to_detect: bool) {
        self.state.write().to_detect_node = to_detect;
    }
    /// `ToDetectNodeMap`; `sens.to_detect_node_map()`.
    pub fn to_detect_node_map(&self) -> bool {
        !self.state.read().detected_node_map.is_null()
    }
    /// `SetDetectNodeMap`; `sens.set_detect_node_map(true)`.
    pub fn set_detect_node_map(&self, to_detect: bool) {
        let mut state = self.state.write();
        state.detected_node_map = detection_map(&state.detected_node_map, to_detect);
    }
    /// `ToDetectEdges`; `sens.to_detect_edges()`.
    pub fn to_detect_edges(&self) -> bool {
        self.state.read().to_detect_edge
    }
    /// `SetDetectEdges`; `sens.set_detect_edges(true)`.
    pub fn set_detect_edges(&self, to_detect: bool) {
        self.state.write().to_detect_edge = to_detect;
    }
    /// `LastDetectedElement`; `sens.last_detected_element()`.
    pub fn last_detected_element(&self) -> i32 {
        self.state.read().detected_elem
    }
    /// `LastDetectedElementMap`; `sens.last_detected_element_map()`.
    pub fn last_detected_element_map(&self) -> Handle<HPackedMapOfInteger> {
        self.state.read().detected_elem_map.clone()
    }
    /// `LastDetectedNode`; `sens.last_detected_node()`.
    pub fn last_detected_node(&self) -> i32 {
        self.state.read().detected_node
    }
    /// `LastDetectedNodeMap`; `sens.last_detected_node_map()`.
    pub fn last_detected_node_map(&self) -> Handle<HPackedMapOfInteger> {
        self.state.read().detected_node_map.clone()
    }
    /// `LastDetectedEdgeNode1`; `sens.last_detected_edge_node1()`.
    pub fn last_detected_edge_node1(&self) -> i32 {
        self.state.read().detected_edge_node1
    }
    /// `LastDetectedEdgeNode2`; `sens.last_detected_edge_node2()`.
    pub fn last_detected_edge_node2(&self) -> i32 {
        self.state.read().detected_edge_node2
    }
    /// `getPosVec3` (protected): the 3D position of a vertex (zero past the buffer).
    pub fn get_pos_vec3(&self, index: i32) -> Vec3<f32> {
        let state = self.state.read();
        read_position(&state, index).unwrap_or_default()
    }
    /// `getPosVec2` (protected): the 2D position of a vertex (zero past the buffer).
    pub fn get_pos_vec2(&self, index: i32) -> Vec2<f32> {
        let state = self.state.read();
        read_position_2d(&state, index).unwrap_or_default()
    }
    /// `GetVertex`: the three positions of a triangle; `sens.get_vertex(0)`.
    pub fn get_vertex(&self, index: i32) -> [Vec3<f32>; 3] {
        let state = self.state.read();
        let nodes = tri_indices(&state.indices, index * 3);
        nodes.map(|n| read_position(&state, n).unwrap_or_default())
    }
}

/// `SetDetect*Map`: a cleared map, a new map, or none.
fn detection_map(
    current: &Handle<HPackedMapOfInteger>,
    to_detect: bool,
) -> Handle<HPackedMapOfInteger> {
    if !to_detect {
        return Handle::default();
    }
    match current.get() {
        Some(map) => {
            map.change_map().clear();
            current.clone()
        }
        None => Handle::new(HPackedMapOfInteger::default()),
    }
}

/// The 3D position of a vertex through `myPosData + myPosStride * index`.
pub(crate) fn read_position(state: &PrimArrayState, index: i32) -> Option<Vec3<f32>> {
    let offset = state
        .pos_offset?
        .wrapping_add(state.pos_stride.wrapping_mul(index as usize));
    state.verts.get()?.read_at::<Vec3<f32>>(offset)
}

/// The 2D position of a vertex.
pub(crate) fn read_position_2d(state: &PrimArrayState, index: i32) -> Option<Vec2<f32>> {
    let offset = state
        .pos_offset?
        .wrapping_add(state.pos_stride.wrapping_mul(index as usize));
    state.verts.get()?.read_at::<Vec2<f32>>(offset)
}

/// The position of a vertex as a point (3D or in the plane).
pub(crate) fn position_point(state: &PrimArrayState, index: i32) -> Point3 {
    if state.is_3d {
        vec3_to_pnt(read_position(state, index).unwrap_or_default())
    } else {
        vec2_to_pnt(read_position_2d(state, index).unwrap_or_default())
    }
}

/// `getTriIndices`: the three vertex indexes of the triangle at an index offset.
pub(crate) fn tri_indices(
    indices: &Handle<dyn Graphic3dIndexBufferSource>,
    offset: i32,
) -> [i32; 3] {
    match indices.get() {
        Some(buffer) => [0, 1, 2].map(|k| buffer.index(offset + k).unwrap_or(0)),
        None => [offset, offset + 1, offset + 2],
    }
}

/// The vertex index of a point at an index offset.
pub(crate) fn point_index(indices: &Handle<dyn Graphic3dIndexBufferSource>, offset: i32) -> i32 {
    indices
        .get()
        .map_or(offset, |b| b.index(offset).unwrap_or(0))
}

/// The position attribute of the vertices: its byte offset, stride and dimension (None when
/// absent or not a 2D, 3D or 4D vector).
pub(crate) fn position_attribute(
    verts: &dyn Graphic3dBufferSource,
) -> Option<(usize, usize, bool)> {
    let (index, offset, stride) = verts.attribute_data(Graphic3dTypeOfAttribute::Pos)?;
    let data_type = verts.attribute(index)?.data_type;
    let is_3d = matches!(
        data_type,
        Graphic3dTypeOfData::Vec3 | Graphic3dTypeOfData::Vec4
    );
    (is_3d || data_type == Graphic3dTypeOfData::Vec2).then_some((offset, stride, is_3d))
}
