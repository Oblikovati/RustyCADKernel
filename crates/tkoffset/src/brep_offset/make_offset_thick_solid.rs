//! Supported no-cap branch of `BRepOffset_MakeOffset::MakeThickSolid`.
use super::BRepOffsetMakeOffset;
use rustycad_tkernel::standard::Failure;

impl BRepOffsetMakeOffset {
    /// Builds the offset shell and reverses positive offsets, matching the native no-cap branch.
    /// `maker.make_thick_solid()?`.
    pub fn make_thick_solid(&mut self) -> Result<(), Failure> {
        if !self.closing_faces.is_empty() {
            return Err(unsupported_closing_faces());
        }
        let reverse_result = self.configuration()?.offset > 0.0;
        self.make_offset_shape()?;
        self.reverse_completed_result(reverse_result);
        Ok(())
    }

    fn reverse_completed_result(&mut self, reverse_result: bool) {
        if self.done && reverse_result {
            self.result_shape.reverse();
        }
    }
}

fn unsupported_closing_faces() -> Failure {
    Failure::with_message(
        "Standard_NotImplemented",
        "BRepOffset_MakeOffset::MakeThickSolid closing faces are not supported yet",
    )
}
