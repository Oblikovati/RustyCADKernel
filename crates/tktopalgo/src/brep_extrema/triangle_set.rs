// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_TriangleSet.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepExtrema_TriangleSet`: the triangles of the faces (and the segments of the edge
//! polygons) of a list of shapes, under a linear BVH.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::exceptions::{no_such_object, null_object};
use rustycad_tkernel::standard::{Failure, StandardTransient};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bvh::constants::{LEAF_NODE_SIZE_DEFAULT, MAX_TREE_DEPTH};
use rustycad_tkmath::bvh::{
    BvhArray3d, BvhArray4i, BvhBox, BvhLinearBuilder, BvhPrimitiveSet, BvhPrimitiveSetAccess,
    BvhSet, BvhTree, BvhVec3d, BvhVec4i, array,
};
use rustycad_tkmath::gp::{Point3, Transform3};
use rustycad_tkmath::poly::mesh_purpose;
use std::sync::{Arc, Mutex, MutexGuard};

/// The fields of a triangle set; the BVH set interface is implemented on them.
struct TriangleSetState {
    /// `myTriangles`: vertex indices and shape index of each triangle.
    triangles: BvhArray4i,
    /// `myVertexArray`: the vertex coordinates.
    vertex_array: BvhArray3d,
    /// `myShapeIdxOfVtxVec`: the shape index of each vertex.
    shape_idx_of_vtx_vec: Vec<i32>,
    /// `myNumVtxInShapeVec`: the vertex count of each shape.
    num_vtx_in_shape_vec: Vec<i32>,
    /// `myNumTrgInShapeVec`: the triangle count of each face.
    num_trg_in_shape_vec: Vec<i32>,
    /// `myTrgIdxMap`: the index before swapping of each triangle (bound from 0 to the
    /// triangle count by `Init`, as the native map).
    trg_idx_map: Vec<i32>,
    primitive_set: BvhPrimitiveSet<BvhVec3d>,
}

/// `NCollection_DynamicArray::SetValue`: the array grows (with zeros) to hold `index`.
fn set_value(values: &mut Vec<i32>, index: i32, value: i32) {
    let index: usize = index as usize;
    if index >= values.len() {
        values.resize(index + 1, 0);
    }
    values[index] = value;
}

/// `NCollection_DynamicArray::Value(int)`: unchecked natively (an index outside the
/// array reads out of bounds); Standard_OutOfRange here.
fn dynamic_value(values: &[i32], index: i32, name: &str) -> Result<i32, Failure> {
    usize::try_from(index)
        .ok()
        .and_then(|i| values.get(i))
        .copied()
        .ok_or_else(|| {
            Failure::out_of_range(&format!(
                "BRepExtrema_TriangleSet::{name}: index {index} outside [0, {})",
                values.len()
            ))
        })
}

impl TriangleSetState {
    fn new() -> Self {
        let builder = BvhLinearBuilder::<BvhVec3d>::new(LEAF_NODE_SIZE_DEFAULT, MAX_TREE_DEPTH);
        Self {
            triangles: BvhArray4i::default(),
            vertex_array: BvhArray3d::default(),
            shape_idx_of_vtx_vec: Vec::new(),
            num_vtx_in_shape_vec: Vec::new(),
            num_trg_in_shape_vec: Vec::new(),
            trg_idx_map: Vec::new(),
            primitive_set: BvhPrimitiveSet::new(Some(Arc::new(builder))),
        }
    }

    fn triangle(&self, index: i32) -> Result<BvhVec4i, Failure> {
        array::value(&self.triangles, index).copied()
    }

    fn vertex(&self, index: i32) -> Result<BvhVec3d, Failure> {
        array::value(&self.vertex_array, index).copied()
    }

    fn vertices_of(&self, index: i32) -> Result<[BvhVec3d; 3], Failure> {
        let triangle: BvhVec4i = self.triangle(index)?;
        Ok([
            self.vertex(triangle.x)?,
            self.vertex(triangle.y)?,
            self.vertex(triangle.z)?,
        ])
    }

    /// `initNodes(theNodes, theTrsf, theIndex)`: the transformed nodes, each of shape
    /// `index`, and the vertex count of the shape.
    fn init_nodes(&mut self, nodes: &[Point3], transformation: Transform3, index: i32) {
        for node in nodes {
            let vertex: Point3 = node.transformed(transformation);
            array::append(
                &mut self.vertex_array,
                BvhVec3d::new(vertex.x(), vertex.y(), vertex.z()),
            );
            self.shape_idx_of_vtx_vec.push(index);
        }
        set_value(&mut self.num_vtx_in_shape_vec, index, nodes.len() as i32);
    }

