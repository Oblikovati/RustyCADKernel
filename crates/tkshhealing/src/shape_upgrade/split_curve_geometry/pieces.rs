//! Native one-piece reuse, knot snapping and Segment fallback semantics.
use super::*;
impl<G: CurveGeometry + 'static> CurveSplitKernel<G> {
    pub(super) fn fill_single(
        &self,
        curve: &Handle<G::Curve>,
        first: f64,
        last: f64,
        segment: bool,
    ) -> Result<bool, Failure> {
        if self.split_curve_data().with(|f| f.nb_curves) != 1 {
            return Ok(false);
        }
        if (G::bounds(curve).0 - first).abs() < precision::p_confusion()
            && (G::bounds(curve).1 - last).abs() < precision::p_confusion()
        {
            self.set_curve(1, curve.clone())?;
            return Ok(true);
        }
        let spline = G::spline(curve);
        if segment && spline && self.status_flag(ShapeExtendStatus::Done2) {
            return Ok(false);
        }
        let piece = if spline && G::SEGMENT_SINGLE {
            segmented_copy::<G>(curve, first, last)?.0
        } else {
            trim_copy::<G>(curve, first, last)?
        };
        self.set_curve(1, piece)?;
        Ok(true)
    }
    pub(super) fn snap_spline_knots(&self, curve: &Handle<G::Curve>) -> Result<(), Failure> {
        let Some(knots) = G::knots(curve)? else {
            return Ok(());
        };
        let mut knot = 0;
        let last = knots.len() - 1;
        let values = self.split_values();
        for index in 1..=values.sequence(|s| s.len()) {
            let parameter = values.sequence(|s| s.value(index).copied())?;
            snap_parameter(&knots, &values, index, parameter, &mut knot, last)?;
            if knot == last {
                break;
            }
        }
        Ok(())
    }
    pub(super) fn build_pieces(
        &self,
        curve: &Handle<G::Curve>,
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
        curve: &Handle<G::Curve>,
        first: f64,
        last: f64,
    ) -> Result<Handle<G::Curve>, Failure> {
        let (copy, segmented) = segmented_copy::<G>(curve, first, last)?;
        if segmented {
            self.add_status(ShapeExtendStatus::Done3);
        }
        Ok(copy)
    }
}
fn trim_copy<G: CurveGeometry>(
    curve: &Handle<G::Curve>,
    first: f64,
    last: f64,
) -> Result<Handle<G::Curve>, Failure> {
    let copy = G::copy(curve)?;
    G::trim(&copy, first, last)
}
fn segmented_copy<G: CurveGeometry>(
    curve: &Handle<G::Curve>,
    first: f64,
    last: f64,
) -> Result<(Handle<G::Curve>, bool), Failure> {
    if !G::spline(curve) {
        return Ok((trim_copy::<G>(curve, first, last)?, false));
    }
    let copy = G::copy(curve)?;
    if G::segment(&copy, first, last).is_err() {
        return Ok((trim_copy::<G>(curve, first, last)?, false));
    }
    Ok((copy, true))
}
fn snap_parameter(
    knots: &[f64],
    values: &Handle<HSequence<f64>>,
    index: usize,
    parameter: f64,
    knot: &mut usize,
    last: usize,
) -> Result<(), Failure> {
    while *knot <= last {
        let value = knots[*knot];
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
