//! Compatibility alpha state belongs to the native view context, not its shared renderer.
use crate::{RenderFrame, legacy_fragment_alpha::LegacyFragmentAlpha};
use rustycad_tkernel::standard::SharedState;
use std::sync::Arc;

#[derive(Clone, Debug, Default)]
pub(crate) struct FragmentAlphaFeedback(Arc<SharedState<LegacyFragmentAlpha>>);

#[derive(Clone, Debug)]
pub(crate) struct FragmentAlphaSnapshot {
    initial: LegacyFragmentAlpha,
    feedback: FragmentAlphaFeedback,
}

impl FragmentAlphaSnapshot {
    pub fn commit(&self, ending: LegacyFragmentAlpha) {
        *self.feedback.0.write() = ending;
    }
}

impl RenderFrame {
    pub(crate) fn attach_fragment_alpha(&mut self, feedback: &FragmentAlphaFeedback) {
        self.fragment_alpha_context = Some(FragmentAlphaSnapshot {
            initial: *feedback.0.read(),
            feedback: feedback.clone(),
        });
    }

    pub(crate) fn initial_fragment_alpha(
        &self,
        fallback: LegacyFragmentAlpha,
    ) -> LegacyFragmentAlpha {
        self.fragment_alpha_context
            .as_ref()
            .map_or(fallback, |snapshot| snapshot.initial)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completed_state_survives_same_view_without_changing_captured_or_adjacent_frames() {
        let first = FragmentAlphaFeedback::default();
        let second = FragmentAlphaFeedback::default();
        let mut captured = RenderFrame::new(2, 2, [0.; 4], vec![]).unwrap();
        captured.attach_fragment_alpha(&first);
        captured
            .fragment_alpha_context
            .as_ref()
            .unwrap()
            .commit(LegacyFragmentAlpha(Some(0.5)));
        assert_eq!(
            captured.initial_fragment_alpha(LegacyFragmentAlpha(Some(0.1))),
            LegacyFragmentAlpha(None)
        );
        let mut next = captured.clone();
        next.attach_fragment_alpha(&first);
        assert_eq!(
            next.initial_fragment_alpha(Default::default()),
            LegacyFragmentAlpha(Some(0.5))
        );
        next.attach_fragment_alpha(&second);
        assert_eq!(
            next.initial_fragment_alpha(Default::default()),
            LegacyFragmentAlpha(None)
        );
    }

    #[test]
    fn standalone_frames_use_their_renderer_context_without_a_retained_view_sink() {
        let frame = RenderFrame::new(2, 2, [0.; 4], vec![]).unwrap();
        assert_eq!(
            frame.initial_fragment_alpha(LegacyFragmentAlpha(Some(0.25))),
            LegacyFragmentAlpha(Some(0.25))
        );
        assert!(frame.fragment_alpha_context.is_none());
    }
}
