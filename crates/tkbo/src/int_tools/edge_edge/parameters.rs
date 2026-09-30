//! Native two-ended curve/box search preserves its last sampled boundary, OCCT 3d097a0.
use super::{edge_math::point_box_distance, edge_resolution::curve_resolution};
use crate::int_tools::Range;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::bnd::BoundingBox3;
const GOLDEN_SECTION: f64 = 0.618_033_988_749_894_9;
pub(super) struct ParameterSearch<'a> {
    pub curve: &'a BRepAdaptorCurve,
    pub range: Range,
    pub tolerance: f64,
    pub resolution: f64,
    pub precision: f64,
    pub coefficient: f64,
    pub bounds: &'a BoundingBox3,
}
struct BoundarySearch {
    parameter: f64,
    step: f64,
    previous_distance: f64,
    multiplier: f64,
    sign: f64,
}
impl ParameterSearch<'_> {
    pub(super) fn find(&self, first: &mut f64, last: &mut f64) -> Result<bool, Failure> {
        let mut inflated = self.bounds.clone();
        inflated.set_gap(inflated.gap() + self.tolerance);
        for end in 0..2 {
            let start = if end == 0 {
                self.range.first()
            } else {
                self.range.last()
            };
            let limit = if end == 0 { self.range.last() } else { *first };
            let mut state = BoundarySearch::new(start, self.resolution, end == 0);
            if !self.find_inside(&mut state, limit)?
                && !state.fallback(end, *first, self.range.last())
            {
                return Ok(false);
            }
            if state.parameter != start {
                self.refine_boundary(&mut state, &inflated)?;
            }
            if end == 0 {
                *first = state.parameter;
            } else {
                *last = state.parameter;
            }
        }
        Ok(true)
    }
    fn find_inside(&self, state: &mut BoundarySearch, limit: f64) -> Result<bool, Failure> {
        while state.sign * (limit - state.parameter) >= 0. {
            let distance = point_box_distance(self.bounds, self.curve.d0(state.parameter)?)?;
            // Native NaN comparison follows the inside branch as well.
            if distance.partial_cmp(&self.tolerance) != Some(std::cmp::Ordering::Greater) {
                return Ok(true);
            }
            if state.previous_distance > 0. {
                self.update_step(state, distance)?;
            }
            state.parameter += state.sign * state.step;
            state.previous_distance = distance;
        }
        Ok(false)
    }
    fn update_step(&self, state: &mut BoundarySearch, distance: f64) -> Result<(), Failure> {
        let mut grow = false;
        if (state.previous_distance - distance).abs() / state.previous_distance < 0.1 {
            state.step = self.distance_resolution(state.multiplier * distance)?;
            if state.step < (self.range.last() - self.range.first()) * 0.01 {
                grow = true;
                state.multiplier *= 2.;
            }
        }
        if !grow {
            state.multiplier = 1.;
            state.step = self.distance_resolution(distance)?;
        }
        Ok(())
    }
    fn distance_resolution(&self, distance: f64) -> Result<f64, Failure> {
        curve_resolution(
            &self.curve.transformed_curve().geom_curve(),
            self.curve.get_type()?,
            self.coefficient,
            distance,
        )
    }
    fn refine_boundary(
        &self,
        state: &mut BoundarySearch,
        inflated: &BoundingBox3,
    ) -> Result<(), Failure> {
        let mut inside = state.parameter;
        let mut outside = state.parameter - state.sign * state.step;
        let mut difference = inside - outside;
        while difference.abs() > self.precision {
            state.parameter = outside + difference * GOLDEN_SECTION;
            if inflated.is_out_point(self.curve.d0(state.parameter)?) {
                outside = state.parameter;
            } else {
                inside = state.parameter;
            }
            difference = inside - outside;
        }
        Ok(())
    }
}
impl BoundarySearch {
    fn new(parameter: f64, step: f64, first: bool) -> Self {
        Self {
            parameter,
            step,
            previous_distance: 0.,
            multiplier: 1.,
            sign: if first { 1. } else { -1. },
        }
    }
    fn fallback(&mut self, end: i32, first: f64, last: f64) -> bool {
        if end == 0 {
            return false;
        }
        self.parameter = first;
        self.step = last - first;
        true
    }
}
