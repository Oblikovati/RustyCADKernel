//! Edge/face candidate preparation and generated-vertex bookkeeping.
#[path = "edge_face/candidates.rs"]
mod candidates;
#[path = "edge_face/common_blocks.rs"]
mod common_blocks;
#[path = "edge_face/vertices.rs"]
mod vertices;
use super::{BOPAlgoPaveFiller, Glue};
use crate::bop_algo::AlertIntersectionOfPairOfShapesFailed;
use crate::{
    bop_ds::{DataStructure, InterfEF, InterfVF, Pave, PaveBlockHandle, ShapeInfo},
    bop_tools::make_new_vertex_edge_face,
    int_tools::{CommonPrt, Context, EdgeFace, Range},
};
use candidates::{
    configured_edge_face, edge_face_candidates, initialize_glued_faces, selected_edge_face_pairs,
};
use common_blocks::{PendingCommonFace, apply_pending_common_faces, common_block_vertices};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Compound, Edge, Face, Vertex},
};
use rustycad_tkernel::{message::AlertHandle, ncollection::Map, standard::Failure};
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::sync::Arc;
use vertices::{append_pave, append_unique, fused_vertex, pending_vertex_groups};

#[derive(Clone)]
pub(super) struct EdgeFaceCandidate {
    edge_index: i32,
    face_index: i32,
    edge: Edge,
    face: Face,
    block: PaveBlockHandle,
    parameter_range: Range,
    shrunk_range: Range,
    express_compute: bool,
}

struct PendingEFVertex {
    candidate: EdgeFaceCandidate,
    common: CommonPrt,
    vertex: Vertex,
    parameter: f64,
}

#[derive(Default)]
struct PendingEFResults {
    edges: Vec<i32>,
    faces: Map<i32>,
    vertices: Vec<PendingEFVertex>,
    common_faces: Vec<PendingCommonFace>,
}

fn collect_edge_face_results(
    filler: &mut BOPAlgoPaveFiller,
    candidates: Vec<EdgeFaceCandidate>,
    results: &mut PendingEFResults,
) -> Result<(), Failure> {
    for candidate in candidates {
        filler.intersect_edge_face_candidate(candidate, results)?;
    }
    Ok(())
}

impl BOPAlgoPaveFiller {
    /// Intersects edge/face candidates and records supported vertex common parts.
    /// `filler.perform_ef()?` executes the prepared Edge/Face stage.
    pub fn perform_ef(&mut self) -> Result<(), Failure> {
        self.fill_shrunk_data(ShapeEnum::Edge, ShapeEnum::Face)?;
        let pairs = selected_edge_face_pairs(self)?;
        if self.glue == Glue::Full {
            return initialize_glued_faces(self, &pairs);
        }
        let candidates = edge_face_candidates(self, &pairs)?;
        self.intersect_edge_face_candidates(candidates)
    }

    fn intersect_edge_face_candidates(
        &mut self,
        candidates: Vec<EdgeFaceCandidate>,
    ) -> Result<(), Failure> {
        let mut results = PendingEFResults::default();
        collect_edge_face_results(self, candidates, &mut results)?;
        self.apply_common_faces(&results.common_faces)?;
        self.update_vertices_of_common_blocks()?;
        self.commit_pending_vertices(results.vertices, &mut results.edges, &mut results.faces)?;
        self.split_intersected_edges(&results.edges)?;
        self.structure_mut()?
            .update_face_info_in_map(&results.faces)
    }

    fn intersect_edge_face_candidate(
        &mut self,
        candidate: EdgeFaceCandidate,
        results: &mut PendingEFResults,
    ) -> Result<(), Failure> {
        let Some(solver) = self.try_edge_face_solver(&candidate)? else {
            return Ok(());
        };
        self.process_edge_face_parts(&candidate, &solver, results)
    }

    fn try_edge_face_solver(
        &self,
        candidate: &EdgeFaceCandidate,
    ) -> Result<Option<EdgeFace>, Failure> {
        let mut solver = configured_edge_face(self, candidate);
        if let Err(error) = solver.perform() {
            if error.exception_type() == "Standard_NotImplemented" {
                return Err(error);
            }
            self.warn_edge_face(candidate)?;
            return Ok(None);
        }
        if !solver.is_done() {
            self.warn_edge_face(candidate)?;
            return Ok(None);
        }
        Ok(Some(solver))
    }

