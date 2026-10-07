//! Read-side `BOPDS_DS` queries: face splits, shared edges, sub-shape tests and solid boxes.
use super::{DataStructure, Failure, ShapeEnum, VertexIndexSet};
use crate::bop_ds::{InterfEZ, InterfFZ, InterfVZ, InterfZZ, PaveBlockHandle, PaveBlockIndexMap};
use crate::bop_tools::{is_inverted_solid, is_open_shell};
use rustycad_tkbrep::topods::{Shell, Solid};
use rustycad_tkernel::ncollection::{DynamicArray, Map};
use rustycad_tkmath::bnd::BoundingBox3;
use std::fmt::Write;

/// Shapes and blocks shared by two faces, as filled by native `BOPDS_DS::SubShapesOnIn`.
/// The maps keep their bucket state across [`SubShapesOnIn::clear`], as the native caller's
/// maps do, because the iteration order of a map depends on it.
/// `let mut shared = SubShapesOnIn::default(); ds.sub_shapes_on_in(f1, f2, &mut shared)?;`.
pub struct SubShapesOnIn {
    /// Vertices of both faces' On/In blocks plus the vertices common to both faces.
    pub vertices_on_in: Map<i32>,
    /// Vertices of the blocks and vertex sets present on both faces.
    pub vertices_common: Map<i32>,
    /// Every On/In pave block of both faces in native insertion order.
    pub pave_blocks_on_in: PaveBlockIndexMap,
    /// The blocks of the first face that are also On/In the second one.
    pub common_pave_blocks: PaveBlockIndexMap,
}

impl Default for SubShapesOnIn {
    /// Maps of 100 buckets, the size MakeBlocks gives them; `SubShapesOnIn::default()`.
    fn default() -> Self {
        Self {
            vertices_on_in: Map::new(100),
            vertices_common: Map::new(100),
            pave_blocks_on_in: PaveBlockIndexMap::default(),
            common_pave_blocks: PaveBlockIndexMap::default(),
        }
    }
}

impl SubShapesOnIn {
    /// Empties all maps keeping their buckets (`Clear()`); `shared.clear()`.
    pub fn clear(&mut self) {
        self.vertices_on_in.clear(false);
        self.vertices_common.clear(false);
        self.pave_blocks_on_in.clear();
        self.common_pave_blocks.clear();
    }
}

impl DataStructure {
    /// Removes from each source face's In blocks those also On the face; `ds.refine_face_info_in()?`.
    pub fn refine_face_info_in(&mut self) -> Result<(), Failure> {
        for index in 0..self.source_shape_count {
            let info = self.shape_info(index)?;
            if info.shape_type() != ShapeEnum::Face || !info.has_reference() {
                continue;
            }
            let slot = self.face_info_slot(index)?;
            self.refine_face_in_slot(slot)?;
        }
        Ok(())
    }

    /// Lists face vertices (In then Section) not bounding any In/Section block, in map order.
    /// `let alone = ds.alone_vertices(face)?;`.
    pub fn alone_vertices(&self, face: i32) -> Result<Vec<i32>, Failure> {
        let mut alone = Vec::new();
        if !self.has_face_info(face)? {
            return Ok(alone);
        }
        let info = self.face_info(face)?;
        let mut bounding = VertexIndexSet::default();
        for blocks in [info.pave_blocks_in(), info.pave_blocks_section()] {
            for block in blocks.iter() {
                let (first, last) = lock(block).indices();
                bounding.add(first);
                bounding.add(last);
            }
        }
        for vertices in [info.vertices_in(), info.vertices_section()] {
            for vertex in vertices.iter().copied() {
                if vertex >= 0 && bounding.add(vertex) {
                    alone.push(vertex);
                }
            }
        }
        Ok(alone)
    }

    /// Adds the On/In blocks and vertices of two faces and those common to both to `shared`.
    /// `ds.sub_shapes_on_in(f1, f2, &mut shared)?;`.
    pub fn sub_shapes_on_in(
        &self,
        face1: i32,
        face2: i32,
        shared: &mut SubShapesOnIn,
    ) -> Result<(), Failure> {
        let info1 = self.face_info(face1)?;
        let info2 = self.face_info(face2)?;
        for blocks in [
            info1.pave_blocks_on(),
            info1.pave_blocks_in(),
            info2.pave_blocks_on(),
            info2.pave_blocks_in(),
        ] {
            add_blocks_and_bounds(blocks, shared);
        }
        for blocks in [info1.pave_blocks_on(), info1.pave_blocks_in()] {
            add_common_blocks(blocks, info2, shared);
        }
        for vertices in [info1.vertices_on(), info1.vertices_in()] {
            for vertex in vertices.iter().copied() {
                if info2.vertices_on().contains(vertex) || info2.vertices_in().contains(vertex) {
                    shared.vertices_on_in.add(vertex);
                    shared.vertices_common.add(vertex);
                }
            }
        }
        Ok(())
    }

