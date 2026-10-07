//! The non-virtual state accessors of `Graphic3d_CView`: identification, activity, shading,
//! visualization, layer target, rendering parameters, background, texture environment and
//! unit factor.
use super::{
    Graphic3dCView, Graphic3dCubeMap, Graphic3dCubeMapOrder, Graphic3dCubeMapPacked,
    Graphic3dRenderingParams, Graphic3dStructureManager, Graphic3dTextureEnv, Graphic3dTextureMap,
    Graphic3dTypeOfBackfacingModel, Graphic3dTypeOfBackground, Graphic3dTypeOfShadingModel,
    Graphic3dTypeOfVisualization, Graphic3dZLayerId,
};
use crate::aspect::{AspectSkydomeBackground, AspectXRSessionMembers as _};
use rustycad_tkernel::standard::exceptions::program_error;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;

/// The non-virtual state members of every view.
pub trait Graphic3dCViewMembers: Graphic3dCView {
    /// `Identification`; `view.identification()`.
    fn identification(&self) -> i32 {
        self.cview_base().fields.read().id
    }
    /// `IsActive`; `view.is_active()`.
    fn is_active(&self) -> bool {
        self.cview_base().fields.read().is_active
    }
    /// `IsRemoved`; `view.is_removed()`.
    fn is_removed(&self) -> bool {
        self.cview_base().fields.read().is_removed
    }
    /// `ShadingModel`; `view.shading_model()`.
    fn shading_model(&self) -> Graphic3dTypeOfShadingModel {
        self.cview_base().fields.read().render_params.shading_model
    }
    /// `SetShadingModel`: the default model raises `Standard_ProgramError`;
    /// `view.set_shading_model(Graphic3dTypeOfShadingModel::Phong)?`.
    fn set_shading_model(&self, model: Graphic3dTypeOfShadingModel) -> Result<(), Failure> {
        if model == Graphic3dTypeOfShadingModel::Default {
            return Err(program_error(&format!(
                "Graphic3d_CView::SetShadingModel() - attempt to set invalid Shading Model: {model:?}"
            )));
        }
        self.cview_base().fields.write().render_params.shading_model = model;
        Ok(())
    }
    /// `BackfacingModel`; `view.backfacing_model()`.
    fn backfacing_model(&self) -> Graphic3dTypeOfBackfacingModel {
        self.cview_base().fields.read().backfacing
    }
    /// `SetBackfacingModel`; `view.set_backfacing_model(Graphic3dTypeOfBackfacingModel::DoubleSided)`.
    fn set_backfacing_model(&self, model: Graphic3dTypeOfBackfacingModel) {
        self.cview_base().fields.write().backfacing = model;
    }
    /// `VisualizationType`; `view.visualization_type()`.
    fn visualization_type(&self) -> Graphic3dTypeOfVisualization {
        self.cview_base().fields.read().visualization
    }
    /// `SetVisualizationType`; `view.set_visualization_type(Graphic3dTypeOfVisualization::Shading)`.
    fn set_visualization_type(&self, visualization: Graphic3dTypeOfVisualization) {
        self.cview_base().fields.write().visualization = visualization;
    }
    /// `ZLayerTarget`; `view.z_layer_target()`.
    fn z_layer_target(&self) -> Graphic3dZLayerId {
        self.cview_base().fields.read().z_layer_target
    }
    /// `SetZLayerTarget`; `view.set_z_layer_target(0)`.
    fn set_z_layer_target(&self, target: Graphic3dZLayerId) {
        self.cview_base().fields.write().z_layer_target = target;
    }
    /// `ZLayerRedrawMode`; `view.z_layer_redraw_mode()`.
    fn z_layer_redraw_mode(&self) -> bool {
        self.cview_base().fields.read().z_layer_redraw_mode
    }
    /// `SetZLayerRedrawMode`; `view.set_z_layer_redraw_mode(true)`.
    fn set_z_layer_redraw_mode(&self, mode: bool) {
        self.cview_base().fields.write().z_layer_redraw_mode = mode;
    }
    /// `ComputedMode`; `view.computed_mode()`.
    fn computed_mode(&self) -> bool {
        self.cview_base().fields.read().is_in_computed_mode
    }
    /// `StructureManager`; `view.structure_manager()`.
    fn structure_manager(&self) -> Handle<Graphic3dStructureManager> {
        self.cview_base().fields.read().structure_manager.clone()
    }
    /// `RenderingParams` (a copy); `view.rendering_params().stats.stats_nb_frames`.
    fn rendering_params(&self) -> Graphic3dRenderingParams {
        self.cview_base().fields.read().render_params.clone()
    }
    /// `ChangeRenderingParams`: edits the parameters; `view.change_rendering_params(&mut |p| p.nb_msaa_samples = 4)`.
    fn change_rendering_params(&self, edit: &mut dyn FnMut(&mut Graphic3dRenderingParams)) {
        edit(&mut self.cview_base().fields.write().render_params);
    }
    /// `BackgroundImage`; `view.background_image()`.
    fn background_image(&self) -> Handle<dyn Graphic3dTextureMap> {
        self.cview_base().fields.read().background_image.clone()
    }
    /// `BackgroundCubeMap`; `view.background_cube_map()`.
    fn background_cube_map(&self) -> Handle<dyn Graphic3dCubeMap> {
        self.cview_base().fields.read().cube_map_background.clone()
    }
    /// `IBLCubeMap`; `view.ibl_cube_map()`.
    fn ibl_cube_map(&self) -> Handle<dyn Graphic3dCubeMap> {
        self.cview_base().fields.read().cube_map_ibl.clone()
    }
    /// `BackgroundType`; `view.background_type()`.
    fn background_type(&self) -> Graphic3dTypeOfBackground {
        self.cview_base().fields.read().background_type
    }
    /// `SetBackgroundType`; `view.set_background_type(Graphic3dTypeOfBackground::Cubemap)`.
    fn set_background_type(&self, background_type: Graphic3dTypeOfBackground) {
        self.cview_base().fields.write().background_type = background_type;
    }
    /// `BackgroundSkydome`; `view.background_skydome()`.
    fn background_skydome(&self) -> AspectSkydomeBackground {
        self.cview_base().fields.read().skydome_aspect
    }
    /// `SetBackgroundSkydome`: a cubemap background generated from the skydome; with an
    /// environment, the image based lighting is switched off and on again;
    /// `view.set_background_skydome(&skydome, true)`.
    fn set_background_skydome(&self, aspect: &AspectSkydomeBackground, to_update_pbr_env: bool) {
        let cube_map = Graphic3dCubeMapPacked::from_file(
            &AsciiString::default(),
            &Graphic3dCubeMapOrder::default_order(),
        );
        let has_ibl = {
            let mut fields = self.cview_base().fields.write();
            fields.to_update_skydome = true;
            fields.skydome_aspect = *aspect;
            fields.cube_map_background = Handle::new(cube_map).cast();
            fields.background_type = Graphic3dTypeOfBackground::Cubemap;
            !fields.cube_map_ibl.is_null()
        };
        if to_update_pbr_env && has_ibl {
            self.set_image_based_lighting(false);
            self.set_image_based_lighting(true);
        }
    }
    /// `TextureEnv`; `view.texture_env()`.
    fn texture_env(&self) -> Handle<Graphic3dTextureEnv> {
        self.cview_base().fields.read().texture_env_data.clone()
    }
    /// `UnitFactor`; `view.unit_factor()`.
    fn unit_factor(&self) -> f64 {
        self.cview_base().fields.read().unit_factor
    }
    /// `SetUnitFactor`: a non-positive factor raises `Standard_ProgramError`; the XR session
    /// gets the factor too; `view.set_unit_factor(0.001)?`.
    fn set_unit_factor(&self, factor: f64) -> Result<(), Failure> {
        if factor <= 0.0 {
            return Err(program_error(&format!(
                "Graphic3d_CView::SetUnitFactor() - invalid unit factor: {factor}; expected > 0"
            )));
        }
        let session = {
            let mut fields = self.cview_base().fields.write();
            fields.unit_factor = factor;
            fields.xr_session.clone()
        };
        if let Some(session) = session.get() {
            session.set_unit_factor(factor);
        }
        Ok(())
    }
}
impl<T: Graphic3dCView + ?Sized> Graphic3dCViewMembers for T {}
