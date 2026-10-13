//! Context state survives frames; each submission captures the actual ordered draw transitions.
use crate::{RenderFrame, legacy_fragment_alpha::LegacyFragmentAlpha};
#[derive(Clone, Debug)]
pub(super) struct FragmentAlphaPlan {
    pub during: Vec<Option<f32>>,
    pub after: Vec<Option<f32>>,
    pub ending: LegacyFragmentAlpha,
}
impl FragmentAlphaPlan {
    pub fn new(frame: &RenderFrame, mut state: LegacyFragmentAlpha) -> Self {
        let mut during = Vec::with_capacity(frame.draws.len());
        let mut after = Vec::with_capacity(frame.draws.len());
        for draw in &frame.draws {
            during.push(state.advance(draw));
            after.push(state.0);
        }
        Self {
            during,
            after,
            ending: state,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn captured_context_and_frame_state_are_independent() {
        let draw = crate::PrimitiveDraw::new(
            vec![crate::RenderVertex::new([0.; 3], [1.; 4]); 3],
            vec![0, 1, 2],
            crate::identity_clip(),
        )
        .unwrap();
        let frame = RenderFrame::new(2, 2, [0.; 4], vec![draw]).unwrap();
        let previous = FragmentAlphaPlan::new(&frame, LegacyFragmentAlpha(Some(0.5)));
        let fresh = FragmentAlphaPlan::new(&frame, LegacyFragmentAlpha::default());
        assert_eq!(previous.during, [Some(0.5)]);
        assert_eq!(previous.after, [Some(0.5)]);
        assert_eq!(previous.ending, LegacyFragmentAlpha(Some(0.5)));
        assert_eq!(fresh.during, [None]);
    }
}
