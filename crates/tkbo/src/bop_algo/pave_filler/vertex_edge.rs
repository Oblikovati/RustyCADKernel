use super::BOPAlgoPaveFiller;
use crate::bop_algo::{AlertIntersectionOfPairOfShapesFailed, Options};
use crate::bop_ds::{DataStructure, InterfVE, Pave, PaveBlockHandle, ShapeInfo, pave_block_handle};
use crate::int_tools::Context;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Compound, Edge, Shape, Vertex},
};
use rustycad_tkernel::{
    message::AlertHandle,
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::sync::Arc;

struct CandidateGroup {
    block: PaveBlockHandle,
    vertices: Vec<i32>,
}

struct SharedVertexCandidate {
    representative: i32,
    originals: Vec<i32>,
}

impl BOPAlgoPaveFiller {
    /// Performs the defined ordinary vertex/edge intersection path; `filler.perform_ve()?`.
    pub fn perform_ve(&mut self) -> Result<(), Failure> {
        self.fill_shrunk_data(ShapeEnum::Vertex, ShapeEnum::Edge)?;
        let groups = self.vertex_edge_candidates()?;
        let mut changed_edges = Vec::new();
        for group in groups {
            self.intersect_vertex_edge_group(&group, &mut changed_edges)?;
        }
        self.split_intersected_edges(&changed_edges)?;
        Ok(())
    }

    fn vertex_edge_candidates(&mut self) -> Result<Vec<CandidateGroup>, Failure> {
        let pairs = self.selected_vertex_edge_pairs()?;
        let structure = self.data_structure.as_ref().ok_or_else(|| {
            Failure::new("BOPAlgo_PaveFiller::PerformVE requires initialized data structure")
        })?;
        let mut groups = Vec::new();
        for (vertex, edge) in pairs {
            append_vertex_edge_candidate(structure, vertex, edge, &mut groups)?;
        }
        Ok(groups)
    }

    fn selected_vertex_edge_pairs(&mut self) -> Result<Vec<(i32, i32)>, Failure> {
        let iterator = self.iterator.as_mut().ok_or_else(|| {
            Failure::new("BOPAlgo_PaveFiller::PerformVE requires initialized iterator")
        })?;
        iterator.initialize(ShapeEnum::Vertex, ShapeEnum::Edge);
        let mut pairs = Vec::new();
        while iterator.more() {
            pairs.push(iterator.value()?);
            iterator.next();
        }
        Ok(pairs)
    }

    fn intersect_vertex_edge_group(
        &mut self,
        group: &CandidateGroup,
        changed_edges: &mut Vec<i32>,
    ) -> Result<(), Failure> {
        let block = &group.block;
        let edge_index = block
            .lock()
            .expect("pave-block lock must not be poisoned")
            .original_edge();
        let candidates = self.shared_vertex_candidates(group, edge_index)?;
        for candidate in candidates {
            self.compute_vertex_edge(&candidate, edge_index, block, changed_edges)?;
        }
        Ok(())
    }

    fn shared_vertex_candidates(
        &self,
        group: &CandidateGroup,
        edge: i32,
    ) -> Result<Vec<SharedVertexCandidate>, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(|| {
            Failure::new("BOPAlgo_PaveFiller::PerformVE requires initialized data structure")
        })?;
        let mut candidates: Vec<SharedVertexCandidate> = Vec::new();
        for original in &group.vertices {
            let representative = structure.shape_sd(*original).unwrap_or(*original);
            if edge_has_vertex_endpoint(structure, edge, representative)? {
                continue;
            }
            append_shared_vertex(&mut candidates, representative, *original);
        }
        Ok(candidates)
    }

    fn compute_vertex_edge(
        &mut self,
        candidate: &SharedVertexCandidate,
        edge_index: i32,
        block: &PaveBlockHandle,
        changed_edges: &mut Vec<i32>,
    ) -> Result<(), Failure> {
        let (vertex, edge, context) =
            self.vertex_edge_inputs(candidate.representative, edge_index)?;
        let Some((parameter, tolerance)) =
            compute_vertex_edge_hit(&self.options, &context, &vertex, &edge)?
        else {
            return Ok(());
        };
        self.accept_vertex_edge_hit(
            candidate,
            edge_index,
            block,
            parameter,
            tolerance,
            changed_edges,
        )
    }

    fn accept_vertex_edge_hit(
        &mut self,
        candidate: &SharedVertexCandidate,
        edge: i32,
        block: &PaveBlockHandle,
        parameter: f64,
        tolerance: f64,
        changed_edges: &mut Vec<i32>,
    ) -> Result<(), Failure> {
        let new_vertex = self.update_vertex(candidate.representative, tolerance)?;
        if !append_block_pave(block, new_vertex, parameter) {
            return Ok(());
        }
        self.append_vertex_edge_interferences(candidate, edge, new_vertex, parameter);
        append_unique(changed_edges, edge);
        Ok(())
    }

    fn vertex_edge_inputs(
        &self,
        vertex_index: i32,
        edge_index: i32,
    ) -> Result<(Vertex, Edge, Handle<Context>), Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(|| {
            Failure::new("BOPAlgo_PaveFiller::PerformVE requires initialized data structure")
        })?;
        let vertex = Vertex::unchecked(structure.shape(vertex_index)?.clone());
        let edge = Edge::unchecked(structure.shape(edge_index)?.clone());
        Ok((vertex, edge, self.context.clone()))
    }

    fn update_vertex(&mut self, index: i32, tolerance: f64) -> Result<i32, Failure> {
        let structure = self.data_structure.as_mut().ok_or_else(|| {
            Failure::new("BOPAlgo_PaveFiller::PerformVE requires initialized data structure")
        })?;
        update_vertex_record(structure, index, tolerance, self.non_destructive)
    }

    fn append_vertex_edge_interferences(
        &mut self,
        candidate: &SharedVertexCandidate,
        edge: i32,
        new_vertex: i32,
        parameter: f64,
    ) {
        let structure = self
            .data_structure
            .as_mut()
            .expect("initialized data structure");
        for original in &candidate.originals {
            append_vertex_edge_record(structure, *original, edge, new_vertex, parameter);
        }
    }

    fn split_intersected_edges(&mut self, edges: &[i32]) -> Result<(), Failure> {
        for edge in edges {
            self.split_edge_blocks(*edge)?;
        }
        if !edges.is_empty() {
            self.fill_shrunk_data(ShapeEnum::Vertex, ShapeEnum::Edge)?;
        }
        Ok(())
    }

    fn split_edge_blocks(&mut self, edge: i32) -> Result<(), Failure> {
        let structure = self.data_structure.as_mut().ok_or_else(|| {
            Failure::new("BOPAlgo_PaveFiller::PerformVE requires initialized data structure")
        })?;
        let source_blocks = structure.change_pave_blocks(edge)?.clone();
        let mut replacement = Vec::new();
        for block in source_blocks {
            split_block(block, &mut replacement)?;
        }
        *structure.change_pave_blocks(edge)? = replacement;
        Ok(())
    }
}