    /// `initFace(theFace, theIndex)`: the nodes and triangles of the face triangulation;
    /// false without one.
    fn init_face(&mut self, face: &Face, index: i32) -> Result<bool, Failure> {
        let (triangulation, location) = BRepTool::triangulation(face, mesh_purpose::NONE)?;
        let Some(triangulation) = triangulation.get() else {
            return Ok(false);
        };
        let vert_offset: i32 = array::size(&self.vertex_array) - 1;
        // A triangulation without nodes has a null node array natively.
        let nodes = triangulation.map_node_array().ok_or_else(|| {
            null_object("BRepExtrema_TriangleSet::initFace: a triangulation without nodes")
        })?;
        self.init_nodes(nodes.as_slice(), location.transformation(), index);
        for tri_idx in 1..=triangulation.nb_triangles() {
            let (v1, v2, v3) = triangulation.triangle(tri_idx)?.get();
            let record = BvhVec4i::new(v1 + vert_offset, v2 + vert_offset, v3 + vert_offset, index);
            array::append(&mut self.triangles, record);
        }
        set_value(
            &mut self.num_trg_in_shape_vec,
            index,
            triangulation.nb_triangles(),
        );
        Ok(true)
    }

    /// `initEdge(theEdge, theIndex)`: the nodes of the 3D polygon and one degenerate
    /// triangle (a segment) per polygon link; false without a polygon.
    fn init_edge(&mut self, edge: &Edge, index: i32) -> Result<bool, Failure> {
        let (polygon, location) = BRepTool::polygon3d(edge)?;
        let Some(polygon) = polygon.get() else {
            return Ok(false);
        };
        let vert_offset: i32 = array::size(&self.vertex_array) - 1;
        self.init_nodes(polygon.nodes().as_slice(), location.transformation(), index);
        for vert_idx in 1..polygon.nb_nodes() {
            let (first, second) = (vert_idx + vert_offset, vert_idx + vert_offset + 1);
            array::append(
                &mut self.triangles,
                BvhVec4i::new(first, second, second, index),
            );
        }
        Ok(true)
    }

    /// `Init(theShapes)`: see [`BRepExtremaTriangleSet::init`].
    fn init(&mut self, shapes: &[Shape]) -> Result<bool, Failure> {
        array::clear(&mut self.triangles);
        array::clear(&mut self.vertex_array);
        let mut is_ok: bool = true;
        for (shape_idx, shape) in shapes.iter().enumerate() {
            if !is_ok {
                break;
            }
            match shape.shape_type()? {
                ShapeEnum::Face => {
                    is_ok = self.init_face(&topods::face(shape)?, shape_idx as i32)?
                }
                ShapeEnum::Edge => {
                    is_ok = self.init_edge(&topods::edge(shape)?, shape_idx as i32)?
                }
                _ => {}
            }
        }
        let num_trg: i32 = array::size(&self.triangles);
        self.trg_idx_map = (0..num_trg).collect();
        self.primitive_set.object_transient_mut().is_dirty = true; // needs BVH rebuilding
        self.update()?;
        Ok(true)
    }
}

impl BvhSet<BvhVec3d> for TriangleSetState {
    fn size(&self) -> i32 {
        array::size(&self.triangles)
    }

    fn element_box(&self, index: i32) -> Result<BvhBox<BvhVec3d>, Failure> {
        let [a, b, c] = self.vertices_of(index)?;
        let min_pnt: BvhVec3d = a.cwise_min(&b.cwise_min(&c));
        let max_pnt: BvhVec3d = a.cwise_max(&b.cwise_max(&c));
        Ok(BvhBox::from_corners(min_pnt, max_pnt))
    }

    fn center(&self, index: i32, axis: i32) -> Result<f64, Failure> {
        let [a, b, c] = self.vertices_of(index)?;
        let axis: usize = match axis {
            0 => 0,
            1 => 1,
            _ => 2,
        };
        Ok((1.0 / 3.0) * (a[axis] + b[axis] + c[axis]))
    }

