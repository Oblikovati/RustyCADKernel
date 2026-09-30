use super::{Failure, FunctionAllRoots};
impl FunctionAllRoots {
    /// Reports completion; `solver.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Counts null intervals; `solver.number_of_intervals()?`.
    pub fn number_of_intervals(&self) -> Result<i32, Failure> {
        self.check_done()?;
        Ok(self.intervals.len() as i32)
    }
    /// Counts isolated points; `solver.number_of_points()?`.
    pub fn number_of_points(&self) -> Result<i32, Failure> {
        self.check_done()?;
        Ok(self.points.len() as i32)
    }
    /// Reads one-based interval endpoints; `solver.interval(1)?`.
    pub fn interval(&self, index: i32) -> Result<[f64; 2], Failure> {
        let pair: &[super::RootState; 2] =
            &self.intervals[self.check_index(index, self.intervals.len())?];
        Ok([pair[0].parameter, pair[1].parameter])
    }
    /// Reads states saved at interval endpoints; `solver.interval_state(1)?`.
    pub fn interval_state(&self, index: i32) -> Result<[i32; 2], Failure> {
        let pair: &[super::RootState; 2] =
            &self.intervals[self.check_index(index, self.intervals.len())?];
        Ok([pair[0].state, pair[1].state])
    }
    /// Reads a one-based isolated point; `solver.point(1)?`.
    pub fn point(&self, index: i32) -> Result<f64, Failure> {
        Ok(self.points[self.check_index(index, self.points.len())?].parameter)
    }
    /// Reads the state saved at an isolated point; `solver.point_state(1)?`.
    pub fn point_state(&self, index: i32) -> Result<i32, Failure> {
        Ok(self.points[self.check_index(index, self.points.len())?].state)
    }
    fn check_done(&self) -> Result<(), Failure> {
        if !self.done {
            return Err(Failure::with_type("StdFail_NotDone", " ", ""));
        }
        Ok(())
    }
    fn check_index(&self, index: i32, count: usize) -> Result<usize, Failure> {
        self.check_done()?;
        if index < 1 || index as usize > count {
            return Err(Failure::with_type(
                "Standard_OutOfRange",
                &format!("index {index}, expected 1..={count}"),
                "",
            ));
        }
        Ok(index as usize - 1)
    }
}
