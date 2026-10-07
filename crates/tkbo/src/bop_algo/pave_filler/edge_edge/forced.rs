//! `BOPAlgo_PaveFiller_3.cxx` forced intersections: ForceInterfVE, UpdateVerticesOfCB and
//! ForceInterfEE (common blocks among pave blocks sharing both vertices).
use super::super::vertex_vertex::pair_compound;
use super::super::{BOPAlgoPaveFiller, missing_structure};
use super::EdgeEdgeTask;
use crate::bop_algo::tools::{PaveBlockLinks, fill_map, pave_block_links, perform_common_blocks};
use crate::bop_algo::{AlertAcquiredSelfIntersection, AlertSelfInterferingShape};
use crate::bop_ds::{InterfEE, InterfVE, Pair, Pave, PaveBlockHandle, PaveBlockHandleIdentity};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::message::AlertHandle;
use rustycad_tkernel::ncollection::{DefaultHashPolicy, IndexedDataMap, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{self, Point3, Vector3};
use std::sync::Arc;

type BlocksByBounds = IndexedDataMap<Pair, Vec<PaveBlockHandle>, DefaultHashPolicy>;

impl BOPAlgoPaveFiller {
    /// Puts vertex `vertex` on pave block `block` when ComputeVE accepts it (status 0 or -4),
    /// recording a VE interference; true also when they already interfere or share topology.
    /// `filler.force_interf_ve(v, &block, &mut edges)?`.
    pub(in crate::bop_algo::pave_filler) fn force_interf_ve(
        &mut self,
        vertex: i32,
        block: &PaveBlockHandle,
        edges: &mut Map<i32>,
    ) -> Result<bool, Failure> {
        let (edge, bounds) = {
            let locked = block.lock().expect("pave-block lock must not be poisoned");
            (locked.original_edge(), locked.indices())
        };
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        if structure.shape_info(edge)?.has_sub_shape(vertex)
            || structure.has_interference_between(vertex, edge)
            || structure.has_interference_shape_subshapes(vertex, edge)?
            || bounds.0 == vertex
            || bounds.1 == vertex
        {
            return Ok(true);
        }
        let mut representative = vertex;
        structure.has_shape_sd_into(vertex, &mut representative);
        let vertex_shape = Vertex::unchecked(structure.shape(representative)?.clone());
        let edge_shape = Edge::unchecked(structure.shape(edge)?.clone());
        let (mut parameter, mut tolerance) = (0., 0.);
        let context = self.context.get().ok_or_else(missing_structure)?;
        let fuzzy = self.options.fuzzy_value();
        let status = context.compute_ve(
            &vertex_shape,
            &edge_shape,
            &mut parameter,
            &mut tolerance,
            fuzzy,
        )?;
        if status != 0 && status != -4 {
            return Ok(false);
        }
        let structure = self.structure_mut()?;
        let position = structure.interf_ve_records().len();
        let record = structure.interf_ve().append(InterfVE::default());
        record.set_indices(vertex, edge);
        record.set_parameter(parameter);
        structure.add_interference(vertex, edge);
        let updated = self.update_vertex(vertex, tolerance)?;
        let structure = self.structure_mut()?;
        if structure.is_new_shape(updated) {
            structure.interf_ve()[position].set_index_new(updated);
        }
        block
            .lock()
            .expect("pave-block lock must not be poisoned")
            .append_ext_pave(Pave::new(updated, parameter));
        edges.add(edge);
        let rank = structure.rank(vertex);
        if rank >= 0 && rank == structure.rank(edge) {
            let pair = pair_compound(&vertex_shape.into_shape(), &edge_shape.into_shape())?;
            let alert: AlertHandle = Arc::new(AlertSelfInterferingShape::new(&pair));
            self.options.add_warning(&alert);
        }
        Ok(true)
    }

    /// Grows the end vertices of every common block to its computed tolerance.
    /// `filler.update_vertices_of_cb()?`.
    pub fn update_vertices_of_cb(&mut self) -> Result<(), Failure> {
        let mut fence: Map<PaveBlockHandle, PaveBlockHandleIdentity> =
            Map::with_hasher(PaveBlockHandleIdentity, 1);
        let mut updates: Vec<(i32, i32, f64)> = Vec::new();
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        for list in structure.pave_blocks_pool() {
            for block in list {
                let Some(common) = structure.common_block(block) else {
                    continue;
                };
                let (real, tolerance) = {
                    let locked = common.lock().expect("common-block lock");
                    (Arc::clone(locked.pave_block1()?), locked.tolerance())
                };
                if !fence.add(Arc::clone(&real)) {
                    continue;
                }
                if tolerance > 0. {
                    let (first, last) = real.lock().expect("pave-block lock").indices();
                    updates.push((first, last, tolerance));
                }
            }
        }
        // The pool is not modified by UpdateVertex, so the updates are applied in pool order.
        for (first, last, tolerance) in updates {
            self.update_vertex(first, tolerance)?;
            self.update_vertex(last, tolerance)?;
        }
        Ok(())
    }

    /// After vertex tolerances grew, finds new common blocks among pave blocks with the same
    /// bounding vertices that come from different arguments (`ForceInterfEE`).
    /// `filler.force_interf_ee()?`.
    pub fn force_interf_ee(&mut self) -> Result<(), Failure> {
        let structure = self.structure_mut()?;
        for index in 0..structure.nb_source_shapes() {
            let info = structure.shape_info(index)?;
            if info.shape_type() == ShapeEnum::Vertex && structure.has_interference_for(index) {
                structure.init_pave_blocks_for_vertex(index)?;
            }
        }
        let by_bounds = self.real_blocks_by_bounds()?;
        if by_bounds.extent() == 0 {
            return Ok(());
        }
        let mut tasks = Vec::new();
        for position in 1..=by_bounds.extent() {
            let blocks = by_bounds.find_from_index(position)?;
            if blocks.len() < 2 {
                continue;
            }
            let bounds = by_bounds.find_key(position)?.indices();
            self.append_forced_ee_tasks(bounds, blocks, &mut tasks)?;
        }
        if tasks.is_empty() {
            return Ok(());
        }
        for task in &mut tasks {
            task.perform();
        }
        let mut links = pave_block_links();
        for task in &tasks {
            self.analyze_forced_ee_task(task, &mut links)?;
        }
        perform_common_blocks(&links, self.structure_mut()?, None)
    }

    fn real_blocks_by_bounds(&self) -> Result<BlocksByBounds, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let mut by_bounds = BlocksByBounds::with_hasher(DefaultHashPolicy, 1);
        let mut fence: Map<PaveBlockHandle, PaveBlockHandleIdentity> =
            Map::with_hasher(PaveBlockHandleIdentity, 1);
        for index in 0..structure.nb_source_shapes() {
            let info = structure.shape_info(index)?;
            if info.shape_type() != ShapeEnum::Edge || !info.has_reference() || info.has_flag() {
                continue;
            }
            for block in structure.pave_blocks(index)? {
                let real = structure.real_pave_block(block)?;
                if !fence.add(Arc::clone(&real)) {
                    continue;
                }
                let (first, last) = real.lock().expect("pave-block lock").indices();
                let key = Pair::new(first, last);
                if let Some(list) = by_bounds.change_seek(&key) {
                    list.push(real);
                } else {
                    by_bounds.add(key, vec![real]);
                }
            }
        }
        Ok(by_bounds)
    }

    fn append_forced_ee_tasks(
        &self,
        (first_vertex, last_vertex): (i32, i32),
        blocks: &[PaveBlockHandle],
        tasks: &mut Vec<EdgeEdgeTask>,
    ) -> Result<(), Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let fuzzy = self.options.fuzzy_value();
        let first = Vertex::unchecked(structure.shape(first_vertex)?.clone());
        let last = Vertex::unchecked(structure.shape(last_vertex)?.clone());
        let check_mode = self.arguments.extent() == 1;
        let additional = if check_mode {
            fuzzy
        } else {
            2. * BRepTool::vertex_tolerance(&first)?.max(BRepTool::vertex_tolerance(&last)?)
        };
        for (position, block1) in blocks.iter().enumerate() {
            let Some(probe) = self.middle_tangent(block1)? else {
                continue;
            };
            for block2 in &blocks[position + 1..] {
                if !self.forced_pair_allowed(block1, block2, (first_vertex, last_vertex))? {
                    continue;
                }
                let Some(use_additional) = self.tangents_parallel(&probe, block2)? else {
                    continue;
                };
                let mut task = self.forced_ee_task(block1, block2)?;
                let fuzzy_value = if use_additional {
                    fuzzy + additional
                } else {
                    fuzzy
                };
                let boxes = task.boxes_from(structure)?;
                task.configure(boxes, fuzzy_value, true);
                tasks.push(task);
            }
        }
        Ok(())
    }

    /// The middle point, unit tangent and curve type of the first block, or `None` for a
    /// degenerate tangent.
    fn middle_tangent(&self, block: &PaveBlockHandle) -> Result<Option<MiddleTangent>, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let (edge, range) = {
            let locked = block.lock().expect("pave-block lock");
            (locked.original_edge(), locked.range())
        };
        let curve = BRepAdaptorCurve::with_edge(&Edge::unchecked(structure.shape(edge)?.clone()))?;
        let (point, mut tangent) = curve.d1((range.0 + range.1) * 0.5)?;
        if tangent.square_magnitude() < gp::resolution() {
            return Ok(None);
        }
        tangent.normalize()?;
        Ok(Some(MiddleTangent {
            point,
            tangent,
            curve_type: curve.get_type()?,
        }))
    }

    /// Checks the argument/vertex origin and existing common block rules of ForceInterfEE.
    fn forced_pair_allowed(
        &self,
        block1: &PaveBlockHandle,
        block2: &PaveBlockHandle,
        (first_vertex, last_vertex): (i32, i32),
    ) -> Result<bool, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let rank1 = structure.rank(block1.lock().expect("pave-block lock").original_edge());
        let rank2 = structure.rank(block2.lock().expect("pave-block lock").original_edge());
        if rank1 == rank2 {
            // Vertices shared originally (not during the operation) exclude the pair.
            let original_first =
                !structure.is_new_shape(first_vertex) && structure.rank(first_vertex) == rank1;
            let original_last =
                !structure.is_new_shape(last_vertex) && structure.rank(last_vertex) == rank2;
            if original_first || original_last {
                return Ok(false);
            }
        }
        let (common1, common2) = (
            structure.common_block(block1),
            structure.common_block(block2),
        );
        if let (Some(common1), Some(common2)) = (common1, common2)
            && Arc::ptr_eq(&common1, &common2)
        {
            return Ok(false);
        }
        Ok(true)
    }

    /// `Some(true)` when the extra tolerance may be used, `Some(false)` when the tangents make
    /// more than about 25 degrees, `None` when the pair must be skipped.
    fn tangents_parallel(
        &self,
        probe: &MiddleTangent,
        block2: &PaveBlockHandle,
    ) -> Result<Option<bool>, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let edge = Edge::unchecked(
            structure
                .shape(block2.lock().expect("pave-block lock").original_edge())?
                .clone(),
        );
        let curve = BRepAdaptorCurve::with_edge(&edge)?;
        if probe.curve_type == CurveType::Line && curve.get_type()? == CurveType::Line {
            return Ok(Some(true));
        }
        let context = self.context.get().ok_or_else(missing_structure)?;
        let projector = context.proj_pc(&edge)?;
        let parameter = {
            let mut projection = projector.write();
            projection.perform(probe.point)?;
            if projection.nb_points() == 0 {
                return Ok(None);
            }
            projection.lower_distance_parameter()?
        };
        let (_, tangent) = curve.d1(parameter)?;
        if tangent.square_magnitude() < gp::resolution() {
            return Ok(None);
        }
        let cosine = probe.tangent.dot(tangent.normalized()?);
        Ok(Some(cosine.abs() >= 0.9063))
    }

    fn forced_ee_task(
        &self,
        block1: &PaveBlockHandle,
        block2: &PaveBlockHandle,
    ) -> Result<EdgeEdgeTask, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let (edge1, range1) = edge_and_range(block1);
        let (edge2, range2) = edge_and_range(block2);
        let shape1 = Edge::unchecked(structure.shape(edge1)?.clone());
        let shape2 = Edge::unchecked(structure.shape(edge2)?.clone());
        Ok(EdgeEdgeTask::new(
            [Arc::clone(block1), Arc::clone(block2)],
            [shape1, shape2],
            [range1, range2],
        ))
    }

    fn analyze_forced_ee_task(
        &mut self,
        task: &EdgeEdgeTask,
        links: &mut PaveBlockLinks,
    ) -> Result<(), Failure> {
        let edges = (
            edge_and_range(&task.block1).0,
            edge_and_range(&task.block2).0,
        );
        if task.failed() {
            let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
            let first = structure.shape(edges.0)?.clone();
            let second = structure.shape(edges.1)?.clone();
            return self.add_intersection_failed_warning(&first, &second);
        }
        let [part] = task.parts.as_slice() else {
            return Ok(());
        };
        if part.shape_type() != ShapeEnum::Edge {
            return Ok(());
        }
        let structure = self.structure_mut()?;
        if structure.rank(edges.0) == structure.rank(edges.1) {
            let pair = pair_compound(structure.shape(edges.0)?, structure.shape(edges.1)?)?;
            let alert: AlertHandle = Arc::new(AlertAcquiredSelfIntersection::new(&pair));
            self.options.add_warning(&alert);
        }
        let structure = self.structure_mut()?;
        let record = structure.interf_ee().append(InterfEE::default());
        record.set_indices(edges.0, edges.1);
        record.set_common_part(part);
        structure.add_interference(edges.0, edges.1);
        for block in [&task.block1, &task.block2] {
            let Some(common) = structure.common_block(block) else {
                continue;
            };
            let members = common
                .lock()
                .expect("common-block lock")
                .pave_blocks()
                .to_vec();
            for member in members {
                fill_map(links, Arc::clone(block), member);
            }
        }
        fill_map(links, Arc::clone(&task.block1), Arc::clone(&task.block2));
        Ok(())
    }
}

/// Middle point data of the first pave block of a ForceInterfEE pair.
struct MiddleTangent {
    point: Point3,
    tangent: Vector3,
    curve_type: CurveType,
}

impl EdgeEdgeTask {
    /// The boxes of the two original edges, as ForceInterfEE passes them.
    fn boxes_from(
        &self,
        structure: &crate::bop_ds::DataStructure,
    ) -> Result<[rustycad_tkmath::bnd::BoundingBox3; 2], Failure> {
        let first = edge_and_range(&self.block1).0;
        let second = edge_and_range(&self.block2).0;
        Ok([
            structure.shape_info(first)?.box_bounds().clone(),
            structure.shape_info(second)?.box_bounds().clone(),
        ])
    }
}

fn edge_and_range(block: &PaveBlockHandle) -> (i32, (f64, f64)) {
    let locked = block.lock().expect("pave-block lock must not be poisoned");
    (locked.original_edge(), locked.range())
}
