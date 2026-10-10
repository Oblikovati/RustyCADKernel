//! Retained text uses the current aspect and view resolution on every extraction.
use crate::{
    Graphic3dDrawGroup, PrimitiveDraw, RenderError, RenderVertex, TextLayoutSource, TexturedStyle,
    error::rejected,
};
use rustycad_tkernel::{ncollection::Vec3, standard::Handle};
use rustycad_tkservice::{aspect::*, graphic3d::*};
pub(crate) struct TextSceneContext<'a> {
    pub group: &'a Graphic3dDrawGroup,
    pub camera: &'a Graphic3dCamera,
    pub extent: Option<[u32; 2]>,
    pub structure: &'a Graphic3dCStructureFields,
    pub rendering: &'a Graphic3dRenderingParams,
    pub fonts: &'a dyn TextLayoutSource,
}
impl TextSceneContext<'_> {
    pub fn draws(
        &self,
        text: &Graphic3dText,
        aspects: &Handle<dyn Graphic3dAspectsSource>,
    ) -> Result<Vec<PrimitiveDraw>, RenderError> {
        let aspect = aspects
            .get()
            .ok_or_else(|| rejected("null text aspect", "a retained text style"))?;
        validate_text_style(aspect)?;
        let Some(layout) = self.fonts.layout(text, aspect, self.rendering)? else {
            return Ok(vec![]);
        };
        let clip = self.transform()?.clip(text, aspect, layout.point_size)?;
        let color = aspect.interior_color_rgba();
        let rgb = color.rgb();
        let rgba = [
            rgb.red() as f32,
            rgb.green() as f32,
            rgb.blue() as f32,
            color.alpha(),
        ];
        let vertices = layout
            .positions
            .iter()
            .map(|p| RenderVertex::new([p[0], p[1], 0.], rgba))
            .collect();
        // Graphic3d_ShaderManager::getStdProgramFont at 3d097a0 hard-codes this cutoff.
        let style = TexturedStyle::new(layout.atlas, layout.uv, 0.285)?
            .with_alpha_blending(aspect.alpha_mode() == Graphic3dAlphaMode::MaskBlend);
        let mut draw = PrimitiveDraw::textured(vertices, layout.indices, clip, style)?;
        draw.annotation = aspect.text_style() == AspectTypeOfStyleText::Annotation;
        Ok(vec![draw])
    }
    fn transform(&self) -> Result<crate::text_transform::TextTransform, RenderError> {
        let extent = self
            .extent
            .ok_or_else(|| rejected(self.extent, "a viewport for text rendering"))?;
        let (projection, model_view) = crate::persistent_transform::text_matrices(
            self.group,
            self.camera,
            extent,
            self.structure,
        )?;
        let mut camera_rotation = self.camera.orientation_matrix();
        camera_rotation.set_column3(3, Vec3::new(0., 0., 0.));
        Ok(crate::text_transform::TextTransform {
            projection,
            model_view,
            camera_rotation,
            viewport: [0, 0, extent[0] as i32, extent[1] as i32],
        })
    }
}
fn validate_text_style(aspect: &dyn Graphic3dAspectsSource) -> Result<(), RenderError> {
    if aspect.text_display_type() != AspectTypeOfDisplayText::Normal
        || !aspect.shader_program().is_null()
    {
        return Err(rejected(
            (
                aspect.text_display_type(),
                aspect.shader_program().object_address(),
            ),
            "normal text display without a custom shader in this slice",
        ));
    }
    Ok(())
}
