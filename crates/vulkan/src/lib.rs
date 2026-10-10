//! Martensite's Vulkan 1.3 renderer with immutable submissions and offscreen captures.
#[allow(unsafe_code)]
mod backend;
mod error;
mod scene;
pub use backend::{VulkanOptions, VulkanRenderer};
pub use error::RenderError;
pub use scene::{
    FrameCapture, RenderBackend, RenderFrame, RenderVertex, TriangleDraw, identity_clip,
};
mod graphic_group;
mod primitive;
pub use graphic_group::Graphic3dDrawGroup;

mod style;
