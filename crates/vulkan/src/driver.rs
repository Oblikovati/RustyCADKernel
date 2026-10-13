//! Graphic3d factory routes ordinary structure/view creation to Martensite rendering.
use crate::{
    RenderBackend, RenderError, VulkanOptions, VulkanRenderer,
    driver_backend::{DriverBackend, SharedRenderDevice},
    driver_structure::RenderStructure,
    driver_view::Graphic3dRenderView,
    error::rejected,
};
use rustycad_tkernel::{
    standard::{Failure, Handle, SharedState, StandardTransient, WeakHandle},
    tcollection::AsciiString,
};
use rustycad_tkservice::{aspect::AspectWindow, graphic3d::*};
use std::sync::Arc;
/// A Graphic3d driver with an injected device; `Graphic3dRenderDriver::new(VulkanOptions::default())?`.
pub struct Graphic3dRenderDriver {
    transient: StandardTransient,
    base: Graphic3dGraphicDriverBase,
    device: SharedRenderDevice,
    text_source: Arc<dyn crate::TextLayoutSource>,
    pub(crate) mesh_edge_pipeline: crate::RenderMeshEdgePipeline,
    pub(crate) lighting_pipeline: crate::RenderLightingPipeline,
    pub(crate) texture_images: Arc<crate::retained_texture_images::RetainedTextureImages>,
    views: SharedState<Vec<WeakHandle<dyn Graphic3dCView>>>,
    limit: u32,
    vertical_sync: Arc<SharedState<bool>>,
}
rustycad_tkernel::impl_transient!(Graphic3dRenderDriver, "Martensite_RenderDriver", StandardTransient, transient, interfaces:[dyn Graphic3dGraphicDriver]);
impl Graphic3dRenderDriver {
    /// Create a Vulkan 1.3 driver; `Graphic3dRenderDriver::new(options)?`.
    pub fn new(options: VulkanOptions) -> Result<Handle<Self>, RenderError> {
        Self::with_texture_coordinate_pipeline(
            options,
            crate::RenderTextureCoordinatePipeline::default(),
        )
    }
    /// Inject legacy coordinate semantics; `Graphic3dRenderDriver::with_texture_coordinate_pipeline(options, RenderTextureCoordinatePipeline::GeneratedCompatibility)?`.
    /// This changes texture coordinates, not the shader lighting family.
    pub fn with_texture_coordinate_pipeline(
        options: VulkanOptions,
        coordinate_pipeline: crate::RenderTextureCoordinatePipeline,
    ) -> Result<Handle<Self>, RenderError> {
        Self::with_surface_pipelines(
            options,
            coordinate_pipeline,
            crate::RenderMeshEdgePipeline::default(),
        )
    }
    /// Select native polygon edge compatibility; `Self::with_mesh_edge_pipeline(options, RenderMeshEdgePipeline::PolygonCompatibility)?`.
    pub fn with_mesh_edge_pipeline(
        options: VulkanOptions,
        pipeline: crate::RenderMeshEdgePipeline,
    ) -> Result<Handle<Self>, RenderError> {
        Self::with_surface_pipelines(
            options,
            crate::RenderTextureCoordinatePipeline::default(),
            pipeline,
        )
    }
    /// Inject independent legacy coordinate and edge policies; `Self::with_surface_pipelines(options, RenderTextureCoordinatePipeline::GeneratedCompatibility, RenderMeshEdgePipeline::PolygonCompatibility)?`.
    pub fn with_surface_pipelines(
        options: VulkanOptions,
        coordinates: crate::RenderTextureCoordinatePipeline,
        edges: crate::RenderMeshEdgePipeline,
    ) -> Result<Handle<Self>, RenderError> {
        Self::with_render_pipelines(
            options,
            coordinates,
            edges,
            crate::RenderLightingPipeline::default(),
        )
    }
    /// Inject lighting, coordinates and edges; `Self::with_render_pipelines(options, coordinates, edges, RenderLightingPipeline::FixedFunction)?`.
    pub fn with_render_pipelines(
        options: VulkanOptions,
        coordinates: crate::RenderTextureCoordinatePipeline,
        edges: crate::RenderMeshEdgePipeline,
        lighting: crate::RenderLightingPipeline,
    ) -> Result<Handle<Self>, RenderError> {
        let renderer = VulkanRenderer::new(options)?;
        let limit = renderer.capture_limit();
        Ok(Self::build(
            DriverBackend::Vulkan(Box::new(renderer)),
            limit,
            Arc::new(crate::FreeTypeTextSource::default()),
            coordinates,
            edges,
            lighting,
        ))
    }
    /// Inject a named backend and its extent limit; `Graphic3dRenderDriver::with_backend(recorder, 4096)?`.
    pub fn with_backend(
        backend: impl RenderBackend + Send + Sync + 'static,
        limit: u32,
    ) -> Result<Handle<Self>, RenderError> {
        Self::with_backend_and_fonts(
            backend,
            limit,
            Arc::new(crate::FreeTypeTextSource::default()),
        )
    }
    /// Inject both graphics and font I/O; `Graphic3dRenderDriver::with_backend_and_fonts(recorder, 4096, fonts)?`.
    pub fn with_backend_and_fonts(
        backend: impl RenderBackend + Send + Sync + 'static,
        limit: u32,
        fonts: Arc<dyn crate::TextLayoutSource>,
    ) -> Result<Handle<Self>, RenderError> {
        Self::with_backend_and_texture_coordinates(
            backend,
            limit,
            fonts,
            crate::RenderTextureCoordinatePipeline::default(),
        )
    }
    /// Inject device/font I/O and coordinate policy; `Self::with_backend_and_texture_coordinates(recorder, 4096, fonts, policy)?`.
    pub fn with_backend_and_texture_coordinates(
        backend: impl RenderBackend + Send + Sync + 'static,
        limit: u32,
        fonts: Arc<dyn crate::TextLayoutSource>,
        coordinate_pipeline: crate::RenderTextureCoordinatePipeline,
    ) -> Result<Handle<Self>, RenderError> {
        if limit == 0 || limit > i32::MAX as u32 {
            return Err(rejected(limit, "a positive 31-bit capture limit"));
        }
        Ok(Self::build(
            DriverBackend::Injected(Box::new(backend)),
            limit,
            fonts,
            coordinate_pipeline,
            crate::RenderMeshEdgePipeline::default(),
            crate::RenderLightingPipeline::default(),
        ))
    }
    fn build(
        backend: DriverBackend,
        limit: u32,
        text_source: Arc<dyn crate::TextLayoutSource>,
        coordinate_pipeline: crate::RenderTextureCoordinatePipeline,
        mesh_edge_pipeline: crate::RenderMeshEdgePipeline,
        lighting_pipeline: crate::RenderLightingPipeline,
    ) -> Handle<Self> {
        let texture_images = Arc::new(
            crate::retained_texture_images::RetainedTextureImages::with_coordinate_pipeline(
                coordinate_pipeline,
            ),
        );
        Handle::new(Self {
            transient: StandardTransient::default(),
            base: Graphic3dGraphicDriverBase::new(&Handle::default()),
            device: Arc::new(SharedState::new(backend)),
            text_source,
            mesh_edge_pipeline,
            lighting_pipeline,
            texture_images,
            views: SharedState::default(),
            limit,
            vertical_sync: Arc::new(SharedState::new(false)),
        })
    }
    /// Inspect Vulkan validation failures; `driver.validation_errors()`.
    pub fn validation_errors(&self) -> Vec<String> {
        self.device.read().validation_errors()
    }
    /// Identify the actual rendering device; `driver.device_name()`.
    pub fn device_name(&self) -> String {
        self.device.read().name().to_owned()
    }
    /// Read exact selected Vulkan provenance, absent for injected backends; `driver.driver_identity()`.
    pub fn driver_identity(&self) -> Option<crate::RenderDriverIdentity> {
        self.device.read().driver_identity()
    }
    fn live_views(&self) -> Vec<Handle<dyn Graphic3dCView>> {
        self.views
            .read()
            .iter()
            .map(WeakHandle::upgrade)
            .filter(|v| !v.is_null() && !v.is_removed())
            .collect()
    }
}
impl Graphic3dGraphicDriver for Graphic3dRenderDriver {
    fn graphic_driver_base(&self) -> &Graphic3dGraphicDriverBase {
        &self.base
    }
    fn inquire_limit(&self, limit: Graphic3dTypeOfLimit) -> i32 {
        match limit {
            Graphic3dTypeOfLimit::MaxNbViews => 32,
            Graphic3dTypeOfLimit::MaxMsaa
            | Graphic3dTypeOfLimit::HasBlendedOit
            | Graphic3dTypeOfLimit::HasBlendedOitMsaa => self.device.read().raster_limit(limit),
            Graphic3dTypeOfLimit::MaxViewDumpSizeX | Graphic3dTypeOfLimit::MaxViewDumpSizeY => {
                self.limit as i32
            }
            _ => 0,
        }
    }
    fn create_structure(
        &self,
        manager: &Handle<Graphic3dStructureManager>,
    ) -> Result<Handle<dyn Graphic3dCStructure>, Failure> {
        Ok(RenderStructure::new(manager, self.text_source.clone())?.cast())
    }
    fn remove_structure(&self, structure: &mut Handle<dyn Graphic3dCStructure>) {
        if let Some(value) = structure.get() {
            value.clear();
        }
        *structure = Handle::default();
    }
    fn create_view(
        &self,
        manager: &Handle<Graphic3dStructureManager>,
    ) -> Result<Handle<dyn Graphic3dCView>, Failure> {
        let view: Handle<dyn Graphic3dCView> = Graphic3dRenderView::create(
            manager,
            &self.layers(),
            self.device.clone(),
            self.limit,
            self.vertical_sync.clone(),
        )?
        .cast();
        self.views.write().push(view.downgrade());
        Ok(view)
    }
    fn remove_view(&self, view: &Handle<dyn Graphic3dCView>) {
        self.views
            .write()
            .retain(|weak| weak.upgrade() != *view && !weak.upgrade().is_null());
    }
    // Vulkan buffer uploads are mandatory; this legacy GL implementation switch has no effect.
    fn enable_vbo(&self, _: bool) {}
    fn is_vertical_sync(&self) -> bool {
        *self.vertical_sync.read()
    }
    fn set_vertical_sync(&self, enabled: bool) {
        *self.vertical_sync.write() = enabled;
    }
    fn memory_info(&self, free_bytes: &mut usize, info: &mut AsciiString) -> bool {
        *free_bytes = 0;
        *info = self.device.read().name().into();
        false
    }
    fn default_text_height(&self) -> f32 {
        16.
    }
    fn text_size(&self, view: &Handle<dyn Graphic3dCView>, text: &str, height: f32) -> [f32; 3] {
        crate::driver_text::text_size(view, text, height)
    }
    fn insert_layer_before(
        &self,
        id: Graphic3dZLayerId,
        settings: &Graphic3dZLayerSettings,
        after: Graphic3dZLayerId,
    ) -> Result<(), Failure> {
        self.base.insert_layer_before(id, settings, after)?;
        for view in self.live_views() {
            view.insert_layer_before(id, settings, after)?;
        }
        Ok(())
    }
    fn insert_layer_after(
        &self,
        id: Graphic3dZLayerId,
        settings: &Graphic3dZLayerSettings,
        before: Graphic3dZLayerId,
    ) -> Result<(), Failure> {
        self.base.insert_layer_after(id, settings, before)?;
        for view in self.live_views() {
            view.insert_layer_after(id, settings, before)?;
        }
        Ok(())
    }
    fn remove_z_layer(&self, id: Graphic3dZLayerId) -> Result<(), Failure> {
        self.base.remove_z_layer(id)?;
        for view in self.live_views() {
            view.remove_z_layer(id)?;
        }
        Ok(())
    }
    fn set_z_layer_settings(
        &self,
        id: Graphic3dZLayerId,
        settings: &Graphic3dZLayerSettings,
    ) -> Result<(), Failure> {
        self.base.set_z_layer_settings(id, settings)?;
        for view in self.live_views() {
            view.set_z_layer_settings(id, settings)?;
        }
        Ok(())
    }
    fn view_exists(
        &self,
        window: &Handle<dyn AspectWindow>,
        output: &mut Handle<dyn Graphic3dCView>,
    ) -> bool {
        *output = self
            .live_views()
            .into_iter()
            .find(|view| view.window() == *window)
            .unwrap_or_default();
        !output.is_null()
    }
}
