//! ShapeUpgrade_SplitCurve3d at OCCT 3d097a0328e71b8: copied geometry and native split state.
use super::split_curve_geometry::{CurveFamily3d, CurveSplitKernel};
use super::{ShapeUpgradeSplitCurve, SplitCurve, SplitCurveData};
use rustycad_tkernel::{
    impl_transient,
    ncollection::HArray1,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom::GeomCurve;
/// Shared resulting curve array; multipart `build(false)` entries may be null.
pub type SplitCurves3d = Handle<HArray1<Handle<dyn GeomCurve>>>;
/// Splits a private curve copy; `ShapeUpgradeSplitCurve3d::new()`.
/// Native offset construction from null pieces crashes; Rust records a typed failure divergence.
#[derive(Default)]
pub struct ShapeUpgradeSplitCurve3d {
    parent: CurveSplitKernel<CurveFamily3d>,
}
impl_transient!(ShapeUpgradeSplitCurve3d,"ShapeUpgrade_SplitCurve3d",ShapeUpgradeSplitCurve,parent,interfaces:[dyn SplitCurve]);
impl SplitCurve for ShapeUpgradeSplitCurve3d {
    fn split_curve_data(&self) -> &SplitCurveData {
        self.parent.split_curve_data()
    }
    fn build(&self, segment: bool) -> Result<(), Failure> {
        self.parent.build(segment)
    }
}
impl ShapeUpgradeSplitCurve3d {
    /// Creates an uninitialized splitter; `ShapeUpgradeSplitCurve3d::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Copy a curve over its natural bounds; `splitter.init_curve(&curve)?`.
    pub fn init_curve(&self, curve: &Handle<dyn GeomCurve>) -> Result<(), Failure> {
        self.parent.init_natural(curve)
    }
    /// Copy and clamp the range as OCC966 requires; `splitter.init_curve_range(&curve,0.,1.)?`.
    pub fn init_curve_range(
        &self,
        curve: &Handle<dyn GeomCurve>,
        first: f64,
        last: f64,
    ) -> Result<(), Failure> {
        self.parent.init_curve_range(curve, first, last)
    }
    pub(super) fn source_curve(&self) -> Handle<dyn GeomCurve> {
        self.parent.source_curve()
    }
    pub(super) fn set_result_curves(&self, curves: SplitCurves3d) {
        self.parent.set_result_curves(curves);
    }
    /// Shared result array; null before Build allocates it; `splitter.curves()`.
    pub fn curves(&self) -> SplitCurves3d {
        self.parent.curves()
    }
}
