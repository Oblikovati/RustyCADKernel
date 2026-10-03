//! Shared native 2D/3D splitter control flow; family policies retain upstream differences.
use super::{ShapeUpgradeSplitCurve, SplitCurve, SplitCurveData};
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkernel::{
    ncollection::{HArray1, HSequence},
    precision,
    standard::{
        Failure, Handle, RuntimeType, RuntimeTyped, SharedState, StandardTransient, Transient,
    },
};
mod build;
mod families;
mod pieces;
pub(super) use families::{CurveFamily2d, CurveFamily3d};
pub(super) trait CurveGeometry {
    type Curve: Transient + RuntimeTyped + ?Sized;
    const NAME: &'static str;
    const CLAMP_BUILD: bool;
    const SEGMENT_SINGLE: bool;
    fn copy(curve: &Handle<Self::Curve>) -> Result<Handle<Self::Curve>, Failure>;
    fn bounds(curve: &Handle<Self::Curve>) -> (f64, f64);
    fn periodic(curve: &Handle<Self::Curve>) -> Result<bool, Failure>;
    fn trimmed_basis(curve: &Handle<Self::Curve>) -> Option<Handle<Self::Curve>>;
    fn offset_basis(curve: &Handle<Self::Curve>) -> Option<Handle<Self::Curve>>;
    fn offset(
        curve: &Handle<Self::Curve>,
        basis: &Handle<Self::Curve>,
    ) -> Result<Handle<Self::Curve>, Failure>;
    fn trim(
        curve: &Handle<Self::Curve>,
        first: f64,
        last: f64,
    ) -> Result<Handle<Self::Curve>, Failure>;
    fn spline(curve: &Handle<Self::Curve>) -> bool;
    fn segment(curve: &Handle<Self::Curve>, first: f64, last: f64) -> Result<(), Failure>;
    fn knots(curve: &Handle<Self::Curve>) -> Result<Option<Vec<f64>>, Failure>;
}
pub(super) type CurveArray<G> = Handle<HArray1<Handle<<G as CurveGeometry>::Curve>>>;
struct GeometryState<G: CurveGeometry> {
    curve: Handle<G::Curve>,
    result: CurveArray<G>,
}
impl<G: CurveGeometry> Default for GeometryState<G> {
    fn default() -> Self {
        Self {
            curve: Handle::default(),
            result: Handle::default(),
        }
    }
}
pub(super) struct CurveSplitKernel<G: CurveGeometry> {
    parent: ShapeUpgradeSplitCurve,
    state: SharedState<GeometryState<G>>,
}
impl<G: CurveGeometry> Default for CurveSplitKernel<G> {
    fn default() -> Self {
        Self {
            parent: ShapeUpgradeSplitCurve::new(),
            state: SharedState::default(),
        }
    }
}
// The real native base lives here so outer-class RTTI projects onto the same split state.
impl<G: CurveGeometry + 'static> Transient for CurveSplitKernel<G> {
    fn dynamic_type(&self) -> &'static RuntimeType {
        self.parent.dynamic_type()
    }
    fn project(&self, requested: std::any::TypeId) -> Option<&dyn std::any::Any> {
        self.parent.project(requested)
    }
    fn transient_base(&self) -> &StandardTransient {
        self.parent.transient_base()
    }
}
impl<G: CurveGeometry + 'static> SplitCurve for CurveSplitKernel<G> {
    fn split_curve_data(&self) -> &SplitCurveData {
        self.parent.split_curve_data()
    }
    fn build(&self, segment: bool) -> Result<(), Failure> {
        self.build_curves(segment)
    }
}
impl<G: CurveGeometry + 'static> CurveSplitKernel<G> {
    pub(super) fn init_natural(&self, curve: &Handle<G::Curve>) -> Result<(), Failure> {
        require_curve::<G>(curve)?;
        let (first, last) = G::bounds(curve);
        self.init_curve_range(curve, first, last)
    }
    pub(super) fn init_curve_range(
        &self,
        curve: &Handle<G::Curve>,
        first: f64,
        last: f64,
    ) -> Result<(), Failure> {
        require_curve::<G>(curve)?;
        let copy = G::copy(curve)?;
        self.state.write().curve = copy.clone();
        let basis = G::trimmed_basis(&copy).unwrap_or(copy);
        let (first, last) = if G::periodic(curve)? {
            (first, last)
        } else {
            init_bounds(G::bounds(&basis), first, last)
        };
        SplitCurve::init(self, first, last);
        Ok(())
    }
    pub(super) fn source_curve(&self) -> Handle<G::Curve> {
        self.state.read().curve.clone()
    }
    pub(super) fn curves(&self) -> CurveArray<G> {
        self.state.read().result.clone()
    }
    pub(super) fn set_result_curves(&self, curves: CurveArray<G>) {
        self.state.write().result = curves;
    }
    fn allocate_curves(&self, count: i32) -> Result<(), Failure> {
        self.split_curve_data()
            .with_mut(|fields| fields.nb_curves = count);
        let array = HArray1::with_value(1, count, Handle::default())
            .map_err(|e| e.into_failure(G::NAME))?;
        self.state.write().result = Handle::new(array);
        Ok(())
    }
    fn set_curve(&self, index: i32, curve: Handle<G::Curve>) -> Result<(), Failure> {
        self.curves()
            .change_array1()
            .set_value(index, curve)
            .map_err(|e| e.into_failure(G::NAME))
    }
    fn add_status(&self, status: ShapeExtendStatus) {
        self.split_curve_data()
            .with_mut(|fields| fields.status |= ShapeExtend::encode_status(status));
    }
}
fn require_curve<G: CurveGeometry>(curve: &Handle<G::Curve>) -> Result<(), Failure> {
    if curve.is_null() {
        return Err(Failure::with_type(
            "Standard_NullObject",
            &format!("{}: null curve; expected initialized curve", G::NAME),
            "",
        ));
    }
    Ok(())
}
fn init_bounds((lower, upper): (f64, f64), mut first: f64, mut last: f64) -> (f64, f64) {
    (first, last) = snap_bounds(first, last, lower, upper);
    if first < lower {
        first = lower;
    }
    if last > upper {
        last = upper;
    }
    // OCC966 expands nonperiodic ranges after clamping, including reversed input.
    if last - first < precision::p_confusion() {
        last = first + 2. * precision::p_confusion();
    }
    (first, last)
}
fn snap_bounds(mut first: f64, mut last: f64, lower: f64, upper: f64) -> (f64, f64) {
    if (first - lower).abs() < precision::p_confusion() {
        first = lower;
    }
    if (last - upper).abs() < precision::p_confusion() {
        last = upper;
    }
    (first, last)
}
