//! Native closure diagnosis composes connection and singular-edge statuses.
use super::*;

impl ShapeAnalysisWire {
    /// Report an incorrectly oriented outer boundary using the requested wire builder.
    /// Example: `analyzer.check_outer_bound(false)?`.
    pub fn check_outer_bound(&self, api_make: bool) -> Result<bool, Failure> {
        self.set_last_status(ShapeExtendStatus::Ok);
        if !self.is_ready() {
            return Ok(false);
        }
        let wire = if api_make {
            self.wire_data().wire_api_make()?
        } else {
            self.wire_data().wire()?
        };
        let mut face = self.face().empty_copied()?;
        rustycad_tkbrep::topods::Builder.add(&mut face, &wire.into_shape())?;
        let face = rustycad_tkbrep::topods::face(&face)?;
        if crate::shape_analysis::ShapeAnalysis::is_outer_bound(&face)? {
            return Ok(false);
        }
        self.set_last_status(ShapeExtendStatus::Done1);
        Ok(true)
    }

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
