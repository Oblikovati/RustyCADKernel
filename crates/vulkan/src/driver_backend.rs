//! Device injection keeps viewer tests independent from Vulkan availability.
use crate::{FrameCapture, RenderBackend, RenderError, RenderFrame, VulkanRenderer};
use rustycad_tkernel::standard::SharedState;
use std::sync::Arc;
pub(crate) type SharedRenderDevice = Arc<SharedState<DriverBackend>>;
pub(crate) enum DriverBackend {
    Vulkan(VulkanRenderer),
    Injected(Box<dyn RenderBackend + Send + Sync>),
}
impl DriverBackend {
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
    pub fn name(&self) -> &str {
        match self {
            Self::Vulkan(device) => device.device_name(),
            Self::Injected(_) => "injected Martensite render backend",
        }
    }
}
