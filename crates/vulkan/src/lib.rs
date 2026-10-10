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
mod primitive_ranges;
mod primitive_topology;
pub use graphic_group::Graphic3dDrawGroup;

mod style;

mod render_layers;
pub use render_layers::{RenderLayer, RenderLayerSpan};

mod driver;
mod driver_backend;
mod driver_structure;
mod driver_text;
mod driver_view;
mod view_capture;
mod view_forward;
mod view_scene;
mod view_target;
pub use driver::Graphic3dRenderDriver;
pub use driver_view::Graphic3dRenderView;
