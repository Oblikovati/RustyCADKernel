use super::*;
impl SurfaceRangeLocalizeData {
    fn grid_length(&self, axis: usize) -> i32 {
        self.parameters[axis]
            .get()
            .map_or(0, |array| array.array1().length())
    }
    fn set_grid_range(&mut self, axis: usize, length: i32) -> Result<(), ArrayError> {
        self.frame = [[0; 2]; 2];
        if !self.parameters[axis].is_null() && self.grid_length(axis) == length {
            return Ok(());
        }
        self.parameters[axis] = Handle::new(HArray1::with_bounds(1, length)?);
        if !self.parameters[1 - axis].is_null() {
            self.points = Handle::new(HArray2::with_bounds(
                1,
                self.grid_length(0),
                1,
                self.grid_length(1),
            )?);
        }
        Ok(())
    }
    /// Sets U grid extent and clears the frame; `cache.set_range_u_grid(3)?`.
    pub fn set_range_u_grid(&mut self, length: i32) -> Result<(), ArrayError> {
        self.set_grid_range(0, length)
    }
    /// Returns zero for a null grid; `cache.get_range_u_grid()`.
    pub fn get_range_u_grid(&self) -> i32 {
        self.grid_length(0)
    }
    /// Writes shared U storage; `cache.set_u_param(1, 0.5)?`.
    pub fn set_u_param(&mut self, index: i32, value: f64) -> Result<(), ArrayError> {
        self.parameters[0].change_array1().set_value(index, value)
    }
    /// Reads shared U storage; `cache.get_u_param(1)?`.
    pub fn get_u_param(&self, index: i32) -> Result<f64, ArrayError> {
        self.parameters[0].array1().value(index).copied()
    }
    /// Sets V grid extent and clears the frame; `cache.set_range_v_grid(3)?`.
    pub fn set_range_v_grid(&mut self, length: i32) -> Result<(), ArrayError> {
        self.set_grid_range(1, length)
    }
    /// Returns zero for a null grid; `cache.get_range_v_grid()`.
    pub fn get_range_v_grid(&self) -> i32 {
        self.grid_length(1)
    }
    /// Writes shared V storage; `cache.set_v_param(1, 0.5)?`.
    pub fn set_v_param(&mut self, index: i32, value: f64) -> Result<(), ArrayError> {
        self.parameters[1].change_array1().set_value(index, value)
    }
    /// Reads shared V storage; `cache.get_v_param(1)?`.
    pub fn get_v_param(&self, index: i32) -> Result<f64, ArrayError> {
        self.parameters[1].array1().value(index).copied()
    }
    /// Writes the shared grid; `cache.set_grid_point(1, 1, Point3::default())?`.
    pub fn set_grid_point(&mut self, u: i32, v: i32, point: Point3) -> Result<(), ArrayError> {
        self.points.change_array2().set_value(u, v, point)
    }
    /// Reads the shared point value; `cache.get_grid_point(1, 1)?`.
    pub fn get_grid_point(&self, u: i32, v: i32) -> Result<Point3, ArrayError> {
        self.points.array2().value(u, v).copied()
    }
}
