//! Native one-piece reuse, knot snapping and Segment fallback semantics.
use super::*;
impl ShapeUpgradeSplitCurve3d {
    pub(super) fn fill_single(
        &self,
        curve: &Handle<dyn GeomCurve>,
        first: f64,
        last: f64,
        segment: bool,
    ) -> Result<bool, Failure> {
        if self.split_curve_data().with(|f| f.nb_curves) != 1 {
            return Ok(false);
        }
        if (curve.first_parameter() - first).abs() < precision::p_confusion()
            && (curve.last_parameter() - last).abs() < precision::p_confusion()
        {
            self.set_curve(1, curve.clone())?;
            return Ok(true);
        }
        let spline = !curve.cast::<GeomBSplineCurve>().is_null()
            || !curve.cast::<GeomBezierCurve>().is_null();
        if segment && spline && self.status_flag(ShapeExtendStatus::Done2) {
            return Ok(false);
        }
        self.set_curve(1, trim_copy(curve, first, last)?)?;
        Ok(true)
    }
    pub(super) fn snap_spline_knots(&self, curve: &Handle<dyn GeomCurve>) -> Result<(), Failure> {
        let curve = curve.cast::<GeomBSplineCurve>();
        let Some(curve) = curve.get() else {
            return Ok(());
        };
        let copy: Handle<GeomBSplineCurve> = curve.copy()?.cast();
        let mut knot = copy.first_u_knot_index()?;
        let last = copy.last_u_knot_index()?;
        let values = self.split_values();
        for index in 1..=values.sequence(|s| s.len()) {
            let parameter = values.sequence(|s| s.value(index).copied())?;
            snap_parameter(&copy, &values, index, parameter, &mut knot, last)?;
            if knot == last {
                break;
            }
        }
        Ok(())
    }
    pub(super) fn build_pieces(
        &self,
        curve: &Handle<dyn GeomCurve>,
        segment: bool,
    ) -> Result<(), Failure> {
        let values = self.split_values();
        for index in 1..=self.split_curve_data().with(|f| f.nb_curves) {
            let first = values.sequence(|s| s.value(index as usize).copied())?;
            let last = values.sequence(|s| s.value(index as usize + 1).copied())?;
            // OCCT3d097a0 leaves multipart entries null when Segment is false, despite the header comment.
            let result = if segment {
                self.segment_piece(curve, first, last)?
            } else {
                Handle::default()
            };
            self.set_curve(index, result)?;
        }
        Ok(())
    }
    fn segment_piece(
        &self,
        curve: &Handle<dyn GeomCurve>,
        first: f64,
        last: f64,
    ) -> Result<Handle<dyn GeomCurve>, Failure> {
        if curve.cast::<GeomBSplineCurve>().is_null() && curve.cast::<GeomBezierCurve>().is_null() {
            return trim_copy(curve, first, last);
        }
        let copy: Handle<dyn GeomCurve> = curve.copy()?.cast();
        if segment_spline(&copy, first, last).is_err() {
            return trim_copy(curve, first, last);
        }
        self.add_status(ShapeExtendStatus::Done3);
        Ok(copy)
    }
}
fn trim_copy(
    curve: &Handle<dyn GeomCurve>,
    first: f64,
    last: f64,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let copy: Handle<dyn GeomCurve> = curve.copy()?.cast();
    Ok(Handle::new(GeomTrimmedCurve::new(&copy, first, last, true, true)?).cast())
}
fn segment_spline(curve: &Handle<dyn GeomCurve>, first: f64, last: f64) -> Result<(), Failure> {
    if let Some(spline) = curve.cast::<GeomBSplineCurve>().get() {
        return spline.segment(first, last, precision::p_confusion());
    }
    curve.cast::<GeomBezierCurve>().segment(first, last)
}
fn snap_parameter(
    curve: &GeomBSplineCurve,
    values: &Handle<rustycad_tkernel::ncollection::HSequence<f64>>,
    index: usize,
    parameter: f64,
    knot: &mut i32,
    last: i32,
) -> Result<(), Failure> {
    while *knot <= last {
        let value = curve.knot(*knot)?;
        if parameter > value + precision::p_confusion() {
            *knot += 1;
            continue;
        }
        if parameter < value - precision::p_confusion() {
            break;
        }
        values.change_sequence(|s| s.set_value(index, value))?;
        *knot += 1;
    }
    Ok(())
}
