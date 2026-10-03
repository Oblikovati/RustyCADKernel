//! OCCT `ShapeUpgrade_ConvertCurve3dToBezier` at the pinned source revision.
use super::{ShapeUpgradeSplitCurve3d, SplitCurve, SplitCurveData, SplitCurves3d};
use rustycad_tkernel::{
    impl_transient,
    ncollection::HSequence,
    standard::{Failure, Handle, SharedState},
};
use rustycad_tkg3d::geom::GeomCurve;

mod build;
mod compute;

struct ConversionState {
    segments: Handle<HSequence<Handle<dyn GeomCurve>>>,
    split_params: Handle<HSequence<f64>>,
    line_mode: bool,
    circle_mode: bool,
    conic_mode: bool,
}
impl Default for ConversionState {
    fn default() -> Self {
        Self {
            segments: Handle::new(HSequence::default()),
            split_params: Handle::new(HSequence::default()),
            line_mode: true,
            circle_mode: true,
            conic_mode: true,
        }
    }
}

/// Convert an initialized curve to Bezier segments while retaining source parameters.
/// `let converter = ShapeUpgradeConvertCurve3dToBezier::new();`
pub struct ShapeUpgradeConvertCurve3dToBezier {
    parent: ShapeUpgradeSplitCurve3d,
    state: SharedState<ConversionState>,
}
impl Default for ShapeUpgradeConvertCurve3dToBezier {
    fn default() -> Self {
        Self {
            parent: ShapeUpgradeSplitCurve3d::new(),
            state: SharedState::new(ConversionState::default()),
        }
    }
}
impl_transient!(ShapeUpgradeConvertCurve3dToBezier, "ShapeUpgrade_ConvertCurve3dToBezier", ShapeUpgradeSplitCurve3d, parent, interfaces: [dyn SplitCurve]);

impl ShapeUpgradeConvertCurve3dToBezier {
    /// Create a converter with native line, circle and conic modes enabled.
    /// `ShapeUpgradeConvertCurve3dToBezier::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Initialize from a curve's natural bounds; `converter.init_curve(&curve)?`.
    pub fn init_curve(&self, curve: &Handle<dyn GeomCurve>) -> Result<(), Failure> {
        self.parent.init_curve(curve)
    }
    /// Initialize over a caller-supplied range; `converter.init_curve_range(&curve, 0., 1.)?`.
    pub fn init_curve_range(
        &self,
        curve: &Handle<dyn GeomCurve>,
        first: f64,
        last: f64,
    ) -> Result<(), Failure> {
        self.parent.init_curve_range(curve, first, last)
    }
    /// Enable or disable line conversion; `converter.set_line_mode(false)`.
    pub fn set_line_mode(&self, enabled: bool) {
        self.state.write().line_mode = enabled;
    }
    /// Read line conversion mode; `assert!(converter.line_mode())`.
    pub fn line_mode(&self) -> bool {
        self.state.read().line_mode
    }
    /// Enable or disable circle conversion; `converter.set_circle_mode(false)`.
    pub fn set_circle_mode(&self, enabled: bool) {
        self.state.write().circle_mode = enabled;
    }
    /// Read circle conversion mode; `assert!(converter.circle_mode())`.
    pub fn circle_mode(&self) -> bool {
        self.state.read().circle_mode
    }
    /// Enable or disable other conic conversion; `converter.set_conic_mode(false)`.
    pub fn set_conic_mode(&self, enabled: bool) {
        self.state.write().conic_mode = enabled;
    }
    /// Read other conic conversion mode; `assert!(converter.conic_mode())`.
    pub fn conic_mode(&self) -> bool {
        self.state.read().conic_mode
    }
    /// Parameters delimiting generated Bezier segments; `converter.split_params()`.
    pub fn split_params(&self) -> Handle<HSequence<f64>> {
        self.state.read().split_params.clone()
    }
    /// Lend the native protected segment sequence to specialized converters.
    /// Example: `let segments = converter.segments();`.
    pub fn segments(&self) -> Handle<HSequence<Handle<dyn GeomCurve>>> {
        self.state.read().segments.clone()
    }
    /// Shared Bezier result geometry inherited from `ShapeUpgradeSplitCurve3d`.
    /// `let curves: SplitCurves3d = converter.curves()`.
    pub fn curves(&self) -> SplitCurves3d {
        self.parent.curves()
    }
}
impl SplitCurve for ShapeUpgradeConvertCurve3dToBezier {
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
