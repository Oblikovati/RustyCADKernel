//! Martensite's Vulkan 1.3 renderer with immutable submissions and offscreen captures.
#[allow(unsafe_code)]
mod backend;
mod error;
mod scene;
pub use backend::{VulkanOptions, VulkanRenderer};
pub use error::RenderError;
pub use scene::{FrameCapture, RenderBackend, RenderFrame, RenderVertex, identity_clip};
mod graphic_group;
mod group_flipping;
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

mod persistent_transform;

mod draw;
pub use draw::{LineStyle, PrimitiveDraw, PrimitiveTopology, TriangleDraw};

mod line_style;

mod highlight_style;

mod point_style;
pub use point_style::PointStyle;

mod raster_image;
pub use raster_image::RasterImage;
mod textured_draw;
pub use textured_draw::TexturedStyle;
mod text_layout;
pub use text_layout::{GlyphLayout, TextLayoutSource};
mod text_font;
pub use text_font::FreeTypeTextSource;
mod text_atlas;

mod text_culling;
mod text_drawing;
mod text_submission;
mod text_transform;

mod clipping;
pub use clipping::RenderClipping;

mod retained_clipping;

mod capping_pass;
pub use capping_pass::RenderCappingPass;

mod face_culling;
pub use face_culling::RenderFaceCulling;

mod capping_geometry;
mod capping_planes;
mod capping_style;
mod retained_capping;

mod material;
mod material_components;
pub use material::{RenderMaterial, RenderMaterialColorSpace};
pub use material_components::{RenderMaterialCommon, RenderMaterialPbr};

mod light_snapshot;
pub use light_snapshot::RenderLightSet;

mod lighting;
pub use lighting::{RenderLighting, RenderShading};

mod retained_lighting;

mod capping_lighting;

mod quad_assembly;

mod polygon_offset;
pub use polygon_offset::RenderPolygonOffset;
