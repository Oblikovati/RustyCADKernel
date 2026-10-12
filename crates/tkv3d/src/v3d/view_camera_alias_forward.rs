//! Named camera-view fake forwards real retained scene operations; only its Camera slot selection differs.
use super::view_camera_alias_fixture::BorrowedCameraView;
use rustycad_tkernel::{
    standard::{Failure, Handle, HandleReference, StandardTransient, WeakHandleFieldRef},
    tcollection::AsciiString,
};
use rustycad_tkservice::{aspect::*, graphic3d::*, image::ImagePixMap};
impl Graphic3dCView for BorrowedCameraView {
    fn activate(&self) {
        if let Some(control) = self.control.light_probe.read().as_ref() {
            super::viewer_light_probe_fixture::reject_light_probe_virtual(control, 13, "Activate");
        }
        self.retained.activate();
        if *self.control.replace_activation_argument.read() {
            *self.control.view_caller.write() =
                self.control.view_replacement.read().observe().clone();
        }
    }
    fn set_camera(&self, camera: &dyn HandleReference<Graphic3dCamera>) {
        if *self.control.replace_camera_argument.read() {
            *self.control.caller.write() = self.control.replacement.read().observe().clone();
        }
        self.cview_base().fields.write().camera = camera.clone_handle();
    }
    fn cview_base(&self) -> &Graphic3dCViewBase {
        self.retained.cview_base()
    }
    fn cview_camera_ref(&self) -> Box<dyn HandleReference<Graphic3dCamera>> {
        self.control
            .bounds_getter_calls
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if *self.control.alternate_selected.read() {
            return Box::new(WeakHandleFieldRef::new(&self.control.alternate, |slot| {
                slot
            }));
        }
        graphic3d_cview_camera_ref(self.cview_base())
    }

    fn redraw(&self) {
        self.retained.redraw()
    }
    fn redraw_immediate(&self) {
        self.retained.redraw_immediate()
    }
    fn invalidate(&self) {
        self.retained.invalidate()
    }
    fn is_invalidated(&self) -> bool {
        self.retained.is_invalidated()
    }
    fn resized(&self) {
        self.retained.resized()
    }
    fn set_immediate_mode_draw_to_front(&self, draw_to_front_buffer: bool) -> bool {
        self.retained
            .set_immediate_mode_draw_to_front(draw_to_front_buffer)
    }
    fn set_window(
        &self,
        parent_view: &Handle<dyn Graphic3dCView>,
        window: &Handle<dyn AspectWindow>,
        context: AspectRenderingContext,
    ) {
        self.retained.set_window(parent_view, window, context)
    }
    fn window(&self) -> Handle<dyn AspectWindow> {
        self.retained.window()
    }
    fn is_defined(&self) -> bool {
        self.retained.is_defined()
    }
    fn buffer_dump(&self, image: &ImagePixMap, buffer_type: &Graphic3dBufferType) -> bool {
        self.retained.buffer_dump(image, buffer_type)
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
    fn fbo_create(&self, width: i32, height: i32) -> Handle<StandardTransient> {
        self.retained.fbo_create(width, height)
    }
    fn fbo_release(&self, fbo: &mut Handle<StandardTransient>) {
        self.retained.fbo_release(fbo)
    }
    fn fbo_get_dimensions(&self, fbo: &Handle<StandardTransient>) -> [i32; 4] {
        self.retained.fbo_get_dimensions(fbo)
    }
    fn fbo_change_viewport(&self, fbo: &Handle<StandardTransient>, width: i32, height: i32) {
        self.retained.fbo_change_viewport(fbo, width, height)
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
    fn lights_ref(
        &self,
    ) -> Box<dyn rustycad_tkernel::standard::HandleReference<Graphic3dLightSet> + '_> {
        self.retained.lights_ref()
    }
    fn set_lights(
        &self,
        lights: &dyn rustycad_tkernel::standard::HandleReference<Graphic3dLightSet>,
    ) {
        if let Some(control) = self.control.light_probe.read().clone() {
            super::viewer_light_probe_callback::record_light_set_callback(
                &control,
                self.identity,
                self.transient.ref_count(),
                lights,
            );
        }
        self.retained.set_lights(lights)
    }
    fn clip_planes_ref(&self) -> Box<dyn HandleReference<dyn Graphic3dClipPlaneSequenceSource>> {
        if let Some(probe) = self.control.clip_probe.read().as_ref() {
            return super::view_clip_retarget_probe::clip_probe_getter(self, probe);
        }
        self.retained.clip_planes_ref()
    }
    fn set_clip_planes(
        &self,
        planes: &dyn rustycad_tkernel::standard::HandleReference<dyn Graphic3dClipPlaneSequenceSource>,
    ) {
        if let Some(probe) = self.control.clip_probe.read().as_ref() {
            super::view_clip_retarget_probe::clip_probe_setter(self, probe, planes);
            return;
        }
        self.retained.set_clip_planes(planes)
    }
    fn diagnostic_information(
        &self,
        dict: &mut Graphic3dFrameStatsDict,
        flags: Graphic3dDiagnosticInfo,
    ) {
        self.retained.diagnostic_information(dict, flags)
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
}
