use super::{Failure, MarkedRangeSet};
impl MarkedRangeSet {
    /// First interval containing a value, inclusive at both ends; `set.get_index(0.5)?`.
    pub fn get_index(&self, value: f64) -> Result<i32, Failure> {
        if value < self.bound(1)? {
            return Ok(0);
        }
        Ok(self
            .bounds
            .iter()
            .skip(1)
            .position(|bound| value <= *bound)
            .map_or(0, |i| i as i32 + 1))
    }
    /// Selects the lower- or upper-inclusive interval; `set.get_index_with_side(0.5, true)?`.
    pub fn get_index_with_side(&self, value: f64, use_lower: bool) -> Result<i32, Failure> {
        let first = self.bound(1)?;
        if (use_lower && value < first) || (!use_lower && value <= first) {
            return Ok(0);
        }
        let position = self
            .bounds
            .iter()
            .skip(1)
            .position(|bound| (use_lower && value < *bound) || (!use_lower && value <= *bound));
        Ok(position.map_or(0, |i| i as i32 + 1))
    }
    /// Returns all matching intervals in a reusable result buffer; `set.get_indices(0.5)?`.
    pub fn get_indices(&mut self, value: f64) -> Result<&[i32], Failure> {
        self.found_indices.clear();
        if value < self.bound(1)? {
            return Ok(&self.found_indices);
        }
        let mut found = false;
        for i in 1..self.bounds.len() {
            if found && value < self.bounds[i - 1] {
                break;
            }
            if found || value <= self.bounds[i] {
                self.found_indices.push(i as i32);
                found = true;
            }
        }
        Ok(&self.found_indices)
    }
}
