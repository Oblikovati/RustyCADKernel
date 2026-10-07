//! `BOPAlgo_PaveFiller_5.cxx`: edge/face intersection (`PerformEF`) and its task class.
use super::edge_edge::{PaveBlockBoxes, VertexCouples, pave_block_box};
use super::{BOPAlgoPaveFiller, EdgeRangeDistance, Glue, missing_structure};
use crate::bop_algo::tools::{
    PaveBlockIndexLinks, fill_map_one, pave_block_index_links, perform_common_blocks_on_faces,
};
use crate::bop_ds::{CoupleOfPaveBlocks, InterfEF, Pair, PaveBlockHandle, VertexIndexSet};
use crate::bop_tools::{compute_vertices, correct_range_face, make_new_vertex_edge_face};
use crate::int_tools::{CommonPrt, Range, Tools};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::Map;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use std::sync::Arc;

#[path = "edge_face/forced.rs"]
mod forced;
#[path = "edge_face/task.rs"]
mod task;
pub(super) use task::EdgeFaceTask;

/// The state shared by the analysis of the EF tasks (native locals of PerformEF).
struct EdgeFaceResults {
    faces: Map<i32>,
    couples: VertexCouples,
    blocks_on_faces: PaveBlockIndexLinks,
}

impl BOPAlgoPaveFiller {
    /// Intersects the pave blocks of edges with the faces whose boxes they touch, making new
    /// vertices and edge/face common blocks (`PerformEF`); `filler.perform_ef()?`.
    pub fn perform_ef(&mut self) -> Result<(), Failure> {
        self.fill_shrunk_data(ShapeEnum::Edge, ShapeEnum::Face)?;
        let pairs = self.iterator_pairs(ShapeEnum::Edge, ShapeEnum::Face)?;
        if pairs.is_empty() {
            return Ok(());
        }
        if self.glue == Glue::Full {
            let structure = self.structure_mut()?;
            for (edge, face) in pairs {
                if !structure.shape_info(edge)?.has_flag() {
                    structure.change_face_info(face)?;
                }
            }
            return Ok(());
        }
        let mut boxes = PaveBlockBoxes::new();
        let mut tasks = Vec::new();
        for (edge, face) in pairs {
            self.append_edge_face_tasks(edge, face, &mut boxes, &mut tasks)?;
        }
        let context = self.context.clone();
        for task in &mut tasks {
            task.perform(&context);
        }
        let mut results = EdgeFaceResults {
            faces: Map::new(100),
            couples: VertexCouples::with_hasher(ShapeMapHasher, 100),
            blocks_on_faces: pave_block_index_links(),
        };
        for task in &tasks {
            self.analyze_edge_face_task(task, &mut results)?;
        }
        perform_common_blocks_on_faces(
            &results.blocks_on_faces,
            self.structure_mut()?,
            context.get(),
        )?;
        self.update_vertices_of_cb()?;
        self.perform_new_vertices(&mut results.couples, false)?;
        self.structure_mut()?
            .update_face_info_in_map(&results.faces)
    }

