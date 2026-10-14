//! A final ordered span follows model transparency without clearing model depth.
use crate::{RenderError, RenderFrame, RenderLayerSpan, view_gui_layer::GuiLayerPacket};
pub(crate) fn append_gui_packet(
    frame: &mut RenderFrame,
    packet: GuiLayerPacket,
) -> Result<(), RenderError> {
    let mut validated = validated_gui_frame(frame, packet.layer)?;
    let begin = frame.draws.len();
    frame.gui_draw_start = Some(begin);
    frame.draws.append(&mut validated.draws);
    for span in validated.layers {
        frame.layers.push(RenderLayerSpan {
            draw_range: begin + span.draw_range.start..begin + span.draw_range.end,
            ..span
        });
    }
    Ok(())
}
fn validated_gui_frame(
    frame: &RenderFrame,
    layer: crate::RenderLayer,
) -> Result<RenderFrame, RenderError> {
    RenderFrame::with_layers(frame.width, frame.height, frame.clear_color, vec![layer])
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PrimitiveDraw, RenderLayer, RenderVertex, identity_clip};
    fn triangle(color: [f32; 4]) -> PrimitiveDraw {
        let vertices = vec![RenderVertex::new([0.; 3], color); 3];
        PrimitiveDraw::new(vertices, vec![0, 1, 2], identity_clip()).unwrap()
    }
    fn model_with_gui() -> RenderFrame {
        let blend = triangle([1., 0., 0., 0.5])
            .with_surface_blending(true)
            .unwrap();
        let model = RenderLayer::new(vec![blend], true, true, false);
        let mut frame = RenderFrame::with_layers(64, 64, [0.; 4], vec![model]).unwrap();
        let gui = RenderLayer::new(vec![triangle([0., 1., 0., 0.5])], false, false, false);
        let packet = GuiLayerPacket {
            extent: [64, 64],
            layer: gui,
        };
        append_gui_packet(&mut frame, packet).unwrap();
        frame
    }
    #[test]
    fn final_gui_follows_deferred_transparency_and_keeps_model_depth() {
        let frame = model_with_gui();
        assert!(frame.draws()[0].is_surface_blended());
        assert_eq!(
            frame.draws().last().unwrap().vertices()[0].color,
            [0., 1., 0., 0.5]
        );
        assert_eq!(
            frame.layers().last().unwrap().depth_policy(),
            (false, false, false)
        );
        assert_eq!(frame.layers()[1].depth_policy(), (true, false, false));
    }
}
