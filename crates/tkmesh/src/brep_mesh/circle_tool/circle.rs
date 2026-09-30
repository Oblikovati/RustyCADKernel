use rustycad_tkmath::gp::Xy;
/// Compact native circle, including negative deletion radii; `BRepMeshCircle::new(Xy::default(),1.)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BRepMeshCircle {
    location: Xy,
    radius: f64,
}
impl BRepMeshCircle {
    /// Store coordinates and radius without validation; `BRepMeshCircle::new(center,-1.)`.
    pub fn new(location: Xy, radius: f64) -> Self {
        Self { location, radius }
    }
    /// Replace the center; `circle.set_location(Xy::default())`.
    pub fn set_location(&mut self, location: Xy) {
        self.location = location;
    }
    /// Retain arbitrary radius bits; `circle.set_radius(-1.)`.
    pub fn set_radius(&mut self, radius: f64) {
        self.radius = radius;
    }
    /// Borrow the circle center; `circle.location()`.
    pub fn location(&self) -> &Xy {
        &self.location
    }
    /// Read the stored radius; `circle.radius()`.
    pub fn radius(&self) -> f64 {
        self.radius
    }
}
