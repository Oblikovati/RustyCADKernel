//! Immutable submissions separate geometry validation from device operations.
use crate::error::{RenderError, rejected};
use crate::{PrimitiveDraw, RenderLayer, RenderLayerSpan, render_layers::flatten_layers};
/// Position and linear RGBA color; `RenderVertex::new([0.,0.,0.],[1.,0.,0.,1.])`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderVertex {
    /// Object-space coordinates.
    pub position: [f32; 3],
    /// Linear, unpremultiplied RGBA.
    pub color: [f32; 4],
}
impl RenderVertex {
    /// Construct a vertex; validation occurs at submission, e.g. `RenderVertex::new([0.;3],[1.;4])`.
    pub fn new(position: [f32; 3], color: [f32; 4]) -> Self {
        Self { position, color }
    }
}
/// OpenGL clip identity; `let transform = identity_clip()`.
pub fn identity_clip() -> [f32; 16] {
    [
        1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
    ]
}
/// Immutable frame input; `RenderFrame::new(64,64,[0.,0.,0.,1.],draws)?`.
#[derive(Clone, Debug)]
pub struct RenderFrame {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) clear_color: [f32; 4],
    pub(crate) draws: Vec<PrimitiveDraw>,
    pub(crate) layers: Vec<RenderLayerSpan>,
}
impl RenderFrame {
    /// Read target dimensions; `frame.extent()`.
    pub fn extent(&self) -> [u32; 2] {
        [self.width, self.height]
    }
    /// Read the linear clear color; `frame.clear_color()`.
    pub fn clear_color(&self) -> [f32; 4] {
        self.clear_color
    }
    /// Read validated draws for an injected backend; `frame.draws()`.
    pub fn draws(&self) -> &[PrimitiveDraw] {
        &self.draws
    }
    /// Validate extent and clear color; `RenderFrame::new(64,64,[0.;4],vec![])?`.
    pub fn new(
        width: u32,
        height: u32,
        clear_color: [f32; 4],
        draws: Vec<PrimitiveDraw>,
    ) -> Result<Self, RenderError> {
        Self::with_layers(
            width,
            height,
            clear_color,
            vec![RenderLayer::new(draws, true, true, true)],
        )
    }
    /// Read ordered depth policies; `frame.layers()`.
    pub fn layers(&self) -> &[RenderLayerSpan] {
        &self.layers
    }
    /// Build an ordered frame; an empty layer list still clears its attachments.
    /// Example: `RenderFrame::with_layers(64, 64, [0.;4], layers)?`.
    pub fn with_layers(
        width: u32,
        height: u32,
        clear_color: [f32; 4],
        mut layers: Vec<RenderLayer>,
    ) -> Result<Self, RenderError> {
        if layers.is_empty() {
            layers.push(RenderLayer::new(vec![], true, true, true));
        }
        let (draws, layers) = flatten_layers(layers);
        validate_extent(width, height)?;
        if !clear_color.iter().all(|v| v.is_finite()) {
            return Err(rejected(clear_color, "finite clear color"));
        }
        Ok(Self {
            width,
            height,
            clear_color,
            draws,
            layers,
        })
    }
}
/// Host capture with top-left origin; `capture.pixel(0,0)`.
#[derive(Clone, Debug)]
pub struct FrameCapture {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Tightly packed sRGB color and linear alpha bytes, row-major from top left.
    pub rgba: Vec<u8>,
    /// Vulkan depth values in `[0,1]`, same pixel order as color.
    pub depth: Vec<f32>,
}
impl FrameCapture {
    /// Read a checked pixel; `capture.pixel(10,20)`.
    pub fn pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let offset = (y as usize * self.width as usize + x as usize) * 4;
        self.rgba.get(offset..offset + 4)?.try_into().ok()
    }
}
/// Device seam: tests inject a named recording backend; `backend.render(&frame)?`.
pub trait RenderBackend {
    /// Execute a complete immutable frame and return its capture; `backend.render(&frame)?`.
    fn render(&mut self, frame: &RenderFrame) -> Result<FrameCapture, RenderError>;
}

pub(crate) fn validate_components(
    vertices: &[RenderVertex],
    clip_transform: &[f32; 16],
) -> Result<(), RenderError> {
    if !clip_transform.iter().all(|v| v.is_finite())
        || vertices.iter().any(|v| {
            !v.position
                .iter()
                .chain(v.color.iter())
                .all(|c| c.is_finite())
        })
    {
        return Err(rejected(
            (vertices, clip_transform),
            "finite vertex components and clip transform",
        ));
    }
    if let Some(vertex) = vertices.iter().find(|v| v.color[3] != 1.0) {
        return Err(rejected(
            vertex.color,
            "opaque vertex colors in this renderer slice",
        ));
    }
    Ok(())
}

fn validate_extent(width: u32, height: u32) -> Result<(), RenderError> {
    if width == 0
        || height == 0
        || width
            .checked_mul(height)
            .and_then(|v| v.checked_mul(4))
            .is_none()
    {
        return Err(rejected(
            [width, height],
            "nonzero extent fitting a 32-bit RGBA byte count",
        ));
    }
    Ok(())
}
