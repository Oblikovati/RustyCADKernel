//! Device injection keeps viewer tests independent from Vulkan availability.
use crate::{FrameCapture, RenderBackend, RenderError, RenderFrame, VulkanRenderer};
use rustycad_tkernel::standard::SharedState;
use std::sync::Arc;
pub(crate) type SharedRenderDevice = Arc<SharedState<DriverBackend>>;
pub(crate) enum DriverBackend {
    Vulkan(Box<VulkanRenderer>),
    Injected(Box<dyn RenderBackend + Send + Sync>),
}
impl DriverBackend {
    pub fn raster_limit(&self, limit: rustycad_tkservice::graphic3d::Graphic3dTypeOfLimit) -> i32 {
        match self {
            Self::Vulkan(device) => device.raster_limit(limit),
            Self::Injected(_) => 0,
        }
    }
    pub fn render(&mut self, frame: &RenderFrame) -> Result<FrameCapture, RenderError> {
        match self {
            Self::Vulkan(device) => device.render(frame),
            Self::Injected(device) => device.render(frame),
        }
    }
    pub fn validation_errors(&self) -> Vec<String> {
        match self {
            Self::Vulkan(device) => device.validation_errors(),
            Self::Injected(_) => vec![],
        }
    }
    /// Return selected Vulkan provenance; injected backends have no Vulkan identity.
    /// Example: `backend.driver_identity().is_none()`.
    pub fn driver_identity(&self) -> Option<crate::RenderDriverIdentity> {
        match self {
            Self::Vulkan(device) => Some(device.driver_identity()),
            Self::Injected(_) => None,
        }
    }
    pub fn name(&self) -> &str {
        match self {
            Self::Vulkan(device) => device.device_name(),
            Self::Injected(_) => "injected Martensite render backend",
        }
    }
}

impl DriverBackend {
    pub fn window_presenter(
        &self,
        window: &rustycad_tkernel::standard::Handle<dyn rustycad_tkservice::aspect::AspectWindow>,
    ) -> Result<crate::backend::WindowPresenter, RenderError> {
        match self {
            Self::Vulkan(device) => device.window_presenter(window),
            Self::Injected(_) => Err(crate::error::rejected(
                "injected backend",
                "a Vulkan device for native window presentation",
            )),
        }
    }
    pub fn render_window(
        &mut self,
        frame: &RenderFrame,
        target: &mut crate::backend::WindowPresenter,
        vsync: bool,
        policy: crate::RenderGuiSurfacePolicy,
    ) -> Result<FrameCapture, RenderError> {
        match self {
            Self::Vulkan(device) => device.render_window(frame, target, vsync, policy),
            Self::Injected(_) => Err(crate::error::rejected(
                "injected backend",
                "a Vulkan window presenter",
            )),
        }
    }
}
