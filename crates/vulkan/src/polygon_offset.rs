//! Native polygon offset flags remain ordered state, independently of draw topology.
use crate::{RenderError, error::rejected};
/// Native `(mode, slope factor, depth units)`; `RenderPolygonOffset::new(1, 1., 1.)?`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RenderPolygonOffset {
    mode: i32,
    factor: f32,
    units: f32,
}
impl RenderPolygonOffset {
    /// Validate the OCCT mask and finite factors; `RenderPolygonOffset::new(8, 0., 0.)?` preserves state.
    pub fn new(mode: i32, factor: f32, units: f32) -> Result<Self, RenderError> {
        if mode & !15 != 0 || !factor.is_finite() || !units.is_finite() {
            return Err(rejected(
                (mode, factor, units),
                "polygon offset mask 0..=15 and finite factor/units",
            ));
        }
        Ok(Self {
            mode,
            factor,
            units,
        })
    }
    /// Read native parameters; `offset.parameters()`.
    pub fn parameters(self) -> (i32, f32, f32) {
        (self.mode, self.factor, self.units)
    }
}
/// Ordered state resolves `Aspect_POM_None` without leaking bias onto lines or text.
#[derive(Default)]
pub(crate) struct PolygonOffsetState {
    current: RenderPolygonOffset,
}
impl PolygonOffsetState {
    pub fn new(layer: RenderPolygonOffset) -> Self {
        Self { current: layer }
    }
    // OpenGl_Text::BackPolygonOffsetSentry at 3d097a0 temporarily forces Fill and adds one unit.
    pub fn shadow_bias(&self) -> [f32; 2] {
        [self.current.units + 1., self.current.factor]
    }
    pub fn advance(&mut self, offset: Option<RenderPolygonOffset>, polygon: bool) -> [f32; 2] {
        if let Some(offset) = offset.filter(|offset| offset.mode & 8 == 0) {
            self.current = offset;
        }
        if !polygon || self.current.mode & 1 == 0 {
            return [0., 0.];
        }
        [self.current.units, self.current.factor]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shadow_fill_bias_is_temporary_and_preserves_factor_even_when_fill_disabled() {
        let mut state = PolygonOffsetState::new(RenderPolygonOffset::new(2, 3., -2.).unwrap());
        assert_eq!(state.shadow_bias(), [-1., 3.]);
        assert_eq!(state.advance(None, true), [0., 0.]);
        state.advance(Some(RenderPolygonOffset::new(1, 5., 8.).unwrap()), true);
        assert_eq!(state.shadow_bias(), [9., 5.]);
        assert_eq!(state.advance(None, true), [8., 5.]);
    }
    #[test]
    fn native_masks_and_finite_parameters_are_validated() {
        for mode in 0..=15 {
            assert_eq!(
                RenderPolygonOffset::new(mode, -2., 3.)
                    .unwrap()
                    .parameters(),
                (mode, -2., 3.)
            );
        }
        for (mode, factor, units) in [
            (-1, 0., 0.),
            (16, 0., 0.),
            (1, f32::NAN, 0.),
            (1, 0., f32::INFINITY),
        ] {
            assert!(RenderPolygonOffset::new(mode, factor, units).is_err());
        }
    }
    #[test]
    fn none_preserves_ordered_offset_and_nonpolygons_temporarily_suppress_it() {
        let mut state = PolygonOffsetState::new(RenderPolygonOffset::new(1, 2., 3.).unwrap());
        assert_eq!(state.advance(None, true), [3., 2.]);
        assert_eq!(
            state.advance(Some(RenderPolygonOffset::new(8, 9., 9.).unwrap()), true),
            [3., 2.]
        );
        assert_eq!(state.advance(None, false), [0., 0.]);
        assert_eq!(state.advance(None, true), [3., 2.]);
        assert_eq!(
            state.advance(Some(RenderPolygonOffset::new(2, 4., 5.).unwrap()), true),
            [0., 0.]
        );
        assert_eq!(
            state.advance(Some(RenderPolygonOffset::default()), true),
            [0., 0.]
        );
    }
}