    /// Returns the edges (split images when present) of the second face also bounding the first.
    /// `let edges = ds.shared_edges(f1, f2)?;`.
    pub fn shared_edges(&self, face1: i32, face2: i32) -> Result<Vec<i32>, Failure> {
        let mut first_edges: Map<i32> = Map::new(100);
        for edge in self.face_edge_images(face1)? {
            first_edges.add(edge);
        }
        Ok(self
            .face_edge_images(face2)?
            .into_iter()
            .filter(|edge| first_edges.contains(edge))
            .collect())
    }

    /// Tests a stored pair between `shape` and any (or every) sub-shape of `parent`; a parent
    /// without sub-shapes gives false in both modes, as the native loop leaves its flag unset.
    /// `ds.has_interference_shape_subshapes_mode(v, e, false)?`.
    pub fn has_interference_shape_subshapes_mode(
        &self,
        shape: i32,
        parent: i32,
        any_interference: bool,
    ) -> Result<bool, Failure> {
        let sub_shapes = self.shape_info(parent)?.sub_shapes();
        if sub_shapes.is_empty() {
            return Ok(false);
        }
        let mut interfering = sub_shapes
            .iter()
            .map(|sub_shape| self.has_interference_between(shape, *sub_shape));
        if any_interference {
            return Ok(interfering.any(|found| found));
        }
        Ok(interfering.all(|found| found))
    }

