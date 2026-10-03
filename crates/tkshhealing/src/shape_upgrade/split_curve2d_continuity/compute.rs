//! Preserve 2D source status order, interval knot cursor, and removal-index quirk.
use super::*;
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkmath::geom_abs::Shape as Continuity;

struct KnotScan {
    first: f64,
    last: f64,
    interval: usize,
    index: i32,
    final_index: i32,
}

impl KnotScan {
    fn new(spline: &Geom2dBSplineCurve, first: f64) -> Result<Self, Failure> {
        Ok(Self {
            first,
            last: first,
            interval: 2,
            index: spline.first_u_knot_index()? + 1,
            final_index: spline.last_u_knot_index()? - 1,
        })
    }
}

impl ShapeUpgradeSplitCurve2dContinuity {
    pub(super) fn compute_continuity(&self) -> Result<(), Failure> {
        let values = self.split_values();
        let curve = self.curve();
        let settings = *self.settings.read();
        if curve.continuity() < settings.criterion {
            self.replace_status(ShapeExtendStatus::Done2);
        }
        if values.sequence(|s| s.len()) > 2 {
            self.replace_status(ShapeExtendStatus::Done1);
        }
        let (first, last) =
            values.sequence(|s| Ok::<_, Failure>((*s.value(1)?, *s.value(s.len())?)))?;
        self.compute_geometry(&curve, first, last, settings)
    }

    fn compute_geometry(
        &self,
        curve: &Handle<dyn Geom2dCurve>,
        first: f64,
        last: f64,
        settings: ContinuitySettings,
    ) -> Result<(), Failure> {
        if let Some((basis, settings)) = wrapped_basis(curve, settings) {
            return self.compute_basis(&basis, first, last, settings);
        }
        let spline = curve.cast::<Geom2dBSplineCurve>();
        if spline.is_null() {
            return Ok(());
        }
        self.compute_spline(&spline, first, settings)
    }

    fn compute_basis(
        &self,
        basis: &Handle<dyn Geom2dCurve>,
        first: f64,
        last: f64,
        settings: ContinuitySettings,
    ) -> Result<(), Failure> {
        let nested = Self::new();
        nested.init_curve_range(basis, first, last)?;
        nested.set_split_values(&self.split_values())?;
        nested.set_tolerance(settings.tolerance);
        nested.set_criterion(settings.criterion);
        nested.compute()?;
        copy_nested_results(self, &nested)
    }

    fn compute_spline(
        &self,
        spline: &Geom2dBSplineCurve,
        first: f64,
        settings: ContinuitySettings,
    ) -> Result<(), Failure> {
        self.split_curve_data()
            .with_mut(|fields| fields.nb_curves = 1);
        if spline.nb_knots() <= 2 {
            return Ok(());
        }
        let mut scan = KnotScan::new(spline, first)?;
        self.scan_intervals(spline, &mut scan, settings)?;
        self.finish_status()
    }

    fn scan_intervals(
        &self,
        spline: &Geom2dBSplineCurve,
        scan: &mut KnotScan,
        settings: ContinuitySettings,
    ) -> Result<(), Failure> {
        while scan.interval <= self.split_values().sequence(|s| s.len()) {
            scan.last = self
                .split_values()
                .sequence(|s| s.value(scan.interval).copied())?;
            self.scan_interval(spline, scan, settings)?;
            scan.first = scan.last;
            scan.interval += 1;
        }
        Ok(())
    }

    fn scan_interval(
        &self,
        spline: &Geom2dBSplineCurve,
        scan: &mut KnotScan,
        settings: ContinuitySettings,
    ) -> Result<(), Failure> {
        while scan.index <= scan.final_index {
            let value = spline.knot(scan.index)?;
            if value <= scan.first + precision::p_confusion() {
                scan.index += 1;
                continue;
            }
            if value >= scan.last - precision::p_confusion() {
                break;
            }
            self.process_knot(spline, scan, settings)?;
            scan.index += 1;
        }
        Ok(())
    }

    fn process_knot(
        &self,
        spline: &Geom2dBSplineCurve,
        scan: &mut KnotScan,
        settings: ContinuitySettings,
    ) -> Result<(), Failure> {
        if spline.degree() - spline.multiplicity(scan.index)? >= settings.order {
            return Ok(());
        }
        let multiplicity = (spline.degree() - settings.order).max(0);
        if remove_for_continuity(spline, scan.index, multiplicity, settings)? {
            self.add_status(ShapeExtendStatus::Done3);
            return Ok(());
        }
        self.split_at_knot(spline.knot(scan.index)?, scan.interval)?;
        scan.interval += 1;
        self.split_curve_data()
            .with_mut(|fields| fields.nb_curves += 1);
        Ok(())
    }

    fn split_at_knot(&self, value: f64, interval: usize) -> Result<(), Failure> {
        self.split_values()
            .change_sequence(|values| values.insert_before(interval, value).map(|_| ()))
    }

    fn finish_status(&self) -> Result<(), Failure> {
        if self.split_values().sequence(|values| values.len()) > 2 {
            self.replace_status(ShapeExtendStatus::Done1);
        }
        Ok(())
    }
}

fn copy_nested_results(
    parent: &ShapeUpgradeSplitCurve2dContinuity,
    nested: &ShapeUpgradeSplitCurve2dContinuity,
) -> Result<(), Failure> {
    let sequence = nested.split_values().sequence(Clone::clone);
    parent
        .split_values()
        .change_sequence(|values| *values = sequence);
    let status = nested.split_curve_data().with(|fields| fields.status);
    parent
        .split_curve_data()
        .with_mut(|fields| fields.status |= status);
    Ok(())
}

fn remove_for_continuity(
    spline: &Geom2dBSplineCurve,
    knot: i32,
    multiplicity: i32,
    settings: ContinuitySettings,
) -> Result<bool, Failure> {
    let corrected = spline
        .remove_knot(knot, multiplicity, settings.tolerance)
        .unwrap_or(false);
    if corrected && multiplicity > 0 {
        return Ok(spline.degree() - spline.multiplicity(knot)? >= settings.order);
    }
    Ok(corrected)
}

fn offset_criterion(criterion: Continuity) -> Continuity {
    match criterion {
        Continuity::C2 => Continuity::C3,
        Continuity::C3 | Continuity::CN => Continuity::CN,
        _ => Continuity::C2,
    }
}

fn wrapped_basis(
    curve: &Handle<dyn Geom2dCurve>,
    settings: ContinuitySettings,
) -> Option<(Handle<dyn Geom2dCurve>, ContinuitySettings)> {
    if let Some(trimmed) = curve.cast::<Geom2dTrimmedCurve>().get() {
        return Some((trimmed.basis_curve(), settings));
    }
    let offset = curve.cast::<Geom2dOffsetCurve>();
    let basis = offset.get()?.basis_curve();
    let criterion = offset_criterion(settings.criterion);
    let adjusted = ContinuitySettings {
        criterion,
        order: continuity_order(criterion),
        ..settings
    };
    Some((basis, adjusted))
}
