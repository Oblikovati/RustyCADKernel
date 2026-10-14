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
        let mut plan = Self {
            during: Vec::with_capacity(frame.draws.len()),
            after: Vec::with_capacity(frame.draws.len()),
            ending: state,
        };
        for (index, draw) in frame.draws.iter().enumerate() {
            let gui = frame.gui_draw_start.is_some_and(|begin| index >= begin);
            plan.advance_draw(draw, &mut state, gui);
        }
        plan
    }
    fn advance_draw(
        &mut self,
        draw: &crate::PrimitiveDraw,
        state: &mut LegacyFragmentAlpha,
        gui: bool,
    ) {
        // GUI packets specify complete draw state; model compatibility alpha survives the overlay.
        if gui {
            *state = LegacyFragmentAlpha::default();
        }
        self.during.push(state.advance(draw));
        self.after.push(state.0);
        if !gui {
            self.ending = *state;
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
#[cfg(test)]
mod gui_state_tests {
    use super::*;
    use crate::{PrimitiveDraw, RenderLayer, RenderVertex, identity_clip};
    fn retained_model_with_gui(reset_material: bool) -> RenderFrame {
        let vertices = vec![RenderVertex::new([0.; 3], [0., 1., 0., 0.4]); 3];
        let model = PrimitiveDraw::new(vertices.clone(), vec![0, 1, 2], identity_clip()).unwrap();
        let mut gui = PrimitiveDraw::new(vertices, vec![0, 1, 2], identity_clip()).unwrap();
        gui.fixed_function_lighting = reset_material;
        let mut frame = RenderFrame::new(2, 2, [0.; 4], vec![model]).unwrap();
        let packet = crate::view_gui_layer::GuiLayerPacket {
            extent: [2, 2],
            layer: RenderLayer::new(vec![gui], false, false, false),
        };
        crate::view_gui_composition::append_gui_packet(&mut frame, packet).unwrap();
        frame
    }
    #[test]
    fn retained_model_mask_does_not_reject_translucent_gui() {
        let frame = retained_model_with_gui(false);
        let plan = FragmentAlphaPlan::new(&frame, LegacyFragmentAlpha(Some(0.9)));
        assert_eq!(plan.during, [Some(0.9), None]);
        assert_eq!(plan.ending, LegacyFragmentAlpha(Some(0.9)));
    }
    #[test]
    fn gui_material_reset_does_not_change_next_model_frame() {
        let frame = retained_model_with_gui(true);
        let plan = FragmentAlphaPlan::new(&frame, LegacyFragmentAlpha(Some(0.9)));
        assert_eq!(plan.during, [Some(0.9), None]);
        assert_eq!(plan.ending, LegacyFragmentAlpha(Some(0.9)));
        let next = RenderFrame::new(2, 2, [0.; 4], frame.draws[..1].to_vec()).unwrap();
        let next_plan = FragmentAlphaPlan::new(&next, plan.ending);
        assert_eq!(next_plan.during, [Some(0.9)]);
    }
}
