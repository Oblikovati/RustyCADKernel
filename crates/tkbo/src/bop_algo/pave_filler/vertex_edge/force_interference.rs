use super::{BOPAlgoPaveFiller, append_unique, failure_alert_shape};
use crate::bop_algo::AlertSelfInterferingShape;
use crate::bop_ds::{InterfVE, Pave, PaveBlockHandle};
use crate::int_tools::Context;
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::{
    message::AlertHandle,
    standard::{Failure, Handle},
};
use std::sync::Arc;

impl BOPAlgoPaveFiller {
    pub(in crate::bop_algo::pave_filler) fn force_interference_vertex_edge(
        &mut self,
        vertex: i32,
        block: &PaveBlockHandle,
        changed_edges: &mut Vec<i32>,
    ) -> Result<bool, Failure> {
        let edge = block_edge_index(block);
        if self.force_ve_already_present(vertex, edge, block)? {
            return Ok(true);
        }
        self.compute_forced_interference(vertex, edge, block, changed_edges)
    }

    fn compute_forced_interference(
        &mut self,
        vertex: i32,
        edge: i32,
        block: &PaveBlockHandle,
        changed_edges: &mut Vec<i32>,
    ) -> Result<bool, Failure> {
        let representative = self
            .data_structure
            .as_ref()
            .ok_or_else(|| Failure::new("BOPAlgo_PaveFiller::ForceInterfVE requires DS"))?
            .shape_sd(vertex)
            .unwrap_or(vertex);
        let (vertex_shape, edge_shape, context) = self.vertex_edge_inputs(representative, edge)?;
        let Some((parameter, tolerance)) = compute_forced_vertex_edge_hit(
            &context,
            &vertex_shape,
            &edge_shape,
            self.options.fuzzy_value(),
        )?
        else {
            return Ok(false);
        };
        self.record_forced_interference(vertex, edge, block, parameter, tolerance, changed_edges)?;
        self.warn_self_interference(vertex, edge, &vertex_shape, &edge_shape)?;
        Ok(true)
    }

    fn record_forced_interference(
        &mut self,
        vertex: i32,
        edge: i32,
        block: &PaveBlockHandle,
        parameter: f64,
        tolerance: f64,
        changed_edges: &mut Vec<i32>,
    ) -> Result<(), Failure> {
        let new_vertex = self.update_vertex(vertex, tolerance)?;
        append_forced_vertex_edge_record(self, vertex, edge, new_vertex, parameter);
        append_unchecked_block_pave(block, new_vertex, parameter);
        append_unique(changed_edges, edge);
        append_unique(&mut self.vertices_to_avoid_extension, vertex);
        Ok(())
    }

    fn force_ve_already_present(
        &self,
        vertex: i32,
        edge: i32,
        block: &PaveBlockHandle,
    ) -> Result<bool, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(|| {
            Failure::new("BOPAlgo_PaveFiller::ForceInterfVE requires initialized data structure")
        })?;
        let info = structure.shape_info(edge)?;
        if info.has_sub_shape(vertex)
            || structure.has_interference_between(vertex, edge)
            || structure.has_interference_shape_subshapes(vertex, edge)?
        {
            return Ok(true);
        }
        let (first, last) = block
            .lock()
            .expect("pave-block lock must not be poisoned")
            .indices();
        Ok(first == vertex || last == vertex)
    }

    fn warn_self_interference(
        &self,
        vertex_index: i32,
        edge_index: i32,
        vertex: &Vertex,
        edge: &Edge,
    ) -> Result<(), Failure> {
        let structure = self
            .data_structure
            .as_ref()
            .expect("initialized data structure");
        if structure.rank(vertex_index) < 0
            || structure.rank(vertex_index) != structure.rank(edge_index)
        {
            return Ok(());
        }
        let shape = failure_alert_shape(vertex, edge)?;
        let alert: AlertHandle = Arc::new(AlertSelfInterferingShape::new(&shape));
        self.options.add_warning(&alert);
        Ok(())
    }
}

fn block_edge_index(block: &PaveBlockHandle) -> i32 {
    block
        .lock()
        .expect("pave-block lock must not be poisoned")
        .original_edge()
}

fn compute_forced_vertex_edge_hit(
    context: &Handle<Context>,
    vertex: &Vertex,
    edge: &Edge,
    fuzzy: f64,
) -> Result<Option<(f64, f64)>, Failure> {
    let mut parameter = -1.0;
    let mut tolerance = -1.0;
    let status = context.compute_ve(vertex, edge, &mut parameter, &mut tolerance, fuzzy)?;
    Ok(matches!(status, 0 | -4).then_some((parameter, tolerance)))
}

fn append_forced_vertex_edge_record(
    filler: &mut BOPAlgoPaveFiller,
    vertex: i32,
    edge: i32,
    new_vertex: i32,
    parameter: f64,
) {
    let structure = filler
        .data_structure
        .as_mut()
        .expect("initialized data structure");
    let is_new = structure.is_new_shape(new_vertex);
    let record = structure.interf_ve().append(InterfVE::default());
    record.set_indices(vertex, edge);
    record.set_parameter(parameter);
    if is_new {
        record.set_index_new(new_vertex);
    }
    structure.add_interference(vertex, edge);
}

fn append_unchecked_block_pave(block: &PaveBlockHandle, vertex: i32, parameter: f64) {
    block
        .lock()
        .expect("pave-block lock must not be poisoned")
        .append_ext_pave(Pave::new(vertex, parameter));
}
