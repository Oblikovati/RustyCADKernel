//! OCCT overloads share initialization and search without changing mutation order.
use super::*;
impl ContapContour {
    /// Create a parallel-view search without performing it.
    ///
    /// `let contour = ContapContour::with_direction(Vector3::new(0., 0., 1.))?;`
    pub fn with_direction(direction: Vector3) -> Result<Self, Failure> {
        let mut contour = Self::new();
        contour.init_direction(direction)?;
        Ok(contour)
    }
    /// Create a draft-angle search without performing it.
    ///
    /// `let contour = ContapContour::with_direction_angle(Vector3::new(0., 0., 1.), 0.2)?;`
    pub fn with_direction_angle(direction: Vector3, angle: f64) -> Result<Self, Failure> {
        let mut contour = Self::new();
        contour.init_direction_angle(direction, angle)?;
        Ok(contour)
    }
    /// Create a perspective search without performing it.
    ///
    /// `let contour = ContapContour::with_eye(Point3::new(3., 4., 5.));`
    pub fn with_eye(eye: Point3) -> Self {
        let mut contour = Self::new();
        contour.init_eye(eye);
        contour
    }
    /// Create and perform a parallel-view contour search.
    ///
    /// `let contour = ContapContour::with_surface_direction(&surface, &domain, direction)?;`
    pub fn with_surface_direction(
        surface: &Adaptor3dSurfaceHandle,
        domain: &(dyn TheTopolTool + 'static),
        direction: Vector3,
    ) -> Result<Self, Failure> {
        let mut contour = Self::with_direction(direction)?;
        contour.perform(surface, domain)?;
        Ok(contour)
    }
    /// Create and perform a draft-angle contour search.
    ///
    /// `let contour = ContapContour::with_surface_direction_angle(&surface, &domain, direction, 0.2)?;`
    pub fn with_surface_direction_angle(
        surface: &Adaptor3dSurfaceHandle,
        domain: &(dyn TheTopolTool + 'static),
        direction: Vector3,
        angle: f64,
    ) -> Result<Self, Failure> {
        let mut contour = Self::with_direction_angle(direction, angle)?;
        contour.perform(surface, domain)?;
        Ok(contour)
    }
    /// Create and perform a perspective contour search.
    ///
    /// `let contour = ContapContour::with_surface_eye(&surface, &domain, eye)?;`
    pub fn with_surface_eye(
        surface: &Adaptor3dSurfaceHandle,
        domain: &(dyn TheTopolTool + 'static),
        eye: Point3,
    ) -> Result<Self, Failure> {
        let mut contour = Self::with_eye(eye);
        contour.perform(surface, domain)?;
        Ok(contour)
    }
    /// Select a parallel view and perform using the native mutation order.
    ///
    /// `contour.perform_direction(&surface, &domain, direction)?;`
    pub fn perform_direction(
        &mut self,
        surface: &Adaptor3dSurfaceHandle,
        domain: &(dyn TheTopolTool + 'static),
        direction: Vector3,
    ) -> Result<(), Failure> {
        self.init_direction(direction)?;
        self.perform(surface, domain)
    }
    /// Select a draft view and perform using the native mutation order.
    ///
    /// `contour.perform_direction_angle(&surface, &domain, direction, 0.2)?;`
    pub fn perform_direction_angle(
        &mut self,
        surface: &Adaptor3dSurfaceHandle,
        domain: &(dyn TheTopolTool + 'static),
        direction: Vector3,
        angle: f64,
    ) -> Result<(), Failure> {
        self.init_direction_angle(direction, angle)?;
        self.perform(surface, domain)
    }
    /// Select a perspective view and perform using the native mutation order.
    ///
    /// `contour.perform_eye(&surface, &domain, eye)?;`
    pub fn perform_eye(
        &mut self,
        surface: &Adaptor3dSurfaceHandle,
        domain: &(dyn TheTopolTool + 'static),
        eye: Point3,
    ) -> Result<(), Failure> {
        self.init_eye(eye);
        self.perform(surface, domain)
    }
}
