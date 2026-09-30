// Rust translation of ModelingData/TKG3d/Geom/Geom_Direction.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_Direction`: a unit vector as transient geometry.
use super::errors;
use super::geometry::GeomGeometry;
use super::protected::ProtectedAccess;
use super::vector::GeomVector;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::SharedState;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::gp::{Direction3, Transform3, Vector3, resolution};

/// A unit vector; `Handle::new(GeomDirection::new(1.0, 1.0, 0.0)?)`.
pub struct GeomDirection {
    parent: StandardTransient,
    state: SharedState<Vector3>,
}
impl_transient!(
    GeomDirection,
    "Geom_Direction",
    dyn GeomVector,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomVector]
);

// (X / D, Y / D, Z / D) after the native zero-length check of `member`.
fn normalized_coordinates(x: f64, y: f64, z: f64, member: &str) -> Result<Vector3, Failure> {
    let d: f64 = (x * x + y * y + z * z).sqrt();
    if d <= resolution() {
        return Err(errors::construction(&format!(
            "{member} - input vector ({x}, {y}, {z}) has zero length; expected a length > gp::Resolution()"
        )));
    }
    Ok(Vector3::new(x / d, y / d, z / d))
}

impl GeomDirection {
    fn stored(vector: Vector3) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(vector),
        }
    }
    /// Unit vector along `(x, y, z)` (`Geom_Direction(X, Y, Z)`);
    /// `GeomDirection::new(3.0, 4.0, 0.0)?`.
    pub fn new(x: f64, y: f64, z: f64) -> Result<Self, Failure> {
        let unit: Vector3 = normalized_coordinates(x, y, z, "Geom_Direction()")?;
        Ok(Self::stored(unit))
    }
    /// Direction from a gp direction (`Geom_Direction(gp_Dir)`);
    /// `GeomDirection::from_dir(direction)`.
    pub fn from_dir(direction: Direction3) -> Self {
        Self::stored(Vector3::from(direction))
    }
    /// Set from `(x, y, z)`, normalized (`SetCoord`); `direction.set_coord(0.0, 2.0, 0.0)?`.
    pub fn set_coord(&self, x: f64, y: f64, z: f64) -> Result<(), Failure> {
        let unit: Vector3 = normalized_coordinates(x, y, z, "Geom_Direction::SetCoord()")?;
        *self.state.write() = unit;
        Ok(())
    }
    /// Replace by a gp direction (`SetDir`); `direction.set_dir(dir)`.
    pub fn set_dir(&self, direction: Direction3) {
        *self.state.write() = Vector3::from(direction);
    }
    /// Set X and renormalize with the current Y and Z (`SetX`); `direction.set_x(1.0)?`.
    pub fn set_x(&self, x: f64) -> Result<(), Failure> {
        let mut state = self.state.write();
        *state = normalized_coordinates(x, state.y(), state.z(), "Geom_Direction::SetX()")?;
        Ok(())
    }
    /// Set Y and renormalize with the current X and Z (`SetY`); `direction.set_y(1.0)?`.
    pub fn set_y(&self, y: f64) -> Result<(), Failure> {
        let mut state = self.state.write();
        *state = normalized_coordinates(state.x(), y, state.z(), "Geom_Direction::SetY()")?;
        Ok(())
    }
    /// Set Z and renormalize with the current X and Y (`SetZ`); `direction.set_z(1.0)?`.
    pub fn set_z(&self, z: f64) -> Result<(), Failure> {
        let mut state = self.state.write();
        *state = normalized_coordinates(state.x(), state.y(), z, "Geom_Direction::SetZ()")?;
        Ok(())
    }
    /// The gp direction (`Dir`); fails only if the stored vector is null; `direction.dir()?`.
    pub fn dir(&self) -> Result<Direction3, Failure> {
        Direction3::from_vector(*self.state.read())
    }
}

// `gp_Dir V(...)`: the unit direction of a product, Standard_ConstructionError when null.
fn unit_of(product: Vector3) -> Result<Vector3, Failure> {
    Ok(Vector3::from(Direction3::from_vector(product)?))
}

impl GeomGeometry for GeomDirection {
    fn transform(&self, transform: Transform3) -> Result<(), Failure> {
        let mut state = self.state.write();
        let mut direction: Direction3 = Direction3::from_vector(*state)?;
        direction.transform(transform);
        *state = Vector3::from(direction);
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        // Quirk kept: the native Copy passes gpVec to the gp_Dir constructor, which
        // normalizes the stored unit vector once more (the last bit can change).
        Ok(Handle::new(Self::from_dir(self.dir()?)).cast())
    }
}
impl GeomVector for GeomDirection {
    fn vector_cell(&self, _access: ProtectedAccess) -> &SharedState<Vector3> {
        &self.state
    }
    fn magnitude(&self) -> f64 {
        1.0
    }
    fn square_magnitude(&self) -> f64 {
        1.0
    }
    fn cross(&self, other: &dyn GeomVector) -> Result<(), Failure> {
        let right: Vector3 = other.vec();
        let mut state = self.state.write();
        *state = unit_of(state.crossed(right))?;
        Ok(())
    }
    fn crossed(&self, other: &dyn GeomVector) -> Result<Handle<dyn GeomVector>, Failure> {
        let unit: Vector3 = unit_of(self.vec().crossed(other.vec()))?;
        Ok(Handle::new(Self::stored(unit)).cast())
    }
    fn cross_cross(&self, v1: &dyn GeomVector, v2: &dyn GeomVector) -> Result<(), Failure> {
        let (first, second) = (v1.vec(), v2.vec());
        let mut state = self.state.write();
        *state = unit_of(state.cross_crossed(first, second))?;
        Ok(())
    }
    fn cross_crossed(
        &self,
        v1: &dyn GeomVector,
        v2: &dyn GeomVector,
    ) -> Result<Handle<dyn GeomVector>, Failure> {
        let unit: Vector3 = unit_of(self.vec().cross_crossed(v1.vec(), v2.vec()))?;
        Ok(Handle::new(Self::stored(unit)).cast())
    }
}
