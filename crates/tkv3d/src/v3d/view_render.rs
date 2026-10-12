//! The rendering members of `V3d_View`: backgrounds and environment, visualization and
//! shading, lights, clip planes, computed mode, back faces, rendering parameters and the
//! driver statistics.
use super::{V3dTypeOfVisualization, V3dView};
use rustycad_tkernel::quantity::{Color, ColorSpace};
use rustycad_tkernel::standard::exceptions::type_mismatch;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::aspect::{
    AspectBackground, AspectFillMethod, AspectGradientBackground, AspectGradientFillMethod,
    AspectSkydomeBackground,
};
use rustycad_tkservice::graphic3d::{
    Graphic3dCLight, Graphic3dCViewComputed as _, Graphic3dCViewMembers as _, Graphic3dClipPlane,
    Graphic3dCubeMap, Graphic3dDiagnosticInfo, Graphic3dFrameStatsDict,
    Graphic3dGraphicDriverMembers as _, Graphic3dLightSet, Graphic3dRenderingParams,
    Graphic3dRenderingParamsFrustumCulling, Graphic3dSequenceOfHClipPlane, Graphic3dTexture2D,
    Graphic3dTextureEnv, Graphic3dTextureMap, Graphic3dTextureMapMembers as _,
    Graphic3dTypeOfBackfacingModel, Graphic3dTypeOfShadingModel, Graphic3dTypeOfVisualization,
};

