//! Backend-independent view storage: layer membership, window, camera and display options.
//! Renderers embed an unregistered value and register their outer Graphic3d view instead.
use crate::aspect::{
    AspectFillMethod, AspectGradientBackground, AspectRenderingContext, AspectWindow,
};
use crate::graphic3d::Graphic3dBvhBuilder3d;
use crate::graphic3d::{
    GRAPHIC3D_ZLAYER_ID_DEFAULT, GRAPHIC3D_ZLAYER_ID_UNKNOWN, Graphic3dBufferType,
    Graphic3dCStructure, Graphic3dCStructureMembers as _, Graphic3dCView, Graphic3dCViewBase,
    Graphic3dDataStructureManager, Graphic3dDiagnosticInfo, Graphic3dDisplayPriority,
    Graphic3dFrameStatsDict, Graphic3dLayer, Graphic3dLightSet, Graphic3dSequenceOfHClipPlane,
    Graphic3dStructureManager, Graphic3dTextureEnv, Graphic3dTextureMap, Graphic3dZLayerId,
    Graphic3dZLayerSettings, graphic3d_cview_diagnostic_information, graphic3d_cview_register,
    graphic3d_cview_resized,
};
use crate::image::ImagePixMap;
use rustycad_tkernel::standard::exceptions::program_error;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::bvh::constants::{LEAF_NODE_SIZE_SINGLE, MAX_TREE_DEPTH};
use rustycad_tkmath::bvh::{BvhLinearBuilder, BvhVec3d};
use std::sync::Arc;

/// The frustum culling builder of the layers, as `OpenGl_LayerList` makes it.
fn culling_builder() -> Graphic3dBvhBuilder3d {
    Some(Arc::new(BvhLinearBuilder::<BvhVec3d>::new(
        LEAF_NODE_SIZE_SINGLE,
        MAX_TREE_DEPTH,
    )))
}

/// Shared scene state used by recording and rendering views.
#[derive(Default)]
struct RetainedViewFields {
    layers: Vec<Handle<Graphic3dLayer>>,
    builder: Graphic3dBvhBuilder3d,
    window: Handle<dyn AspectWindow>,
    is_invalidated: bool,
    gradient: AspectGradientBackground,
    image_style: Option<AspectFillMethod>,
    texture_env: Handle<Graphic3dTextureEnv>,
    lights: Handle<Graphic3dLightSet>,
    clip_planes: Handle<Graphic3dSequenceOfHClipPlane>,
    fbo: Handle<StandardTransient>,
}

/// Shared driver-view storage (with an intentionally non-rendering standalone adapter); `Graphic3dRetainedView::create(&manager, &layers)?`.
pub struct Graphic3dRetainedView {
    transient: StandardTransient,
    base: Graphic3dCViewBase,
    state: SharedState<RetainedViewFields>,
}
rustycad_tkernel::impl_transient!(Graphic3dRetainedView, "Martensite_RetainedView", StandardTransient, transient,
    interfaces: [dyn Graphic3dCView, dyn Graphic3dDataStructureManager]);

