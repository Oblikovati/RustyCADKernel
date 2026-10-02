//! Shape storage and the ported topology-preparation slice of `BOPDS_DS::Init`.
use super::{
    FaceInfo, IndexRange, Pave, PaveBlock, PaveBlockHandle, ShapeInfo, VertexIndexSet,
    pave_block_handle,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Face, Shape, ShapeIterator, Vertex};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use std::collections::HashMap;

/// The shape and argument storage shared by Boolean operation algorithms.
/// `let mut ds = DataStructure::default(); ds.append_shape(shape);`.
#[derive(Default)]
pub struct DataStructure {
    arguments: Vec<Shape>,
    ranges: Vec<IndexRange>,
    lines: Vec<ShapeInfo>,
    shape_indices: Option<DataMap<Shape, i32, ShapeMapHasher>>,
    vertex_edges: DataMap<i32, Vec<i32>>,
    pave_blocks_pool: Vec<Vec<PaveBlockHandle>>,
    face_info_pool: Vec<FaceInfo>,
    source_shape_count: i32,
}

impl DataStructure {
    /// Initializes source shape metadata, bounds, shared-vertex links and empty pools.
    /// `ds.init(1e-5)?`.
    pub fn init(&mut self, fuzz: f64) -> Result<(), Failure> {
        if self.arguments.is_empty() {
            return Ok(());
        }
        self.append_unindexed_arguments()?;
        self.source_shape_count = self.nb_shapes();
        let additional_tolerance = additional_tolerance(fuzz);
        self.prepare_vertex_bounds(additional_tolerance)?;
        let edge_count = self.prepare_edges(additional_tolerance)?;
        let face_count = self.prepare_faces(additional_tolerance)?;
        self.prepare_solids()?;
        self.build_vertex_edge_map()?;
        self.pave_blocks_pool.reserve(edge_count);
        self.face_info_pool.reserve(face_count);
        Ok(())
    }

    /// Replaces operation arguments with a copy of `shapes`.
    /// `ds.set_arguments(&[shape.clone()]);`.
    pub fn set_arguments(&mut self, shapes: &[Shape]) {
        self.arguments = shapes.to_vec();
    }

    /// Borrows the ordered operation arguments; `ds.arguments().len()`.
    pub fn arguments(&self) -> &[Shape] {
        &self.arguments
    }

    /// Returns the number of stored shape records; `ds.nb_shapes()`.
    pub fn nb_shapes(&self) -> i32 {
        self.lines.len() as i32
    }

    /// Returns the number of source records initialized from arguments.
    /// Before `Init`, OCCT reports zero; `ds.nb_source_shapes()`.
    pub fn nb_source_shapes(&self) -> i32 {
        self.source_shape_count
    }

    /// Returns the number of source index ranges; `ds.nb_ranges()`.
    pub fn nb_ranges(&self) -> i32 {
        self.ranges.len() as i32
    }

    /// Borrows a zero-based range, reporting the offending index on failure.
    /// `let range = ds.range(0)?;`.
    pub fn range(&self, index: i32) -> Result<&IndexRange, Failure> {
        let slot = checked_slot(index, self.ranges.len(), "BOPDS_DS::Range")?;
        Ok(&self.ranges[slot])
    }

    /// Returns the first range containing `index`, or `-1` when none does.
    /// `ds.rank(2) == -1` for an uninitialized structure.
    pub fn rank(&self, index: i32) -> i32 {
        self.ranges
            .iter()
            .position(|range| range.contains(index))
            .map_or(-1, |rank| rank as i32)
    }

    /// Reports whether the index is at or beyond the initialized source count.
    /// `ds.is_new_shape(0)` is true before `Init`.
    pub fn is_new_shape(&self, index: i32) -> bool {
        index >= self.source_shape_count
    }

    /// Appends a shape-info copy and binds its shape to the new zero-based index.
    /// `let index = ds.append_shape_info(&info);`.
    pub fn append_shape_info(&mut self, info: &ShapeInfo) -> i32 {
        let index = self.nb_shapes();
        self.lines.push(info.clone());
        self.bind_shape_index(info.shape(), index);
        index
    }

    /// Appends default metadata for `shape` and returns its zero-based index.
    /// `let index = ds.append_shape(&shape);`.
    pub fn append_shape(&mut self, shape: &Shape) -> i32 {
        let mut info = ShapeInfo::default();
        info.set_shape(shape);
        self.append_shape_info(&info)
    }