fn append_vertex_edge_candidate(
    structure: &DataStructure,
    vertex: i32,
    edge: i32,
    groups: &mut Vec<CandidateGroup>,
) -> Result<(), Failure> {
    if !is_vertex_edge_candidate(structure, vertex, edge)? {
        return Ok(());
    }
    let Some(block) = first_splittable_block(structure, edge)? else {
        return Ok(());
    };
    group_candidate(groups, block, vertex);
    Ok(())
}

fn is_vertex_edge_candidate(
    structure: &DataStructure,
    vertex: i32,
    edge: i32,
) -> Result<bool, Failure> {
    let info = structure.shape_info(edge)?;
    Ok(!info.has_sub_shape(vertex)
        && !info.has_flag()
        && !structure.has_interference_between(vertex, edge)
        && !structure.has_interference_shape_subshapes(vertex, edge)?)
}

fn first_splittable_block(
    structure: &DataStructure,
    edge: i32,
) -> Result<Option<PaveBlockHandle>, Failure> {
    let Some(block) = structure.pave_blocks(edge)?.first() else {
        return Ok(None);
    };
    let splittable = block
        .lock()
        .expect("pave-block lock must not be poisoned")
        .is_splittable();
    Ok(splittable.then(|| Arc::clone(block)))
}

fn group_candidate(groups: &mut Vec<CandidateGroup>, block: PaveBlockHandle, vertex: i32) {
    if let Some(group) = groups
        .iter_mut()
        .find(|group| Arc::ptr_eq(&group.block, &block))
    {
        append_unique(&mut group.vertices, vertex);
        return;
    }
    groups.push(CandidateGroup {
        block,
        vertices: vec![vertex],
    });
}

fn append_shared_vertex(
    candidates: &mut Vec<SharedVertexCandidate>,
    representative: i32,
    original: i32,
) {
    if let Some(candidate) = candidates
        .iter_mut()
        .find(|candidate| candidate.representative == representative)
    {
        append_unique(&mut candidate.originals, original);
        return;
    }
    candidates.push(SharedVertexCandidate {
        representative,
        originals: vec![original],
    });
}

fn edge_has_vertex_endpoint(
    structure: &DataStructure,
    edge: i32,
    vertex: i32,
) -> Result<bool, Failure> {
    for block in structure.pave_blocks(edge)? {
        let locked = block.lock().expect("pave-block lock must not be poisoned");
        let (first, last) = locked.indices();
        if first == vertex || last == vertex {
            return Ok(true);
        }
    }
    Ok(false)
}

