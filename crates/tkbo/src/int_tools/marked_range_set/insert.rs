use super::{Failure, MarkedRangeSet, Range};
impl MarkedRangeSet {
    /// Inserts a flagged interval while retaining existing boundaries; `set.insert_range(0.2, 0.8, 3)?`.
    pub fn insert_range(&mut self, first: f64, last: f64, flag: i32) -> Result<bool, Failure> {
        let mut start = self.get_index_with_side(first, true)?;
        if start == 0 {
            return Ok(false);
        }
        let mut end = self.get_index_with_side(last, false)?;
        if end == 0 {
            return Ok(false);
        }
        if end < start {
            std::mem::swap(&mut start, &mut end);
            if last < first {
                return Ok(false);
            }
        }
        self.insert_across(first, last, flag, start as usize, end as usize);
        Ok(true)
    }
    /// Inserts the supplied Range; `set.insert(&Range::new(0.2, 0.8), 3)?`.
    pub fn insert(&mut self, range: &Range, flag: i32) -> Result<bool, Failure> {
        self.insert_range(range.first(), range.last(), flag)
    }
    /// Inserts within a hinted one-based interval; `set.insert_range_at(0.2, 0.8, 3, 1)?`.
    pub fn insert_range_at(
        &mut self,
        first: f64,
        last: f64,
        flag: i32,
        index: i32,
    ) -> Result<bool, Failure> {
        if index <= 0 || index > self.range_number {
            return Ok(false);
        }
        if first < self.bound(index)?
            || last > self.bound(index + 1)?
            || (first - last).abs() < 1e-15
        {
            return self.insert_range(first, last, flag);
        }
        self.insert_inside(first, last, flag, index as usize);
        Ok(true)
    }
    /// Inserts a Range with a hint; `set.insert_at(&Range::new(0.2, 0.8), 3, 1)?`.
    pub fn insert_at(&mut self, range: &Range, flag: i32, index: i32) -> Result<bool, Failure> {
        self.insert_range_at(range.first(), range.last(), flag, index)
    }
    fn insert_across(&mut self, first: f64, last: f64, flag: i32, start: usize, end: usize) {
        let previous = self.flags[start - 1];
        self.bounds.insert(start, first);
        self.flags.insert(start, flag);
        self.bounds.insert(end + 1, last);
        if start == end {
            self.flags.insert(end + 1, previous);
        } else {
            self.flags.insert(end, flag);
            self.flags[start..=end].fill(flag);
        }
        self.range_number = self.bounds.len() as i32 - 1;
    }
    fn insert_inside(&mut self, first: f64, last: f64, flag: i32, mut index: usize) {
        let previous = self.flags[index - 1];
        if self.is_interior(first, index) {
            self.bounds.insert(index, first);
            self.flags.insert(index, flag);
            index += 1;
        } else {
            self.flags[index - 1] = flag;
        }
        if self.is_interior(last, index) {
            self.bounds.insert(index, last);
            self.flags.insert(index, previous);
        }
        self.range_number = self.bounds.len() as i32 - 1;
    }
    fn is_interior(&self, value: f64, index: usize) -> bool {
        (value - self.bounds[index - 1]).abs() > 1e-15 && (value - self.bounds[index]).abs() > 1e-15
    }
}
