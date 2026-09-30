// Rust translation of FoundationClasses/TKMath/Poly/Poly_TriangulationParameters.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use rustycad_tkernel::standard::{Handle, StandardTransient};
/// `Poly_TriangulationParameters`: the settings a mesh was built for, `-1` when undefined;
/// `TriangulationParameters::new(0.1, 0.5, -1.0)`.
pub struct TriangulationParameters {
    base: StandardTransient,
    deflection: f64,
    angle: f64,
    min_size: f64,
}
rustycad_tkernel::impl_transient!(
    TriangulationParameters,
    "Poly_TriangulationParameters",
    StandardTransient,
    base
);
impl Default for TriangulationParameters {
    fn default() -> Self {
        Self::new(-1.0, -1.0, -1.0)
    }
}
impl TriangulationParameters {
    /// Stores linear deflection, angular deflection and minimum size; `TriangulationParameters::new(0.1, -1.0, -1.0)`.
    pub fn new(deflection: f64, angle: f64, min_size: f64) -> Self {
        Self {
            base: StandardTransient::default(),
            deflection,
            angle,
            min_size,
        }
    }
    /// Returns a new handle with the same values; `parameters.copy()`.
    pub fn copy(&self) -> Handle<Self> {
        Handle::new(Self::new(self.deflection, self.angle, self.min_size))
    }
    /// True unless the deflection is negative (NaN counts as defined); `parameters.has_deflection()`.
    pub fn has_deflection(&self) -> bool {
        is_defined(self.deflection)
    }
    /// True unless the angle is negative; `parameters.has_angle()`.
    pub fn has_angle(&self) -> bool {
        is_defined(self.angle)
    }
    /// True unless the minimum size is negative; `parameters.has_min_size()`.
    pub fn has_min_size(&self) -> bool {
        is_defined(self.min_size)
    }
    /// Linear deflection or -1; `parameters.deflection()`.
    pub const fn deflection(&self) -> f64 {
        self.deflection
    }
    /// Angular deflection or -1; `parameters.angle()`.
    pub const fn angle(&self) -> f64 {
        self.angle
    }
    /// Minimum size or -1; `parameters.min_size()`.
    pub const fn min_size(&self) -> f64 {
        self.min_size
    }
}
/// `!(value < 0.)`: a NaN setting counts as defined, as natively; `is_defined(f64::NAN)`.
fn is_defined(value: f64) -> bool {
    value.partial_cmp(&0.0) != Some(std::cmp::Ordering::Less)
}
