//! `BOPAlgo_PaveFiller::MakeBlocks` (`BOPAlgo_PaveFiller_6.cxx`): section edges from the
//! face/face curves, their post treatment and the face record updates.
#[path = "make_blocks/curve_ends.rs"]
mod curve_ends;
#[path = "make_blocks/existing_blocks.rs"]
mod existing_blocks;
#[path = "make_blocks/face_info_update.rs"]
mod face_info_update;
#[path = "make_blocks/paves_on_curve.rs"]
mod paves_on_curve;
#[path = "make_blocks/pool_updates.rs"]
mod pool_updates;
#[path = "make_blocks/post_treat.rs"]
mod post_treat;
#[path = "make_blocks/post_treat_map.rs"]
mod post_treat_map;
#[path = "make_blocks/process_existing.rs"]
mod process_existing;
#[path = "make_blocks/se_tolerance.rs"]
mod se_tolerance;
#[path = "make_blocks/section_edges.rs"]
mod section_edges;
#[path = "make_blocks/stick_paves.rs"]
mod stick_paves;

use super::{BOPAlgoPaveFiller, Glue, missing_structure};
use crate::bop_ds::{
    BopdsCurve, CoupleOfPaveBlocks, PaveBlockHandle, PaveBlockHandleIdentity, SubShapesOnIn,
};
use crate::bop_tools::{BoxSet, make_new_vertex_point};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Face, Shape, Vertex};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, DefaultHashPolicy, IndexedDataMap, IndexedMap, Map};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::{BoundingBox3, box3_to_bvh};
use rustycad_tkmath::bvh::BvhVec3d;
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;

/// A section curve: the face/face interference index and the curve index in it.
pub(super) type CurveRef = (usize, usize);
/// Initial vertex tolerances saved before putting vertices on curves (`aMVTol`).
pub(super) type VertexTolerances = DataMap<i32, f64, DefaultHashPolicy>;
/// Groups of vertices sharing one pave of a curve (`aDMVLV`).
pub(super) type VertexGroups = DataMap<i32, Vec<i32>, DefaultHashPolicy>;
/// Box tree of the On/In pave blocks of a face pair (`BOPTools_BoxTree`).
pub(super) type BlockTree = BoxSet<BvhVec3d>;
/// Faces a pave block has to be added to (`BOPAlgo_DataMapOfPaveBlockListOfInteger`).
pub(super) type BlockFaces = DataMap<PaveBlockHandle, Vec<i32>, PaveBlockHandleIdentity>;
/// Pave block fence map (`NCollection_Map<handle<BOPDS_PaveBlock>>`).
pub(super) type SectionBlocks = Map<PaveBlockHandle, PaveBlockHandleIdentity>;
/// Existing edges and the pave blocks replacing them (`aDMExEdges`).
pub(super) type ExistingEdges =
    DataMap<PaveBlockHandle, Vec<PaveBlockHandle>, PaveBlockHandleIdentity>;

/// Section shapes kept for the post treatment (`aMSCPB`) and their vertex indices (`aMVI`).
pub(super) struct SectionShapes {
    pub(super) couples: IndexedDataMap<Shape, CoupleOfPaveBlocks, ShapeMapHasher>,
    pub(super) vertex_indices: DataMap<Shape, i32, ShapeMapHasher>,
}

/// The maps of MakeBlocks: per face pair ones are cleared, keeping their buckets, as natively.
struct MakeBlocksState {
    shared: SubShapesOnIn,
    shared_edges: Vec<i32>,
    vertices_stick: Map<i32>,
    vertices_ef: Map<i32>,
    vertices_bounds: Map<i32>,
    shapes: Map<i32>,
    bound_vertices: DataMap<i32, Vec<i32>, DefaultHashPolicy>,
    tolerances: VertexTolerances,
    added: SectionBlocks,
    sections: SectionShapes,
    existing_edges: ExistingEdges,
    new_sd: DataMap<i32, i32, DefaultHashPolicy>,
    groups: VertexGroups,
    micro_blocks: crate::bop_ds::PaveBlockIndexMap,
    vertices_on_rejected: IndexedMap<Shape, ShapeMapHasher>,
    block_faces: BlockFaces,
}

