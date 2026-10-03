//! Native closure diagnosis composes connection and singular-edge statuses.
use super::*;

impl ShapeAnalysisWire {
    /// Check the closing edge pair, resetting only the aggregate closure status.
    /// Example: `analyzer.check_closed(0.0)?`.
    pub fn check_closed(&self, precision: f64) -> Result<bool, Failure> {
        self.fields.write().status_closed = ShapeExtend::encode_status(ShapeExtendStatus::Ok);
        if !self.is_ready() || self.nb_edges() < 1 {
            return Ok(false);
        }
        self.check_connected_at(1, precision)?;
        self.accumulate_closed_status(ShapeExtendStatus::Done1, ShapeExtendStatus::Fail1);
        self.check_degenerated_edge(1)?;
        self.accumulate_closed_status(ShapeExtendStatus::Done2, ShapeExtendStatus::Fail2);
        Ok(self.status_closed(ShapeExtendStatus::Done))
    }

    /// Query findings from the last closure check; `analyzer.status_closed(ShapeExtendStatus::Fail1)`.
    pub fn status_closed(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.fields.read().status_closed, status)
    }

    fn accumulate_closed_status(&self, done: ShapeExtendStatus, fail: ShapeExtendStatus) {
        if self.last_check_status(ShapeExtendStatus::Done) {
            self.fields.write().status_closed |= ShapeExtend::encode_status(done);
        }
        if self.last_check_status(ShapeExtendStatus::Fail) {
            self.fields.write().status_closed |= ShapeExtend::encode_status(fail);
        }
    }
}
