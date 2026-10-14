//! Immutable GUI behavior selection leaves existing driver constructors unchanged.
use crate::{
    Graphic3dRenderDriver, RenderBackend, RenderError, RenderGuiSurfacePolicy, VulkanOptions,
};
use rustycad_tkernel::standard::Handle;
impl Graphic3dRenderDriver {
    /// Select checked GUI outcomes; `Self::with_gui_surface_policy(options, RenderGuiSurfacePolicy::CheckedV2)?`.
    pub fn with_gui_surface_policy(
        options: VulkanOptions,
        policy: RenderGuiSurfacePolicy,
    ) -> Result<Handle<Self>, RenderError> {
        Self::with_pipeline_policies(
            options,
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            policy,
        )
    }
    /// Inject named device I/O and GUI semantics; `Self::with_backend_and_gui_surface_policy(recorder, 4096, RenderGuiSurfacePolicy::CheckedV2)?`.
    pub fn with_backend_and_gui_surface_policy(
        backend: impl RenderBackend + Send + Sync + 'static,
        limit: u32,
        policy: RenderGuiSurfacePolicy,
    ) -> Result<Handle<Self>, RenderError> {
        Self::with_injected_policies(
            backend,
            limit,
            std::sync::Arc::new(crate::FreeTypeTextSource::default()),
            Default::default(),
            Default::default(),
            policy,
        )
    }
}