    /// Borrows shape metadata by zero-based index.
    /// `let info = ds.shape_info(0)?;`.
    pub fn shape_info(&self, index: i32) -> Result<&ShapeInfo, Failure> {
        let slot = checked_slot(index, self.lines.len(), "BOPDS_DS::ShapeInfo")?;
        Ok(&self.lines[slot])
    }

    /// Mutably borrows shape metadata by zero-based index.
    /// `ds.change_shape_info(0)?.set_flag(1);`.
    pub fn change_shape_info(&mut self, index: i32) -> Result<&mut ShapeInfo, Failure> {
        let slot = checked_slot(index, self.lines.len(), "BOPDS_DS::ChangeShapeInfo")?;
        Ok(&mut self.lines[slot])
    }

    /// Borrows the shape stored at `index`.
    /// `let shape = ds.shape(0)?;`.
    pub fn shape(&self, index: i32) -> Result<&Shape, Failure> {
        Ok(self.shape_info(index)?.shape())
    }

    /// Finds a shape using OCCT `IsSame` identity, returning `-1` when absent.
    /// `ds.index(&shape) == 0` after appending it.
    pub fn index(&self, shape: &Shape) -> i32 {
        self.shape_indices
            .as_ref()
            .and_then(|indices| indices.find(shape).ok())
            .copied()
            .unwrap_or(-1)
    }

    /// Reads the pave-block pool length; `ds.pave_blocks_pool().len()`.
    pub fn pave_blocks_pool(&self) -> &[Vec<PaveBlockHandle>] {
        &self.pave_blocks_pool
    }

    /// Mutably borrows the pave-block pool; `ds.change_pave_blocks_pool().clear()`.
    pub fn change_pave_blocks_pool(&mut self) -> &mut Vec<Vec<PaveBlockHandle>> {
        &mut self.pave_blocks_pool
    }

    /// Borrows the face-state pool; `ds.face_info_pool().len()`.
    pub fn face_info_pool(&self) -> &[FaceInfo] {
        &self.face_info_pool
    }

    /// Returns whether shape metadata points to a pave-block pool entry.
    /// `ds.has_pave_blocks(edge_index)`.
    pub fn has_pave_blocks(&self, index: i32) -> Result<bool, Failure> {
        Ok(self.shape_info(index)?.has_reference())
    }

    /// Reads the edge's pave-block list, empty when it has no pool reference.
    /// `let blocks = ds.pave_blocks(edge_index)?;`.
    pub fn pave_blocks(&self, index: i32) -> Result<&[PaveBlockHandle], Failure> {
        let info = self.shape_info(index)?;
        if !info.has_reference() {
            return Ok(&[]);
        }
        let slot = checked_slot(
            info.reference(),
            self.pave_blocks_pool.len(),
            "BOPDS_DS::PaveBlocks",
        )?;
        Ok(&self.pave_blocks_pool[slot])
    }

    /// Finds edges sharing a source vertex in native insertion order.
    /// `ds.edges_at_vertex(vertex_index)`.
    pub fn edges_at_vertex(&self, index: i32) -> &[i32] {
        self.vertex_edges.seek(&index).map_or(&[], Vec::as_slice)
    }

    /// Initializes pave-block lists for edges incident to the source vertex.
    /// `ds.init_pave_blocks_for_vertex(vertex_index)?`.
    pub fn init_pave_blocks_for_vertex(&mut self, index: i32) -> Result<(), Failure> {
        for edge in self.edges_at_vertex(index).to_vec() {
            self.change_pave_blocks(edge)?;
        }
        Ok(())
    }

    /// Returns the edge's mutable pave list, creating its native endpoint block first.
    /// `ds.change_pave_blocks(edge_index)?`.
    pub fn change_pave_blocks(&mut self, index: i32) -> Result<&mut Vec<PaveBlockHandle>, Failure> {
        if !self.shape_info(index)?.has_reference() {
            self.init_pave_blocks(index)?;
        }
        let reference = self.shape_info(index)?.reference();
        let slot = checked_slot(
            reference,
            self.pave_blocks_pool.len(),
            "BOPDS_DS::ChangePaveBlocks",
        )?;
        Ok(&mut self.pave_blocks_pool[slot])
    }