    fn swap(&mut self, first: i32, second: i32) -> Result<(), Failure> {
        let (i, j) = (first as usize, second as usize);
        let count: usize = self.triangles.len();
        if i >= count || j >= count || i >= self.trg_idx_map.len() || j >= self.trg_idx_map.len() {
            return Err(no_such_object(&format!(
                "BRepExtrema_TriangleSet::Swap: triangles {first} and {second}, expected indices below {count}"
            )));
        }
        self.triangles.as_mut_slice().swap(i, j);
        self.trg_idx_map.swap(i, j);
        Ok(())
    }

    fn bounding_box(&self) -> Result<BvhBox<BvhVec3d>, Failure> {
        self.cached_box()
    }
}

impl BvhPrimitiveSetAccess<BvhVec3d> for TriangleSetState {
    fn primitive_set(&self) -> &BvhPrimitiveSet<BvhVec3d> {
        &self.primitive_set
    }
    fn primitive_set_mut(&mut self) -> &mut BvhPrimitiveSet<BvhVec3d> {
        &mut self.primitive_set
    }
}

/// `BRepExtrema_TriangleSet` (docs/topalgo-design.md section 13): the triangles of the
/// faces and the segments (degenerate triangles) of the edge polygons of a list of
/// shapes, in world coordinates, under a linear BVH (5 elements a leaf, depth 32).
/// Each triangle keeps the index of its shape in the list.
///
/// The vectors of the shape of each vertex and of the vertex and triangle counts of
/// each shape are never cleared, only appended or overwritten (native quirk): after a
/// second `Init` they keep entries of the first one.
///
/// ```
/// use rustycad_tktopalgo::brep_extrema::BRepExtremaTriangleSet;
/// let set = BRepExtremaTriangleSet::new();
/// assert_eq!(set.size(), 0);
/// assert!(set.init(&[])?);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BRepExtremaTriangleSet {
    parent: StandardTransient,
    // A mutex, not a SharedState: the BVH primitive-set state caches its box in a Cell.
    state: Mutex<TriangleSetState>,
}
impl_transient!(
    BRepExtremaTriangleSet,
    "BRepExtrema_TriangleSet",
    StandardTransient,
    parent
);

impl Default for BRepExtremaTriangleSet {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepExtremaTriangleSet {
    /// `BRepExtrema_TriangleSet()`: an empty set with a linear builder;
    /// `BRepExtremaTriangleSet::new()`.
    pub fn new() -> Self {
        Self {
            parent: StandardTransient::default(),
            state: Mutex::new(TriangleSetState::new()),
        }
    }

    /// `BRepExtrema_TriangleSet(theFaces)`: a set initialized from the shapes (the
    /// result of `Init` is ignored, as natively);
    /// `BRepExtremaTriangleSet::with_shapes(&faces)?`.
    pub fn with_shapes(shapes: &[Shape]) -> Result<Self, Failure> {
        let set: Self = Self::new();
        set.init(shapes)?;
        Ok(set)
    }

    /// The locked state; a poisoned lock is recovered (no member panics while holding it).
    fn lock(&self) -> MutexGuard<'_, TriangleSetState> {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }

    /// `Init(theShapes)`: clears the triangles and vertices, then adds each face (its
    /// triangulation) and each edge (its 3D polygon) of the list, stopping after the
    /// first one that has none; rebuilds the BVH. Returns true in every case (native
    /// quirk); `set.init(&shapes)?`.
    pub fn init(&self, shapes: &[Shape]) -> Result<bool, Failure> {
        self.lock().init(shapes)
    }

    /// `Clear()`: the triangles and the vertices only; `set.clear()`.
    pub fn clear(&self) {
        let mut state = self.lock();
        array::clear(&mut state.triangles);
        array::clear(&mut state.vertex_array);
    }

    /// `Size()`: the number of triangles; `set.size()`.
    pub fn size(&self) -> i32 {
        self.lock().size()
    }

    /// `Box(theIndex)`: the box of a triangle (Standard_OutOfRange outside the set);
    /// `set.element_box(0)?`.
    pub fn element_box(&self, index: i32) -> Result<BvhBox<BvhVec3d>, Failure> {
        self.lock().element_box(index)
    }

    /// `Box()` of `BVH_PrimitiveSet`: the box of the whole set; `set.bounding_box()?`.
    pub fn bounding_box(&self) -> Result<BvhBox<BvhVec3d>, Failure> {
        self.lock().cached_box()
    }

