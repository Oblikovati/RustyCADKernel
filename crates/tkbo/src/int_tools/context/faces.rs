use super::*;
impl Context {
    /// Classifies using the cached face classifier and periodic recentering; `context.state_point_face(&face, uv)?`.
    pub fn state_point_face(&self, face: &Face, point: Point2) -> Result<State, Failure> {
        self.fclass2d(face)?.read().perform(point, true)
    }
    /// Accepts every state except OUT and ON, matching native handling of UNKNOWN; `context.is_point_in_face(&face, uv)?`.
    pub fn is_point_in_face(&self, face: &Face, point: Point2) -> Result<bool, Failure> {
        Ok(!matches!(
            self.state_point_face(face, point)?,
            State::Out | State::On
        ))
    }
    /// Accepts every state except OUT; `context.is_point_in_on_face(&face, uv)?`.
    pub fn is_point_in_on_face(&self, face: &Face, point: Point2) -> Result<bool, Failure> {
        Ok(self.state_point_face(face, point)? != State::Out)
    }
    /// Projects a point and uses the native strict distance test; `context.is_point3d_in_face(p, &face, 1e-7)?`.
    pub fn is_point3d_in_face(
        &self,
        point: Point3,
        face: &Face,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        let service = self.proj_ps(face)?;
        let mut projector = service.write();
        projector.perform(point)?;
        if !projector.is_done() {
            return Ok(false);
        }
        if projector.lower_distance()?.partial_cmp(&tolerance) != Some(std::cmp::Ordering::Less) {
            return Ok(false);
        }
        let (u, v) = projector.lower_distance_parameters()?;
        self.is_point_in_face(face, Point2::new(u, v))
    }
    /// Projects a point and accepts distance equal to tolerance; `context.is_valid_point_for_face(p, &face, 1e-7)?`.
    pub fn is_valid_point_for_face(
        &self,
        point: Point3,
        face: &Face,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        let service = self.proj_ps(face)?;
        let mut projector = service.write();
        projector.perform(point)?;
        if !projector.is_done() || projector.lower_distance()? > tolerance {
            return Ok(false);
        }
        let (u, v) = projector.lower_distance_parameters()?;
        self.is_point_in_on_face(face, Point2::new(u, v))
    }
    /// Requires valid projection onto both faces, short-circuiting on the first; `context.is_valid_point_for_faces(p, &f1, &f2, 1e-7)?`.
    pub fn is_valid_point_for_faces(
        &self,
        point: Point3,
        first: &Face,
        last: &Face,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        Ok(self.is_valid_point_for_face(point, first, tolerance)?
            && self.is_valid_point_for_face(point, last, tolerance)?)
    }
}
