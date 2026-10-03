//! OCCT `ShapeUpgrade_ConvertCurve2dToBezier` at the pinned source revision.
use super::{ShapeUpgradeSplitCurve2d, SplitCurve, SplitCurveData, SplitCurves2d};
use rustycad_tkernel::{
    impl_transient,
    ncollection::HSequence,
    standard::{Failure, Handle, SharedState},
};
use rustycad_tkg2d::geom2d::Geom2dCurve;

mod build;
mod compute;
mod spline;

struct ConversionState {
    segments: Handle<HSequence<Handle<dyn Geom2dCurve>>>,
    split_params: Handle<HSequence<f64>>,
}
impl Default for ConversionState {
    fn default() -> Self {
        Self {
            segments: Handle::new(HSequence::default()),
            split_params: Handle::new(HSequence::default()),
        }
    }
}

/// Convert an initialized p-curve into Bezier segments.
/// `let converter = ShapeUpgradeConvertCurve2dToBezier::new()`.
#[derive(Default)]
pub struct ShapeUpgradeConvertCurve2dToBezier {
    parent: ShapeUpgradeSplitCurve2d,
    state: SharedState<ConversionState>,
}
impl_transient!(ShapeUpgradeConvertCurve2dToBezier, "ShapeUpgrade_ConvertCurve2dToBezier", ShapeUpgradeSplitCurve2d, parent, interfaces: [dyn SplitCurve]);

impl ShapeUpgradeConvertCurve2dToBezier {
    /// Create an uninitialized p-curve converter; `ShapeUpgradeConvertCurve2dToBezier::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Initialize from a curve's natural bounds; `converter.init_curve(&curve)?`.
    pub fn init_curve(&self, curve: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
        self.parent.init_curve(curve)
    }
    /// Initialize over a caller-supplied range; `converter.init_curve_range(&curve, 0., 1.)?`.
    pub fn init_curve_range(
        &self,
        curve: &Handle<dyn Geom2dCurve>,
        first: f64,
        last: f64,
    ) -> Result<(), Failure> {
        self.parent.init_curve_range(curve, first, last)
    }
    /// Parameters delimiting generated Bezier segments; `converter.split_params()`.
    pub fn split_params(&self) -> Handle<HSequence<f64>> {
        self.state.read().split_params.clone()
    }
    /// Lend the native protected segment sequence to specialized converters.
    /// Example: `let segments = converter.segments();`.
    pub fn segments(&self) -> Handle<HSequence<Handle<dyn Geom2dCurve>>> {
        self.state.read().segments.clone()
    }
    /// Shared Bezier result geometry inherited from `ShapeUpgradeSplitCurve2d`.
    /// `let curves: SplitCurves2d = converter.curves()`.
    pub fn curves(&self) -> SplitCurves2d {
        self.parent.curves()
    }
}
impl SplitCurve for ShapeUpgradeConvertCurve2dToBezier {
    fn split_curve_data(&self) -> &SplitCurveData {
        self.parent.split_curve_data()
    }
    fn build(&self, _segment: bool) -> Result<(), Failure> {
        build::build(self)
    }
    fn compute(&self) -> Result<(), Failure> {
        compute::compute(self)
    }
}
