//! A native cap aspect classifies its entire mask batch for the transparent pass.
use crate::{PrimitiveDraw, RenderCappingPass};
pub(crate) fn order_layer_draws(draws: &mut [PrimitiveDraw]) {
    // Preserve malformed input until RenderFrame rejects it; sorting must not hide interruptions.
    if crate::capping_pass::validate_capping_sequence(draws).is_err() {
        return;
    }
    classify_capping_batches(draws);
    // Native BLEND_UNORDERED defers whole cap batches while retaining their submission order.
    draws.sort_by_key(PrimitiveDraw::is_surface_blended);
}
pub(crate) fn classify_capping_batches(draws: &mut [PrimitiveDraw]) {
    let mut start = None;
    for index in 0..draws.len() {
        match draws[index].capping_pass {
            RenderCappingPass::MaskStart => start = Some(index),
            RenderCappingPass::MaskContinue => {}
            RenderCappingPass::Surface => assign_cap_batch_alpha(draws, start.take(), index),
            RenderCappingPass::None => start = None,
        }
    }
}
fn assign_cap_batch_alpha(draws: &mut [PrimitiveDraw], begin: Option<usize>, end: usize) {
    let Some(begin) = begin else {
        return;
    };
    let blended = draws[end].surface_blend;
    for draw in &mut draws[begin..=end] {
        draw.surface_blend = blended;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn cap_phase(pass: RenderCappingPass, blended: bool) -> PrimitiveDraw {
        let mut draw = PrimitiveDraw::new(
            vec![crate::RenderVertex::new([0.; 3], [1.; 4]); 3],
            vec![0, 1, 2],
            crate::identity_clip(),
        )
        .unwrap();
        draw.capping_pass = pass;
        draw.surface_blend = blended;
        draw
    }
    #[test]
    fn surface_alpha_defers_the_whole_mask_batch_and_not_adjacent_geometry() {
        use RenderCappingPass::*;
        let draws = vec![
            cap_phase(MaskStart, false),
            cap_phase(MaskContinue, false),
            cap_phase(Surface, true),
            cap_phase(None, false),
        ];
        let layer = crate::RenderLayer::new(draws, true, true, false);
        let phases: Vec<_> = layer.draws.iter().map(|draw| draw.capping_pass).collect();
        let blended: Vec<_> = layer.draws.iter().map(|draw| draw.surface_blend).collect();
        assert_eq!(phases, [None, MaskStart, MaskContinue, Surface]);
        assert_eq!(blended, [false, true, true, true]);
    }
    #[test]
    fn opaque_cap_does_not_defer_its_transparent_source_mask() {
        use RenderCappingPass::*;
        let mut draws = vec![cap_phase(MaskStart, true), cap_phase(Surface, false)];
        classify_capping_batches(&mut draws);
        assert!(draws.iter().all(|draw| !draw.surface_blend));
    }
    #[test]
    fn deferred_ordering_cannot_hide_an_interrupted_stencil_batch() {
        use RenderCappingPass::*;
        let draws = vec![
            cap_phase(MaskStart, false),
            cap_phase(None, true),
            cap_phase(Surface, false),
        ];
        let layer = crate::RenderLayer::new(draws, true, true, false);
        assert!(crate::RenderFrame::with_layers(4, 4, [0.; 4], vec![layer]).is_err());
    }
}