    fn process_edge_face_parts(
        &mut self,
        candidate: &EdgeFaceCandidate,
        solver: &EdgeFace,
        results: &mut PendingEFResults,
    ) -> Result<(), Failure> {
        for common in solver.common_parts().iter() {
            self.accept_edge_face_common(candidate, common, results)?;
        }
        Ok(())
    }

    fn commit_pending_vertices(
        &mut self,
        pending: Vec<PendingEFVertex>,
        changed_edges: &mut Vec<i32>,
        changed_faces: &mut Map<i32>,
    ) -> Result<(), Failure> {
        let groups = pending_vertex_groups(&pending, self.options.fuzzy_value())?;
        for group in groups {
            self.commit_vertex_group(&pending, &group, changed_edges, changed_faces)?;
        }
        Ok(())
    }

    fn commit_vertex_group(
        &mut self,
        pending: &[PendingEFVertex],
        group: &[usize],
        changed_edges: &mut Vec<i32>,
        changed_faces: &mut Map<i32>,
    ) -> Result<(), Failure> {
        let vertex = fused_vertex(pending, group)?;
        let index = self.append_generated_vertex(&vertex)?;
        for event in group {
            self.commit_vertex_event(&pending[*event], index, changed_edges, changed_faces);
        }
        Ok(())
    }

    fn commit_vertex_event(
        &mut self,
        event: &PendingEFVertex,
        vertex: i32,
        changed_edges: &mut Vec<i32>,
        changed_faces: &mut Map<i32>,
    ) {
        self.append_edge_face_record(&event.candidate, &event.common, vertex);
        append_pave(&event.candidate.block, Pave::new(vertex, event.parameter));
        append_unique(changed_edges, event.candidate.edge_index);
        changed_faces.add(event.candidate.face_index);
    }

