use super::*;
impl SurfaceRangeLocalizeData {
    /// Selects grid parameters strictly inside bounds; `cache.set_frame(0.0, 1.0, 0.0, 1.0)`.
    pub fn set_frame(&mut self, min_u: f64, max_u: f64, min_v: f64, max_v: f64) {
        self.frame = [[0; 2]; 2];
        if self.parameters.iter().any(Handle::is_null) {
            return;
        }
        self.frame[0] = parameter_frame(&self.parameters[0], min_u, max_u);
        self.frame[1] = parameter_frame(&self.parameters[1], min_v, max_v);
    }
    /// Native signed frame extent, which may be negative; `cache.get_nb_u_points_in_frame()`.
    pub fn get_nb_u_points_in_frame(&self) -> i32 {
        self.frame[0][1] - self.frame[0][0] + 1
    }
    /// Native signed frame extent, which may be negative; `cache.get_nb_v_points_in_frame()`.
    pub fn get_nb_v_points_in_frame(&self) -> i32 {
        self.frame[1][1] - self.frame[1][0] + 1
    }
    /// Returns the origin past the frame maximum; `cache.get_point_in_frame(1, 1)?`.
    pub fn get_point_in_frame(&self, u: i32, v: i32) -> Result<Point3, ArrayError> {
        let u = u + self.frame[0][0] - 1;
        let v = v + self.frame[1][0] - 1;
        if self.points.is_null() || u > self.frame[0][1] || v > self.frame[1][1] {
            return Ok(Point3::default());
        }
        self.get_grid_point(u, v)
    }
    fn frame_parameter(&self, axis: usize, index: i32) -> Result<f64, ArrayError> {
        let index = index + self.frame[axis][0] - 1;
        if self.parameters[axis].is_null() || index > self.frame[axis][1] {
            return Ok(rustycad_tkernel::precision::infinite());
        }
        self.parameters[axis].array1().value(index).copied()
    }
    /// Returns Precision::Infinite past the frame maximum; `cache.get_u_param_in_frame(1)?`.
    pub fn get_u_param_in_frame(&self, index: i32) -> Result<f64, ArrayError> {
        self.frame_parameter(0, index)
    }
    /// Returns Precision::Infinite past the frame maximum; `cache.get_v_param_in_frame(1)?`.
    pub fn get_v_param_in_frame(&self, index: i32) -> Result<f64, ArrayError> {
        self.frame_parameter(1, index)
    }
}
fn parameter_frame(parameters: &HArray1<f64>, minimum: f64, maximum: f64) -> [i32; 2] {
    let parameters = parameters.array1();
    let length = parameters.length();
    let mut frame = [0, 0];
    for index in 1..=length {
        if frame[0] == 0 && minimum < *parameters.value(index).unwrap() {
            frame[0] = index;
        }
        let reverse = length - index + 1;
        if frame[1] == 0 && maximum > *parameters.value(reverse).unwrap() {
            frame[1] = reverse;
        }
    }
    if frame[0] == 0 {
        frame[0] = length + 1;
    }
    frame
}
