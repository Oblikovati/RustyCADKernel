//! Shape storage and the ported topology-preparation slice of `BOPDS_DS::Init`.
use super::{
    CommonBlockHandle, FaceInfo, IndexRange, InterfEE, InterfEF, InterfFF, InterfVE, InterfVF,
    InterfVV, Pair, Pave, PaveBlock, PaveBlockHandle, ShapeInfo, VertexIndexSet, pave_block_handle,
};
use rustycad_tkbrep::topods::{Edge, Face, Shape, ShapeIterator, Vertex};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    brep_adaptor::BRepAdaptorCurve,
};
use rustycad_tkernel::ncollection::{DataMap, DynamicArray, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use std::collections::HashMap;
use std::sync::Arc;

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
    pave_block_common_blocks: Vec<(PaveBlockHandle, CommonBlockHandle)>,
    face_info_pool: Vec<FaceInfo>,
    interf_vv: DynamicArray<InterfVV>,
    interf_ve: DynamicArray<InterfVE>,
    interf_vf: DynamicArray<InterfVF>,
    interf_ee: DynamicArray<InterfEE>,
    interf_ef: DynamicArray<InterfEF>,
    interf_ff: DynamicArray<InterfFF>,
    interference_table: Map<Pair>,
    interfered_shapes: Map<i32>,
    same_domain_vertices: DataMap<i32, i32>,
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

    /// Adds an unordered pair to the interference summary, reporting first insertion.
    /// `ds.add_interference(3, 8)`.
    pub fn add_interference(&mut self, first: i32, second: i32) -> bool {
        if !self.interference_table.add(Pair::new(first, second)) {
            return false;
        }
        self.interfered_shapes.add(first);
        self.interfered_shapes.add(second);
        true
    }

    /// Binds a source vertex to its same-domain representative; `ds.add_shape_sd(3, 8)`.
    pub fn add_shape_sd(&mut self, source: i32, representative: i32) {
        self.same_domain_vertices.bind(source, representative);
    }

    /// Reads a same-domain representative; `ds.shape_sd(3) == Some(8)`.
    pub fn shape_sd(&self, source: i32) -> Option<i32> {
        self.same_domain_vertices.find(&source).ok().copied()
    }

    /// Traverses same-domain entries in native hash-bucket order; `ds.shape_sd_items().count()`.
    pub fn shape_sd_items(&self) -> impl Iterator<Item = (&i32, &i32)> {
        self.same_domain_vertices.items()
    }

    /// Reports whether any stored pair contains `index`; `ds.has_interference_for(3)`.
    pub fn has_interference_for(&self, index: i32) -> bool {
        self.interfered_shapes.contains(&index)
    }

    /// Reports whether the two shapes form a stored unordered pair.
    /// `ds.has_interference_between(3, 8)`.
    pub fn has_interference_between(&self, first: i32, second: i32) -> bool {
        self.interference_table.contains(&Pair::new(first, second))
    }

    /// Detects a vertex interference with any direct subshape of `index`; `ds.has_interference_shape_subshapes(vertex, edge)`.
    pub fn has_interference_shape_subshapes(
        &self,
        shape: i32,
        parent: i32,
    ) -> Result<bool, Failure> {
        let info = self.shape_info(parent)?;
        Ok(info
            .sub_shapes()
            .iter()
            .any(|subshape| self.has_interference_between(shape, *subshape)))
    }

    /// Borrows the deduplicated pair table in native hash-map traversal order.
    /// `ds.interferences().iter()`.
    pub fn interferences(&self) -> &Map<Pair> {
        &self.interference_table
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

    /// Mutably borrows vertex/vertex records; `ds.interf_vv().append(InterfVV::default())`.
    pub fn interf_vv(&mut self) -> &mut DynamicArray<InterfVV> {
        &mut self.interf_vv
    }

    /// Borrows vertex/vertex records in native insertion order; `ds.interf_vv_records().len()`.
    pub fn interf_vv_records(&self) -> &DynamicArray<InterfVV> {
        &self.interf_vv
    }

    /// Mutably borrows vertex/edge records; `ds.interf_ve().append(InterfVE::default())`.
    pub fn interf_ve(&mut self) -> &mut DynamicArray<InterfVE> {
        &mut self.interf_ve
    }

    /// Borrows vertex/edge interference records in insertion order; `ds.interf_ve_records().len()`.
    pub fn interf_ve_records(&self) -> &DynamicArray<InterfVE> {
        &self.interf_ve
    }

    /// Mutably borrows vertex/face records; `ds.interf_vf().append(InterfVF::default())`.
    pub fn interf_vf(&mut self) -> &mut DynamicArray<InterfVF> {
        &mut self.interf_vf
    }

    /// Mutably borrows edge/edge records; `ds.interf_ee().append(InterfEE::default())`.
    pub fn interf_ee(&mut self) -> &mut DynamicArray<InterfEE> {
        &mut self.interf_ee
    }

    /// Mutably borrows edge/face records; `ds.interf_ef().append(InterfEF::default())`.
    pub fn interf_ef(&mut self) -> &mut DynamicArray<InterfEF> {
        &mut self.interf_ef
    }

    /// Mutably borrows face/face records; `ds.interf_ff().append(InterfFF::default())`.
    pub fn interf_ff(&mut self) -> &mut DynamicArray<InterfFF> {
        &mut self.interf_ff
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

    /// Validates cached shrunk endpoints against the source edge; `ds.is_valid_shrunk_data(&block)?`.
    pub fn is_valid_shrunk_data(&self, block: &PaveBlockHandle) -> Result<bool, Failure> {
        let locked = block.lock().expect("pave-block lock must not be poisoned");
        if !locked.has_shrunk_data() {
            return Ok(false);
        }
        valid_shrunk_endpoints(self, &locked)
    }

    /// Replaces split endpoints by same-domain representatives; `ds.update_pave_block_with_sd_vertices(&block)`.
    pub fn update_pave_block_with_sd_vertices(&self, block: &PaveBlockHandle) {
        let mut block = block.lock().expect("pave-block lock must not be poisoned");
        let first = block.pave1();
        let last = block.pave2();
        block.set_pave1(resolve_pave_vertex(self, first));
        block.set_pave2(resolve_pave_vertex(self, last));
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

    /// Tests whether this exact pave-block handle has common-block metadata.
    /// `ds.is_common_block(&block)`.
    pub fn is_common_block(&self, block: &PaveBlockHandle) -> bool {
        self.common_block_entry(block).is_some()
    }

    /// Clones common-block metadata for this exact pave-block handle, if present.
    /// `let group = ds.common_block(&block);`.
    pub fn common_block(&self, block: &PaveBlockHandle) -> Option<CommonBlockHandle> {
        self.common_block_entry(block)
            .map(|(_, common)| Arc::clone(common))
    }

    /// Associates a pave-block handle with common-block metadata; `ds.set_common_block(&pb, cb)`.
    pub fn set_common_block(&mut self, block: &PaveBlockHandle, common: &CommonBlockHandle) {
        if let Some((_, existing)) = self.common_block_entry_mut(block) {
            *existing = Arc::clone(common);
            return;
        }
        self.pave_block_common_blocks
            .push((Arc::clone(block), Arc::clone(common)));
    }

    /// Returns the common block's first pave block, or `block` when ungrouped.
    /// `let real = ds.real_pave_block(&block);`.
    pub fn real_pave_block(&self, block: &PaveBlockHandle) -> Result<PaveBlockHandle, Failure> {
        let Some(common) = self.common_block(block) else {
            return Ok(Arc::clone(block));
        };
        let common = common
            .lock()
            .expect("common-block lock must not be poisoned");
        Ok(Arc::clone(common.pave_block1()?))
    }

    /// Reports whether this block belongs to a common group with multiple blocks.
    /// `ds.is_common_block_on_edge(&block)`.
    pub fn is_common_block_on_edge(&self, block: &PaveBlockHandle) -> bool {
        let Some(common) = self.common_block(block) else {
            return false;
        };
        common
            .lock()
            .expect("common-block lock must not be poisoned")
            .pave_blocks()
            .len()
            > 1
    }

    /// Clears arguments and all currently implemented DS storage.
    /// `ds.clear(); assert_eq!(ds.nb_shapes(), 0);`.
    pub fn clear(&mut self) {
        self.arguments.clear();
        self.ranges.clear();
        self.lines.clear();
        self.vertex_edges.clear(false);
        self.pave_blocks_pool.clear();
        self.pave_block_common_blocks.clear();
        self.face_info_pool.clear();
        self.interf_vv.clear(false);
        self.interf_ve.clear(false);
        self.interf_vf.clear(false);
        self.interf_ee.clear(false);
        self.interf_ef.clear(false);
        self.interf_ff.clear(false);
        self.same_domain_vertices.clear(false);
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

    fn common_block_entry(
        &self,
        block: &PaveBlockHandle,
    ) -> Option<&(PaveBlockHandle, CommonBlockHandle)> {
        self.pave_block_common_blocks
            .iter()
            .find(|(candidate, _)| Arc::ptr_eq(candidate, block))
    }

    fn common_block_entry_mut(
        &mut self,
        block: &PaveBlockHandle,
    ) -> Option<&mut (PaveBlockHandle, CommonBlockHandle)> {
        self.pave_block_common_blocks
            .iter_mut()
            .find(|(candidate, _)| Arc::ptr_eq(candidate, block))
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
        if BRepTool::degenerated(&edge)? {
            self.change_shape_info(index)?.set_flag(index);
        } else {
            self.append_infinite_edge_vertices(index, &edge)?;
        }
        let mut bounds = self.shape_info(index)?.box_bounds().clone();
        BRepBndLib::add(&edge, &mut bounds, true)?;
        self.add_child_boxes(index, &mut bounds)?;
        bounds.set_gap(bounds.gap() + added_gap);
        self.change_shape_info(index)?.set_box(&bounds);
        Ok(())
    }

    fn append_infinite_edge_vertices(
        &mut self,
        edge_index: i32,
        edge: &Edge,
    ) -> Result<(), Failure> {
        let forward = Edge::unchecked(edge.oriented(Orientation::Forward));
        let (curve, first, last) = BRepTool::curve(&forward)?;
        self.append_infinite_edge_endpoint(edge_index, &curve, first)?;
        self.append_infinite_edge_endpoint(edge_index, &curve, last)?;
        Ok(())
    }

    fn append_infinite_edge_endpoint(
        &mut self,
        edge_index: i32,
        curve: &rustycad_tkernel::standard::Handle<dyn rustycad_tkg3d::geom::GeomCurve>,
        parameter: f64,
    ) -> Result<(), Failure> {
        if !rustycad_tkernel::precision::is_infinite(parameter) {
            return Ok(());
        }
        let point = curve
            .get()
            .expect("BOPDS_DS: infinite edge range requires a curve")
            .d0(parameter)?;
        let edge = Edge::unchecked(self.shape(edge_index)?.clone());
        let tolerance = BRepTool::edge_tolerance(&edge)?;
        let mut vertex = Vertex::default();
        BRepBuilder::default().make_vertex_at(&mut vertex, point, tolerance)?;
        let mut info = ShapeInfo::default();
        info.set_shape_type(ShapeEnum::Vertex);
        info.set_shape(&vertex.into_shape());
        info.set_flag(1);
        let vertex_index = self.append_shape_info(&info);
        self.change_shape_info(edge_index)?
            .change_sub_shapes()
            .push(vertex_index);
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
        if edge.orientation() == Orientation::Internal {
            self.append_internal_edge_paves(&edge, &mut block)?;
        } else {
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
        for vertex_index in vertex_indices {
            self.append_endpoint_pave(edge_index, *vertex_index, edge, block, false)?;
        }
        if let [vertex_index] = vertex_indices {
            self.append_endpoint_pave(edge_index, *vertex_index, edge, block, true)?;
        }
        Ok(())
    }

    fn append_endpoint_pave(
        &self,
        edge_index: i32,
        vertex_index: i32,
        edge: &Edge,
        block: &mut PaveBlock,
        reverse_vertex: bool,
    ) -> Result<(), Failure> {
        let mut vertex = Vertex::unchecked(self.shape(vertex_index)?.clone());
        if reverse_vertex {
            vertex = Vertex::unchecked(vertex.reversed());
        }
        let parameter = endpoint_parameter(self, vertex_index, &vertex, edge)?;
        let pave = Pave::new(vertex_index, parameter);
        if self.shape_info(edge_index)?.has_flag() || reverse_vertex {
            block.append_ext_pave1(pave);
        } else {
            block.append_ext_pave(pave);
        }
        Ok(())
    }

    fn append_internal_edge_paves(
        &self,
        edge: &Edge,
        block: &mut PaveBlock,
    ) -> Result<(), Failure> {
        let mut vertices = ShapeIterator::new(edge, false, true);
        while vertices.more() {
            let vertex_shape = vertices.value()?.clone();
            let vertex_index = self.index(&vertex_shape);
            let vertex = Vertex::unchecked(vertex_shape);
            let parameter = endpoint_parameter(self, vertex_index, &vertex, edge)?;
            block.append_ext_pave1(Pave::new(vertex_index, parameter));
            vertices.next();
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

fn endpoint_parameter(
    structure: &DataStructure,
    vertex_index: i32,
    vertex: &Vertex,
    edge: &Edge,
) -> Result<f64, Failure> {
    if structure.shape_info(vertex_index)?.has_flag() {
        return compute_endpoint_parameter(vertex, edge);
    }
    BRepTool::parameter(vertex, edge)
}

fn compute_endpoint_parameter(vertex: &Vertex, edge: &Edge) -> Result<f64, Failure> {
    let tolerance = BRepTool::edge_tolerance(edge)?;
    let tolerance_square = tolerance * tolerance;
    let vertex_point = BRepTool::pnt(vertex)?;
    let forward = Edge::unchecked(edge.oriented(Orientation::Forward));
    let (curve, first, last) = BRepTool::curve(&forward)?;
    let curve = curve
        .get()
        .expect("BOPDS_DS: flagged infinite endpoint requires an edge curve");
    if curve.d0(first)?.square_distance(&vertex_point) < tolerance_square {
        return Ok(first);
    }
    if curve.d0(last)?.square_distance(&vertex_point) < tolerance_square {
        return Ok(last);
    }
    Ok(0.0)
}

fn valid_shrunk_endpoints(structure: &DataStructure, block: &PaveBlock) -> Result<bool, Failure> {
    let edge = Edge::unchecked(structure.shape(block.original_edge())?.clone());
    let curve = BRepAdaptorCurve::with_edge(&edge)?;
    let epsilon = BRepTool::edge_tolerance(&edge)? * 0.01;
    let (first, last, _, _) = block.shrunk_data();
    let (first_vertex, last_vertex) = block.indices();
    for (index, parameter) in [(first_vertex, first), (last_vertex, last)] {
        if !valid_shrunk_endpoint(structure, &curve, index, parameter, epsilon)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn resolve_pave_vertex(structure: &DataStructure, pave: Pave) -> Pave {
    let index = structure.shape_sd(pave.index()).unwrap_or(pave.index());
    Pave::new(index, pave.parameter())
}

fn valid_shrunk_endpoint(
    structure: &DataStructure,
    curve: &BRepAdaptorCurve,
    vertex_index: i32,
    parameter: f64,
    epsilon: f64,
) -> Result<bool, Failure> {
    let vertex = Vertex::unchecked(structure.shape(vertex_index)?.clone());
    let tolerance = BRepTool::vertex_tolerance(&vertex)? + rustycad_tkernel::precision::confusion();
    let vertex_point = BRepTool::pnt(&vertex)?;
    let edge_point = curve.value(parameter)?;
    Ok(tolerance - vertex_point.distance(&edge_point) <= epsilon)
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