impl MakeBlocksState {
    fn new() -> Self {
        Self {
            shared: SubShapesOnIn::default(),
            shared_edges: Vec::new(),
            vertices_stick: Map::new(100),
            vertices_ef: Map::new(100),
            vertices_bounds: Map::new(100),
            shapes: Map::new(100),
            bound_vertices: DataMap::with_hasher(DefaultHashPolicy, 100),
            tolerances: DataMap::with_hasher(DefaultHashPolicy, 100),
            added: Map::with_hasher(PaveBlockHandleIdentity, 100),
            sections: SectionShapes {
                couples: IndexedDataMap::with_hasher(ShapeMapHasher, 100),
                vertex_indices: DataMap::with_hasher(ShapeMapHasher, 100),
            },
            existing_edges: DataMap::with_hasher(PaveBlockHandleIdentity, 100),
            new_sd: DataMap::with_hasher(DefaultHashPolicy, 100),
            groups: DataMap::with_hasher(DefaultHashPolicy, 100),
            micro_blocks: crate::bop_ds::PaveBlockIndexMap::default(),
            vertices_on_rejected: IndexedMap::with_hasher(ShapeMapHasher, 100),
            block_faces: DataMap::with_hasher(PaveBlockHandleIdentity, 100),
        }
    }

    fn clear_pair_maps(&mut self) {
        self.shared.clear();
        self.bound_vertices.clear(false);
        self.tolerances.clear(false);
        self.shared_edges.clear();
        self.vertices_stick.clear(false);
        self.vertices_ef.clear(false);
        self.vertices_bounds.clear(false);
        self.shapes.clear(false);
    }
}

impl BOPAlgoPaveFiller {
    /// Builds the section edges of all face/face curves, post-treats them with a nested
    /// intersection, and updates tolerances, face records and pave blocks (`MakeBlocks`).
    /// `filler.make_blocks()?`.
    pub fn make_blocks(&mut self) -> Result<(), Failure> {
        if self.glue != Glue::Off {
            return Ok(());
        }
        let mut count = self.ff_count()?;
        if count == 0 {
            return Ok(());
        }
        let mut state = MakeBlocksState::new();
        let mut to_recheck: Vec<usize> = Vec::new();
        let previous = count;
        let mut index = 0;
        while index < count {
            // Pairs with curves are processed again at the end so that no section edge is
            // missed because of the processing order.
            let current = if index < previous {
                index
            } else {
                to_recheck[index - previous]
            };
            if self.make_pair_blocks(index, current, previous, &mut state)? {
                to_recheck.push(current);
                count += 1;
            }
            index += 1;
        }
        self.finish_make_blocks(state)
    }

    fn finish_make_blocks(&mut self, mut state: MakeBlocksState) -> Result<(), Failure> {
        self.remove_micro_section_edges(&mut state.sections, &mut state.micro_blocks)?;
        self.make_sd_vertices_ff(&state.groups, &mut state.new_sd)?;
        self.post_treat_ff(&mut state)?;
        if self.options.has_errors() {
            return Ok(());
        }
        self.correct_tolerance_of_se()?;
        self.update_face_info(&mut state.existing_edges, &state.new_sd, &state.block_faces)?;
        self.update_pave_blocks_sd(&state.new_sd)?;
        self.put_se_in_other_faces()
    }

