use super::super::{Axis1, Axis3, Direction3, Failure};
use rustycad_tkernel::precision;
impl Axis3 {
    /// Returns the stored X direction; `frame.x_direction()`.
    pub const fn x_direction(&self) -> Direction3 {
        self.x_direction
    }
    /// Returns the stored Y direction; `frame.y_direction()`.
    pub const fn y_direction(&self) -> Direction3 {
        self.y_direction
    }
    /// Tests handedness using the native normalized cross product; `frame.direct()?`.
    pub fn direct(&self) -> Result<bool, Failure> {
        Ok(self
            .x_direction
            .crossed(self.y_direction)?
            .dot(self.direction)
            > 0.0)
    }
    /// Reverses X without adjusting other directions; `frame.x_reverse()`.
    pub fn x_reverse(&mut self) {
        self.x_direction.reverse();
    }
    /// Reverses Y without adjusting other directions; `frame.y_reverse()`.
    pub fn y_reverse(&mut self) {
        self.y_direction.reverse();
    }
    /// Reverses Z without adjusting other directions; `frame.z_reverse()`.
    pub fn z_reverse(&mut self) {
        self.direction.reverse();
    }
    /// Moves the origin before updating orientation as OCCT does; `frame.set_axis(axis)?`.
    pub fn set_axis(&mut self, axis: Axis1) -> Result<(), Failure> {
        self.location = axis.location();
        self.set_direction(axis.direction())
    }
    /// Changes Z while preserving handedness (OCC29406); `frame.set_direction(normal)?`.
    pub fn set_direction(&mut self, normal: Direction3) -> Result<(), Failure> {
        let cosine = normal.dot(self.x_direction);
        if 1.0 - cosine.abs() <= precision::angular() {
            self.align_z_to_x(normal, cosine);
            return Ok(());
        }
        let direct = self.direct()?;
        self.direction = normal;
        self.x_direction = normal.cross_crossed(self.x_direction, normal)?;
        self.y_direction = self.orthogonal_y(direct)?;
        Ok(())
    }
    fn align_z_to_x(&mut self, normal: Direction3, cosine: f64) {
        if cosine > 0.0 {
            self.x_direction = self.y_direction;
            self.y_direction = self.direction;
        } else {
            self.x_direction = self.direction;
        }
        self.direction = normal;
    }
    fn orthogonal_y(&self, direct: bool) -> Result<Direction3, Failure> {
        if direct {
            self.direction.crossed(self.x_direction)
        } else {
            self.x_direction.crossed(self.direction)
        }
    }
    /// Changes X, including the native collinear-axis swap; `frame.set_x_direction(direction)?`.
    pub fn set_x_direction(&mut self, direction: Direction3) -> Result<(), Failure> {
        let cosine = direction.dot(self.direction);
        if 1.0 - cosine.abs() <= precision::angular() {
            self.direction = self.x_direction;
            if cosine > 0.0 {
                self.y_direction.reverse();
            }
            self.x_direction = direction;
            return Ok(());
        }
        let direct = self.direct()?;
        self.x_direction = self.direction.cross_crossed(direction, self.direction)?;
        self.y_direction = self.orthogonal_y(direct)?;
        Ok(())
    }
    /// Changes Y while preserving handedness (OCC29406); `frame.set_y_direction(direction)?`.
    pub fn set_y_direction(&mut self, direction: Direction3) -> Result<(), Failure> {
        let cosine = direction.dot(self.direction);
        if 1.0 - cosine.abs() <= precision::angular() {
            self.direction = self.y_direction;
            if cosine > 0.0 {
                self.x_direction.reverse();
            }
            self.y_direction = direction;
            return Ok(());
        }
        let direct = self.direct()?;
        self.x_direction = direction.crossed(self.direction)?;
        self.y_direction = self.direction.crossed(self.x_direction)?;
        if !direct {
            self.x_direction.reverse();
        }
        Ok(())
    }
}
