//! Native connection statuses and distance fallback at OCCT 3d097a0328e71b8.
use super::*;
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkmath::gp;
impl ShapeAnalysisWire {
    /// Accumulate connection statuses over every edge, including closure.
    /// Example: `analyzer.check_connected(1e-6)?`.
    pub fn check_connected(&self, precision: f64) -> Result<bool, Failure> {
        let count = self.wire_data().nb_edges();
        for index in 1..=count {
            self.check_connected_at(index, precision)?;
            let mut fields = self.fields.write();
            fields.status_connected |= fields.status;
        }
        Ok(self.status_connected(ShapeExtendStatus::Done))
    }
    /// Check the preceding edge connection; nonpositive indices select the last edge.
    /// Example: `analyzer.check_connected_at(1, 1e-6)?`.
    pub fn check_connected_at(&self, index: i32, precision: f64) -> Result<bool, Failure> {
        self.set_last_status(ShapeExtendStatus::Ok);
        let count = self.nb_edges();
        if !self.is_loaded() || count < 1 {
            return Ok(false);
        }
        let second = if index > 0 { index } else { count };
        let first = if second > 1 { second - 1 } else { count };
        let wire = self.wire_data();
        self.connection_vertices(
            &wire.edge(first)?,
            &wire.edge(second)?,
            first == second,
            precision,
        )
    }
    /// Accumulated public connection-check status; `analyzer.status_connected(ShapeExtendStatus::Fail)`.
    pub fn status_connected(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.fields.read().status_connected, status)
    }
    /// Most recent measured three-dimensional distance; `analyzer.min_distance_3d()`.
    pub fn min_distance_3d(&self) -> f64 {
        self.fields.read().min_3d
    }
    fn connection_vertices(
        &self,
        first: &Edge,
        second: &Edge,
        same: bool,
        precision: f64,
    ) -> Result<bool, Failure> {
        let analyzer = ShapeAnalysisEdge::new();
        let previous = analyzer.last_vertex(first);
        let following = analyzer.first_vertex(second);
        if previous.is_null() || following.is_null() {
            self.set_last_status(ShapeExtendStatus::Fail2);
            return Ok(false);
        }
        if previous.is_same(&following) {
            return Ok(false);
        }
        let distance = BRepTool::pnt(&previous)?.distance(&BRepTool::pnt(&following)?);
        self.fields.write().min_3d = distance;
        if self.classify_connection_distance(distance, precision) {
            return Ok(true);
        }
        self.connection_reverse_fallback(&previous, second, same)?;
        Ok(false)
    }
    fn classify_connection_distance(&self, distance: f64, precision: f64) -> bool {
        let status = if distance <= gp::resolution() {
            ShapeExtendStatus::Done1
        } else if distance <= self.precision() {
            ShapeExtendStatus::Done2
        } else if distance <= precision {
            ShapeExtendStatus::Done3
        } else {
            return false;
        };
        self.set_last_status(status);
        true
    }
    fn connection_reverse_fallback(
        &self,
        previous: &Vertex,
        second: &Edge,
        same: bool,
    ) -> Result<(), Failure> {
        self.set_last_status(ShapeExtendStatus::Fail1);
        if same {
            return Ok(());
        }
        let last = ShapeAnalysisEdge::new().last_vertex(second);
        let distance = BRepTool::pnt(previous)?.distance(&BRepTool::pnt(&last)?);
        if distance <= self.precision() {
            self.fields.write().min_3d = distance;
            self.set_last_status(ShapeExtendStatus::Fail2);
        }
        Ok(())
    }
}
