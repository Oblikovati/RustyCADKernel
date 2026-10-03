//! OCCT ShapeUpgrade_SplitCurve3dContinuity at 3d097a0328e71b8.
use super::{
    ShapeUpgradeSplitCurve3d, SplitCurve, SplitCurveData, SplitCurves3d,
    curve_continuity::ContinuitySettings,
};
use rustycad_tkernel::{
    impl_transient, precision,
    standard::{Failure, Handle, SharedState},
};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve, GeomOffsetCurve, GeomTrimmedCurve};
use rustycad_tkmath::geom_abs::Shape;
mod compute;

/// Correct or split a private curve copy at insufficiently continuous knots.
/// `let splitter = ShapeUpgradeSplitCurve3dContinuity::new();`
pub struct ShapeUpgradeSplitCurve3dContinuity {
    parent: ShapeUpgradeSplitCurve3d,
    settings: SharedState<ContinuitySettings>,
}
impl Default for ShapeUpgradeSplitCurve3dContinuity {
    fn default() -> Self {
        Self {
            parent: ShapeUpgradeSplitCurve3d::default(),
            settings: SharedState::new(ContinuitySettings::with_tolerance(precision::confusion())),
        }
    }
}
impl_transient!(ShapeUpgradeSplitCurve3dContinuity, "ShapeUpgrade_SplitCurve3dContinuity",
    ShapeUpgradeSplitCurve3d, parent, interfaces: [dyn SplitCurve]);

impl ShapeUpgradeSplitCurve3dContinuity {
    /// Use native C1 and Confusion defaults; `ShapeUpgradeSplitCurve3dContinuity::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Set the required continuity; geometric criteria use order one as in OCCT.
    /// `splitter.set_criterion(Shape::C2)`.
    pub fn set_criterion(&self, criterion: Shape) {
        self.settings.write().set_criterion(criterion);
    }
    /// Set the knot-removal tolerance without clamping; `splitter.set_tolerance(1e-6)`.
    pub fn set_tolerance(&self, tolerance: f64) {
        self.settings.write().tolerance = tolerance;
    }
    /// Copy a curve over its natural range; `splitter.init_curve(&curve)?`.
    pub fn init_curve(&self, curve: &Handle<dyn GeomCurve>) -> Result<(), Failure> {
        self.parent.init_curve(curve)
    }
    /// Copy a curve over the native clamped range; `splitter.init_curve_range(&curve, 0., 1.)?`.
    pub fn init_curve_range(
        &self,
        curve: &Handle<dyn GeomCurve>,
        first: f64,
        last: f64,
    ) -> Result<(), Failure> {
        self.parent.init_curve_range(curve, first, last)
    }
    /// Share the corrected private curve; `let curve = splitter.curve()`.
    pub fn curve(&self) -> Handle<dyn GeomCurve> {
        self.parent.source_curve()
    }
    /// Share the built pieces; `let pieces = splitter.curves()`.
    pub fn curves(&self) -> SplitCurves3d {
        self.parent.curves()
    }
}
impl SplitCurve for ShapeUpgradeSplitCurve3dContinuity {
    fn split_curve_data(&self) -> &SplitCurveData {
        self.parent.split_curve_data()
    }
    fn build(&self, segment: bool) -> Result<(), Failure> {
        self.parent.build(segment)
    }
    fn compute(&self) -> Result<(), Failure> {
        self.compute_continuity()
    }
}
