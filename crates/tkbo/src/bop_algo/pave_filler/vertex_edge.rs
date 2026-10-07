//! `BOPAlgo_PaveFiller_2.cxx`: vertex/edge intersection and the splitting of pave blocks.
use super::vertex_vertex::pair_compound;
use super::{BOPAlgoPaveFiller, missing_structure};
use crate::bop_algo::AlertIntersectionOfPairOfShapesFailed;
use crate::bop_algo::tools::{PaveBlockIndexLinks, fill_map_one, pave_block_index_links};
use crate::bop_ds::{InterfVE, Pair, Pave, PaveBlockHandle};
use rustycad_tkbrep::topods::{Edge, Shape, Vertex};
use rustycad_tkernel::{
    message::AlertHandle,
    ncollection::{DataMap, Map},
    standard::Failure,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::sync::Arc;

#[path = "vertex_edge/split.rs"]
mod split;

/// One prepared `BOPAlgo_VertexEdge` solver: indices, its pave block and computed result.
struct VertexEdgeSolver {
    vertex: i32,
    edge: i32,
    flag: i32,
    parameter: f64,
    new_tolerance: f64,
    failed: bool,
}

impl BOPAlgoPaveFiller {
    /// Intersects the vertices with the first splittable pave block of each edge whose box
    /// they touch (`PerformVE`); `filler.perform_ve()?`.
    pub fn perform_ve(&mut self) -> Result<(), Failure> {
        self.fill_shrunk_data(ShapeEnum::Vertex, ShapeEnum::Edge)?;
        let pairs = self.iterator_pairs(ShapeEnum::Vertex, ShapeEnum::Edge)?;
        if pairs.is_empty() {
            return Ok(());
        }
        let mut vertex_edge_pairs = pave_block_index_links();
        for (vertex, edge) in pairs {
            if let Some(block) = self.ve_candidate_block(vertex, edge)? {
                fill_map_one(&mut vertex_edge_pairs, block, vertex);
            }
        }
        self.intersect_ve(&vertex_edge_pairs, true)
    }

    /// Computes the parameters of the vertices on the pave blocks' edges, adds extra paves
    /// (and VE interferences when `add_interfs`) and splits the touched edges (`IntersectVE`).
    /// `filler.intersect_ve(&pairs, true)?`.
    pub fn intersect_ve(
        &mut self,
        pairs: &PaveBlockIndexLinks,
        add_interfs: bool,
    ) -> Result<(), Failure> {
        if pairs.extent() == 0 {
            return Ok(());
        }
        let mut same_domain: DataMap<Pair, Vec<i32>> = DataMap::default();
        let mut solvers = self.prepare_ve_solvers(pairs, &mut same_domain)?;
        for solver in &mut solvers {
            self.compute_ve_solver(solver)?;
        }
        let mut edges: Map<i32> = Map::new(1);
        for solver in &solvers {
            self.apply_ve_solver(solver, &same_domain, add_interfs, &mut edges)?;
        }
        self.split_pave_blocks(&edges, add_interfs)
    }

    /// Warns that the intersection of two sub-shapes failed; `filler.add_intersection_failed_warning(&a, &b)?`.
    pub fn add_intersection_failed_warning(
        &mut self,
        first: &Shape,
        second: &Shape,
    ) -> Result<(), Failure> {
        let pair = pair_compound(first, second)?;
        let alert: AlertHandle = Arc::new(AlertIntersectionOfPairOfShapesFailed::new(&pair));
        self.options.add_warning(&alert);
        Ok(())
    }

    fn ve_candidate_block(
        &self,
        vertex: i32,
        edge: i32,
    ) -> Result<Option<PaveBlockHandle>, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let info = structure.shape_info(edge)?;
        if info.has_sub_shape(vertex) || info.has_flag() {
            return Ok(None);
        }
        if structure.has_interference_between(vertex, edge)
            || structure.has_interference_shape_subshapes(vertex, edge)?
        {
            return Ok(None);
        }
        let Some(first) = structure.pave_blocks(edge)?.first() else {
            return Ok(None);
        };
        // A non-splittable first block marks a micro edge, which is ignored.
        if !first.lock().expect("pave-block lock").is_splittable() {
            return Ok(None);
        }
        Ok(Some(Arc::clone(first)))
    }

    fn prepare_ve_solvers(
        &self,
        pairs: &PaveBlockIndexLinks,
        same_domain: &mut DataMap<Pair, Vec<i32>>,
    ) -> Result<Vec<VertexEdgeSolver>, Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let mut solvers = Vec::new();
        for index in 1..=pairs.extent() {
            let edge = pairs
                .find_key(index)?
                .lock()
                .expect("pave-block lock")
                .original_edge();
            let mut bounding: Map<i32> = Map::new(1);
            for block in structure.pave_blocks(edge)? {
                let (first, last) = block.lock().expect("pave-block lock").indices();
                bounding.add(first);
                bounding.add(last);
            }
            for vertex in pairs.find_from_index(index)?.iter().copied() {
                let mut representative = vertex;
                structure.has_shape_sd_into(vertex, &mut representative);
                if bounding.contains(&representative) {
                    continue;
                }
                let key = Pair::new(representative, edge);
                if let Some(list) = same_domain.change_seek(&key) {
                    list.push(vertex);
                    continue;
                }
                same_domain.bind(key, vec![vertex]);
                solvers.push(VertexEdgeSolver {
                    vertex: representative,
                    edge,
                    flag: -1,
                    parameter: -1.,
                    new_tolerance: -1.,
                    failed: false,
                });
            }
        }
        Ok(solvers)
    }

    fn compute_ve_solver(&self, solver: &mut VertexEdgeSolver) -> Result<(), Failure> {
        let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
        let vertex = Vertex::unchecked(structure.shape(solver.vertex)?.clone());
        let edge = Edge::unchecked(structure.shape(solver.edge)?.clone());
        let context = self.context.get().ok_or_else(missing_structure)?;
        // An exception of ComputeVE becomes the solver's error; its flag stays -1.
        match context.compute_ve(
            &vertex,
            &edge,
            &mut solver.parameter,
            &mut solver.new_tolerance,
            self.options.fuzzy_value(),
        ) {
            Ok(flag) => solver.flag = flag,
            Err(_) => solver.failed = true,
        }
        Ok(())
    }

    fn apply_ve_solver(
        &mut self,
        solver: &VertexEdgeSolver,
        same_domain: &DataMap<Pair, Vec<i32>>,
        add_interfs: bool,
        edges: &mut Map<i32>,
    ) -> Result<(), Failure> {
        if solver.flag != 0 {
            if solver.failed {
                let structure = self.data_structure.as_ref().ok_or_else(missing_structure)?;
                let vertex = structure.shape(solver.vertex)?.clone();
                let edge = structure.shape(solver.edge)?.clone();
                self.add_intersection_failed_warning(&vertex, &edge)?;
            }
            return Ok(());
        }
        let new_vertex = self.update_vertex(solver.vertex, solver.new_tolerance)?;
        let structure = self.data_structure.as_mut().ok_or_else(missing_structure)?;
        let target = structure.pave_blocks(solver.edge)?.iter().find(|block| {
            let (first, last) = block.lock().expect("pave-block lock").range();
            solver.parameter > first && solver.parameter < last
        });
        let Some(target) = target.cloned() else {
            return Ok(());
        };
        let pave = Pave::new(new_vertex, solver.parameter);
        target
            .lock()
            .expect("pave-block lock")
            .append_ext_pave(pave);
        edges.add(solver.edge);
        if !add_interfs {
            return Ok(());
        }
        let originals = same_domain
            .find(&Pair::new(solver.vertex, solver.edge))?
            .clone();
        let is_new = structure.is_new_shape(new_vertex);
        for original in originals {
            let record = structure.interf_ve().append(InterfVE::default());
            record.set_indices(original, solver.edge);
            record.set_parameter(solver.parameter);
            if is_new {
                record.set_index_new(new_vertex);
            }
            structure.add_interference(original, solver.edge);
        }
        Ok(())
    }
}
