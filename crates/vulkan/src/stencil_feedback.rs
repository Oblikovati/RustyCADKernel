//! Only a completed submission advances the actual view's context; no view is retained here.
use crate::{RenderFrame, RenderStencilState};
use rustycad_tkernel::standard::SharedState;
use std::sync::Arc;
#[derive(Clone, Debug, Default)]
pub(crate) struct StencilFeedback(Arc<SharedState<RenderStencilState>>);
#[derive(Clone, Debug)]
pub(crate) struct StencilSnapshot {
    initial: RenderStencilState,
    feedback: StencilFeedback,
}
impl StencilFeedback {
    pub fn current(&self) -> RenderStencilState {
        *self.0.read()
    }
}
impl StencilSnapshot {
    pub fn commit(&self, ending: RenderStencilState) {
        *self.feedback.0.write() = ending;
    }
}
impl RenderFrame {
    pub(crate) fn attach_stencil_context(&mut self, feedback: &StencilFeedback) {
        self.stencil_context = Some(StencilSnapshot {
            initial: *feedback.0.read(),
            feedback: feedback.clone(),
        });
    }
    pub(crate) fn initial_stencil(&self, fallback: RenderStencilState) -> RenderStencilState {
        self.stencil_context
            .as_ref()
            .map_or(fallback, |snapshot| snapshot.initial)
    }
    /// Inspect this immutable frame's captured native context; `frame.stencil_state()`.
    pub fn stencil_state(&self) -> RenderStencilState {
        self.initial_stencil(Default::default())
    }
    /// Inspect ordered draw states without executing or committing them; `frame.stencil_draw_states()`.
    pub fn stencil_draw_states(&self) -> Vec<RenderStencilState> {
        crate::stencil_plan::StencilPlan::new(self, self.stencil_state()).during
    }
}
#[cfg(test)]
#[path = "stencil_feedback_tests.rs"]
mod tests;
