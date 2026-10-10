//! Typed forwarding of backend-independent Graphic3d view state.
//! Keeping these operations in TKService gives recording and Vulkan views the same layer semantics.
macro_rules! retained_view_methods {
    () => {
        fn invalidate(&self) {
            self.retained.invalidate()
        }
        fn is_invalidated(&self) -> bool {
            self.retained.is_invalidated()
        }
        fn set_immediate_mode_draw_to_front(&self, draw_to_front_buffer: bool) -> bool {
            self.retained
                .set_immediate_mode_draw_to_front(draw_to_front_buffer)
        }
        fn window(&self) -> Handle<dyn AspectWindow> {
            self.retained.window()
        }
        fn is_defined(&self) -> bool {
            self.retained.is_defined()
        }
        fn shadow_map_dump(&self, image: &ImagePixMap, light_name: &AsciiString) -> bool {
            self.retained.shadow_map_dump(image, light_name)
        }
        fn invalidate_bvh_data(&self, layer_id: Graphic3dZLayerId) {
            self.retained.invalidate_bvh_data(layer_id)
        }
        fn insert_layer_before(
            &self,
            new_layer_id: Graphic3dZLayerId,
            settings: &Graphic3dZLayerSettings,
            layer_after: Graphic3dZLayerId,
        ) -> Result<(), Failure> {
            self.retained
                .insert_layer_before(new_layer_id, settings, layer_after)
        }
        fn insert_layer_after(
            &self,
            new_layer_id: Graphic3dZLayerId,
            settings: &Graphic3dZLayerSettings,
            layer_before: Graphic3dZLayerId,
        ) -> Result<(), Failure> {
            self.retained
                .insert_layer_after(new_layer_id, settings, layer_before)
        }
        fn z_layer_max(&self) -> i32 {
            self.retained.z_layer_max()
        }
        fn layers(&self) -> Vec<Handle<Graphic3dLayer>> {
            self.retained.layers()
        }
        fn layer(&self, layer_id: Graphic3dZLayerId) -> Handle<Graphic3dLayer> {
            self.retained.layer(layer_id)
        }
        fn remove_z_layer(&self, layer_id: Graphic3dZLayerId) -> Result<(), Failure> {
            self.retained.remove_z_layer(layer_id)
        }
        fn set_z_layer_settings(
            &self,
            layer_id: Graphic3dZLayerId,
            settings: &Graphic3dZLayerSettings,
        ) -> Result<(), Failure> {
            self.retained.set_z_layer_settings(layer_id, settings)
        }
        fn fbo(&self) -> Handle<StandardTransient> {
            self.retained.fbo()
        }
        fn set_fbo(&self, fbo: &Handle<StandardTransient>) {
            self.retained.set_fbo(fbo)
        }
        fn gradient_background(&self) -> AspectGradientBackground {
            self.retained.gradient_background()
        }
        fn set_gradient_background(&self, background: &AspectGradientBackground) {
            self.retained.set_gradient_background(background)
        }
        fn set_background_image(
            &self,
            texture: &Handle<dyn Graphic3dTextureMap>,
            to_update_pbr_env: bool,
        ) {
            self.retained
                .set_background_image(texture, to_update_pbr_env)
        }
        fn background_image_style(&self) -> AspectFillMethod {
            self.retained.background_image_style()
        }
        fn set_background_image_style(&self, style: AspectFillMethod) {
            self.retained.set_background_image_style(style)
        }
        fn set_image_based_lighting(&self, to_enable_ibl: bool) {
            self.retained.set_image_based_lighting(to_enable_ibl)
        }
        fn set_texture_env(&self, texture_env: &Handle<Graphic3dTextureEnv>) {
            self.retained.set_texture_env(texture_env)
        }
        fn lights(&self) -> Handle<Graphic3dLightSet> {
            self.retained.lights()
        }
        fn set_lights(&self, lights: &Handle<Graphic3dLightSet>) {
            self.retained.set_lights(lights)
        }
        fn clip_planes(&self) -> Handle<Graphic3dSequenceOfHClipPlane> {
            self.retained.clip_planes()
        }
        fn set_clip_planes(&self, planes: &Handle<Graphic3dSequenceOfHClipPlane>) {
            self.retained.set_clip_planes(planes)
        }
        fn statistic_information(&self) -> AsciiString {
            self.retained.statistic_information()
        }
        fn statistic_information_dict(&self, dict: &mut Graphic3dFrameStatsDict) {
            self.retained.statistic_information_dict(dict)
        }
        fn display_cstructure(
            &self,
            cstructure: &Handle<dyn Graphic3dCStructure>,
            priority: Graphic3dDisplayPriority,
        ) {
            self.retained.display_cstructure(cstructure, priority)
        }
        fn erase_cstructure(&self, cstructure: &Handle<dyn Graphic3dCStructure>) {
            self.retained.erase_cstructure(cstructure)
        }
        fn change_cstructure_z_layer(
            &self,
            cstructure: &Handle<dyn Graphic3dCStructure>,
            layer_id: Graphic3dZLayerId,
        ) {
            self.retained
                .change_cstructure_z_layer(cstructure, layer_id)
        }
        fn change_cstructure_priority(
            &self,
            cstructure: &Handle<dyn Graphic3dCStructure>,
            priority: Graphic3dDisplayPriority,
        ) {
            self.retained
                .change_cstructure_priority(cstructure, priority)
        }
    };
}
pub(crate) use retained_view_methods;