    /// Clears arguments and all currently implemented DS storage.
    /// `ds.clear(); assert_eq!(ds.nb_shapes(), 0);`.
    pub fn clear(&mut self) {
        self.arguments.clear();
        self.ranges.clear();
        self.lines.clear();
        self.vertex_edges.clear(false);
        self.pave_blocks_pool.clear();
        self.face_info_pool.clear();
        self.source_shape_count = 0;
        if let Some(indices) = &mut self.shape_indices {
            indices.clear(false);
        }
    }

    fn bind_shape_index(&mut self, shape: &Shape, index: i32) {
        self.shape_indices
            .get_or_insert_with(|| DataMap::with_hasher(ShapeMapHasher, 100))
            .bind(shape.clone(), index);
    }

    fn append_unindexed_arguments(&mut self) -> Result<(), Failure> {
        let mut first_index = 0;
        for shape in self.arguments.clone() {
            if self.index(&shape) >= 0 {
                continue;
            }
            let shape_index = self.append_shape(&shape);
            self.init_shape(shape_index, &shape)?;
            let last_index = self.nb_shapes() - 1;
            self.ranges.push(IndexRange::new(first_index, last_index));
            first_index = last_index + 1;
        }
        Ok(())
    }

    fn init_shape(&mut self, shape_index: i32, shape: &Shape) -> Result<(), Failure> {
        let shape_type = shape.shape_type()?;
        self.change_shape_info(shape_index)?
            .set_shape_type(shape_type);
        let mut iterator = ShapeIterator::new(shape, true, true);
        while iterator.more() {
            let child = iterator.value()?.clone();
            let child_index = self.append_or_find(&child)?;
            self.init_shape(child_index, &child)?;
            self.append_unique_child(shape_index, child_index)?;
            iterator.next();
        }
        Ok(())
    }

    fn append_or_find(&mut self, shape: &Shape) -> Result<i32, Failure> {
        let found = self.index(shape);
        if found >= 0 {
            return Ok(found);
        }
        Ok(self.append_shape(shape))
    }

    fn append_unique_child(&mut self, parent: i32, child: i32) -> Result<(), Failure> {
        let children = self.change_shape_info(parent)?.change_sub_shapes();
        if !children.contains(&child) {
            children.push(child);
        }
        Ok(())
    }

    fn prepare_vertex_bounds(&mut self, added_gap: f64) -> Result<(), Failure> {
        for index in 0..self.source_shape_count {
            let info = self.shape_info(index)?;
            if info.shape_type() != rustycad_tkg3d::top_abs::ShapeEnum::Vertex {
                continue;
            }
            let vertex = Vertex::unchecked(self.shape(index)?.clone());
            let point = BRepTool::pnt(&vertex)?;
            let gap = BRepTool::vertex_tolerance(&vertex)? + added_gap;
            let mut bounds = info.box_bounds().clone();
            bounds.set_gap(gap);
            bounds.add_point(point);
            self.change_shape_info(index)?.set_box(&bounds);
        }
        Ok(())
    }

    fn prepare_edges(&mut self, added_gap: f64) -> Result<usize, Failure> {
        let mut edge_count = 0;
        for index in 0..self.source_shape_count {
            if self.shape_info(index)?.shape_type() != ShapeEnum::Edge {
                continue;
            }
            edge_count += 1;
            self.prepare_edge(index, added_gap)?;
        }
        Ok(edge_count)
    }

    fn prepare_edge(&mut self, index: i32, added_gap: f64) -> Result<(), Failure> {
        let edge = Edge::unchecked(self.shape(index)?.clone());
        let mut bounds = self.shape_info(index)?.box_bounds().clone();
        if BRepTool::degenerated(&edge)? {
            self.change_shape_info(index)?.set_flag(index);
        }
        BRepBndLib::add(&edge, &mut bounds, true)?;
        self.add_child_boxes(index, &mut bounds)?;
        bounds.set_gap(bounds.gap() + added_gap);
        self.change_shape_info(index)?.set_box(&bounds);
        Ok(())
    }

    fn add_child_boxes(&self, index: i32, bounds: &mut BoundingBox3) -> Result<(), Failure> {
        for child in self.shape_info(index)?.sub_shapes() {
            bounds.add_box(self.shape_info(*child)?.box_bounds());
        }
        Ok(())
    }

