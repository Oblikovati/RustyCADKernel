//! ShapeUpgrade_SplitCurve3d at OCCT 3d097a0328e71b8: copied geometry and native split state.
use super::{ShapeUpgradeSplitCurve, SplitCurve, SplitCurveData};
use crate::{
    shape_analysis::ShapeAnalysisCurve,
    shape_extend::{ShapeExtend, ShapeExtendStatus},
};
use rustycad_tkernel::{
    impl_transient,
    ncollection::HArray1,
    precision,
    standard::{Failure, Handle, SharedState},
};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCurve, GeomGeometry, GeomOffsetCurve, GeomTrimmedCurve,
};
mod build;
mod pieces;
/// Shared array of resulting curves; native unbuilt entries may be null.
pub type SplitCurves3d = Handle<HArray1<Handle<dyn GeomCurve>>>;
#[derive(Default)]
struct CurveSplitState {
    curve: Handle<dyn GeomCurve>,
    result: SplitCurves3d,
}
/// Splits a private curve copy; `ShapeUpgradeSplitCurve3d::new()`.
/// Multipart `build(false)` retains native null entries. Offset recursion then returns
/// `Standard_ConstructionError` where OCCT3d097a0 crashes; this is a recorded parity divergence.
#[derive(Default)]
pub struct ShapeUpgradeSplitCurve3d {
    parent: ShapeUpgradeSplitCurve,
    state: SharedState<CurveSplitState>,
}
impl_transient!(ShapeUpgradeSplitCurve3d,"ShapeUpgrade_SplitCurve3d",ShapeUpgradeSplitCurve,parent,interfaces:[dyn SplitCurve]);
impl SplitCurve for ShapeUpgradeSplitCurve3d {
    fn split_curve_data(&self) -> &SplitCurveData {
        self.parent.split_curve_data()
    }
    fn build(&self, segment: bool) -> Result<(), Failure> {
        self.build_curves(segment)
    }
}
impl ShapeUpgradeSplitCurve3d {
    /// Creates an uninitialized splitter; `ShapeUpgradeSplitCurve3d::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Copy a curve over its natural bounds; `splitter.init_curve(&curve)?`.
    pub fn init_curve(&self, curve: &Handle<dyn GeomCurve>) -> Result<(), Failure> {
        require_curve(curve)?;
        self.init_curve_range(curve, curve.first_parameter(), curve.last_parameter())
    }
    /// Copy a curve and clamp its range as OCC966 requires for nonperiodic curves.
    /// Example: `splitter.init_curve_range(&curve,0.,1.)?`.
    pub fn init_curve_range(
        &self,
        curve: &Handle<dyn GeomCurve>,
        first: f64,
        last: f64,
    ) -> Result<(), Failure> {
        require_curve(curve)?;
        let copy: Handle<dyn GeomCurve> = curve.copy()?.cast();
        self.state.write().curve = copy.clone();
        let basis = copy
            .cast::<GeomTrimmedCurve>()
            .get()
            .map_or(copy.clone(), GeomTrimmedCurve::basis_curve);
        let (first, last) = if ShapeAnalysisCurve::is_periodic(curve)? {
            (first, last)
        } else {
            init_bounds(&basis, first, last)
        };
        SplitCurve::init(self, first, last);
        Ok(())
    }
    /// Shared native result array; null until Build allocates it; `splitter.curves()`.
    pub fn curves(&self) -> SplitCurves3d {
        self.state.read().result.clone()
    }
    fn allocate_curves(&self, count: i32) -> Result<(), Failure> {
        self.split_curve_data()
            .with_mut(|fields| fields.nb_curves = count);
        let array = HArray1::with_value(1, count, Handle::default())
            .map_err(|error| error.into_failure("ShapeUpgrade_SplitCurve3d::Build"))?;
        self.state.write().result = Handle::new(array);
        Ok(())
    }
    fn set_curve(&self, index: i32, curve: Handle<dyn GeomCurve>) -> Result<(), Failure> {
        self.curves()
            .change_array1()
            .set_value(index, curve)
            .map_err(|error| error.into_failure("ShapeUpgrade_SplitCurve3d::Build"))
    }
    fn add_status(&self, status: ShapeExtendStatus) {
        self.split_curve_data()
            .with_mut(|fields| fields.status |= ShapeExtend::encode_status(status));
    }
}
fn require_curve(curve: &Handle<dyn GeomCurve>) -> Result<(), Failure> {
    if curve.is_null() {
        return Err(Failure::with_type(
            "Standard_NullObject",
            "ShapeUpgrade_SplitCurve3d: null curve; expected initialized Geom_Curve",
            "",
        ));
    }
    Ok(())
}
fn init_bounds(curve: &Handle<dyn GeomCurve>, mut first: f64, mut last: f64) -> (f64, f64) {
    let precision = precision::p_confusion();
    let lower = curve.first_parameter();
    let upper = curve.last_parameter();
    (first, last) = snap_bounds(first, last, lower, upper);
    if first < lower {
        first = lower;
    }
    if last > upper {
        last = upper;
    }
    if last - first < precision {
        last = first + 2. * precision;
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
