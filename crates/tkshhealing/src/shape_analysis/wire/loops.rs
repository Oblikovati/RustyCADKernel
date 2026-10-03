//! Vertex incidence checks, including native output accumulation and early-exit precision.
// ShapeAnalysis_Wire.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::*;
use rustycad_tkbrep::{
    topexp,
    topods::{Edge, Shape, Vertex},
    toptools::{IndexedMapOfShape, MapOfShape, ShapeMapHasher},
};
use rustycad_tkernel::ncollection::{DataMap, List};

struct LoopMaps<'a> {
    vertices: &'a mut IndexedMapOfShape,
    incidence: &'a mut DataMap<Shape, List<Shape>, ShapeMapHasher>,
    small: &'a mut MapOfShape,
    seams: &'a mut MapOfShape,
}

impl ShapeAnalysisWire {
    /// Accumulate vertices with more than two significant incident edges;
    /// `analyzer.check_loop(&mut loops, &mut incidence, &mut small, &mut seams)?`.
    pub fn check_loop(
        &self,
        vertices: &mut IndexedMapOfShape,
        incidence: &mut DataMap<Shape, List<Shape>, ShapeMapHasher>,
        small: &mut MapOfShape,
        seams: &mut MapOfShape,
    ) -> Result<bool, Failure> {
        self.set_last_status(ShapeExtendStatus::Ok);
        if !self.is_loaded() || self.nb_edges() < 2 {
            return Ok(false);
        }
        self.check_loop_incidence(&mut LoopMaps {
            vertices,
            incidence,
            small,
            seams,
        })
    }

    /// Query accumulated loop flags; `analyzer.status_loop(ShapeExtendStatus::Done1)`.
    pub fn status_loop(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.fields.read().status_loop, status)
    }

    fn check_loop_incidence(&self, maps: &mut LoopMaps<'_>) -> Result<bool, Failure> {
        let saved_precision = self.precision();
        self.set_precision(precision::infinite());
        for index in 1..=self.nb_edges() {
            if !self.append_loop_edge(index, maps)? {
                // Native FAIL2 exits before restoring precision; callers can observe it.
                self.set_last_status(ShapeExtendStatus::Fail2);
                return Ok(false);
            }
        }
        self.set_precision(saved_precision);
        if maps.vertices.is_empty() {
            return Ok(false);
        }
        self.set_last_status(ShapeExtendStatus::Done1);
        self.fields.write().status_loop |= ShapeExtend::encode_status(ShapeExtendStatus::Done1);
        Ok(true)
    }

    fn append_loop_edge(&self, index: i32, maps: &mut LoopMaps<'_>) -> Result<bool, Failure> {
        let edge = self.wire_data().edge(index)?;
        let (first, last) = topexp::vertices(&edge, false);
        if first.is_null() || last.is_null() {
            return Ok(false);
        }
        self.classify_loop_edge(index, &edge, &first, &last, maps)?;
        for vertex in [&first, &last] {
            let shape = vertex.clone().into_shape();
            if !maps.incidence.is_bound(&shape) {
                maps.incidence.bind(shape, List::default());
            }
        }
        maps.append_incidence(&first, &edge)?;
        maps.append_incidence(&last, &edge)?;
        Ok(true)
    }

    fn classify_loop_edge(
        &self,
        index: i32,
        edge: &Edge,
        first: &Vertex,
        last: &Vertex,
        maps: &mut LoopMaps<'_>,
    ) -> Result<(), Failure> {
        if self.wire_data().is_seam(index) {
            maps.seams.add(edge.clone().into_shape());
        } else if BRepTool::degenerated(edge)?
            || (first
                .clone()
                .into_shape()
                .is_same(&last.clone().into_shape())
                && self.check_small_at(index, BRepTool::vertex_tolerance(first)?)?)
        {
            maps.small.add(edge.clone().into_shape());
        }
        Ok(())
    }
}

impl LoopMaps<'_> {
    fn append_incidence(&mut self, vertex: &Vertex, edge: &Edge) -> Result<(), Failure> {
        let key = vertex.clone().into_shape();
        let edges = self.incidence.change_find(&key)?;
        edges.append(edge.clone().into_shape());
        let significant = edges
            .iter()
            .filter(|edge| !self.small.contains(edge) && !self.seams.contains(edge))
            .count();
        if significant > 2 {
            self.vertices.add(key);
        }
        Ok(())
    }
}
