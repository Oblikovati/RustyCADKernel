//! Native ordered closure repairs from ShapeFix_Wire.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::ShapeFixWire;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkernel::standard::Failure;

impl ShapeFixWire {
    /// Repairs the closing connection, singularity, then p-curve gap in native order.
    /// Example: `fixer.fix_closed(1e-7)?`.
    /// Inherits the remaining geometric limitations of `fix_lacking_at`.
    pub fn fix_closed(&self, precision: f64) -> Result<bool, Failure> {
        self.state.write().closed_status = ShapeExtend::encode_status(ShapeExtendStatus::Ok);
        if self.loaded_wire_analyzer()?.is_none() {
            return Ok(false);
        }
        self.fix_connected_at(1, precision, true)?;
        self.accumulate_closed_step(ShapeExtendStatus::Done1, ShapeExtendStatus::Fail1);
        self.fix_degenerated_at(1)?;
        self.accumulate_closed_step(ShapeExtendStatus::Done2, ShapeExtendStatus::Fail2);
        self.fix_lacking_at(1, false)?;
        self.accumulate_closed_step(ShapeExtendStatus::Done3, ShapeExtendStatus::Fail3);
        Ok(self.status_closed(ShapeExtendStatus::Done))
    }

    /// Reads the combined flags from the last closure repair.
    /// Example: `fixer.status_closed(ShapeExtendStatus::Done1)`.
    pub fn status_closed(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.state.read().closed_status, status)
    }

    fn accumulate_closed_step(&self, done: ShapeExtendStatus, failure: ShapeExtendStatus) {
        let mut flags = 0;
        if self.last_fix_status(ShapeExtendStatus::Done) {
            flags |= ShapeExtend::encode_status(done);
        }
        if self.last_fix_status(ShapeExtendStatus::Fail) {
            flags |= ShapeExtend::encode_status(failure);
        }
        self.state.write().closed_status |= flags;
    }
}
