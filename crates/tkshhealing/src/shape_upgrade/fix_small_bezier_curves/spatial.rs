//! The trim constructor is outside the native 3D catch; approximation and splitting are inside.
use super::*;
impl ShapeUpgradeFixSmallBezierCurves {
    pub(super) fn approximate_spatial(
        &self,
        edge: &Edge,
        output: &mut Handle<dyn GeomCurve>,
        first: &mut f64,
        last: &mut f64,
    ) -> Result<bool, Failure> {
        let (curve, lower, upper) = ShapeAnalysisEdge::new().curve_3d(edge, false)?;
        if curve.is_null() {
            return Ok(true);
        }
        clamp_range(first, last, (lower, upper));
        let trimmed: Handle<dyn GeomCurve> =
            Handle::new(GeomTrimmedCurve::new(&curve, *first, *last, true, true)?).cast();
        let continuity = match trimmed.continuity() {
            Shape::C3 | Shape::CN => Shape::C2,
            other => other,
        };
        Ok(self
            .convert_spatial(&trimmed, continuity, output, *first, *last)
            .unwrap_or(false))
    }
    fn convert_spatial(
        &self,
        trimmed: &Handle<dyn GeomCurve>,
        continuity: Shape,
        output: &mut Handle<dyn GeomCurve>,
        first: f64,
        last: f64,
    ) -> Result<bool, Failure> {
        let approximation = ApproxCurve3d::from_curve(trimmed, self.precision(), continuity, 1, 9)?;
        if !approximation.is_done() {
            return Ok(true);
        }
        let tool = self
            .fix_small_curves_data()
            .with(|f| f.curve_3d_tool.clone());
        tool.init_curve_range(&approximation.curve().cast(), first, last)?;
        let criterion = tool.cast::<dyn SplitCurve>();
        criterion.perform(true)?;
        if criterion.status_flag(ShapeExtendStatus::Fail) {
            return Ok(true);
        }
        assign_single_segment(&tool.curves(), output)
    }
}
