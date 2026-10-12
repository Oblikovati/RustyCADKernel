//! Native 2D setupMatrix uses pixel positions, Y reversal and angle, without size/zoom scaling.
use super::*;
impl TextTransform {
    pub(super) fn pixel_clip(
        &self,
        text: &Graphic3dText,
        aspect: &dyn Graphic3dAspectsSource,
        offset: [f64; 2],
    ) -> [f32; 16] {
        let model = pixel_text_model(text, aspect, offset);
        let model = if text.has_plane() && !text.has_own_anchor_point() {
            self.model_view * model
        } else {
            model
        };
        // Native only uploads the computed OwnAnchor projection for !Is2D (OCCT3d097a0).
        (self.projection * model).data().map(|v| v as f32)
    }
    pub(super) fn pixel_pass_clips(
        &self,
        text: &Graphic3dText,
        aspect: &dyn Graphic3dAspectsSource,
        offsets: &[[f64; 2]],
    ) -> Vec<[f32; 16]> {
        if !text.has_plane() || text.has_own_anchor_point() {
            return offsets
                .iter()
                .map(|offset| self.pixel_clip(text, aspect, *offset))
                .collect();
        }
        self.pixel_local_pass_clips(text, aspect, offsets)
    }
    fn pixel_local_pass_clips(
        &self,
        text: &Graphic3dText,
        aspect: &dyn Graphic3dAspectsSource,
        offsets: &[[f64; 2]],
    ) -> Vec<[f32; 16]> {
        let mut current = self.model_view;
        offsets
            .iter()
            .enumerate()
            .map(|(pass, offset)| {
                let local = pixel_text_model(text, aspect, *offset);
                self.pixel_current_pass_clip(&mut current, local, pass)
            })
            .collect()
    }
    fn pixel_current_pass_clip(
        &self,
        current: &mut Mat4<f64>,
        local: Mat4<f64>,
        pass: usize,
    ) -> [f32; 16] {
        self.advance_local_pass(current, local, pass);
        (self.projection * *current).data().map(|v| v as f32)
    }
}
fn pixel_text_model(
    text: &Graphic3dText,
    aspect: &dyn Graphic3dAspectsSource,
    offset: [f64; 2],
) -> Mat4<f64> {
    let point = text.position();
    let mut model = Mat4::identity();
    tr::translate(
        &mut model,
        [point.x() + offset[0], point.y() + offset[1], 0.],
    );
    tr::scale(&mut model, [1., -1., 1.]);
    tr::rotate(&mut model, f64::from(aspect.text_angle()), [0., 0., 1.]);
    model
}