impl V3dView {
    /// Redraws when the immediate mode is on or when asked.
    fn redraw_if(&self, to_update: bool) {
        if self.state.read().immediate_update || to_update {
            self.redraw();
        }
    }
    /// `SetBackgroundColor(Type, V1, V2, V3)`: components clamped to `[0, 1]`.
    /// Example: `view.set_background_color_values(ColorSpace::Rgb, 0.1, 0.2, 0.3)?`.
    pub fn set_background_color_values(
        &self,
        space: ColorSpace,
        v1: f64,
        v2: f64,
        v3: f64,
    ) -> Result<(), Failure> {
        // std::max(std::min(v, 1.0), 0.0) keeps a NaN natively, unlike f64::clamp.
        let clamp =
            |v: f64| super::view_mapping::std_max(super::view_mapping::std_min(v, 1.0), 0.0);
        self.set_background_color(Color::new(clamp(v1), clamp(v2), clamp(v3), space)?);
        Ok(())
    }
    /// `SetBackgroundColor(Color)`; `view.set_background_color(black)`.
    pub fn set_background_color(&self, color: Color) {
        self.view().set_background(&AspectBackground::new(color));
        self.redraw_if(false);
    }
    /// `SetBgGradientColors`; `view.set_bg_gradient_colors(c1, c2, method, false)`.
    pub fn set_bg_gradient_colors(
        &self,
        color1: Color,
        color2: Color,
        fill_style: AspectGradientFillMethod,
        to_update: bool,
    ) {
        self.view()
            .set_gradient_background(&AspectGradientBackground::new(color1, color2, fill_style));
        self.redraw_if(to_update);
    }
    /// `SetBgGradientStyle`: keeps the colors; `view.set_bg_gradient_style(method, false)`.
    pub fn set_bg_gradient_style(&self, fill_style: AspectGradientFillMethod, to_update: bool) {
        let (color1, color2) = self.gradient_background().colors();
        self.set_bg_gradient_colors(color1, color2, fill_style, to_update);
    }
    /// `SetBackgroundImage(file)`: a non-modulated 2D texture of a file.
    /// Example: `view.set_background_image_file("bg.png", AspectFillMethod::Centered, false)`.
    pub fn set_background_image_file(
        &self,
        file_name: &str,
        fill_style: AspectFillMethod,
        to_update: bool,
    ) {
        let texture = Graphic3dTexture2D::from_file(&AsciiString::from(file_name));
        texture.disable_modulate();
        self.set_background_image(&Handle::new(texture).cast(), fill_style, to_update);
    }
    /// `SetBackgroundImage(texture)`; `view.set_background_image(&texture, AspectFillMethod::Stretch, false)`.
    pub fn set_background_image(
        &self,
        texture: &Handle<dyn Graphic3dTextureMap>,
        fill_style: AspectFillMethod,
        to_update: bool,
    ) {
        self.view().set_background_image(texture, true);
        self.view().set_background_image_style(fill_style);
        self.redraw_if(to_update);
    }
    /// `SetBgImageStyle`; `view.set_bg_image_style(AspectFillMethod::Tiled, false)`.
    pub fn set_bg_image_style(&self, fill_style: AspectFillMethod, to_update: bool) {
        self.view().set_background_image_style(fill_style);
        self.redraw_if(to_update);
    }
    /// `SetBackgroundCubeMap`; `view.set_background_cube_map(&cube_map, true, false)`.
    pub fn set_background_cube_map(
        &self,
        cube_map: &Handle<dyn Graphic3dCubeMap>,
        to_update_pbr_env: bool,
        to_update: bool,
    ) {
        self.view()
            .set_background_image(&cube_map.clone().cast(), to_update_pbr_env);
        self.redraw_if(to_update);
    }
    /// `BackgroundSkydome`; `view.background_skydome()`.
    pub fn background_skydome(&self) -> AspectSkydomeBackground {
        self.view().background_skydome()
    }
    /// `SetBackgroundSkydome`; `view.set_background_skydome(&aspect, true)`.
    pub fn set_background_skydome(
        &self,
        aspect: &AspectSkydomeBackground,
        to_update_pbr_env: bool,
    ) {
        self.view()
            .set_background_skydome(aspect, to_update_pbr_env);
    }
    /// `IsImageBasedLighting`: whether an IBL cube map is set; `view.is_image_based_lighting()`.
    pub fn is_image_based_lighting(&self) -> bool {
        !self.view().ibl_cube_map().is_null()
    }
    /// `SetImageBasedLighting`; `view.set_image_based_lighting(true, false)`.
    pub fn set_image_based_lighting(&self, to_enable_ibl: bool, to_update: bool) {
        self.view().set_image_based_lighting(to_enable_ibl);
        self.redraw_if(to_update);
    }
    /// `GeneratePBREnvironment`: enables image based lighting; `view.generate_pbr_environment(false)`.
    pub fn generate_pbr_environment(&self, to_update: bool) {
        self.set_image_based_lighting(true, to_update);
    }
    /// `ClearPBREnvironment`; native quirk kept: it enables image based lighting too.
    /// Example: `view.clear_pbr_environment(false)`.
    pub fn clear_pbr_environment(&self, to_update: bool) {
        self.set_image_based_lighting(true, to_update);
    }
    /// `SetTextureEnv`; `view.set_texture_env(&texture)`.
    pub fn set_texture_env(&self, texture: &Handle<Graphic3dTextureEnv>) {
        self.view().set_texture_env(texture);
        self.redraw_if(false);
    }
    /// `TextureEnv`; `view.texture_env()`.
    pub fn texture_env(&self) -> Handle<Graphic3dTextureEnv> {
        self.view().texture_env()
    }
    /// `SetVisualization`; `view.set_visualization(V3dTypeOfVisualization::Wireframe)`.
    pub fn set_visualization(&self, kind: V3dTypeOfVisualization) {
        let visualization = match kind {
            V3dTypeOfVisualization::Wireframe => Graphic3dTypeOfVisualization::Wireframe,
            V3dTypeOfVisualization::Zbuffer => Graphic3dTypeOfVisualization::Shading,
        };
        self.view().set_visualization_type(visualization);
        self.redraw_if(false);
    }
    /// `Visualization`; `view.visualization()`.
    pub fn visualization(&self) -> V3dTypeOfVisualization {
        match self.view().visualization_type() {
            Graphic3dTypeOfVisualization::Wireframe => V3dTypeOfVisualization::Wireframe,
            Graphic3dTypeOfVisualization::Shading => V3dTypeOfVisualization::Zbuffer,
        }
    }
    /// `ShadingModel`; `view.shading_model()`.
    pub fn shading_model(&self) -> Graphic3dTypeOfShadingModel {
        self.view().shading_model()
    }
    /// `SetShadingModel`; `Graphic3d_TypeOfShadingModel_DEFAULT` raises
    /// `Standard_ProgramError`. Example: `view.set_shading_model(Graphic3dTypeOfShadingModel::Phong)?`.
    pub fn set_shading_model(&self, model: Graphic3dTypeOfShadingModel) -> Result<(), Failure> {
        self.view().set_shading_model(model)
    }
    /// `BackgroundColor`; `view.background_color()`.
    pub fn background_color(&self) -> Color {
        self.view().background().color()
    }
    /// `BackgroundColor(Type, V1, V2, V3)`: the components in a color space.
    /// Example: `view.background_color_values(ColorSpace::Hls)`.
    pub fn background_color_values(&self, space: ColorSpace) -> (f64, f64, f64) {
        let [v1, v2, v3] = self.background_color().values(space);
        (v1, v2, v3)
    }
    /// `GradientBackgroundColors`; `view.gradient_background_colors()`.
    pub fn gradient_background_colors(&self) -> (Color, Color) {
        self.view().gradient_background().colors()
    }
    /// `GradientBackground`; `view.gradient_background()`.
    pub fn gradient_background(&self) -> AspectGradientBackground {
        self.view().gradient_background()
    }
}