    fn prepare_faces(&mut self, added_gap: f64) -> Result<usize, Failure> {
        let mut face_count = 0;
        for index in 0..self.source_shape_count {
            if self.shape_info(index)?.shape_type() != ShapeEnum::Face {
                continue;
            }
            face_count += 1;
            self.prepare_face(index, added_gap)?;
        }
        Ok(face_count)
    }

    fn prepare_face(&mut self, index: i32, added_gap: f64) -> Result<(), Failure> {
        let face = Face::unchecked(self.shape(index)?.clone());
        let mut bounds = self.shape_info(index)?.box_bounds().clone();
        BRepBndLib::add(&face, &mut bounds, true)?;
        let sub_shapes = self.shape_info(index)?.sub_shapes().to_vec();
        let mut flattened = VertexIndexSet::default();
        self.collect_face_wire_children(index, &sub_shapes, &mut flattened, &mut bounds)?;
        self.collect_face_isolated_vertices(&face, &mut flattened);
        let ordered = flattened.iter().copied().collect();
        bounds.set_gap(bounds.gap() + added_gap);
        let info = self.change_shape_info(index)?;
        *info.change_sub_shapes() = ordered;
        info.set_box(&bounds);
        Ok(())
    }

    fn collect_face_wire_children(
        &mut self,
        face_index: i32,
        wire_indices: &[i32],
        flattened: &mut VertexIndexSet,
        bounds: &mut BoundingBox3,
    ) -> Result<(), Failure> {
        for wire_index in wire_indices {
            let edge_indices = self.shape_info(*wire_index)?.sub_shapes().to_vec();
            for edge_index in edge_indices {
                bounds.add_box(self.shape_info(edge_index)?.box_bounds());
                flattened.add(edge_index);
                self.mark_degenerated_edge(face_index, edge_index)?;
                self.add_edge_vertices(edge_index, flattened)?;
            }
        }
        Ok(())
    }

    fn mark_degenerated_edge(&mut self, face_index: i32, edge_index: i32) -> Result<(), Failure> {
        let edge = Edge::unchecked(self.shape(edge_index)?.clone());
        if BRepTool::degenerated(&edge)? {
            self.change_shape_info(edge_index)?.set_flag(face_index);
        }
        Ok(())
    }

    fn add_edge_vertices(
        &self,
        edge_index: i32,
        indices: &mut VertexIndexSet,
    ) -> Result<(), Failure> {
        for vertex in self.shape_info(edge_index)?.sub_shapes() {
            indices.add(*vertex);
        }
        Ok(())
    }

    fn collect_face_isolated_vertices(&self, face: &Face, indices: &mut VertexIndexSet) {
        let mut children = ShapeIterator::new(face, true, true);
        while children.more() {
            if let Ok(child) = children.value()
                && child.shape_type().ok() == Some(ShapeEnum::Vertex)
            {
                indices.add(self.index(child));
            }
            children.next();
        }
    }

    fn prepare_solids(&mut self) -> Result<(), Failure> {
        if self.arguments.len() != 1 {
            return Ok(());
        }
        for index in 0..self.source_shape_count {
            if self.shape_info(index)?.shape_type() == ShapeEnum::Solid {
                self.prepare_solid(index)?;
            }
        }
        Ok(())
    }

    fn prepare_solid(&mut self, index: i32) -> Result<(), Failure> {
        let shells = self.shape_info(index)?.sub_shapes().to_vec();
        let mut bounds = BoundingBox3::default();
        let mut flattened = VertexIndexSet::default();
        for shell in shells {
            self.collect_shell_children(shell, &mut bounds, &mut flattened)?;
            if self.shell_is_open(shell)? {
                bounds.set_whole();
                break;
            }
        }
        let ordered = flattened.iter().copied().collect();
        let info = self.change_shape_info(index)?;
        *info.change_sub_shapes() = ordered;
        info.set_box(&bounds);
        Ok(())
    }

    fn collect_shell_children(
        &self,
        shell_index: i32,
        bounds: &mut BoundingBox3,
        indices: &mut VertexIndexSet,
    ) -> Result<(), Failure> {
        let faces = self.shape_info(shell_index)?.sub_shapes().to_vec();
        for face_index in faces {
            if self.shape_info(face_index)?.shape_type() != ShapeEnum::Face {
                continue;
            }
            indices.add(face_index);
            bounds.add_box(self.shape_info(face_index)?.box_bounds());
            for child in self.shape_info(face_index)?.sub_shapes() {
                indices.add(*child);
            }
        }
        Ok(())
    }