    /// Processes one face/face interference; returns true when it has to be rechecked.
    fn make_pair_blocks(
        &mut self,
        index: usize,
        current: usize,
        previous: usize,
        state: &mut MakeBlocksState,
    ) -> Result<bool, Failure> {
        let (faces, curve_count, point_count) = self.ff_summary(current)?;
        if curve_count == 0 && point_count == 0 {
            return Ok(false);
        }
        let shapes = self.face_pair(faces.0, faces.1)?;
        let tolerance =
            BRepTool::face_tolerance(&shapes[0])?.max(BRepTool::face_tolerance(&shapes[1])?);
        let structure = self.structure_mut()?;
        structure.change_face_info(faces.0)?;
        structure.change_face_info(faces.1)?;
        state.clear_pair_maps();
        structure.sub_shapes_on_in(faces.0, faces.1, &mut state.shared)?;
        state.shared_edges = structure.shared_edges(faces.0, faces.1)?;
        self.make_point_vertices(index, current, point_count, tolerance, state)?;
        self.put_vertices_on_curves(current, &shapes, faces, curve_count, state)?;
        let mut tree = self.on_in_block_tree(&state.shared.pave_blocks_on_in)?;
        let mut recheck = curve_count > 0 && index < previous;
        for curve in 0..curve_count {
            if self.make_section_edges((current, curve), &shapes, faces, &mut tree, state)? {
                recheck = false;
            }
        }
        self.restore_unused_tolerances(state)?;
        self.process_existing_blocks_at_bounds(
            current,
            faces,
            &state.shared.pave_blocks_on_in,
            &mut tree,
            &state.bound_vertices,
            &mut state.sections,
            &mut state.block_faces,
            &mut state.added,
        )?;
        Ok(recheck)
    }

    /// Makes vertices of the section points not covered by an On/In vertex. The native code
    /// stores the loop counter (not the rechecked interference index) as interference index.
    fn make_point_vertices(
        &mut self,
        index: usize,
        current: usize,
        point_count: usize,
        tolerance: f64,
        state: &mut MakeBlocksState,
    ) -> Result<(), Failure> {
        for point_index in 0..point_count {
            let point = *self
                .data_structure
                .as_ref()
                .ok_or_else(missing_structure)?
                .interf_ff_records()[current]
                .points()[point_index]
                .pnt();
            if self.is_existing_vertex(point, tolerance, &state.shared.vertices_on_in)? {
                continue;
            }
            let mut vertex = Vertex::default();
            make_new_vertex_point(point, tolerance, &mut vertex)?;
            let mut couple = CoupleOfPaveBlocks::new();
            couple.set_interference_index(index as i32);
            couple.set_index(point_index as i32);
            state.sections.couples.add(vertex.into_shape(), couple);
        }
        Ok(())
    }

    fn put_vertices_on_curves(
        &mut self,
        current: usize,
        shapes: &[Face; 2],
        faces: (i32, i32),
        curve_count: usize,
        state: &mut MakeBlocksState,
    ) -> Result<(), Failure> {
        self.get_stick_vertices(
            faces,
            &mut state.vertices_stick,
            &mut state.vertices_ef,
            &mut state.shapes,
        )?;
        for curve in 0..curve_count {
            self.ff_curve_mut((current, curve))?.init_pave_block1();
            // Vertices not common to both faces are filtered by boxes before being put, to
            // avoid extending tolerances (see the native bug26789_1 comment).
            self.put_paves_on_curve(
                &state.shared.vertices_on_in,
                &state.shared.vertices_common,
                (current, curve),
                &state.shapes,
                &state.vertices_ef,
                &mut state.tolerances,
                &mut state.groups,
            )?;
        }
        // A vertex put on a curve through a large EF range is removed from curves it misses.
        self.filter_paves_on_curves(current, &state.tolerances)?;
        for curve in 0..curve_count {
            self.put_curve_end_vertices((current, curve), shapes, curve_count, state)?;
        }
        for curve in 0..curve_count {
            self.put_closing_pave_on_curve((current, curve))?;
        }
        Ok(())
    }

    fn put_curve_end_vertices(
        &mut self,
        curve: CurveRef,
        shapes: &[Face; 2],
        curve_count: usize,
        state: &mut MakeBlocksState,
    ) -> Result<(), Failure> {
        self.put_stick_paves_on_curve(
            shapes,
            &state.shapes,
            curve,
            &state.vertices_stick,
            &mut state.tolerances,
            &mut state.groups,
        )?;
        if curve_count == 1 {
            self.put_ef_paves_on_curve(
                curve,
                &state.shapes,
                &state.vertices_ef,
                &mut state.tolerances,
                &mut state.groups,
            )?;
        }
        if !self.ff_curve(curve)?.curve().has_bounds() {
            return Ok(());
        }
        let created = self.put_bound_pave_on_curve(shapes, curve)?;
        if created.is_empty() {
            return Ok(());
        }
        for vertex in &created {
            state.vertices_bounds.add(*vertex);
        }
        state.bound_vertices.bind(curve.1 as i32, created);
        Ok(())
    }