    /// `Center(theIndex, theAxis)`: a third of the sum of the triangle coordinates along
    /// the axis (0 X, 1 Y, any other Z); `set.center(0, 2)?`.
    pub fn center(&self, index: i32, axis: i32) -> Result<f64, Failure> {
        self.lock().center(index, axis)
    }

    /// `Swap(theIndex1, theIndex2)`: exchanges two triangles and their original indices;
    /// `set.swap(0, 1)?`.
    pub fn swap(&self, first: i32, second: i32) -> Result<(), Failure> {
        self.lock().swap(first, second)
    }

    /// `GetVertices()`: all the vertices; `set.vertices().len()`.
    pub fn vertices(&self) -> BvhArray3d {
        self.lock().vertex_array.clone()
    }

    /// `GetVertices(theIndex, V1, V2, V3)`: the vertices of a triangle;
    /// `let [a, b, c] = set.get_vertices(0)?;`.
    pub fn get_vertices(&self, index: i32) -> Result<[BvhVec3d; 3], Failure> {
        self.lock().vertices_of(index)
    }

    /// `GetVtxIndices(theIndex, theVtxIndices)`: the vertex indices of a triangle (the
    /// native array has bounds 0..2); `let [a, b, c] = set.get_vtx_indices(0)?;`.
    pub fn get_vtx_indices(&self, index: i32) -> Result<[i32; 3], Failure> {
        let triangle: BvhVec4i = self.lock().triangle(index)?;
        Ok([triangle.x, triangle.y, triangle.z])
    }

    /// `GetFaceID(theIndex)`: the shape index of a triangle; `set.get_face_id(0)?`.
    pub fn get_face_id(&self, index: i32) -> Result<i32, Failure> {
        Ok(self.lock().triangle(index)?.w)
    }

    /// `GetShapeIDOfVtx(theIndex)`: the shape index of a vertex;
    /// `set.get_shape_id_of_vtx(0)?`.
    pub fn get_shape_id_of_vtx(&self, index: i32) -> Result<i32, Failure> {
        dynamic_value(&self.lock().shape_idx_of_vtx_vec, index, "GetShapeIDOfVtx")
    }

    /// `GetVtxIdxInShape(theIndex)`: the index of a vertex in the triangulation (or
    /// polygon) of its shape, from 0; `set.get_vtx_idx_in_shape(3)?`.
    pub fn get_vtx_idx_in_shape(&self, index: i32) -> Result<i32, Failure> {
        let state = self.lock();
        let sh_id: i32 = dynamic_value(&state.shape_idx_of_vtx_vec, index, "GetVtxIdxInShape")?;
        let mut num_vertices: i32 = 0;
        for s_idx in 0..sh_id {
            num_vertices += dynamic_value(&state.num_vtx_in_shape_vec, s_idx, "GetVtxIdxInShape")?;
        }
        Ok(index - num_vertices)
    }

    /// `GetTrgIdxInShape(theIndex)`: the index (before the BVH swaps) of a triangle in
    /// the triangulation of its face, from 0; `set.get_trg_idx_in_shape(3)?`.
    pub fn get_trg_idx_in_shape(&self, index: i32) -> Result<i32, Failure> {
        let state = self.lock();
        let sh_id: i32 = state.triangle(index)?.w;
        let mut num_triangles: i32 = 0;
        for s_idx in 0..sh_id {
            num_triangles += dynamic_value(&state.num_trg_in_shape_vec, s_idx, "GetTrgIdxInShape")?;
        }
        let original: i32 = usize::try_from(index)
            .ok()
            .and_then(|i| state.trg_idx_map.get(i))
            .copied()
            .ok_or_else(|| {
                no_such_object(&format!(
                    "BRepExtrema_TriangleSet::GetTrgIdxInShape: triangle {index} not in myTrgIdxMap"
                ))
            })?;
        Ok(original - num_triangles)
    }

    /// `BVH()` for a traversal: the tree (built first while dirty) moved out of the set;
    /// [`BRepExtremaTriangleSet::restore_bvh`] moves it back.
    pub(crate) fn take_bvh(&self) -> Result<BvhTree<BvhVec3d>, Failure> {
        self.lock().take_primitive_bvh()
    }

    /// Moves a tree taken by [`BRepExtremaTriangleSet::take_bvh`] back into the set.
    pub(crate) fn restore_bvh(&self, bvh: BvhTree<BvhVec3d>) {
        self.lock().restore_primitive_bvh(bvh);
    }
}