    fn warn_edge_face(&self, candidate: &EdgeFaceCandidate) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let mut compound = Compound::default();
        builder.make_compound(&mut compound);
        let mut shape = compound.into_shape();
        builder.add(&mut shape, &candidate.edge.clone().into_shape())?;
        builder.add(&mut shape, &candidate.face.clone().into_shape())?;
        let alert: AlertHandle = Arc::new(AlertIntersectionOfPairOfShapesFailed::new(&shape));
        self.options.add_warning(&alert);
        Ok(())
    }

    fn accept_edge_face_common(
        &mut self,
        candidate: &EdgeFaceCandidate,
        common: &CommonPrt,
        results: &mut PendingEFResults,
    ) -> Result<(), Failure> {
        match common.shape_type() {
            ShapeEnum::Vertex => self.accept_vertex_common(candidate, common, results),
            ShapeEnum::Edge => self.accept_edge_common(candidate, common, results),
            shape_type => Err(unsupported_common_part(shape_type)),
        }
    }

    fn accept_vertex_common(
        &mut self,
        candidate: &EdgeFaceCandidate,
        common: &CommonPrt,
        results: &mut PendingEFResults,
    ) -> Result<(), Failure> {
        if self.process_endpoint_common(candidate, common)? {
            return Ok(());
        }
        self.process_interior_vertex_common(candidate, common, results)
    }

    fn process_interior_vertex_common(
        &mut self,
        candidate: &EdgeFaceCandidate,
        common: &CommonPrt,
        results: &mut PendingEFResults,
    ) -> Result<(), Failure> {
        let parameter = common.vertex_parameter1();
        let Some(vertex) = self.make_edge_face_vertex(candidate, parameter)? else {
            return Ok(());
        };
        results.vertices.push(PendingEFVertex {
            candidate: candidate.clone(),
            common: common.clone(),
            vertex,
            parameter,
        });
        results.faces.add(candidate.face_index);
        Ok(())
    }

    fn process_endpoint_common(
        &mut self,
        candidate: &EdgeFaceCandidate,
        common: &CommonPrt,
    ) -> Result<bool, Failure> {
        let Some(endpoint) = common_block_endpoint(candidate, common) else {
            return Ok(false);
        };
        self.force_interference_vertex_face(endpoint, candidate.face_index)?;
        Ok(true)
    }

    fn make_edge_face_vertex(
        &mut self,
        candidate: &EdgeFaceCandidate,
        parameter: f64,
    ) -> Result<Option<Vertex>, Failure> {
        let mut vertex = Vertex::default();
        make_new_vertex_edge_face(&candidate.edge, parameter, &candidate.face, &mut vertex)?;
        if !self.vertex_is_on_face(&vertex, &candidate.face)? {
            return Ok(None);
        }
        Ok(Some(vertex))
    }

    fn accept_edge_common(
        &mut self,
        candidate: &EdgeFaceCandidate,
        common: &CommonPrt,
        results: &mut PendingEFResults,
    ) -> Result<(), Failure> {
        let endpoints_on_face = self.block_endpoints_on_face(candidate)?;
        let mut record = InterfEF::default();
        record.set_indices(candidate.edge_index, candidate.face_index);
        if endpoints_on_face {
            record.set_common_part(common);
            results.common_faces.push(PendingCommonFace::new(
                Arc::clone(&candidate.block),
                candidate.face_index,
            ));
        }
        self.append_interference_record(record, candidate.edge_index, candidate.face_index);
        results.faces.add(candidate.face_index);
        Ok(())
    }

    fn block_endpoints_on_face(&self, candidate: &EdgeFaceCandidate) -> Result<bool, Failure> {
        let block = candidate
            .block
            .lock()
            .expect("pave-block lock must not be poisoned");
        let (first, last) = block.indices();
        drop(block);
        Ok(self.face_has_vertex(candidate.face_index, first)?
            && self.face_has_vertex(candidate.face_index, last)?)
    }

    fn apply_common_faces(&mut self, pending: &[PendingCommonFace]) -> Result<(), Failure> {
        let context = self.context.clone();
        let structure = self.structure_mut()?;
        apply_pending_common_faces(
            structure,
            pending,
            context.get().expect("initialized EF context"),
        )
    }

    fn update_vertices_of_common_blocks(&mut self) -> Result<(), Failure> {
        let vertices = common_block_vertices(self.structure_mut()?);
        for (vertex, tolerance) in vertices {
            let _ = self.update_vertex(vertex, tolerance)?;
        }
        Ok(())
    }

    fn append_interference_record(&mut self, record: InterfEF, edge: i32, face: i32) {
        let ds = self.data_structure.as_mut().expect("prepared EF DS");
        ds.interf_ef().append(record);
        ds.add_interference(edge, face);
    }

    fn vertex_is_on_face(&self, vertex: &Vertex, face: &Face) -> Result<bool, Failure> {
        let point = BRepTool::pnt(vertex)?;
        let tolerance = BRepTool::vertex_tolerance(vertex)?;
        self.context
            .get()
            .expect("initialized EF context")
            .is_valid_point_for_face(point, face, tolerance)
    }

    fn force_interference_vertex_face(&mut self, vertex: i32, face: i32) -> Result<bool, Failure> {
        if self.face_has_vertex(face, vertex)? {
            return Ok(true);
        }
        let (vertex_shape, face_shape) = self.vertex_face_shapes(vertex, face)?;
        let Some((u, v, tolerance)) =
            self.compute_forced_vertex_face(&vertex_shape, &face_shape)?
        else {
            return Ok(false);
        };
        self.record_forced_vertex_face(vertex, face, u, v, tolerance)?;
        Ok(true)
    }

    fn face_has_vertex(&self, face: i32, vertex: i32) -> Result<bool, Failure> {
        let ds = self
            .data_structure
            .as_ref()
            .ok_or_else(missing_ef_structure)?;
        let info = ds.face_info(face)?;
        Ok(info.vertices_in().contains(vertex) || info.vertices_on().contains(vertex))
    }

    fn vertex_face_shapes(&self, vertex: i32, face: i32) -> Result<(Vertex, Face), Failure> {
        let ds = self
            .data_structure
            .as_ref()
            .ok_or_else(missing_ef_structure)?;
        Ok((
            Vertex::unchecked(ds.shape(vertex)?.clone()),
            Face::unchecked(ds.shape(face)?.clone()),
        ))
    }

    fn compute_forced_vertex_face(
        &self,
        vertex: &Vertex,
        face: &Face,
    ) -> Result<Option<(f64, f64, f64)>, Failure> {
        let context = self.context.get().expect("initialized EF context");
        compute_forced_vf(context, vertex, face, self.options.fuzzy_value())
    }

    fn record_forced_vertex_face(
        &mut self,
        vertex: i32,
        face: i32,
        u: f64,
        v: f64,
        tolerance: f64,
    ) -> Result<(), Failure> {
        let new_index = self.update_vertex(vertex, tolerance)?;
        let ds = self.structure_mut()?;
        append_forced_vf_record(ds, vertex, face, u, v, new_index);
        ds.change_face_info(face)?
            .change_vertices_in()
            .add(new_index);
        Ok(())
    }

    fn append_generated_vertex(&mut self, vertex: &Vertex) -> Result<i32, Failure> {
        let tolerance = BRepTool::vertex_tolerance(vertex)? + self.options.fuzzy_value() * 0.5;
        let point = BRepTool::pnt(vertex)?;
        let mut bounds = rustycad_tkmath::bnd::BoundingBox3::default();
        bounds.add_point(point);
        bounds.set_gap(tolerance);
        let mut info = ShapeInfo::default();
        info.set_shape_type(ShapeEnum::Vertex);
        info.set_shape(&vertex.clone().into_shape());
        info.set_box(&bounds);
        Ok(self.structure_mut()?.append_shape_info(&info))
    }

    fn append_edge_face_record(
        &mut self,
        candidate: &EdgeFaceCandidate,
        common: &CommonPrt,
        vertex: i32,
    ) {
        let mut record = InterfEF::default();
        record.set_indices(candidate.edge_index, candidate.face_index);
        record.set_common_part(common);
        record.set_index_new(vertex);
        let structure = self.data_structure.as_mut().expect("prepared EF DS");
        structure.interf_ef().append(record);
        structure.add_interference(candidate.edge_index, candidate.face_index);
    }

    pub(super) fn structure_mut(&mut self) -> Result<&mut DataStructure, Failure> {
        self.data_structure.as_mut().ok_or_else(|| {
            Failure::new("BOPAlgo_PaveFiller data structure is None; expected initialize() to prepare source shapes")
        })
    }
}