impl Graphic3dRetainedView {
    /// A registered view with a copy of `layers` (same identifiers and settings);
    /// `Graphic3dRetainedView::create(&manager, &driver.layers())?`.
    pub fn create(
        manager: &Handle<Graphic3dStructureManager>,
        layers: &[Handle<Graphic3dLayer>],
    ) -> Result<Handle<dyn Graphic3dCView>, Failure> {
        let handle: Handle<dyn Graphic3dCView> = Handle::new(Self::new(manager, layers)?).cast();
        graphic3d_cview_register(&handle)?;
        Ok(handle)
    }
    /// Build storage before registering the renderer's outer view; `Graphic3dRetainedView::new(manager, layers)?`.
    pub fn new(
        manager: &Handle<Graphic3dStructureManager>,
        layers: &[Handle<Graphic3dLayer>],
    ) -> Result<Self, Failure> {
        let builder = culling_builder();
        let copies = layers.iter().filter_map(Handle::get).map(|layer| {
            let copy = Graphic3dLayer::new(layer.layer_id(), builder.clone());
            copy.set_layer_settings(&layer.layer_settings());
            Handle::new(copy)
        });
        let state = RetainedViewFields {
            layers: copies.collect(),
            builder,
            ..Default::default()
        };
        Ok(Self {
            transient: StandardTransient::default(),
            base: Graphic3dCViewBase::new(manager)?,
            state: SharedState::new(state),
        })
    }
    /// The layer of a structure, or the default layer.
    fn layer_of(&self, layer_id: Graphic3dZLayerId) -> Handle<Graphic3dLayer> {
        let found = self.layer(layer_id);
        if found.is_null() {
            self.layer(GRAPHIC3D_ZLAYER_ID_DEFAULT)
        } else {
            found
        }
    }
    /// Removes a structure from its layer, else from the first layer that has it; the
    /// priority it had, or `Invalid`.
    fn remove_from_layers(
        &self,
        cstructure: &Handle<dyn Graphic3dCStructure>,
        layer_id: Graphic3dZLayerId,
        is_for_change_priority: bool,
    ) -> Graphic3dDisplayPriority {
        let own = self.layer_of(layer_id);
        let mut priority = own.get().map_or(Graphic3dDisplayPriority::Invalid, |l| {
            l.remove(cstructure, is_for_change_priority)
        });
        if priority != Graphic3dDisplayPriority::Invalid {
            return priority;
        }
        for layer in self
            .layers()
            .iter()
            .filter(|l| **l != own)
            .filter_map(Handle::get)
        {
            priority = layer.remove(cstructure, is_for_change_priority);
            if priority != Graphic3dDisplayPriority::Invalid {
                break;
            }
        }
        priority
    }
    /// Inserts a layer copy at `index` of the list (the end for `None`).
    fn insert_layer(
        &self,
        new_layer_id: Graphic3dZLayerId,
        settings: &Graphic3dZLayerSettings,
        index: Option<usize>,
    ) -> Result<(), Failure> {
        if !self.layer(new_layer_id).is_null() {
            return Err(program_error(&format!(
                "Graphic3dRetainedView: layer {new_layer_id} already exists"
            )));
        }
        let layer = Graphic3dLayer::new(new_layer_id, self.state.read().builder.clone());
        layer.set_layer_settings(settings);
        let mut state = self.state.write();
        let at = index.unwrap_or(state.layers.len()).min(state.layers.len());
        state.layers.insert(at, Handle::new(layer));
        Ok(())
    }
    /// The position of a layer in the list.
    fn position_of(&self, layer_id: Graphic3dZLayerId) -> Option<usize> {
        self.state
            .read()
            .layers
            .iter()
            .position(|l| l.get().is_some_and(|l| l.layer_id() == layer_id))
    }
    /// Sets the window of the view (null to undefine it); `view.set_null_window(&window)`.
    pub fn set_null_window(&self, window: &Handle<dyn AspectWindow>) {
        self.state.write().window = window.clone();
    }
}

