// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_Direction.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2d_Direction`: a unit vector as transient geometry.
use super::errors;
use super::geometry::Geom2dGeometry;
use super::protected::ProtectedAccess;
use super::vector::Geom2dVector;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::gp::{Direction2, Transform2, Vector2, resolution};

/// A unit vector; `Handle::new(Geom2dDirection::new(1.0, 1.0)?)`.
pub struct Geom2dDirection {
    parent: StandardTransient,
    state: SharedState<Vector2>,
}
impl_transient!(
    Geom2dDirection,
    "Geom2d_Direction",
    dyn Geom2dVector,
    parent,
    interfaces: [dyn Geom2dGeometry, dyn Geom2dVector]
);

// X / D and Y / D after the native zero-length check of `where_` (Standard_ConstructionError).
fn normalized_coordinates(x: f64, y: f64, where_: &str) -> Result<Vector2, Failure> {
    let d: f64 = (x * x + y * y).sqrt();
    if d <= resolution() {
        return Err(errors::construction(&format!(
            "{where_} - input vector ({x}, {y}) has zero length; expected a length > gp::Resolution()"
        )));
    }
    Ok(Vector2::new(x / d, y / d))
}

impl Geom2dDirection {
    fn stored(vector: Vector2) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(vector),
        }
    }
    /// Unit vector along `(x, y)` (`Geom2d_Direction(X, Y)`); `Geom2dDirection::new(3.0, 4.0)?`.
    pub fn new(x: f64, y: f64) -> Result<Self, Failure> {
        let unit: Vector2 = normalized_coordinates(x, y, "Geom2d_Direction()")?;
        Ok(Self::stored(unit))
    }
    /// Direction from a gp direction (`Geom2d_Direction(gp_Dir2d)`);
    /// `Geom2dDirection::from_dir2d(direction)`.
    pub fn from_dir2d(direction: Direction2) -> Self {
        Self::stored(Vector2::from(direction))
    }
    /// Set from `(x, y)`, normalized (`SetCoord`); `direction.set_coord(0.0, 2.0)?`.
    pub fn set_coord(&self, x: f64, y: f64) -> Result<(), Failure> {
        let unit: Vector2 = normalized_coordinates(x, y, "Geom2d_Direction::SetCoord()")?;
        *self.state.write() = unit;
        Ok(())
    }
    /// Replace by a gp direction (`SetDir2d`); `direction.set_dir2d(dir)`.
    pub fn set_dir2d(&self, direction: Direction2) {
        *self.state.write() = Vector2::from(direction);
    }
    /// Set X and renormalize with the current Y (`SetX`); `direction.set_x(1.0)?`.
    pub fn set_x(&self, x: f64) -> Result<(), Failure> {
        let mut state = self.state.write();
        *state = normalized_coordinates(x, state.y(), "Geom2d_Direction::SetX()")?;
        Ok(())
    }
    /// Set Y and renormalize with the current X (`SetY`); `direction.set_y(1.0)?`.
    pub fn set_y(&self, y: f64) -> Result<(), Failure> {
        let mut state = self.state.write();
        *state = normalized_coordinates(state.x(), y, "Geom2d_Direction::SetY()")?;
        Ok(())
    }
    /// The gp direction (`Dir2d`); fails only if the stored vector is null;
    /// `direction.dir2d()?`.
    pub fn dir2d(&self) -> Result<Direction2, Failure> {
        Direction2::from_vector(*self.state.read())
    }
}
impl Geom2dGeometry for Geom2dDirection {
    fn transform(&self, transform: Transform2) -> Result<(), Failure> {
        let mut state = self.state.write();
        let mut direction: Direction2 = Direction2::from_vector(*state)?;
        direction.transform(transform);
        *state = Vector2::from(direction);
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        // Quirk kept: the native Copy passes gpVec2d to the gp_Dir2d constructor, which
        // normalizes the stored unit vector once more (the last bit can change).
        Ok(Handle::new(Self::from_dir2d(self.dir2d()?)).cast())
    }
}
impl Geom2dVector for Geom2dDirection {
    fn vector_cell(&self, _access: ProtectedAccess) -> &SharedState<Vector2> {
        &self.state
    }
    fn magnitude(&self) -> f64 {
        1.0
    }
    fn square_magnitude(&self) -> f64 {
        1.0
    }
    fn crossed(&self, other: &dyn Geom2dVector) -> f64 {
        let own: Vector2 = self.vec2d();
        own.crossed(other.vec2d())
    }
}
