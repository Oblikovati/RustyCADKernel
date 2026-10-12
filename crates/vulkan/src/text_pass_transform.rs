//! Native setupMatrix consumes the CURRENT local-plane world-view on every text pass (3d097a0).
use super::*;
impl TextTransform {
    /// Preserve native cumulative local-plane state; `transform.pass_clips(text, aspect, 16, offsets)` retains ordered decoration and foreground.
    pub(crate) fn pass_clips(
        &self,
        text: &Graphic3dText,
        aspect: &dyn Graphic3dAspectsSource,
        size: u32,
        offsets: &[[f64; 2]],
    ) -> Result<Vec<[f32; 16]>, RenderError> {
        if self.pixel_coordinates {
            return Ok(self.pixel_pass_clips(text, aspect, offsets));
        }
        if text.has_plane() && !text.has_own_anchor_point() {
            return self.local_pass_clips(text, aspect, size, offsets.len());
        }
        offsets
            .iter()
            .map(|offset| self.ordinary_pass_clip(text, aspect, size, *offset))
            .collect()
    }
    fn ordinary_pass_clip(
        &self,
        text: &Graphic3dText,
        aspect: &dyn Graphic3dAspectsSource,
        size: u32,
        offset: [f64; 2],
    ) -> Result<[f32; 16], RenderError> {
        if offset == [0.; 2] {
            return self.clip(text, aspect, size);
        }
        self.clip_offset(text, aspect, size, offset)
    }
    fn local_pass_clips(
        &self,
        text: &Graphic3dText,
        aspect: &dyn Graphic3dAspectsSource,
        size: u32,
        count: usize,
    ) -> Result<Vec<[f32; 16]>, RenderError> {
        let local = self.local_pass_model(text, aspect, size)?;
        let mut current = self.model_view;
        let mut clips = Vec::with_capacity(count);
        for pass in 0..count {
            self.advance_local_pass(&mut current, local, pass);
            clips.push((self.projection * current).data().map(|v| v as f32));
        }
        Ok(clips)
    }
    pub(super) fn advance_local_pass(
        &self,
        current: &mut Mat4<f64>,
        local: Mat4<f64>,
        pass: usize,
    ) {
        if pass != 0 {
            *current *= self.model_world;
        }
        *current = Mat4::from_other(&Mat4::<f32>::from_other(&(*current * local)));
    }
    fn local_pass_model(
        &self,
        text: &Graphic3dText,
        aspect: &dyn Graphic3dAspectsSource,
        size: u32,
    ) -> Result<Mat4<f64>, RenderError> {
        let mut local = super::plane_model(text, [0.; 3]);
        if !aspect.is_text_zoomable() {
            let scale = self.pixel_scale(self.projected_anchor(text)?, size)?;
            tr::scale(&mut local, [scale; 3]);
        }
        Ok(local)
    }
}