/// Lights.
impl V3dView {
    /// `UpdateLights`: the active lights to the driver view; `view.update_lights()`.
    pub fn update_lights(&self) {
        let lights = Graphic3dLightSet::default();
        for light in &self.state.read().active_lights {
            // A light already in the set is skipped; a set error leaves the light out.
            let _ = lights.add(light);
        }
        self.view().set_lights(&Handle::new(lights));
    }
    /// `SetLightOn(light)`: activates a light in this view; `view.set_light_on_one(&light)?`.
    pub fn set_light_on_one(&self, light: &Handle<Graphic3dCLight>) -> Result<(), Failure> {
        {
            let mut state = self.state.write();
            if state.active_lights.contains(light) {
                return Ok(());
            }
            state.active_lights.push(light.clone());
        }
        self.update_lights();
        Ok(())
    }
    /// `SetLightOff(light)`: deactivates a view light; a global light raises
    /// `Standard_TypeMismatch`. Example: `view.set_light_off_one(&light)?`.
    pub fn set_light_off_one(&self, light: &Handle<Graphic3dCLight>) -> Result<(), Failure> {
        if self
            .viewer()
            .get()
            .is_some_and(|v| v.is_global_light(light))
        {
            return Err(type_mismatch("V3d_View::SetLightOff, the light is global"));
        }
        self.state.write().active_lights.retain(|l| l != light);
        self.update_lights();
        Ok(())
    }
    /// `IsActiveLight`; `view.is_active_light(&light)`.
    pub fn is_active_light(&self, light: &Handle<Graphic3dCLight>) -> bool {
        !light.is_null() && self.state.read().active_lights.contains(light)
    }
    /// `SetLightOn()`: activates every light defined in the viewer; `view.set_light_on()`.
    pub fn set_light_on(&self) {
        let defined = self
            .viewer()
            .get()
            .map(|v| v.defined_lights())
            .unwrap_or_default();
        {
            let mut state = self.state.write();
            for light in defined {
                if !state.active_lights.contains(&light) {
                    state.active_lights.push(light);
                }
            }
        }
        self.update_lights();
    }
    /// `SetLightOff()`: deactivates every non-global light; `view.set_light_off()`.
    pub fn set_light_off(&self) {
        let viewer = self.viewer();
        let is_global = |light: &Handle<Graphic3dCLight>| {
            viewer.get().is_some_and(|v| v.is_global_light(light))
        };
        self.state.write().active_lights.retain(is_global);
        self.update_lights();
    }
    /// `ActiveLights`; `view.active_lights()`.
    pub fn active_lights(&self) -> Vec<Handle<Graphic3dCLight>> {
        self.state.read().active_lights.clone()
    }
    /// `ActiveLightIterator`; `view.active_light_iterator()`.
    pub fn active_light_iterator(&self) -> std::vec::IntoIter<Handle<Graphic3dCLight>> {
        self.active_lights().into_iter()
    }
    /// `LightLimit`: the driver limit; `view.light_limit()`.
    pub fn light_limit(&self) -> i32 {
        self.viewer()
            .get()
            .and_then(|v| v.driver().get().map(|d| d.inquire_light_limit()))
            .unwrap_or(0)
    }
    /// Deprecated `IfMoreLights`; `view.if_more_lights()`.
    pub fn if_more_lights(&self) -> bool {
        (self.state.read().active_lights.len() as i32) < self.light_limit()
    }
    /// Deprecated `InitActiveLights`; `view.init_active_lights()`.
    pub fn init_active_lights(&self) {
        self.state.write().active_lights_cursor = 0;
    }
    /// Deprecated `MoreActiveLights`; `view.more_active_lights()`.
    pub fn more_active_lights(&self) -> bool {
        let state = self.state.read();
        state.active_lights_cursor < state.active_lights.len()
    }
    /// Deprecated `NextActiveLights`; `view.next_active_lights()`.
    pub fn next_active_lights(&self) {
        self.state.write().active_lights_cursor += 1;
    }
    /// Deprecated `ActiveLight`; null past the end; `view.active_light()`.
    pub fn active_light(&self) -> Handle<Graphic3dCLight> {
        let state = self.state.read();
        state
            .active_lights
            .get(state.active_lights_cursor)
            .cloned()
            .unwrap_or_default()
    }
}

