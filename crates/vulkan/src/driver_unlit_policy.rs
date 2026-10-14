//! Additive opt-in constructors share the existing driver initialization paths.
use crate::{
    Graphic3dRenderDriver, RenderBackend, RenderError, RenderUnlitFaceColorPolicy, VulkanOptions,
};
use rustycad_tkernel::standard::Handle;
use std::sync::Arc;
impl Graphic3dRenderDriver {
    /// Select unlit replay semantics; `Self::with_unlit_face_color_policy(options, RenderUnlitFaceColorPolicy::SideColorsV2)?`.
    pub fn with_unlit_face_color_policy(
        options: VulkanOptions,
        policy: RenderUnlitFaceColorPolicy,
    ) -> Result<Handle<Self>, RenderError> {
        Self::with_pipeline_policies(
            options,
            Default::default(),
            Default::default(),
            Default::default(),
            policy,
            Default::default(),
        )
    }
    /// Inject device I/O and unlit semantics; `Self::with_backend_and_unlit_face_color_policy(recorder, 4096, RenderUnlitFaceColorPolicy::SideColorsV2)?`.
    pub fn with_backend_and_unlit_face_color_policy(
        backend: impl RenderBackend + Send + Sync + 'static,
        limit: u32,
        policy: RenderUnlitFaceColorPolicy,
    ) -> Result<Handle<Self>, RenderError> {
        Self::with_injected_policies(
            backend,
            limit,
            Arc::new(crate::FreeTypeTextSource::default()),
            Default::default(),
            policy,
            Default::default(),
        )
    }
}
