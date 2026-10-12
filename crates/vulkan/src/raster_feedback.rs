//! Native fallback belongs to a view; immutable frames capture it without reading future state.
use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};
#[derive(Clone, Debug, Default)]
pub(crate) struct RasterFailureFeedback(Arc<AtomicU8>);
impl RasterFailureFeedback {
    pub fn snapshot(&self) -> u8 {
        self.0.load(Ordering::Acquire)
    }
    pub fn disable_multisampling(&self) {
        self.0.fetch_or(1, Ordering::AcqRel);
    }
    pub fn disable_oit(&self) {
        self.0.fetch_or(2, Ordering::AcqRel);
    }
}
impl crate::RenderFrame {
    pub(crate) fn attach_raster_feedback(&mut self, feedback: &RasterFailureFeedback) {
        self.raster.disabled_capabilities = feedback.snapshot();
        self.raster_feedback = feedback.clone();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn captured_frames_keep_old_state_and_view_feedback_does_not_cross_views() {
        let first = RasterFailureFeedback::default();
        let second = RasterFailureFeedback::default();
        let mut frame = crate::RenderFrame::new(2, 2, [0.; 4], vec![]).unwrap();
        frame.attach_raster_feedback(&first);
        first.disable_multisampling();
        assert_eq!(frame.raster.disabled_capabilities, 0);
        assert_eq!(first.snapshot(), 1);
        let mut next = frame.clone();
        next.attach_raster_feedback(&first);
        assert_eq!(next.raster.disabled_capabilities, 1);
        assert_eq!(second.snapshot(), 0);
        first.disable_oit();
        assert_eq!(first.snapshot(), 3);
        assert_eq!(next.raster.disabled_capabilities, 1);
    }
}
