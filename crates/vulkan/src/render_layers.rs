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
}
impl RenderLayer {
    /// Retain a layer's draws and depth policy; `RenderLayer::new(draws, true, true, false)`.
    pub fn new(
        mut draws: Vec<PrimitiveDraw>,
        depth_test: bool,
        depth_write: bool,
        clear_depth: bool,
    ) -> Self {
        // Native BLEND_UNORDERED defers transparent surfaces while retaining their submission order.
        draws.sort_by_key(PrimitiveDraw::is_surface_blended);
        Self {
            draws,
            depth_test,
            depth_write,
            clear_depth,
        }
    }
}
/// Range of frame draws sharing one depth policy; `frame.layers()[0].draw_range()`.
#[derive(Clone, Debug)]
pub struct RenderLayerSpan {
    pub(crate) draw_range: Range<usize>,
    pub(crate) depth_test: bool,
    pub(crate) depth_write: bool,
    pub(crate) clear_depth: bool,
}
impl RenderLayerSpan {
    /// Index into `frame.draws()`; `frame.layers()[0].draw_range()`.
    pub fn draw_range(&self) -> Range<usize> {
        self.draw_range.clone()
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
fn split_blended_layer(layer: RenderLayer) -> (RenderLayer, Option<RenderLayer>) {
    let RenderLayer {
        draws,
        depth_test,
        depth_write,
        clear_depth,
    } = layer;
    let (blended, opaque): (Vec<_>, Vec<_>) = draws
        .into_iter()
        .partition(PrimitiveDraw::is_surface_blended);
    let transparent =
        (!blended.is_empty()).then(|| RenderLayer::new(blended, depth_test, false, false));
    (
        RenderLayer::new(opaque, depth_test, depth_write, clear_depth),
        transparent,
    )
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
    });
}
