//! ShapeUpgrade_SplitSurfaceAngle at OCCT 3d097a0328e71b8.
use super::{ShapeUpgradeSplitSurface, SplitSurface, SplitSurfaceData};
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkernel::{
    impl_transient,
    ncollection::HSequence,
    precision,
    standard::{Failure, Handle, SharedState},
};
use rustycad_tkg3d::geom::{
    GeomConicalSurface, GeomCylindricalSurface, GeomOffsetSurface, GeomRectangularTrimmedSurface,
    GeomSphericalSurface, GeomSurface, GeomSurfaceOfRevolution, GeomToroidalSurface,
};

/// Divide supported rotational surfaces into angular intervals.
/// `let splitter = ShapeUpgradeSplitSurfaceAngle::new(std::f64::consts::PI);`
pub struct ShapeUpgradeSplitSurfaceAngle {
    parent: ShapeUpgradeSplitSurface,
    maximum_angle: SharedState<f64>,
}
impl_transient!(ShapeUpgradeSplitSurfaceAngle, "ShapeUpgrade_SplitSurfaceAngle", ShapeUpgradeSplitSurface, parent, interfaces: [dyn SplitSurface]);
impl ShapeUpgradeSplitSurfaceAngle {
    /// Retain the supplied angle without clamping; `ShapeUpgradeSplitSurfaceAngle::new(1.)`.
    pub fn new(maximum_angle: f64) -> Self {
        Self {
            parent: ShapeUpgradeSplitSurface::new(),
            maximum_angle: SharedState::new(maximum_angle),
        }
    }
    /// Set the angular limit in radians; `splitter.set_max_angle(1.)`.
    pub fn set_max_angle(&self, maximum_angle: f64) {
        *self.maximum_angle.write() = maximum_angle;
    }
    /// Return the retained angular limit; `assert_eq!(splitter.max_angle(), 1.)`.
    pub fn max_angle(&self) -> f64 {
        *self.maximum_angle.read()
    }
    fn compute_angles(&self) -> Result<(), Failure> {
        let surface = self.split_surface_data().with(|f| f.surface.clone());
        let (basis, trimmed_range) = angle_basis(&surface);
        if !supports_angle_split(&basis) {
            return Ok(());
        }
        let values = self.u_split_values();
        let (first, last) =
            values.sequence(|s| Ok::<_, Failure>((*s.value(1)?, *s.value(s.len())?)))?;
        let length = last - first;
        let maximum = self.max_angle();
        let segments = segment_count(length, maximum)?;
        if segments == 1 && !strictly_inside_trim(trimmed_range, length, maximum) {
            self.split_surface_data()
                .with_mut(|f| f.status = ShapeExtend::encode_status(ShapeExtendStatus::Done2));
        }
        self.set_u_split_values(&angular_parameters(first, length, segments))
    }
}
impl SplitSurface for ShapeUpgradeSplitSurfaceAngle {
    fn split_surface_data(&self) -> &SplitSurfaceData {
        self.parent.split_surface_data()
    }
    fn compute(&self, _segment: bool) -> Result<(), Failure> {
        self.compute_angles()
    }
}
fn angle_basis(surface: &Handle<dyn GeomSurface>) -> (Handle<dyn GeomSurface>, Option<(f64, f64)>) {
    if let Some(trimmed) = surface.cast::<GeomRectangularTrimmedSurface>().get() {
        let (first, last, _, _) = trimmed.bounds();
        return (trimmed.basis_surface(), Some((first, last)));
    }
    if let Some(offset) = surface.cast::<GeomOffsetSurface>().get() {
        return (offset.basis_surface(), None);
    }
    (surface.clone(), None)
}
fn supports_angle_split(surface: &Handle<dyn GeomSurface>) -> bool {
    !surface.cast::<GeomSurfaceOfRevolution>().is_null()
        || !surface.cast::<GeomConicalSurface>().is_null()
        || !surface.cast::<GeomToroidalSurface>().is_null()
        || !surface.cast::<GeomCylindricalSurface>().is_null()
        || !surface.cast::<GeomSphericalSurface>().is_null()
}
fn strictly_inside_trim(range: Option<(f64, f64)>, length: f64, maximum: f64) -> bool {
    let Some((first, last)) = range else {
        return false;
    };
    length < maximum && last - first < maximum
}
fn segment_count(length: f64, maximum: f64) -> Result<i32, Failure> {
    let quotient = ((length - precision::angular()) / maximum).trunc();
    // Native float-to-int conversion or subsequent addition is undefined outside this range.
    if !quotient.is_finite() || quotient < i32::MIN as f64 || quotient >= i32::MAX as f64 {
        return Err(Failure::with_type(
            "Standard_OutOfRange",
            &format!(
                "angular segment quotient {quotient} from length {length}, maximum {maximum}; expected finite integer in [{}, {})",
                i32::MIN,
                i32::MAX
            ),
            "",
        ));
    }
    Ok(quotient as i32 + 1)
}
fn angular_parameters(first: f64, length: f64, segments: i32) -> Handle<HSequence<f64>> {
    let values = Handle::new(HSequence::default());
    let step = length / f64::from(segments);
    let mut current = first + step;
    for _ in 1..segments {
        values.append(current);
        current += step;
    }
    values
}