fn common_block_endpoint(candidate: &EdgeFaceCandidate, common: &CommonPrt) -> Option<i32> {
    let (first, last) = candidate.parameter_range.range();
    let shrunk_first = candidate.shrunk_range.first();
    let shrunk_last = candidate.shrunk_range.last();
    let parameter = common.vertex_parameter1();
    let locked = candidate
        .block
        .lock()
        .expect("pave-block lock must not be poisoned");
    let (first_vertex, last_vertex) = locked.indices();
    if parameter <= shrunk_first.max(first) + 5.0e-8 {
        return Some(first_vertex);
    }
    if parameter >= shrunk_last.min(last) - 5.0e-8 {
        return Some(last_vertex);
    }
    None
}

fn append_forced_vf_record(
    ds: &mut DataStructure,
    vertex: i32,
    face: i32,
    u: f64,
    v: f64,
    new_index: i32,
) {
    let mut record = InterfVF::default();
    record.set_indices(vertex, face);
    record.set_uv(u, v);
    if ds.is_new_shape(new_index) {
        record.set_index_new(new_index);
    }
    ds.interf_vf().append(record);
    ds.add_interference(vertex, face);
}

fn compute_forced_vf(
    context: &Context,
    vertex: &Vertex,
    face: &Face,
    fuzzy: f64,
) -> Result<Option<(f64, f64, f64)>, Failure> {
    let (mut u, mut v, mut tolerance) = (0.0, 0.0, 0.0);
    let status = context.compute_vf(vertex, face, &mut u, &mut v, &mut tolerance, fuzzy)?;
    Ok(matches!(status, 0 | -2).then_some((u, v, tolerance)))
}

fn missing_ef_structure() -> Failure {
    Failure::new(
        "BOPAlgo_PaveFiller data structure is None; expected initialize() to prepare source shapes",
    )
}

fn unsupported_common_part(shape_type: ShapeEnum) -> Failure {
    Failure::new(&format!(
        "BOPAlgo_PaveFiller::PerformEF received {shape_type:?}; expected vertex common parts"
    ))
}
