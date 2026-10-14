//! Martensite's Vulkan 1.3 renderer with immutable submissions and offscreen captures.
#[allow(unsafe_code)]
mod backend;
mod error;
mod render_driver_identity;
pub use render_driver_identity::RenderDriverIdentity;
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

mod capture_image_destination;
#[cfg(test)]
mod capture_image_destination_fixture;
#[cfg(test)]
mod capture_image_destination_tests;
mod capture_image_error;
mod capture_image_layout;
#[cfg(test)]
mod capture_image_rejection_tests;
#[cfg(test)]
mod capture_owned_fixture;
mod driver;
mod driver_backend;
mod driver_structure;
mod driver_text;
mod driver_view;
mod view_capture;
pub use capture_image_error::{
    CaptureImageError, CaptureImageFailure, CaptureImageLayout, CaptureImageOperation,
};
#[cfg(test)]
mod view_capture_owned_tests;
mod view_forward;
mod view_gui_attachment;
mod view_gui_composition;
mod view_gui_layer;
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
mod pixel_text;
mod text_layout;
mod text_logic;
pub use pixel_text::{PixelTextView, lower_pixel_text};
pub use text_layout::{GlyphLayout, TextLayoutSource};
mod text_font;
pub use text_font::FreeTypeTextSource;
mod text_atlas;

mod text_background;
mod text_culling;
mod text_drawing;
mod text_submission;
mod text_transform;

mod clipping;
pub use clipping::RenderClipping;

mod retained_clipping;

mod capping_order;
mod capping_pass;
pub use capping_pass::RenderCappingPass;

mod face_culling;
pub use face_culling::RenderFaceCulling;

mod capping_geometry;
mod capping_planes;
mod capping_style;
mod capping_texture;
mod mesh_edges;
mod retained_aspect_textures;
mod retained_cap_textures;
mod retained_capping;
mod retained_eye_edge_matrix;
mod retained_mesh_edges;
mod surface_alpha;
pub use mesh_edges::{RenderMeshEdgePipeline, RenderMeshEdges};

mod material;
mod material_components;
pub use material::{RenderMaterial, RenderMaterialColorSpace};
pub use material_components::{RenderMaterialCommon, RenderMaterialPbr};

mod light_snapshot;
pub use light_snapshot::RenderLightSet;

mod lighting;
mod lighting_pipeline;
pub use lighting::{RenderLighting, RenderShading};
pub use lighting_pipeline::RenderLightingPipeline;

mod retained_lighting;

mod capping_lighting;
mod capping_mask_aspect;

mod quad_assembly;

mod fixed_flat_sources;
mod fixed_light_snapshot;
mod fixed_lighting;
mod fixed_line_stipple;
mod fixed_marker_environment;
mod fixed_normalization;
mod fixed_surface_environment;
mod fixed_view_lighting;
mod polygon_offset;
pub use polygon_offset::RenderPolygonOffset;

mod retained_surface_texture;
mod retained_texture_images;
mod retained_texture_samplers;
mod surface_texgen;
mod surface_texture;
mod surface_texture_compressed;
mod surface_texture_lifetime;
mod surface_texture_pixels;
mod surface_texture_uv;
mod texture_coordinate_frame;
mod texture_coordinate_pipeline;
pub use texture_coordinate_pipeline::RenderTextureCoordinatePipeline;
mod view_render_camera;

mod compatibility_text_color;
mod fragment_alpha_feedback;
mod legacy_fragment_alpha;
mod transparency;
pub use transparency::RenderTransparency;

mod raster_feedback;
mod retained_transparency;

mod stencil_context;
mod stencil_program;
pub use stencil_context::RenderStencilState;
mod stencil_feedback;
mod stencil_plan;