/// Clip planes, computed mode, back faces, rendering parameters and statistics.
impl V3dView {
    /// `AddClipPlane`: appends a plane unless present; `view.add_clip_plane(&plane)`.
    pub fn add_clip_plane(&self, plane: &Handle<Graphic3dClipPlane>) {
        let source = self.this.read().observe();
        if let Some(source) = source.get() {
            source.add_clip_plane(plane);
            return;
        }
        super::view_source_clipping::v3d_view_add_clip_plane(self, plane);
    }
    /// `RemoveClipPlane`: dispatches the actual native subclass; `view.remove_clip_plane(&plane)`.
    pub fn remove_clip_plane(&self, plane: &Handle<Graphic3dClipPlane>) {
        let source = self.this.read().observe();
        if let Some(source) = source.get() {
            source.remove_clip_plane(plane);
            return;
        }
        super::view_source_clipping::v3d_view_remove_clip_plane(self, plane);
    }
    /// `SetClipPlanes`; `view.set_clip_planes(&planes)`.
    pub fn set_clip_planes(&self, planes: &Handle<Graphic3dSequenceOfHClipPlane>) {
        self.view().set_clip_planes(planes);
    }
    /// `ClipPlanes`; `view.clip_planes()`.
    pub fn clip_planes(&self) -> Handle<Graphic3dSequenceOfHClipPlane> {
        self.view().clip_planes()
    }
    /// `PlaneLimit`: the driver limit; `view.plane_limit()`.
    pub fn plane_limit(&self) -> i32 {
        self.viewer()
            .get()
            .and_then(|v| v.driver().get().map(|d| d.inquire_plane_limit()))
            .unwrap_or(0)
    }
    /// `SetComputedMode`: on only when the viewer allows it; `view.set_computed_mode(true)`.
    pub fn set_computed_mode(&self, mode: bool) {
        if mode {
            if self.state.read().computed_mode {
                self.view().set_computed_mode(true);
            }
        } else {
            self.view().set_computed_mode(false);
        }
    }
    /// `ComputedMode`; `view.computed_mode()`.
    pub fn computed_mode(&self) -> bool {
        self.view().computed_mode()
    }
    /// `SetBackFacingModel`: then redraws; `view.set_back_facing_model(Graphic3dTypeOfBackfacingModel::Auto)`.
    pub fn set_back_facing_model(&self, model: Graphic3dTypeOfBackfacingModel) {
        self.view().set_backfacing_model(model);
        self.redraw();
    }
    /// `BackFacingModel`; `view.back_facing_model()`.
    pub fn back_facing_model(&self) -> Graphic3dTypeOfBackfacingModel {
        self.view().backfacing_model()
    }
    /// `RenderingParams`; `view.rendering_params()`.
    pub fn rendering_params(&self) -> Graphic3dRenderingParams {
        self.view().rendering_params()
    }
    /// `ChangeRenderingParams`: edits the parameters; `view.change_rendering_params(&mut |p| p.nb_msaa_samples = 4)`.
    pub fn change_rendering_params(&self, edit: &mut dyn FnMut(&mut Graphic3dRenderingParams)) {
        self.view().change_rendering_params(edit);
    }
    /// `IsCullingEnabled`; `view.is_culling_enabled()`.
    pub fn is_culling_enabled(&self) -> bool {
        self.rendering_params().ray_tracing.frustum_culling_state
            == Graphic3dRenderingParamsFrustumCulling::On
    }
    /// `SetFrustumCulling`; `view.set_frustum_culling(false)`.
    pub fn set_frustum_culling(&self, mode: bool) {
        let state = if mode {
            Graphic3dRenderingParamsFrustumCulling::On
        } else {
            Graphic3dRenderingParamsFrustumCulling::Off
        };
        self.change_rendering_params(&mut |p| p.ray_tracing.frustum_culling_state = state);
    }
    /// `DiagnosticInformation`; `view.diagnostic_information(&mut dict, flags)`.
    pub fn diagnostic_information(
        &self,
        dict: &mut Graphic3dFrameStatsDict,
        flags: Graphic3dDiagnosticInfo,
    ) {
        self.view().diagnostic_information(dict, flags);
    }
    /// `StatisticInformation()`; `view.statistic_information()`.
    pub fn statistic_information(&self) -> AsciiString {
        self.view().statistic_information()
    }
    /// `StatisticInformation(dict)`; `view.statistic_information_dict(&mut dict)`.
    pub fn statistic_information_dict(&self, dict: &mut Graphic3dFrameStatsDict) {
        self.view().statistic_information_dict(dict);
    }
}
