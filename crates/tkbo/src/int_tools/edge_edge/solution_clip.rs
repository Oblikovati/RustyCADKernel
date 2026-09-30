//! Alternate clipping retains the native thin-box test before each parameter search.
use super::{RangeBoxes, SolutionSearch};
use crate::int_tools::Range;
use rustycad_tkernel::standard::Failure;
impl SolutionSearch<'_> {
    pub(super) fn clip(&self, patch: &mut RangeBoxes) -> Result<Option<bool>, Failure> {
        loop {
            let previous = patch.ranges;
            let first = self.clip_side(patch, previous, 0)?;
            if first != Some(false) {
                return Ok(first);
            }
            patch.boxes[1] = self.curves[1].bounds(patch.ranges[1])?;
            let second = self.clip_side(patch, previous, 1)?;
            if second != Some(false) {
                return Ok(second);
            }
            if self.small_steps(previous, patch.ranges) {
                return Ok(Some(false));
            }
            patch.boxes[0] = self.curves[0].bounds(patch.ranges[0])?;
        }
    }
    fn clip_side(
        &self,
        patch: &mut RangeBoxes,
        previous: [Range; 2],
        side: usize,
    ) -> Result<Option<bool>, Failure> {
        if patch.boxes[0].is_out_box(&patch.boxes[1]) {
            return Ok(None);
        }
        let range = patch.ranges[side];
        let bounds = &patch.boxes[side];
        let thin = range.last() - range.first() < self.curves[side].resolution
            || (bounds.is_x_thin(self.tolerance)
                && bounds.is_y_thin(self.tolerance)
                && bounds.is_z_thin(self.tolerance));
        let target = 1 - side;
        if !self.curves[target].parameters(previous[target], bounds, &mut patch.ranges[target])? {
            return Ok(None);
        }
        Ok(Some(thin))
    }
    fn small_steps(&self, previous: [Range; 2], current: [Range; 2]) -> bool {
        (0..2).all(|index| {
            let mut step = (previous[index].last() - previous[index].first()) / 250.;
            if step < self.curves[index].resolution {
                step = self.curves[index].resolution;
            }
            current[index].first() - previous[index].first() < step
                && previous[index].last() - current[index].last() < step
        })
    }
}
