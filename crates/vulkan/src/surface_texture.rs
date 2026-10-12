//! Sampled surface images stay independent from coverage atlases and point sprites.
use crate::{RenderError, error::rejected};
use rustycad_tkservice::graphic3d::*;
use std::sync::Arc;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SurfaceImageEncoding {
    Srgb8,
    Linear8,
    Linear16,
    Float16,
    Float32,
    Bc1Rgb(bool),
    Bc1Rgba(bool),
    Bc2(bool),
    Bc3(bool),
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SurfaceTextureImage {
    pub lifetime: Arc<crate::surface_texture_lifetime::SurfaceTextureLifetime>,
    pub id: Vec<u8>,
    pub revision: usize,
    pub extent: [u32; 2],
    pub encoding: SurfaceImageEncoding,
    pub bytes: Arc<Vec<u8>>,
    pub top_down: bool,
    pub mipmaps: bool,
    pub supplied_levels: Vec<usize>,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SurfaceSamplerPolicy {
    pub repeat: bool,
    pub filter: Graphic3dTypeOfTextureFilter,
    pub anisotropy: Graphic3dLevelOfTextureAnisotropy,
    pub first_level: i32,
    pub last_level: i32,
}
impl SurfaceSamplerPolicy {
    pub fn read(parameters: &Graphic3dTextureParams) -> Self {
        Self {
            repeat: parameters.is_repeat(),
            filter: parameters.filter(),
            anisotropy: parameters.aniso_filter(),
            first_level: parameters.base_level(),
            last_level: parameters.max_level(),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SurfaceTextureStyle {
    pub image: Arc<SurfaceTextureImage>,
    pub pending_texgen: Option<Box<crate::surface_texgen::SurfaceTexgen>>,
    pub sampler: SurfaceSamplerPolicy,
    pub uv: Vec<[f32; 4]>,
    pub back_vertex: bool,
    pub vertex_override: bool,
    pub modulate: bool,
}
impl SurfaceTextureStyle {
    pub fn validate(&self, vertices: usize) -> Result<(), RenderError> {
        if self.uv.len() != vertices {
            return Err(rejected(
                (self.uv.len(), vertices),
                "one texture coordinate per vertex",
            ));
        }
        if let Some(uv) = self
            .uv
            .iter()
            .find(|uv| uv.iter().any(|value| !value.is_finite()))
        {
            return Err(rejected(uv, "finite Vec4 texture coordinates"));
        }
        Ok(())
    }
}

/// Explicit legacy incomplete-sampler policy; no image was decoded or uploaded.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SurfaceTextureUnavailable {
    pub resource_id: Vec<u8>,
    pub modulate: bool,
    pub vertex_override: bool,
    pub back_vertex: bool,
}