    fn append_edge_face_tasks(
        &mut self,
        edge: i32,
        face: i32,
        boxes: &mut PaveBlockBoxes,
        tasks: &mut Vec<EdgeFaceTask>,
    ) -> Result<(), Failure> {
        let fuzzy = self.options.fuzzy_value();
        let structure = self.structure_mut()?;
        if structure.shape_info(edge)?.has_flag() {
            return Ok(());
        }
        let edge_shape = Edge::unchecked(structure.shape(edge)?.clone());
        let face_shape = Face::unchecked(structure.shape(face)?.clone());
        let face_box = structure.shape_info(face)?.box_bounds().clone();
        let edge_box = structure.shape_info(edge)?.box_bounds().clone();
        structure.change_face_info(face)?;
        let blocks = structure.change_pave_blocks(edge)?.clone();
        for block in blocks {
            let structure = self.structure_mut()?;
            let real = structure.real_pave_block(&block)?;
            let info = structure.face_info(face)?;
            if info.pave_blocks_on().contains(&real) {
                continue;
            }
            let Some((range, block_box)) = pave_block_box(&edge_shape, &block, boxes)? else {
                continue;
            };
            if face_box.is_out_box(&block_box) {
                continue;
            }
            let (first, last) = real.lock().expect("pave-block lock").indices();
            let known = |vertex: i32| {
                info.vertices_in().contains(vertex) || info.vertices_on().contains(vertex)
            };
            let quick = known(first) && known(last);
            let shrunk = shrunk_or_whole(&block, range);
            let mut task = EdgeFaceTask::new((edge, face), &block, &edge_shape, &face_shape);
            let mut new_shrunk = shrunk;
            correct_range_face(&edge_shape, &face_shape, &shrunk, &mut new_shrunk)?;
            task.new_shrunk_range = new_shrunk;
            let mut block_range = Range::new(range.0, range.1);
            correct_range_face(
                &edge_shape,
                &face_shape,
                &Range::new(range.0, range.1),
                &mut block_range,
            )?;
            task.configure(
                [edge_box.clone(), face_box.clone()],
                fuzzy,
                quick,
                block_range,
            );
            tasks.push(task);
            self.mark_face_block_done(face, &block);
        }
        Ok(())
    }

    /// Saves the face/pave block pair to avoid its forced intersection later (`myFPBDone`).
    fn mark_face_block_done(&mut self, face: i32, block: &PaveBlockHandle) {
        let done = self.fpb_done.entry(face).or_default();
        if !done.iter().any(|known| Arc::ptr_eq(known, block)) {
            done.push(Arc::clone(block));
        }
    }

    fn analyze_edge_face_task(
        &mut self,
        task: &EdgeFaceTask,
        results: &mut EdgeFaceResults,
    ) -> Result<(), Failure> {
        if task.failed() {
            let (edge, face) = (
                task.edge.clone().into_shape(),
                task.face.clone().into_shape(),
            );
            return self.add_intersection_failed_warning(&edge, &face);
        }
        let edge_tolerance = BRepTool::edge_tolerance(&task.edge)?;
        let face_tolerance = BRepTool::face_tolerance(&task.face)?;
        if task.parts.is_empty() {
            if task.minimal_distance < f64::MAX
                && task.minimal_distance > edge_tolerance + face_tolerance
            {
                let range = task.block.lock().expect("pave-block lock").range();
                let distance = EdgeRangeDistance {
                    first: range.0,
                    last: range.1,
                    distance: task.minimal_distance,
                };
                self.record_distance(Pair::new(task.edge_index, task.face_index), distance);
            }
            return Ok(());
        }
        let (range, vertices, splittable) = {
            let locked = task.block.lock().expect("pave-block lock");
            (locked.range(), locked.indices(), locked.is_splittable())
        };
        let (mut shrunk_first, mut shrunk_last) = task.new_shrunk_range.range();
        if task.parts[0].shape_type() == ShapeEnum::Vertex {
            self.reduce_intersection_range(
                vertices,
                task.edge_index,
                task.face_index,
                &mut shrunk_first,
                &mut shrunk_last,
            )?;
        }
        let sides = [
            Range::new(range.0, shrunk_first),
            Range::new(shrunk_last, range.1),
        ];
        let line_plane = BRepAdaptorCurve::with_edge(&task.edge)?.get_type()? == CurveType::Line
            && self
                .context_ref()?
                .surface_adaptor(&task.face)?
                .read()
                .get_type()?
                == SurfaceType::Plane;
        let bounds = EfBounds {
            vertices,
            splittable,
            sides,
            line_plane,
            tolerances: (edge_tolerance, face_tolerance),
        };
        for part in &task.parts {
            match part.shape_type() {
                ShapeEnum::Vertex => self.analyze_ef_vertex(task, part, &bounds, results)?,
                ShapeEnum::Edge => self.analyze_ef_edge(task, part, vertices, results)?,
                _ => {}
            }
        }
        Ok(())
    }