    fn shell_is_open(&self, shell_index: i32) -> Result<bool, Failure> {
        let mut edge_faces: HashMap<i32, Vec<Orientation>> = HashMap::new();
        for face_index in self.shape_info(shell_index)?.sub_shapes() {
            if self.shape_info(*face_index)?.shape_type() != ShapeEnum::Face {
                continue;
            }
            self.add_face_edge_orientations(*face_index, &mut edge_faces)?;
        }
        Ok(edge_faces.values().any(|faces| faces.len() == 1))
    }

    fn add_face_edge_orientations(
        &self,
        face_index: i32,
        edge_faces: &mut HashMap<i32, Vec<Orientation>>,
    ) -> Result<(), Failure> {
        let face = self.shape(face_index)?;
        let orientation = face.orientation();
        for child in self.shape_info(face_index)?.sub_shapes() {
            if self.shape_info(*child)?.shape_type() == ShapeEnum::Edge {
                let edge = Edge::unchecked(self.shape(*child)?.clone());
                if !BRepTool::degenerated(&edge)?
                    && !matches!(orientation, Orientation::Internal | Orientation::External)
                {
                    edge_faces.entry(*child).or_default().push(orientation);
                }
            }
        }
        Ok(())
    }

    fn build_vertex_edge_map(&mut self) -> Result<(), Failure> {
        for index in 0..self.source_shape_count {
            if self.shape_info(index)?.shape_type() != ShapeEnum::Edge {
                continue;
            }
            for vertex_index in self.shape_info(index)?.sub_shapes().to_vec() {
                if !self.vertex_edges.is_bound(&vertex_index) {
                    self.vertex_edges.bind(vertex_index, Vec::new());
                }
                let edge_indices = self.vertex_edges.change_seek(&vertex_index).unwrap();
                if !edge_indices.contains(&index) {
                    edge_indices.push(index);
                }
            }
        }
        Ok(())
    }

    fn init_pave_blocks(&mut self, index: i32) -> Result<(), Failure> {
        let edge = Edge::unchecked(self.shape(index)?.clone());
        let vertex_indices = self.shape_info(index)?.sub_shapes().to_vec();
        if vertex_indices.is_empty() {
            return Ok(());
        }
        let mut block = PaveBlock::default();
        block.set_original_edge(index);
        if edge.orientation() != Orientation::Internal {
            self.append_endpoint_paves(index, &vertex_indices, &edge, &mut block)?;
        }
        let mut splits = Vec::new();
        block.update(&mut splits, false);
        self.pave_blocks_pool.push(
            splits
                .into_iter()
                .map(pave_block_handle)
                .collect::<Vec<_>>(),
        );
        let pool_index = self.pave_blocks_pool.len() as i32 - 1;
        self.change_shape_info(index)?.set_reference(pool_index);
        Ok(())
    }

    fn append_endpoint_paves(
        &self,
        edge_index: i32,
        vertex_indices: &[i32],
        edge: &Edge,
        block: &mut PaveBlock,
    ) -> Result<(), Failure> {
        let degenerated = self.shape_info(edge_index)?.has_flag();
        for vertex_index in vertex_indices {
            let vertex = Vertex::unchecked(self.shape(*vertex_index)?.clone());
            let parameter = BRepTool::parameter(&vertex, edge)?;
            let pave = Pave::new(*vertex_index, parameter);
            if degenerated {
                block.append_ext_pave1(pave);
            } else {
                block.append_ext_pave(pave);
            }
        }
        Ok(())
    }
}

fn additional_tolerance(fuzz: f64) -> f64 {
    let confusion = rustycad_tkernel::precision::confusion();
    if fuzz < confusion {
        confusion * 0.5
    } else {
        fuzz * 0.5
    }
}

fn checked_slot(index: i32, length: usize, operation: &str) -> Result<usize, Failure> {
    let slot = usize::try_from(index).ok().filter(|slot| *slot < length);
    slot.ok_or_else(|| {
        Failure::with_type(
            "Standard_OutOfRange",
            &format!("{operation}: index {index}; expected 0..{length}"),
            "",
        )
    })
}
