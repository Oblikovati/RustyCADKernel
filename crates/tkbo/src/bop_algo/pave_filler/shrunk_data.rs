use super::{BOPAlgoPaveFiller, Options};
use crate::bop_algo::{AlertBadPositioning, AlertNotSplittableEdge, AlertTooSmallEdge};
use crate::bop_ds::{DataStructure, PaveBlockHandle};
use crate::int_tools::ShrunkRange;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Compound, Edge, Shape, Vertex},
};
use rustycad_tkernel::{message::AlertHandle, standard::Failure};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox3;
use std::sync::Arc;

impl BOPAlgoPaveFiller {
    /// Computes cached shrunk ranges for unique candidate edges; `filler.fill_shrunk_data(Vertex, Edge)?`.
    pub fn fill_shrunk_data(
        &mut self,
        first_type: ShapeEnum,
        second_type: ShapeEnum,
    ) -> Result<(), Failure> {
        let edges = self.candidate_edges(first_type, second_type)?;
        if edges.is_empty() {
            return Ok(());
        }
        self.compute_edge_shrunk_ranges(&edges)
    }

    fn candidate_edges(
        &mut self,
        first_type: ShapeEnum,
        second_type: ShapeEnum,
    ) -> Result<Vec<i32>, Failure> {
        let iterator = self.iterator.as_mut().ok_or_else(|| {
            Failure::new("BOPAlgo_PaveFiller::FillShrunkData requires initialized iterator")
        })?;
        iterator.initialize(first_type, second_type);
        let mut edges = Vec::new();
        while iterator.more() {
            let (first, second) = iterator.value()?;
            append_unique_edges(&mut edges, first_type, second_type, first, second);
            iterator.next();
        }
        Ok(edges)
    }

    fn compute_edge_shrunk_ranges(&mut self, edges: &[i32]) -> Result<(), Failure> {
        let context = self.context.clone();
        let options = &self.options;
        for edge_index in edges {
            let structure = self.data_structure.as_mut().ok_or_else(|| {
                Failure::new(
                    "BOPAlgo_PaveFiller::FillShrunkData requires initialized data structure",
                )
            })?;
            compute_edge_ranges(structure, options, &context, *edge_index)?;
        }
        Ok(())
    }
}

fn compute_edge_ranges(
    structure: &mut DataStructure,
    options: &Options,
    context: &rustycad_tkernel::standard::Handle<crate::int_tools::Context>,
    edge_index: i32,
) -> Result<(), Failure> {
    let info = structure.shape_info(edge_index)?;
    if info.has_flag() {
        return Ok(());
    }
    let edge = Edge::unchecked(info.shape().clone());
    let blocks = structure.change_pave_blocks(edge_index)?.clone();
    for block in blocks {
        compute_block_range(structure, options, context, &edge, &block)?;
    }
    Ok(())
}

fn compute_block_range(
    structure: &DataStructure,
    options: &Options,
    context: &rustycad_tkernel::standard::Handle<crate::int_tools::Context>,
    edge: &Edge,
    block: &PaveBlockHandle,
) -> Result<(), Failure> {
    if structure.is_valid_shrunk_data(block)? {
        return Ok(());
    }
    let (first, last, first_vertex, last_vertex) = block_inputs(structure, block)?;
    let mut shrunk = ShrunkRange::new();
    shrunk.set_data(edge, first, last, &first_vertex, &last_vertex);
    shrunk.set_context(context);
    shrunk.perform()?;
    analyze_shrunk_result(options, structure, edge, block, &shrunk)
}

fn append_unique_edges(
    edges: &mut Vec<i32>,
    first_type: ShapeEnum,
    second_type: ShapeEnum,
    first: i32,
    second: i32,
) {
    if first_type == ShapeEnum::Edge {
        append_unique(edges, first);
    }
    if second_type == ShapeEnum::Edge {
        append_unique(edges, second);
    }
}

fn append_unique(edges: &mut Vec<i32>, edge: i32) {
    if !edges.contains(&edge) {
        edges.push(edge);
    }
}

fn block_inputs(
    structure: &DataStructure,
    block: &PaveBlockHandle,
) -> Result<(f64, f64, Vertex, Vertex), Failure> {
    let locked = block.lock().expect("pave-block lock must not be poisoned");
    let (first, last) = locked.range();
    let (first_index, last_index) = locked.indices();
    let first_vertex = Vertex::unchecked(structure.shape(first_index)?.clone());
    let last_vertex = Vertex::unchecked(structure.shape(last_index)?.clone());
    Ok((first, last, first_vertex, last_vertex))
}

fn analyze_shrunk_result(
    options: &Options,
    structure: &DataStructure,
    edge: &Edge,
    block: &PaveBlockHandle,
    shrunk: &ShrunkRange,
) -> Result<(), Failure> {
    let (first, last) = shrunk.shrunk_range();
    if !shrunk.is_done() {
        report_shrunk_failure(options, structure, edge, block)?;
        set_shrunk(block, first, last, &BoundingBox3::default(), false);
        return Ok(());
    }
    report_unsplittable(options, structure, edge, block, shrunk.is_splittable())?;
    let mut bounds = shrunk.bounding_box().clone();
    bounds.set_gap(bounds.gap() + options.fuzzy_value() / 2.0);
    set_shrunk(block, first, last, &bounds, shrunk.is_splittable());
    Ok(())
}

fn report_shrunk_failure(
    options: &Options,
    structure: &DataStructure,
    edge: &Edge,
    block: &PaveBlockHandle,
) -> Result<(), Failure> {
    let whole = is_whole_edge(edge, block)?;
    let alert: AlertHandle = if whole {
        Arc::new(AlertTooSmallEdge::new(&edge.clone().into_shape()))
    } else {
        Arc::new(AlertBadPositioning::new(&warning_shape(
            structure, edge, block,
        )?))
    };
    options.add_warning(&alert);
    Ok(())
}

fn report_unsplittable(
    options: &Options,
    structure: &DataStructure,
    edge: &Edge,
    block: &PaveBlockHandle,
    splittable: bool,
) -> Result<(), Failure> {
    if splittable {
        return Ok(());
    }
    let alert: AlertHandle = if is_whole_edge(edge, block)? {
        Arc::new(AlertNotSplittableEdge::new(&edge.clone().into_shape()))
    } else {
        Arc::new(AlertBadPositioning::new(&warning_shape(
            structure, edge, block,
        )?))
    };
    options.add_warning(&alert);
    Ok(())
}

fn is_whole_edge(edge: &Edge, block: &PaveBlockHandle) -> Result<bool, Failure> {
    let (_, edge_first, edge_last) = BRepTool::curve(edge)?;
    let locked = block.lock().expect("pave-block lock must not be poisoned");
    let (block_first, block_last) = locked.range();
    Ok(block_first <= edge_first && block_last >= edge_last)
}

fn warning_shape(
    structure: &DataStructure,
    edge: &Edge,
    block: &PaveBlockHandle,
) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let locked = block.lock().expect("pave-block lock must not be poisoned");
    let (first_index, last_index) = locked.indices();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    let mut shape = compound.into_shape();
    builder.add(&mut shape, &edge.clone().into_shape())?;
    builder.add(&mut shape, structure.shape(first_index)?)?;
    builder.add(&mut shape, structure.shape(last_index)?)?;
    Ok(shape)
}

fn set_shrunk(
    block: &PaveBlockHandle,
    first: f64,
    last: f64,
    bounds: &BoundingBox3,
    splittable: bool,
) {
    block
        .lock()
        .expect("pave-block lock must not be poisoned")
        .set_shrunk_data(first, last, bounds, splittable);
}
