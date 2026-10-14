//! Layer ordering and depth policy stay explicit in immutable frame submissions.
use crate::PrimitiveDraw;
use std::ops::Range;
/// One ordered pass; `RenderLayer::new(draws, true, true, false)`.
#[derive(Clone, Debug)]
pub struct RenderLayer {
    pub(crate) draws: Vec<PrimitiveDraw>,
    pub(crate) depth_test: bool,
    pub(crate) depth_write: bool,
    pub(crate) clear_depth: bool,
    pub(crate) polygon_offset: crate::RenderPolygonOffset,
    pub(crate) stencil_boundaries: Vec<crate::stencil_program::StencilBoundary>,
}
impl RenderLayer {
    /// Retain a layer's draws and depth policy; `RenderLayer::new(draws, true, true, false)`.
    pub fn new(
        mut draws: Vec<PrimitiveDraw>,
        depth_test: bool,
        depth_write: bool,
        clear_depth: bool,
    ) -> Self {
        crate::capping_order::order_layer_draws(&mut draws);
        let stencil_boundaries = crate::stencil_program::implicit_cap_restores(&draws);
        Self {
            draws,
            depth_test,
            depth_write,
            clear_depth,
            polygon_offset: crate::RenderPolygonOffset::default(),
            stencil_boundaries,
        }
    }
}
impl RenderLayer {
    pub(crate) fn from_stencil_program(
        mut program: crate::stencil_program::StencilProgram,
        depth_test: bool,
        depth_write: bool,
        clear_depth: bool,
    ) -> Self {
        crate::capping_order::classify_capping_batches(&mut program.draws);
        Self {
            draws: program.draws,
            depth_test,
            depth_write,
            clear_depth,
            polygon_offset: Default::default(),
            stencil_boundaries: program.boundaries,
        }
    }
    /// Retain the layer's default polygon offset; `layer.with_polygon_offset(RenderPolygonOffset::new(1, 1., 1.)?)`.
    pub fn with_polygon_offset(mut self, offset: crate::RenderPolygonOffset) -> Self {
        self.polygon_offset = offset;
        self
    }
}
/// Range of frame draws sharing one depth policy; `frame.layers()[0].draw_range()`.
#[derive(Clone, Debug)]
pub struct RenderLayerSpan {
    pub(crate) draw_range: Range<usize>,
    pub(crate) depth_test: bool,
    pub(crate) depth_write: bool,
    pub(crate) clear_depth: bool,
    pub(crate) polygon_offset: crate::RenderPolygonOffset,
    pub(crate) stencil_boundaries: Vec<crate::stencil_program::StencilBoundary>,
}
impl RenderLayerSpan {
    /// Index into `frame.draws()`; `frame.layers()[0].draw_range()`.
    pub fn draw_range(&self) -> Range<usize> {
        self.draw_range.clone()
    }
    /// Read the retained layer default; `frame.layers()[0].polygon_offset()`.
    pub fn polygon_offset(&self) -> crate::RenderPolygonOffset {
        self.polygon_offset
    }
    /// Read `(test, write, clear)` flags; `frame.layers()[0].depth_policy()`.
    pub fn depth_policy(&self) -> (bool, bool, bool) {
        (self.depth_test, self.depth_write, self.clear_depth)
    }
}
pub(crate) fn flatten_layers(
    layers: Vec<RenderLayer>,
) -> (Vec<PrimitiveDraw>, Vec<RenderLayerSpan>) {
    let mut draws = Vec::new();
    let mut spans = Vec::new();
    let mut deferred = Vec::new();
    for layer in layers {
        if layer.clear_depth {
            flush_blended_layers(&mut deferred, &mut draws, &mut spans);
        }
        let (opaque, blended) = split_blended_layer(layer);
        append_layer_span(opaque, &mut draws, &mut spans);
        deferred.extend(blended);
    }
    flush_blended_layers(&mut deferred, &mut draws, &mut spans);
    (draws, spans)
}
fn split_blended_layer(mut layer: RenderLayer) -> (RenderLayer, Option<RenderLayer>) {
    let deferred: Vec<_> = layer
        .draws
        .iter()
        .map(PrimitiveDraw::is_surface_blended)
        .collect();
    let (blended, opaque): (Vec<_>, Vec<_>) = std::mem::take(&mut layer.draws)
        .into_iter()
        .partition(PrimitiveDraw::is_surface_blended);
    let transparent =
        (!blended.is_empty()).then(|| layer.stencil_partition_branch(blended, &deferred, true));
    (
        layer.stencil_partition_branch(opaque, &deferred, false),
        transparent,
    )
}
impl RenderLayer {
    fn stencil_partition_branch(
        &self,
        draws: Vec<PrimitiveDraw>,
        deferred: &[bool],
        transparent: bool,
    ) -> Self {
        Self {
            draws,
            depth_test: self.depth_test,
            depth_write: self.depth_write && !transparent,
            clear_depth: self.clear_depth && !transparent,
            polygon_offset: self.polygon_offset,
            stencil_boundaries: crate::stencil_program::filtered_boundaries(
                &self.stencil_boundaries,
                deferred,
                transparent,
            ),
        }
    }
}
fn flush_blended_layers(
    pending: &mut Vec<RenderLayer>,
    draws: &mut Vec<PrimitiveDraw>,
    spans: &mut Vec<RenderLayerSpan>,
) {
    // OpenGl_LayerList::Render at 3d097a0 flushes transparent layers before the next depth clear.
    for layer in pending.drain(..) {
        append_layer_span(layer, draws, spans);
    }
}
fn append_layer_span(
    layer: RenderLayer,
    draws: &mut Vec<PrimitiveDraw>,
    spans: &mut Vec<RenderLayerSpan>,
) {
    let start = draws.len();
    draws.extend(layer.draws);
    spans.push(RenderLayerSpan {
        draw_range: start..draws.len(),
        depth_test: layer.depth_test,
        depth_write: layer.depth_write,
        clear_depth: layer.clear_depth,
        polygon_offset: layer.polygon_offset,
        stencil_boundaries: layer.stencil_boundaries,
    });
}
