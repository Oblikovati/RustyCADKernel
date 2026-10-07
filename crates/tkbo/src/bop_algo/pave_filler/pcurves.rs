//! `BOPAlgo_PaveFiller::MakePCurves` and the `BOPAlgo_MPC` task of `BOPAlgo_PaveFiller_7.cxx`.
use super::{BOPAlgoPaveFiller, missing_structure};
use crate::bop_algo::AlertBuildingPCurveFailed;
use crate::bop_ds::{Pair, PaveBlockHandle};
use crate::bop_tools::algo_tools2d::{
    adjust_pcurve_on_surface, attach_existing_pcurve, build_pcurve_for_edge_on_face,
    has_curve_on_surface_simple,
};
use crate::bop_tools::{copy_edge, d_tolerance, make_split_edge};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::message::AlertHandle;
use rustycad_tkernel::ncollection::Map;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::Orientation;
use std::sync::Arc;

impl BOPAlgoPaveFiller {
    /// Builds the missing p-curves of In/On blocks and updates the vertices of section edges
    /// (`MakePCurves`); `filler.make_pcurves()?`.
    pub fn make_pcurves(&mut self) -> Result<(), Failure> {
        let attribute = self.section_attribute;
        if self.avoid_build_pcurve || (!attribute.pcurve_on_s1() && !attribute.pcurve_on_s2()) {
            return Ok(());
        }
        let mut tasks = self.face_block_pcurve_tasks()?;
        tasks.extend(
            self.section_pcurve_tasks([attribute.pcurve_on_s1(), attribute.pcurve_on_s2()])?,
        );
        for task in &mut tasks {
            self.perform_pcurve_task(task);
        }
        for task in tasks {
            self.store_pcurve(task)?;
        }
        Ok(())
    }
    fn face_block_pcurve_tasks(&self) -> Result<Vec<PCurveTask>, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let mut tasks = Vec::new();
        for info in structure.face_info_pool() {
            let face = Face::unchecked(
                structure
                    .shape(info.index())?
                    .oriented(Orientation::Forward),
            );
            for block in info.pave_blocks_in().iter() {
                let edge = block.lock().expect("pave-block lock").edge();
                if edge < 0 {
                    // Native: "Face information is not up to date" assertion, then skipped.
                    continue;
                }
                tasks.push(PCurveTask::new(
                    Edge::unchecked(structure.shape(edge)?.clone()),
                    face.clone(),
                ));
            }
            for block in info.pave_blocks_on().iter() {
                let edge = Edge::unchecked(
                    structure
                        .shape(block.lock().expect("pave-block lock").edge())?
                        .clone(),
                );
                if has_curve_on_surface_simple(&edge, &face)? {
                    continue;
                }
                let mut task = PCurveTask::new(edge, face.clone());
                task.source = self.coinciding_pcurve_source(block, &face)?;
                tasks.push(task);
            }
        }
        Ok(tasks)
    }
    /// The first other edge of the block's common block that has a p-curve on the face, with
    /// the block's bounds on it.
    fn coinciding_pcurve_source(
        &self,
        block: &PaveBlockHandle,
        face: &Face,
    ) -> Result<Option<PCurveSource>, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let Some(common) = structure.common_block(block) else {
            return Ok(None);
        };
        let members = common
            .lock()
            .expect("common-block lock")
            .pave_blocks()
            .to_vec();
        if members.len() < 2 {
            return Ok(None);
        }
        for member in members {
            if Arc::ptr_eq(&member, block) {
                continue;
            }
            let (edge, bounds, range) = {
                let locked = member.lock().expect("pave-block lock");
                (locked.original_edge(), locked.indices(), locked.range())
            };
            let edge = Edge::unchecked(structure.shape(edge)?.clone());
            if !has_curve_on_surface_simple(&edge, face)? {
                continue;
            }
            return Ok(Some((
                Edge::unchecked(edge.oriented(Orientation::Forward)),
                Vertex::unchecked(structure.shape(bounds.0)?.oriented(Orientation::Forward)),
                range.0,
                Vertex::unchecked(structure.shape(bounds.1)?.oriented(Orientation::Reversed)),
                range.1,
            )));
        }
        Ok(None)
    }
    fn section_pcurve_tasks(&self, sides: [bool; 2]) -> Result<Vec<PCurveTask>, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let mut pairs: Map<Pair> = Map::new(1);
        let mut tasks = Vec::new();
        for record in structure.interf_ff_records().iter() {
            if record.curves().is_empty() {
                continue;
            }
            let (first, second) = record.indices();
            let faces = [
                Face::unchecked(structure.shape(first)?.oriented(Orientation::Forward)),
                Face::unchecked(structure.shape(second)?.oriented(Orientation::Forward)),
            ];
            for curve in record.curves().iter() {
                for block in curve.pave_blocks() {
                    let edge = block.lock().expect("pave-block lock").edge();
                    let shape = Edge::unchecked(structure.shape(edge)?.clone());
                    for (side, face_index) in [first, second].into_iter().enumerate() {
                        if sides[side] && pairs.add(Pair::new(edge, face_index)) {
                            let mut task = PCurveTask::new(shape.clone(), faces[side].clone());
                            task.update_vertices = true;
                            tasks.push(task);
                        }
                    }
                }
            }
        }
        Ok(tasks)
    }
    /// Native `BOPAlgo_MPC::Perform`; exceptions mark the task as failed.
    fn perform_pcurve_task(&self, task: &mut PCurveTask) {
        if self.run_pcurve_task(task).is_err() {
            task.failed = true;
        }
    }
    fn run_pcurve_task(&self, task: &mut PCurveTask) -> Result<(), Failure> {
        let context = self.context_ref()?;
        let mut edge = task.edge.clone();
        let found = BRepTool::curve_on_face(&edge, &task.face)?;
        if found.pcurve.is_null() {
            // Work on a copy to avoid data races, as natively.
            edge = copy_edge(&edge)?;
            let mut status = 1;
            if let Some((source, v1, t1, v2, t2)) = &task.source {
                let mut split = Edge::default();
                make_split_edge(source, v1, *t1, v2, *t2, &mut split)?;
                status = attach_existing_pcurve(&split, &edge, &task.face, context)?;
            }
            if status != 0 {
                build_pcurve_for_edge_on_face(&edge, &task.face, Some(context))?;
            }
            task.pcurve = BRepTool::curve_on_face(&edge, &task.face)?.pcurve;
            if task.pcurve.is_null() {
                task.failed = true;
                return Ok(());
            }
            task.tolerance = BRepTool::edge_tolerance(&edge)?;
        } else {
            let surface = context.surface_adaptor(&task.face)?;
            let surface = surface.read();
            if surface.is_u_periodic()? || surface.is_v_periodic()? {
                let adjusted =
                    adjust_pcurve_on_surface(&surface, found.first, found.last, &found.pcurve)?;
                if adjusted != found.pcurve {
                    task.pcurve = adjusted;
                    task.tolerance = BRepTool::edge_tolerance(&edge)?;
                }
            }
        }
        if task.update_vertices {
            update_vertices(&edge, &task.face)?;
        }
        Ok(())
    }
    fn store_pcurve(&mut self, task: PCurveTask) -> Result<(), Failure> {
        if task.failed {
            let builder = BRepBuilder::default();
            let mut compound = rustycad_tkbrep::topods::Compound::default();
            builder.make_compound(&mut compound);
            let mut compound = compound.into_shape();
            builder.add(&mut compound, &task.edge.clone().into_shape())?;
            builder.add(&mut compound, &task.face.clone().into_shape())?;
            let alert: AlertHandle = Arc::new(AlertBuildingPCurveFailed::new(&compound));
            self.options.add_warning(&alert);
            return Ok(());
        }
        // A null p-curve means the edge was already valid and only its vertices changed.
        if !task.pcurve.is_null() {
            BRepBuilder::default().update_edge_pcurve_on_face(
                &task.edge,
                &task.pcurve,
                &task.face,
                task.tolerance,
            )?;
        }
        Ok(())
    }
}