    fn analyze_ef_vertex(
        &mut self,
        task: &EdgeFaceTask,
        part: &CommonPrt,
        bounds: &EfBounds,
        results: &mut EdgeFaceResults,
    ) -> Result<(), Failure> {
        let mut parameter = 0.;
        Tools::vertex_parameter(part, &mut parameter);
        let mut vertex = Vertex::default();
        make_new_vertex_edge_face(&task.edge, parameter, &task.face, &mut vertex)?;
        let common = *part.range1();
        let mut on_pave = [
            Tools::is_in_range(&bounds.sides[0], &common, 5.0e-8),
            Tools::is_in_range(&bounds.sides[1], &common, 5.0e-8),
        ];
        if (on_pave[0] && on_pave[1]) || (bounds.line_plane && (on_pave[0] || on_pave[1])) {
            let face = task.face_index;
            if self.face_has_vertex(face, bounds.vertices.0)?
                && self.face_has_vertex(face, bounds.vertices.1)?
            {
                let mut edge_part = part.clone();
                edge_part.set_type(ShapeEnum::Edge);
                self.append_ef_record(task, Some(&edge_part));
                results.faces.add(task.face_index);
                fill_map_one(
                    &mut results.blocks_on_faces,
                    Arc::clone(&task.block),
                    task.face_index,
                );
                return Ok(());
            }
        }
        if !bounds.splittable {
            return Ok(());
        }
        let ends = [bounds.vertices.0, bounds.vertices.1];
        // The face maps are read live: ForceInterfVF may add the first end to the In vertices.
        for (position, flag) in on_pave.iter_mut().enumerate() {
            if *flag && !self.face_has_vertex(task.face_index, ends[position])? {
                *flag = self.force_interf_vf(ends[position], task.face_index)?;
            }
        }
        if on_pave[0] || on_pave[1] {
            return self.extend_existing_ef_vertices(task, &vertex, on_pave, ends);
        }
        let (face_on, _) = self.face_vertex_sets(task.face_index)?;
        if self.vertex_near_face_vertices(&vertex, &face_on)? {
            return Ok(());
        }
        let mut new_tolerance = BRepTool::vertex_tolerance(&vertex)?;
        new_tolerance = new_tolerance.max(bounds.tolerances.0.max(bounds.tolerances.1));
        BRepBuilder::default().update_vertex_tolerance(&vertex, new_tolerance)?;
        if bounds.line_plane {
            // Line/Plane hits get half the common range, applied only at the next intersection.
            new_tolerance = new_tolerance.max((common.last() - common.first()) / 2.);
        }
        let point = BRepTool::pnt(&vertex)?;
        if !self
            .context_ref()?
            .is_point3d_in_face(point, &task.face, new_tolerance)?
        {
            return Ok(());
        }
        results.faces.add(task.face_index);
        let index = self.append_ef_record(task, Some(part));
        let mut couple = CoupleOfPaveBlocks::new();
        couple.set_pave_blocks(Some(Arc::clone(&task.block)), Some(Arc::clone(&task.block)));
        couple.set_interference_index(index);
        couple.set_tolerance(new_tolerance);
        results.couples.add(vertex.into_shape(), couple);
        Ok(())
    }

    /// The new point is close to a block end: grows that vertex when the point really lies on
    /// the face (within 1e4 of the vertex tolerance, at most 0.1 for small tolerances).
    fn extend_existing_ef_vertices(
        &mut self,
        task: &EdgeFaceTask,
        vertex: &Vertex,
        on_pave: [bool; 2],
        ends: [i32; 2],
    ) -> Result<(), Failure> {
        let point = BRepTool::pnt(vertex)?;
        let projector = self.context_ref()?.proj_ps(&task.face)?;
        let distance = {
            let mut projection = projector.write();
            projection.perform(point)?;
            if projection.is_done() && projection.nb_points()? > 0 {
                projection.lower_distance()?
            } else {
                precision::infinite()
            }
        };
        if distance >= precision::intersection() {
            return Ok(());
        }
        for (position, flag) in on_pave.iter().enumerate() {
            if !*flag {
                continue;
            }
            let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
            let existing = Vertex::unchecked(structure.shape(ends[position])?.clone());
            let gap = BRepTool::pnt(&existing)?.distance(&point);
            let tolerance = BRepTool::vertex_tolerance(&existing)?;
            let mut maximum = 1.0e4 * tolerance;
            if tolerance < 0.01 {
                maximum = maximum.min(0.1);
            }
            if gap < maximum {
                self.update_vertex(ends[position], gap)?;
                if !self.vertices_to_avoid_extension.contains(&ends[position]) {
                    self.vertices_to_avoid_extension.push(ends[position]);
                }
            }
        }
        Ok(())
    }

