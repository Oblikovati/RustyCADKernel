//! ShapeUpgrade_SplitCurve2d at OCCT 3d097a0328e71b8: copied geometry and native split state.
use super::split_curve_geometry::{CurveFamily2d, CurveSplitKernel};
use super::{ShapeUpgradeSplitCurve, SplitCurve, SplitCurveData};
use rustycad_tkernel::{
    impl_transient,
    ncollection::HArray1,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::Geom2dCurve;
/// Shared resulting curve array; multipart `build(false)` entries may be null.
pub type SplitCurves2d = Handle<HArray1<Handle<dyn Geom2dCurve>>>;
/// Splits a private curve copy; `ShapeUpgradeSplitCurve2d::new()`.
/// Native offset construction from null pieces crashes; Rust records a typed failure divergence.
#[derive(Default)]
pub struct ShapeUpgradeSplitCurve2d {
    parent: CurveSplitKernel<CurveFamily2d>,
}
impl_transient!(ShapeUpgradeSplitCurve2d,"ShapeUpgrade_SplitCurve2d",ShapeUpgradeSplitCurve,parent,interfaces:[dyn SplitCurve]);
impl SplitCurve for ShapeUpgradeSplitCurve2d {
    fn split_curve_data(&self) -> &SplitCurveData {
        self.parent.split_curve_data()
    }
    fn build(&self, segment: bool) -> Result<(), Failure> {
        self.parent.build(segment)
    }
}
impl ShapeUpgradeSplitCurve2d {
    /// Creates an uninitialized splitter; `ShapeUpgradeSplitCurve2d::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Copy a curve over its natural bounds; `splitter.init_curve(&curve)?`.
    pub fn init_curve(&self, curve: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
        self.parent.init_natural(curve)
    }
    /// Copy and clamp the range as OCC966 requires; `splitter.init_curve_range(&curve,0.,1.)?`.
    pub fn init_curve_range(
        &self,
        curve: &Handle<dyn Geom2dCurve>,
        first: f64,
        last: f64,
    ) -> Result<(), Failure> {
        self.parent.init_curve_range(curve, first, last)
    }
    /// Shared result array; null before Build allocates it; `splitter.curves()`.
    pub fn curves(&self) -> SplitCurves2d {
        self.parent.curves()
    }
}
