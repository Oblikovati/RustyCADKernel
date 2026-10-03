//! Classify the closest endpoint pairing with an edge or wire, preserving native ties.
use super::*;
use crate::shape_analysis::ShapeAnalysis;
use rustycad_tkbrep::topods::{self, Shape, Vertex};
use rustycad_tkg3d::top_abs::ShapeEnum;

impl ShapeAnalysisWire {
    /// Check which end of a shape can join the loaded wire.
    /// Example: `analyzer.check_shape_connect(&edge.into_shape(), 1e-6)?`.
    pub fn check_shape_connect(&self, shape: &Shape, precision: f64) -> Result<bool, Failure> {
        self.check_shape_connect_into(&mut [0.; 4], shape, precision)
    }

    /// Write distances in native tail-head, tail-tail, head-tail, head-head order.
    /// Outputs remain unchanged for unloaded, null, or unsupported inputs.
    /// Example: `analyzer.check_shape_connect_into(&mut distances, &shape, 1e-6)?`.
    pub fn check_shape_connect_into(
        &self,
        distances: &mut [f64; 4],
        shape: &Shape,
        precision: f64,
    ) -> Result<bool, Failure> {
        self.set_last_status(ShapeExtendStatus::Fail1);
        if !self.is_loaded() || shape.is_null() {
            return Ok(false);
        }
        let Some(vertices) = connection_shape_vertices(shape)? else {
            return Ok(false);
        };
        self.set_last_status(ShapeExtendStatus::Done1);
        *distances = self.shape_connection_distances(vertices)?;
        self.classify_shape_connection(*distances);
        if self.min_distance_3d() > self.precision().max(precision) {
            self.set_last_status(ShapeExtendStatus::Fail2);
        }
        Ok(self.last_check_status(ShapeExtendStatus::Done))
    }

    fn shape_connection_distances(&self, vertices: (Vertex, Vertex)) -> Result<[f64; 4], Failure> {
        let first = BRepTool::pnt(&vertices.0)?;
        let last = BRepTool::pnt(&vertices.1)?;
        let analysis = ShapeAnalysisEdge::new();
        let wire = self.wire_data();
        let wire_first = BRepTool::pnt(&analysis.first_vertex(&wire.edge(1)?))?;
        let wire_last = BRepTool::pnt(&analysis.last_vertex(&wire.edge(self.nb_edges())?))?;
        Ok([
            first.distance(&wire_last),
            last.distance(&wire_last),
            last.distance(&wire_first),
            first.distance(&wire_first),
        ])
    }

    fn classify_shape_connection(&self, distances: [f64; 4]) {
        let tail_reversed = distances[0] > distances[1];
        let head_reversed = distances[2] > distances[3];
        let tail_distance = distances[usize::from(tail_reversed)];
        let head_distance = distances[2 + usize::from(head_reversed)];
        let result = if tail_distance > head_distance {
            2 + usize::from(head_reversed)
        } else {
            usize::from(tail_reversed)
        };
        let mut fields = self.fields.write();
        fields.min_3d = tail_distance.min(head_distance);
        fields.max_3d = tail_distance.max(head_distance);
        fields.status = connection_status(result, tail_reversed, head_reversed);
    }
}

fn connection_shape_vertices(shape: &Shape) -> Result<Option<(Vertex, Vertex)>, Failure> {
    match shape.shape_type()? {
        ShapeEnum::Edge => {
            let edge = topods::edge(shape)?;
            let analysis = ShapeAnalysisEdge::new();
            Ok(Some((
                analysis.first_vertex(&edge),
                analysis.last_vertex(&edge),
            )))
        }
        ShapeEnum::Wire => Ok(Some(ShapeAnalysis::find_bounds(shape)?)),
        _ => Ok(None),
    }
}

fn connection_status(result: usize, tail_reversed: bool, head_reversed: bool) -> i32 {
    let statuses = [
        ShapeExtendStatus::Done1,
        ShapeExtendStatus::Done2,
        ShapeExtendStatus::Done3,
        ShapeExtendStatus::Done4,
    ];
    let mut status = ShapeExtend::encode_status(statuses[result]);
    if !tail_reversed {
        status |= ShapeExtend::encode_status(ShapeExtendStatus::Done5);
    }
    if !head_reversed {
        status |= ShapeExtend::encode_status(ShapeExtendStatus::Done6);
    }
    status
}