    /// Tests whether any sub-shape of `first` interferes with any sub-shape of `second`.
    /// `ds.has_interference_sub_shapes(e1, e2)?`.
    pub fn has_interference_sub_shapes(&self, first: i32, second: i32) -> Result<bool, Failure> {
        for sub_shape in self.shape_info(first)?.sub_shapes() {
            if self.has_interference_shape_subshapes(*sub_shape, second)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Tests direct sub-shape membership by index; `ds.is_sub_shape(vertex, edge)?`.
    pub fn is_sub_shape(&self, candidate: i32, parent: i32) -> Result<bool, Failure> {
        Ok(self.shape_info(parent)?.sub_shapes().contains(&candidate))
    }

    /// Accumulates the face boxes of a solid, opening the box for open or inverted solids.
    /// `ds.build_bnd_box_solid(solid, &mut bounds, true)?`.
    pub fn build_bnd_box_solid(
        &self,
        index: i32,
        bounds: &mut BoundingBox3,
        check_inverted: bool,
    ) -> Result<(), Failure> {
        let mut is_open = false;
        for shell in self.shape_info(index)?.sub_shapes() {
            if self.shape_info(*shell)?.shape_type() != ShapeEnum::Shell {
                continue;
            }
            is_open = self.add_shell_face_boxes(*shell, bounds)?;
            if is_open {
                break;
            }
        }
        if is_open {
            bounds.set_whole();
            return Ok(());
        }
        if check_inverted && is_inverted_solid(&Solid::unchecked(self.shape(index)?.clone()))? {
            bounds.set_whole();
        }
        Ok(())
    }

    /// Writes native `Dump()` text: ranges, then every shape record; `ds.dump(&mut text)?`.
    pub fn dump(&self, output: &mut dyn Write) -> Result<(), Failure> {
        self.dump_text(output)
            .map_err(|_| Failure::with_message("Standard_Failure", "BOPDS_DS::Dump write failed"))
    }

    /// Mutably borrows vertex/solid records; `ds.interf_vz().len()`.
    pub fn interf_vz(&mut self) -> &mut DynamicArray<InterfVZ> {
        &mut self.interf_vz
    }

    /// Mutably borrows edge/solid records; `ds.interf_ez().len()`.
    pub fn interf_ez(&mut self) -> &mut DynamicArray<InterfEZ> {
        &mut self.interf_ez
    }

    /// Mutably borrows face/solid records; `ds.interf_fz().len()`.
    pub fn interf_fz(&mut self) -> &mut DynamicArray<InterfFZ> {
        &mut self.interf_fz
    }

    /// Mutably borrows solid/solid records; `ds.interf_zz().len()`.
    pub fn interf_zz(&mut self) -> &mut DynamicArray<InterfZZ> {
        &mut self.interf_zz
    }

    /// The pair table itself, which the self-interference check clears and refills with its
    /// own pairs (native `BOPAlgo_CheckerSI` casts away the constness of `Interferences()`).
    pub(crate) fn interference_table_mut(&mut self) -> &mut Map<super::Pair> {
        &mut self.interference_table
    }

    /// Native number of interference kinds (VV..ZZ); `DataStructure::nb_interf_types() == 10`.
    pub fn nb_interf_types() -> i32 {
        10
    }

    fn refine_face_in_slot(&mut self, slot: usize) -> Result<(), Failure> {
        let info = &mut self.face_info_pool[slot];
        if info.pave_blocks_in().extent() == 0 || info.pave_blocks_on().extent() == 0 {
            return Ok(());
        }
        let on: Vec<PaveBlockHandle> = info.pave_blocks_on().iter().cloned().collect();
        let blocks_in = info.change_pave_blocks_in();
        for position in (1..=blocks_in.extent()).rev() {
            let block = blocks_in.find_key(position)?;
            if on
                .iter()
                .any(|candidate| std::sync::Arc::ptr_eq(candidate, block))
            {
                blocks_in.remove_from_index(position)?;
            }
        }
        Ok(())
    }

    fn face_edge_images(&self, face: i32) -> Result<Vec<i32>, Failure> {
        let mut edges = Vec::new();
        for sub_shape in self.shape_info(face)?.sub_shapes() {
            if self.shape_info(*sub_shape)?.shape_type() != ShapeEnum::Edge {
                continue;
            }
            let blocks = self.pave_blocks(*sub_shape)?;
            if blocks.is_empty() {
                edges.push(*sub_shape);
                continue;
            }
            for block in blocks {
                edges.push(lock(&self.real_pave_block(block)?).edge());
            }
        }
        Ok(edges)
    }

    fn add_shell_face_boxes(&self, shell: i32, bounds: &mut BoundingBox3) -> Result<bool, Failure> {
        let mut is_open = false;
        for face in self.shape_info(shell)?.sub_shapes() {
            let info = self.shape_info(*face)?;
            if info.shape_type() != ShapeEnum::Face {
                continue;
            }
            bounds.add_box(info.box_bounds());
            is_open = info.box_bounds().is_open();
            if is_open {
                break;
            }
        }
        if !is_open {
            is_open = is_open_shell(&Shell::unchecked(self.shape(shell)?.clone()))?;
        }
        Ok(is_open)
    }

    fn dump_text(&self, output: &mut dyn Write) -> std::fmt::Result {
        writeln!(output, " *** DS ***")?;
        writeln!(output, " Ranges:{}", self.ranges.len())?;
        for range in &self.ranges {
            range.dump(output)?;
            writeln!(output)?;
        }
        writeln!(output, " Shapes:{}", self.source_shape_count)?;
        for (index, info) in self.lines.iter().enumerate() {
            write!(output, " {index} :")?;
            info.dump(output)?;
            writeln!(output)?;
            if index as i32 == self.source_shape_count - 1 {
                writeln!(output, " ****** adds")?;
            }
        }
        writeln!(output, " ******")
    }
}

fn add_blocks_and_bounds(blocks: &PaveBlockIndexMap, shared: &mut SubShapesOnIn) {
    for block in blocks.iter() {
        shared.pave_blocks_on_in.add(block.clone());
        let (first, last) = lock(block).indices();
        shared.vertices_on_in.add(first);
        shared.vertices_on_in.add(last);
    }
}

fn add_common_blocks(
    blocks: &PaveBlockIndexMap,
    other: &crate::bop_ds::FaceInfo,
    shared: &mut SubShapesOnIn,
) {
    for block in blocks.iter() {
        if !other.pave_blocks_on().contains(block) && !other.pave_blocks_in().contains(block) {
            continue;
        }
        shared.common_pave_blocks.add(block.clone());
        let (first, last) = lock(block).indices();
        shared.vertices_common.add(first);
        shared.vertices_common.add(last);
    }
}

fn lock(block: &PaveBlockHandle) -> std::sync::MutexGuard<'_, crate::bop_ds::PaveBlock> {
    block.lock().expect("pave-block lock must not be poisoned")
}