/// Native static `UpdateVertices`: grows the end vertices of the edge to cover the gap
/// between the 3D curve and the p-curve ends.
fn update_vertices(edge: &Edge, face: &Face) -> Result<(), Failure> {
    let forward = Edge::unchecked(edge.oriented(Orientation::Forward));
    let (first, last) = topexp::vertices(&forward, false);
    let surface = BRepTool::surface(face)?;
    let (curve, t1, t2) = BRepTool::curve(&forward)?;
    let on_face = BRepTool::curve_on_face(&forward, face)?;
    let parameters = [on_face.first, on_face.last];
    let _ = (t1, t2);
    for (position, vertex) in [first, last].into_iter().enumerate() {
        let tolerance = BRepTool::vertex_tolerance(&vertex)?;
        let square_tolerance = tolerance * tolerance;
        let point = curve.eval_d0(parameters[position])?;
        let uv = on_face.pcurve.value(parameters[position])?;
        let on_surface = surface.value(uv.x(), uv.y())?;
        let square = point.square_distance(&on_surface);
        if square > square_tolerance {
            BRepBuilder::default()
                .update_vertex_tolerance(&vertex, square.sqrt() + d_tolerance())?;
        }
    }
    Ok(())
}

/// A coinciding edge with p-curve and the block bounds on it (`BOPAlgo_MPC::SetData`).
type PCurveSource = (Edge, Vertex, f64, Vertex, f64);

/// `BOPAlgo_MPC`: the p-curve of an edge on a face, optionally from a coinciding edge.
struct PCurveTask {
    edge: Edge,
    face: Face,
    source: Option<PCurveSource>,
    update_vertices: bool,
    pcurve: Handle<dyn Geom2dCurve>,
    tolerance: f64,
    failed: bool,
}

impl PCurveTask {
    fn new(edge: Edge, face: Face) -> Self {
        Self {
            edge,
            face,
            source: None,
            update_vertices: false,
            pcurve: Handle::default(),
            tolerance: 0.,
            failed: false,
        }
    }
}
