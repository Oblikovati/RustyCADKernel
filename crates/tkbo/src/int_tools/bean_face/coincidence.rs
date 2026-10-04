//! Native coincidence sampling grows intervals before falling back to zero-width markers.
use super::{BeanFaceIntersector, Failure, empty_range::set_empty_result};

impl BeanFaceIntersector {
    /// Checks whole-range coincidence with native adaptive sampling; `solver.test_compute_coinside()?`.
    pub fn test_compute_coinside(&mut self) -> Result<bool, Failure> {
        let (first, last) = (self.range.first(), self.range.last());
        let mut uv = (0., 0.);
        if self.distance_uv(first, &mut uv.0, &mut uv.1)? > self.criteria {
            return Ok(false);
        }
        self.compute_range_from_start_point(true, first, uv.0, uv.1)?;
        let index = self.range_manager.get_index_with_side(last, false)?;
        if index != 0 && self.range_manager.flag(index)? == 2 {
            return Ok(true);
        }
        if self.distance_uv(last, &mut uv.0, &mut uv.1)? > self.criteria {
            return Ok(false);
        }
        self.compute_range_from_start_point(false, last, uv.0, uv.1)?;
        self.sample_coincident_interior(first, last)
    }

    fn sample_coincident_interior(&mut self, first: f64, last: f64) -> Result<bool, Failure> {
        let step = (last - first) / 23.;
        for index in 1..23 {
            let parameter = first + f64::from(index) * step;
            let mut uv = (0., 0.);
            if self.distance_uv(parameter, &mut uv.0, &mut uv.1)? > self.criteria {
                return Ok(false);
            }
            let count = self.range_manager.length();
            self.compute_range_from_start_point(false, parameter, uv.0, uv.1)?;
            self.compute_range_from_start_point(true, parameter, uv.0, uv.1)?;
            if count == self.range_manager.length() {
                set_empty_result(parameter, &mut self.range_manager)?;
            }
        }
        Ok(true)
    }
}