    /// A box tree over the On/In blocks with an edge on a non-degenerated original edge.
    fn on_in_block_tree(
        &self,
        on_in: &crate::bop_ds::PaveBlockIndexMap,
    ) -> Result<BlockTree, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let mut tree = BlockTree::default();
        tree.set_size(on_in.extent());
        for (position, block) in on_in.iter().enumerate() {
            let (has_edge, edge, original) = {
                let locked = block.lock().expect("pave-block lock");
                (locked.has_edge(), locked.edge(), locked.original_edge())
            };
            if !has_edge || structure.shape_info(original)?.has_flag() {
                continue;
            }
            tree.add(
                position as i32 + 1,
                box3_to_bvh(structure.shape_info(edge)?.box_bounds())?,
            );
        }
        tree.build()?;
        Ok(tree)
    }

    /// Gives the vertices whose tolerance was raised for an unused pave their saved tolerance
    /// back and forgets their same-domain groups.
    fn restore_unused_tolerances(&mut self, state: &mut MakeBlocksState) -> Result<(), Failure> {
        let saved: Vec<(i32, f64)> = state
            .tolerances
            .items()
            .map(|(vertex, tolerance)| (*vertex, *tolerance))
            .collect();
        for (vertex, tolerance) in saved {
            let shape = self.vertex_shape(vertex)?;
            Self::set_vertex_tolerance(&shape, tolerance)?;
            let bounds = self
                .structure_mut()?
                .change_shape_info(vertex)?
                .change_box();
            *bounds = BoundingBox3::default();
            BRepBndLib::add(&shape, bounds, true)?;
            bounds.set_gap(bounds.gap() + precision::confusion());
            if state.groups.is_bound(&vertex) {
                state.groups.unbind(&vertex);
            }
        }
        Ok(())
    }

    /// Makes one same-domain vertex per group of coinciding curve vertices (`MakeSDVerticesFF`).
    fn make_sd_vertices_ff(
        &mut self,
        groups: &VertexGroups,
        new_sd: &mut DataMap<i32, i32, DefaultHashPolicy>,
    ) -> Result<(), Failure> {
        let lists: Vec<Vec<i32>> = groups.values().cloned().collect();
        for list in lists {
            let merged = self.make_sd_vertices(&list, false)?;
            for vertex in list {
                new_sd.bind(vertex, merged);
            }
        }
        Ok(())
    }

    /// Raises the saved tolerances of the edge's vertices to the new edge tolerance
    /// (native static `UpdateSavedTolerance`).
    fn update_saved_tolerance(
        &self,
        edge: i32,
        tolerance: f64,
        saved: &mut VertexTolerances,
    ) -> Result<(), Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        for vertex in structure.shape_info(edge)?.sub_shapes() {
            if let Some(value) = saved.change_seek(vertex)
                && *value < tolerance
            {
                *value = tolerance;
            }
        }
        Ok(())
    }

    fn ff_count(&self) -> Result<usize, Failure> {
        Ok(self
            .data_structure
            .as_ref()
            .ok_or_else(missing_structure)?
            .interf_ff_records()
            .len())
    }

    fn ff_summary(&self, ff: usize) -> Result<((i32, i32), usize, usize), Failure> {
        let record = &self
            .data_structure
            .as_ref()
            .ok_or_else(missing_structure)?
            .interf_ff_records()[ff];
        Ok((
            record.indices(),
            record.curves().len(),
            record.points().len(),
        ))
    }

    pub(super) fn ff_curve(&self, (ff, curve): CurveRef) -> Result<&BopdsCurve, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        Ok(&structure.interf_ff_records()[ff].curves()[curve])
    }

    pub(super) fn ff_curve_mut(
        &mut self,
        (ff, curve): CurveRef,
    ) -> Result<&mut BopdsCurve, Failure> {
        Ok(&mut self.structure_mut()?.interf_ff()[ff].change_curves()[curve])
    }
}