fn append_block_pave(block: &PaveBlockHandle, vertex: i32, parameter: f64) -> bool {
    let mut locked = block.lock().expect("pave-block lock must not be poisoned");
    let (first, last) = locked.range();
    if parameter <= first || parameter >= last {
        return false;
    }
    locked.append_ext_pave(Pave::new(vertex, parameter));
    true
}

fn update_vertex_record(
    structure: &mut DataStructure,
    index: i32,
    new_tolerance: f64,
    non_destructive: bool,
) -> Result<i32, Failure> {
    let representative = structure.shape_sd(index).unwrap_or(index);
    let has_same_domain = structure.shape_sd(index).is_some();
    let vertex = Vertex::unchecked(structure.shape(representative)?.clone());
    if structure.is_new_shape(representative) || has_same_domain || !non_destructive {
        enlarge_source_vertex(structure, representative, &vertex, new_tolerance)?;
        return Ok(representative);
    }
    append_toleranced_vertex(structure, index, &vertex, new_tolerance)
}

fn enlarge_source_vertex(
    structure: &mut DataStructure,
    index: i32,
    vertex: &Vertex,
    new_tolerance: f64,
) -> Result<(), Failure> {
    if BRepTool::vertex_tolerance(vertex)? >= new_tolerance {
        return Ok(());
    }
    BRepBuilder::default().update_vertex_tolerance(vertex, new_tolerance)?;
    expand_vertex_info(structure, index, vertex)
}

fn append_toleranced_vertex(
    structure: &mut DataStructure,
    original: i32,
    vertex: &Vertex,
    tolerance: f64,
) -> Result<i32, Failure> {
    let point = BRepTool::pnt(vertex)?;
    let total_tolerance = BRepTool::vertex_tolerance(vertex)?.max(tolerance);
    let mut copy = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut copy, point, total_tolerance)?;
    let mut info = ShapeInfo::default();
    info.set_shape_type(ShapeEnum::Vertex);
    info.set_shape(&copy.clone().into_shape());
    let new_index = structure.append_shape_info(&info);
    expand_vertex_info(structure, new_index, &copy)?;
    structure.add_shape_sd(original, new_index);
    Ok(new_index)
}

fn expand_vertex_info(
    structure: &mut DataStructure,
    index: i32,
    vertex: &Vertex,
) -> Result<(), Failure> {
    let point = BRepTool::pnt(vertex)?;
    let gap = BRepTool::vertex_tolerance(vertex)? + precision::confusion();
    let bounds = structure.change_shape_info(index)?.change_box();
    bounds.add_point(point);
    bounds.set_gap(gap);
    Ok(())
}

fn split_block(block: PaveBlockHandle, output: &mut Vec<PaveBlockHandle>) -> Result<(), Failure> {
    let mut source = block.lock().expect("pave-block lock must not be poisoned");
    if !source.is_to_update() {
        output.push(Arc::clone(&block));
        return Ok(());
    }
    let mut split = Vec::new();
    source.update(&mut split, true);
    output.extend(split.into_iter().map(pave_block_handle));
    Ok(())
}

fn append_unique(values: &mut Vec<i32>, value: i32) {
    if !values.contains(&value) {
        values.push(value);
    }
}

fn failure_alert_shape(vertex: &Vertex, edge: &Edge) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    let mut shape = compound.into_shape();
    builder.add(&mut shape, &vertex.clone().into_shape())?;
    builder.add(&mut shape, &edge.clone().into_shape())?;
    Ok(shape)
}

fn warning(options: &Options, vertex: &Vertex, edge: &Edge) -> Result<(), Failure> {
    let shape = failure_alert_shape(vertex, edge)?;
    let alert: AlertHandle = Arc::new(AlertIntersectionOfPairOfShapesFailed::new(&shape));
    options.add_warning(&alert);
    Ok(())
}

fn compute_vertex_edge_hit(
    options: &Options,
    context: &Context,
    vertex: &Vertex,
    edge: &Edge,
) -> Result<Option<(f64, f64)>, Failure> {
    let mut parameter = -1.0;
    let mut tolerance = -1.0;
    let status = context.compute_ve(
        vertex,
        edge,
        &mut parameter,
        &mut tolerance,
        options.fuzzy_value(),
    );
    let Ok(status) = status else {
        warning(options, vertex, edge)?;
        return Ok(None);
    };
    Ok((status == 0).then_some((parameter, tolerance)))
}

fn append_vertex_edge_record(
    structure: &mut DataStructure,
    original: i32,
    edge: i32,
    new_vertex: i32,
    parameter: f64,
) {
    if !structure.add_interference(original, edge) {
        return;
    }
    let is_new_shape = structure.is_new_shape(new_vertex);
    let record = structure.interf_ve().append(InterfVE::default());
    record.set_indices(original, edge);
    record.set_parameter(parameter);
    if is_new_shape {
        record.set_index_new(new_vertex);
    }
}
