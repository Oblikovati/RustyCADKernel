//! Executed opaque events and cap sentries determine the inherited transparent context.
use crate::{RenderFrame, RenderLayerSpan, RenderStencilState, stencil_program::StencilBoundary};
type BoundaryCursor<'a> = std::iter::Peekable<std::slice::Iter<'a, StencilBoundary>>;
#[derive(Clone, Debug)]
pub(crate) struct StencilPlan {
    pub during: Vec<RenderStencilState>,
    pub after_layer: Vec<RenderStencilState>,
    pub clear_before: Vec<bool>,
    pub clear_after_layer: Vec<bool>,
    pub ending: RenderStencilState,
}
impl StencilPlan {
    pub fn new(frame: &RenderFrame, state: RenderStencilState) -> Self {
        let mut plan = Self::allocated(frame, state);
        for layer in &frame.layers {
            plan.layer(frame, layer);
        }
        plan
    }
    fn allocated(frame: &RenderFrame, state: RenderStencilState) -> Self {
        Self {
            during: Vec::with_capacity(frame.draws.len()),
            after_layer: Vec::with_capacity(frame.layers.len()),
            clear_before: Vec::with_capacity(frame.draws.len()),
            clear_after_layer: Vec::with_capacity(frame.layers.len()),
            ending: state,
        }
    }
    fn layer(&mut self, frame: &RenderFrame, layer: &RenderLayerSpan) {
        let draws = &frame.draws[layer.draw_range.clone()];
        let mut cursor = layer.stencil_boundaries.iter().peekable();
        for (index, draw) in draws.iter().enumerate() {
            self.clear_before
                .push(execute_boundaries(&mut self.ending, &mut cursor, index));
            self.ending.capping(draw.capping_pass);
            self.during.push(self.ending);
        }
        self.clear_after_layer.push(execute_boundaries(
            &mut self.ending,
            &mut cursor,
            draws.len(),
        ));
        self.after_layer.push(self.ending);
    }
}
fn execute_boundaries(
    state: &mut RenderStencilState,
    cursor: &mut BoundaryCursor<'_>,
    index: usize,
) -> bool {
    let mut clear = false;
    while let Some(boundary) = cursor.next_if(|event| event.before_draw == index) {
        state.command(boundary.command);
        clear |= boundary.command == crate::stencil_program::StencilCommand::CappingRestore;
    }
    clear
}
#[cfg(test)]
#[path = "stencil_plan_tests.rs"]
mod tests;
