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
    ) -> Result<FrameCapture, RenderError> {
        match self {
            Self::Vulkan(device) => device.render_window(frame, target, vsync),
            Self::Injected(_) => Err(crate::error::rejected(
                "injected backend",
                "a Vulkan window presenter",
            )),
        }
    }
}
