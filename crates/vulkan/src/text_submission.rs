//! Retained text uses the current aspect and view resolution on every extraction.
use crate::{Graphic3dDrawGroup, PrimitiveDraw, RenderError, TextLayoutSource, error::rejected};
use rustycad_tkernel::{
    ncollection::{Mat4, Vec3},
    standard::Handle,
};
use rustycad_tkservice::{aspect::*, graphic3d::*};
#[derive(Clone, Copy)]
pub(crate) struct TextSceneContext<'a> {
    pub group: &'a Graphic3dDrawGroup,
    pub camera: &'a Graphic3dCamera,
    pub extent: Option<[u32; 2]>,
    pub structure: &'a Graphic3dCStructureFields,
    pub rendering: &'a Graphic3dRenderingParams,
    pub fonts: &'a dyn TextLayoutSource,
    pub model_override: Option<Mat4<f32>>,
    pub clipping: Option<&'a crate::RenderClipping>,
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
        crate::text_drawing::lower_text_draws(text, aspect, &layout, &self.transform()?)
    }

    fn transform(&self) -> Result<crate::text_transform::TextTransform, RenderError> {
        let extent = self
            .extent
            .ok_or_else(|| rejected(self.extent, "a viewport for text rendering"))?;
        let (projection, model_view) = self.text_scene_matrices(extent)?;
        let mut camera_rotation = self.camera.orientation_matrix();
        camera_rotation.set_column3(3, Vec3::new(0., 0., 0.));
        Ok(crate::text_transform::TextTransform {
            projection,
            model_view,
            camera_rotation,
            viewport: [0, 0, extent[0] as i32, extent[1] as i32],
        })
    }
    fn text_scene_matrices(&self, extent: [u32; 2]) -> Result<(Mat4<f64>, Mat4<f64>), RenderError> {
        let (mut projection, mut model_view) = crate::persistent_transform::text_matrices(
            self.group,
            self.camera,
            extent,
            self.structure,
        )?;
        if let Some(model) = self.model_override {
            let (_, view) = crate::persistent_transform::lighting_matrices(self)?;
            // OpenGl_Text promotes the float GL matrices before multiplying (3d097a0).
            model_view = Mat4::<f64>::from_other(&Mat4::<f32>::from_data(view))
                * Mat4::<f64>::from_other(&model);
            projection = Mat4::from_other(&Mat4::<f32>::from_other(&projection));
        }
        Ok((projection, model_view))
    }
}
fn validate_text_style(aspect: &dyn Graphic3dAspectsSource) -> Result<(), RenderError> {
    if !matches!(
        aspect.text_display_type(),
        AspectTypeOfDisplayText::Normal
            | AspectTypeOfDisplayText::Shadow
            | AspectTypeOfDisplayText::Subtitle
    ) || !aspect.shader_program().is_null()
    {
        return Err(rejected(
            (
                aspect.text_display_type(),
                aspect.shader_program().object_address(),
            ),
            "Normal, Shadow or Subtitle text display without a custom shader",
        ));
    }
    Ok(())
}
