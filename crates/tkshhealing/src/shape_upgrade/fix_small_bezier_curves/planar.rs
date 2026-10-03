//! Preserve the seam's forward-range reuse and its narrower native catch boundary.
use super::*;
impl ShapeUpgradeFixSmallBezierCurves {
    pub(super) fn approximate_planar_pair(
        &self,
        edge: &Edge,
        face: &Face,
        output: &mut Handle<dyn Geom2dCurve>,
        reversed: &mut Handle<dyn Geom2dCurve>,
        first: &mut f64,
        last: &mut f64,
    ) -> Result<bool, Failure> {
        let tolerance = parameter_tolerance(face, self.precision())?;
        let (curve, lower, upper) = ShapeAnalysisEdge::new().pcurve_on_face(edge, face, false)?;
        if !curve.is_null()
            && !self.approximate_planar(&curve, output, first, last, (lower, upper), tolerance)?
        {
            return Ok(false);
        }
        if !BRepTool::is_closed_on_face(edge, face)? {
            return Ok(true);
        }
        self.approximate_reversed(edge, face, reversed, first, last, (lower, upper), tolerance)
    }
    fn approximate_planar(
        &self,
        curve: &Handle<dyn Geom2dCurve>,
        output: &mut Handle<dyn Geom2dCurve>,
        first: &mut f64,
        last: &mut f64,
        bounds: (f64, f64),
        tolerance: f64,
    ) -> Result<bool, Failure> {
        clamp_range(first, last, bounds);
        let trimmed: Handle<dyn Geom2dCurve> =
            Handle::new(Geom2dTrimmedCurve::new(curve, *first, *last, true, true)?).cast();
        Ok(self
            .convert_planar(&trimmed, output, *first, *last, tolerance)
            .unwrap_or(false))
    }
    fn convert_planar(
        &self,
        curve: &Handle<dyn Geom2dCurve>,
        output: &mut Handle<dyn Geom2dCurve>,
        first: f64,
        last: f64,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        let approximation = ApproxCurve2d::from_curve(curve, tolerance, curve.continuity(), 1, 9)?;
        self.adopt_planar(&approximation, output, first, last, false)
    }
    #[allow(clippy::too_many_arguments)] // The seam reuses the previous pcurve bounds, not its own f2/l2.
    fn approximate_reversed(
        &self,
        edge: &Edge,
        face: &Face,
        output: &mut Handle<dyn Geom2dCurve>,
        first: &mut f64,
        last: &mut f64,
        forward_bounds: (f64, f64),
        tolerance: f64,
    ) -> Result<bool, Failure> {
        let reversed = topods::edge(&edge.reversed())?;
        let (curve, _, _) = ShapeAnalysisEdge::new().pcurve_on_face(&reversed, face, false)?;
        if curve.is_null() {
            return Ok(true);
        }
        clamp_reversed(first, last, forward_bounds);
        let trimmed: Handle<dyn Geom2dCurve> =
            Handle::new(Geom2dTrimmedCurve::new(&curve, *first, *last, true, true)?).cast();
        // Unlike the forward pcurve, native constructs this approximation before entering try.
        let approximation =
            ApproxCurve2d::from_curve(&trimmed, tolerance, trimmed.continuity(), 1, 9)?;
        Ok(self
            .adopt_planar(&approximation, output, *first, *last, true)
            .unwrap_or(false))
    }
    fn adopt_planar(
        &self,
        approximation: &ApproxCurve2d,
        output: &mut Handle<dyn Geom2dCurve>,
        first: f64,
        last: f64,
        reversed: bool,
    ) -> Result<bool, Failure> {
        if !approximation.is_done() {
            return Ok(true);
        }
        let tool = self
            .fix_small_curves_data()
            .with(|f| f.curve_2d_tool.clone());
        tool.init_curve_range(&approximation.curve().cast(), first, last)?;
        if !planar_split_accepted(&tool, reversed)? {
            return Ok(false);
        }
        assign_single_segment(&tool.curves(), output)
    }
}

fn planar_split_accepted(
    tool: &Handle<ShapeUpgradeSplitCurve2d>,
    reversed: bool,
) -> Result<bool, Failure> {
    let criterion = tool.cast::<dyn SplitCurve>();
    criterion.perform(true)?;
    Ok(if reversed {
        criterion.status_flag(ShapeExtendStatus::Done)
    } else {
        !criterion.status_flag(ShapeExtendStatus::Fail)
    })
}