impl Graphic3dCView for Graphic3dRetainedView {
    fn cview_base(&self) -> &Graphic3dCViewBase {
        &self.base
    }
    fn redraw(&self) {
        self.state.write().is_invalidated = false;
    }
    fn redraw_immediate(&self) {}
    fn invalidate(&self) {
        self.state.write().is_invalidated = true;
    }
    fn is_invalidated(&self) -> bool {
        self.state.read().is_invalidated
    }
    fn resized(&self) {
        graphic3d_cview_resized(self);
    }
    fn set_immediate_mode_draw_to_front(&self, draw_to_front_buffer: bool) -> bool {
        let _ = draw_to_front_buffer;
        false
    }
    fn set_window(
        &self,
        parent_view: &Handle<dyn Graphic3dCView>,
        window: &Handle<dyn AspectWindow>,
        context: AspectRenderingContext,
    ) {
        let _ = (parent_view, context);
        self.set_null_window(window);
    }
    fn window(&self) -> Handle<dyn AspectWindow> {
        self.state.read().window.clone()
    }
    fn is_defined(&self) -> bool {
        !self.state.read().window.is_null()
    }
    fn buffer_dump(&self, image: &ImagePixMap, buffer_type: &Graphic3dBufferType) -> bool {
        let _ = (image, buffer_type);
        false
    }
    fn shadow_map_dump(&self, image: &ImagePixMap, light_name: &AsciiString) -> bool {
        let _ = (image, light_name);
        false
    }
    fn invalidate_bvh_data(&self, layer_id: Graphic3dZLayerId) {
        if let Some(layer) = self.layer(layer_id).get() {
            layer.invalidate_bvh_data();
        }
    }
    fn insert_layer_before(
        &self,
        new_layer_id: Graphic3dZLayerId,
        settings: &Graphic3dZLayerSettings,
        layer_after: Graphic3dZLayerId,
    ) -> Result<(), Failure> {
        let index = if layer_after == GRAPHIC3D_ZLAYER_ID_UNKNOWN {
            Some(0)
        } else {
            self.position_of(layer_after).or(Some(0))
        };
        self.insert_layer(new_layer_id, settings, index)
    }
    fn insert_layer_after(
        &self,
        new_layer_id: Graphic3dZLayerId,
        settings: &Graphic3dZLayerSettings,
        layer_before: Graphic3dZLayerId,
    ) -> Result<(), Failure> {
        let index = self.position_of(layer_before).map(|i| i + 1);
        self.insert_layer(new_layer_id, settings, index)
    }
    fn z_layer_max(&self) -> i32 {
        self.layers()
            .iter()
            .filter_map(Handle::get)
            .map(Graphic3dLayer::layer_id)
            .max()
            .unwrap_or(0)
    }
    fn layers(&self) -> Vec<Handle<Graphic3dLayer>> {
        self.state.read().layers.clone()
    }
    fn layer(&self, layer_id: Graphic3dZLayerId) -> Handle<Graphic3dLayer> {
        let state = self.state.read();
        state
            .layers
            .iter()
            .find(|l| l.get().is_some_and(|l| l.layer_id() == layer_id))
            .cloned()
            .unwrap_or_default()
    }
    fn remove_z_layer(&self, layer_id: Graphic3dZLayerId) -> Result<(), Failure> {
        if layer_id <= 0 {
            return Ok(());
        }
        // OpenGl_LayerList::RemoveLayer at 3d097a0328e71b826377d4814ab05ec3c3d23871
        // preserves displayed structures when an application removes their layer.
        if let Some(removed) = self.layer(layer_id).get() {
            self.layer(GRAPHIC3D_ZLAYER_ID_DEFAULT).append(removed);
        }
        self.state
            .write()
            .layers
            .retain(|l| l.get().is_none_or(|l| l.layer_id() != layer_id));
        Ok(())
    }
    fn set_z_layer_settings(
        &self,
        layer_id: Graphic3dZLayerId,
        settings: &Graphic3dZLayerSettings,
    ) -> Result<(), Failure> {
        if let Some(layer) = self.layer(layer_id).get() {
            layer.set_layer_settings(settings);
        }
        Ok(())
    }
    fn fbo(&self) -> Handle<StandardTransient> {
        self.state.read().fbo.clone()
    }
    fn set_fbo(&self, fbo: &Handle<StandardTransient>) {
        self.state.write().fbo = fbo.clone();
    }
    fn fbo_create(&self, width: i32, height: i32) -> Handle<StandardTransient> {
        let _ = (width, height);
        Handle::default()
    }
    fn fbo_release(&self, fbo: &mut Handle<StandardTransient>) {
        *fbo = Handle::default();
    }
    fn fbo_get_dimensions(&self, fbo: &Handle<StandardTransient>) -> [i32; 4] {
        let _ = fbo;
        [0; 4]
    }
    fn fbo_change_viewport(&self, fbo: &Handle<StandardTransient>, width: i32, height: i32) {
        let _ = (fbo, width, height);
    }
    fn gradient_background(&self) -> AspectGradientBackground {
        self.state.read().gradient
    }
    fn set_gradient_background(&self, background: &AspectGradientBackground) {
        self.state.write().gradient = *background;
    }
    fn set_background_image(
        &self,
        texture: &Handle<dyn Graphic3dTextureMap>,
        to_update_pbr_env: bool,
    ) {
        let _ = to_update_pbr_env;
        self.cview_base().fields.write().background_image = texture.clone();
    }
    fn background_image_style(&self) -> AspectFillMethod {
        self.state
            .read()
            .image_style
            .unwrap_or(AspectFillMethod::None)
    }
    fn set_background_image_style(&self, style: AspectFillMethod) {
        self.state.write().image_style = Some(style);
    }
    fn set_image_based_lighting(&self, to_enable_ibl: bool) {
        let _ = to_enable_ibl;
    }
    fn set_texture_env(&self, texture_env: &Handle<Graphic3dTextureEnv>) {
        self.state.write().texture_env = texture_env.clone();
        self.cview_base().fields.write().texture_env_data = texture_env.clone();
    }
    fn lights(&self) -> Handle<Graphic3dLightSet> {
        self.state.read().lights.clone()
    }
    fn set_lights(&self, lights: &Handle<Graphic3dLightSet>) {
        self.state.write().lights = lights.clone();
    }
    fn clip_planes(&self) -> Handle<Graphic3dSequenceOfHClipPlane> {
        self.state.read().clip_planes.clone()
    }
    fn set_clip_planes(&self, planes: &Handle<Graphic3dSequenceOfHClipPlane>) {
        self.state.write().clip_planes = planes.clone();
    }
    fn diagnostic_information(
        &self,
        dict: &mut Graphic3dFrameStatsDict,
        flags: Graphic3dDiagnosticInfo,
    ) {
        graphic3d_cview_diagnostic_information(self, dict, flags);
    }
    fn statistic_information(&self) -> AsciiString {
        AsciiString::default()
    }
    fn statistic_information_dict(&self, dict: &mut Graphic3dFrameStatsDict) {
        let _ = dict;
    }
    fn display_cstructure(
        &self,
        cstructure: &Handle<dyn Graphic3dCStructure>,
        priority: Graphic3dDisplayPriority,
    ) {
        let layer_id = cstructure
            .get()
            .map_or(GRAPHIC3D_ZLAYER_ID_DEFAULT, |c| c.z_layer());
        if let Some(layer) = self.layer_of(layer_id).get() {
            layer.add(cstructure, priority, false);
        }
    }
    fn erase_cstructure(&self, cstructure: &Handle<dyn Graphic3dCStructure>) {
        let layer_id = cstructure
            .get()
            .map_or(GRAPHIC3D_ZLAYER_ID_DEFAULT, |c| c.z_layer());
        self.remove_from_layers(cstructure, layer_id, false);
    }
    /// Moves the structure from its current layer to `layer_id`, keeping its priority.
    fn change_cstructure_z_layer(
        &self,
        cstructure: &Handle<dyn Graphic3dCStructure>,
        layer_id: Graphic3dZLayerId,
    ) {
        let old_layer = cstructure
            .get()
            .map_or(GRAPHIC3D_ZLAYER_ID_DEFAULT, |c| c.z_layer());
        let priority = self.remove_from_layers(cstructure, old_layer, false);
        if let Some(layer) = self.layer_of(layer_id).get() {
            layer.add(cstructure, priority, false);
        }
    }
    /// Moves the structure to a priority in its layer.
    fn change_cstructure_priority(
        &self,
        cstructure: &Handle<dyn Graphic3dCStructure>,
        priority: Graphic3dDisplayPriority,
    ) {
        let layer_id = cstructure
            .get()
            .map_or(GRAPHIC3D_ZLAYER_ID_DEFAULT, |c| c.z_layer());
        if self.remove_from_layers(cstructure, layer_id, true) == Graphic3dDisplayPriority::Invalid
        {
            return;
        }
        if let Some(layer) = self.layer_of(layer_id).get() {
            layer.add(cstructure, priority, true);
        }
    }
}