    fn analyze_ef_edge(
        &mut self,
        task: &EdgeFaceTask,
        part: &CommonPrt,
        vertices: (i32, i32),
        results: &mut EdgeFaceResults,
    ) -> Result<(), Failure> {
        results.faces.add(task.face_index);
        let face = task.face_index;
        if !self.face_has_vertex(face, vertices.0)? || !self.face_has_vertex(face, vertices.1)? {
            self.append_ef_record(task, None);
            return Ok(());
        }
        self.append_ef_record(task, Some(part));
        fill_map_one(
            &mut results.blocks_on_faces,
            Arc::clone(&task.block),
            task.face_index,
        );
        Ok(())
    }

    /// Native `CheckFacePaves(aVnew, aMIF)`: true when the new vertex interferes with one of
    /// the face's On vertices.
    fn vertex_near_face_vertices(
        &self,
        vertex: &Vertex,
        face_on: &VertexIndexSet,
    ) -> Result<bool, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        for index in face_on.iter().copied() {
            let other = Vertex::unchecked(structure.shape(index)?.clone());
            if compute_vertices(vertex, &other, 0.)? == 0 {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Native `CheckFacePaves(nV, aMIFOn, aMIFIn)` on the live face record.
    fn face_has_vertex(&mut self, face: i32, vertex: i32) -> Result<bool, Failure> {
        let info = self.structure_mut()?.change_face_info(face)?;
        Ok(info.vertices_on().contains(vertex) || info.vertices_in().contains(vertex))
    }

    fn face_vertex_sets(&mut self, face: i32) -> Result<(VertexIndexSet, VertexIndexSet), Failure> {
        let info = self.structure_mut()?.change_face_info(face)?;
        let mut on = VertexIndexSet::default();
        let mut inner = VertexIndexSet::default();
        for vertex in info.vertices_on().iter().copied() {
            on.add(vertex);
        }
        for vertex in info.vertices_in().iter().copied() {
            inner.add(vertex);
        }
        Ok((on, inner))
    }

    /// Appends an EF record (with its common part when given) and its interference pair.
    fn append_ef_record(&mut self, task: &EdgeFaceTask, part: Option<&CommonPrt>) -> i32 {
        let structure = self
            .data_structure
            .as_mut()
            .expect("PerformEF runs on an initialized DS");
        let record = structure.interf_ef().append(InterfEF::default());
        record.set_indices(task.edge_index, task.face_index);
        if let Some(part) = part {
            record.set_common_part(part);
        }
        let index = structure.interf_ef().len() as i32 - 1;
        structure.add_interference(task.edge_index, task.face_index);
        index
    }

    fn record_distance(&mut self, key: Pair, distance: EdgeRangeDistance) {
        if let Some(list) = self.distances.change_seek(&key) {
            list.push(distance);
            return;
        }
        self.distances.bind(key, vec![distance]);
    }

    pub(super) fn context_ref(&self) -> Result<&crate::int_tools::Context, Failure> {
        self.context.get().ok_or_else(missing_structure)
    }
}

/// Values of the pave block of an EF task used by every common part.
struct EfBounds {
    vertices: (i32, i32),
    splittable: bool,
    sides: [Range; 2],
    line_plane: bool,
    tolerances: (f64, f64),
}

fn shrunk_or_whole(block: &PaveBlockHandle, range: (f64, f64)) -> Range {
    let locked = block.lock().expect("pave-block lock must not be poisoned");
    if locked.has_shrunk_data() {
        let (first, last, _, _) = locked.shrunk_data();
        return Range::new(first, last);
    }
    Range::new(range.0, range.1)
}
