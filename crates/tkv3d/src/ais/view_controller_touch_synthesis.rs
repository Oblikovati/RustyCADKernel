//! Native touch synthesis preserves pending render requests and contact origins.
use super::{AisInteractiveContext, AisViewController};
use crate::v3d::V3dViewSource;
use rustycad_tkernel::standard::{Failure, Handle};
impl AisViewController {
    /// Synthesize render gestures from contacts; `controller.flush_gestures(&context, &view)?`.
    pub fn flush_gestures(
        &mut self,
        _context: &Handle<AisInteractiveContext>,
        view: &Handle<dyn V3dViewSource>,
    ) -> Result<(), Failure> {
        let count = self.input_base.touch_points.extent();
        if self.touch_dispatch.last_touch_count != count {
            self.touch_dispatch.last_touch_count = count;
            self.render_input.is_new_gesture = true;
        }
        match count {
            1 => self.synthesize_touch_rotation(view),
            2 => self.synthesize_touch_pinch(view),
            _ => Ok(()),
        }
    }
    pub(super) fn advance_synthesized_touch_origins(&mut self, count: usize) {
        for index in 1..=count {
            let touch = self
                .input_base
                .touch_points
                .change_from_index(index)
                .expect("active touch");
            touch.from = touch.to;
        }
    }
}
pub(super) fn invalidate_touch_view(view: &Handle<dyn V3dViewSource>) -> Result<(), Failure> {
    let view = view.get().ok_or_else(|| {
        Failure::new(
            "touch gesture view=null; expected nonnull V3dView when starting rotation or pan",
        )
    })?;
    view.invalidate();
    Ok(())
}
