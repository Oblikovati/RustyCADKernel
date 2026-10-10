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
        draws: Vec<PrimitiveDraw>,
        depth_test: bool,
        depth_write: bool,
        clear_depth: bool,
    ) -> Self {
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
    for layer in layers {
        let start = draws.len();
        draws.extend(layer.draws);
        spans.push(RenderLayerSpan {
            draw_range: start..draws.len(),
            depth_test: layer.depth_test,
            depth_write: layer.depth_write,
            clear_depth: layer.clear_depth,
        });
    }
    (draws, spans)
}
