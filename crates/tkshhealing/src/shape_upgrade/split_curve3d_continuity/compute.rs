//! Preserve native status overwrites, copied wrapper recursion and knot-removal order.
use super::*;
use crate::shape_extend::ShapeExtendStatus;
use crate::shape_upgrade::curve_continuity::{
    KnotRemoval, add_status, copy_nested_results, offset_criterion, remove_knot_and_recheck,
    replace_status,
};

impl ShapeUpgradeSplitCurve3dContinuity {
    pub(super) fn compute_continuity(&self) -> Result<(), Failure> {
        let values = self.split_values();
        let (first, last) =
            values.sequence(|s| Ok::<_, Failure>((*s.value(1)?, *s.value(s.len())?)))?;
        let curve = self.curve();
        let settings = *self.settings.read();
        if curve.continuity() < settings.criterion {
            replace_status(self.split_curve_data(), ShapeExtendStatus::Done2);
        }
        self.mark_existing_splits();
        self.compute_geometry(&curve, first, last, settings)
    }
    fn compute_geometry(
        &self,
        curve: &Handle<dyn GeomCurve>,
        first: f64,
        last: f64,
        settings: ContinuitySettings,
    ) -> Result<(), Failure> {
        if let Some(trimmed) = curve.cast::<GeomTrimmedCurve>().get() {
            return self.compute_basis(&trimmed.basis_curve(), first, last, settings);
        }
        if let Some(offset) = curve.cast::<GeomOffsetCurve>().get() {
            let settings = ContinuitySettings {
                criterion: offset_criterion(settings.criterion),
                ..settings
            };
            return self.compute_basis(&offset.basis_curve(), first, last, settings);
        }
        let spline = curve.cast::<GeomBSplineCurve>();
        if spline.is_null() {
            return Ok(());
        }
        self.compute_spline(&spline, first, settings)
    }
    fn mark_existing_splits(&self) {
        if self.split_values().sequence(|s| s.len()) > 2 {
            replace_status(self.split_curve_data(), ShapeExtendStatus::Done1);
        }
    }
    fn compute_basis(
        &self,
        basis: &Handle<dyn GeomCurve>,
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
        // Native recursion copies the sequence into the existing shared handle.
        copy_nested_results(
            nested.split_curve_data(),
            &nested.split_values(),
            self.split_curve_data(),
            &self.split_values(),
        )
    }
    fn compute_spline(
        &self,
        spline: &GeomBSplineCurve,
        mut first: f64,
        settings: ContinuitySettings,
    ) -> Result<(), Failure> {
        self.split_curve_data().with_mut(|f| f.nb_curves = 1);
        if spline.nb_knots() <= 2 {
            return Ok(());
        }
        let mut interval = 2;
        while interval <= self.split_values().sequence(|s| s.len()) {
            let last = self
                .split_values()
                .sequence(|s| s.value(interval).copied())?;
            self.scan_interval(spline, first, last, &mut interval, settings)?;
            first = last;
            interval += 1;
        }
        self.mark_existing_splits();
        Ok(())
    }
    fn scan_interval(
        &self,
        spline: &GeomBSplineCurve,
        first: f64,
        last: f64,
        interval: &mut usize,
        settings: ContinuitySettings,
    ) -> Result<(), Failure> {
        let mut knot = spline.first_u_knot_index()? + 1;
        let mut final_knot = spline.last_u_knot_index()? - 1;
        while knot <= final_knot {
            let value = spline.knot(knot)?;
            if value <= first + precision::p_confusion() {
                knot += 1;
                continue;
            }
            if value > last - precision::p_confusion() {
                break;
            }
            if self.correct_knot(spline, knot, interval, settings)? {
                final_knot = spline.last_u_knot_index()? - 1;
                continue;
            }
            knot += 1;
        }
        Ok(())
    }
    fn correct_knot(
        &self,
        spline: &GeomBSplineCurve,
        knot: i32,
        interval: &mut usize,
        settings: ContinuitySettings,
    ) -> Result<bool, Failure> {
        if spline.degree() - spline.multiplicity(knot)? >= settings.order {
            return Ok(false);
        }
        let multiplicity = (spline.degree() - settings.order).max(0);
        if remove_for_continuity(spline, knot, multiplicity, settings)? {
            add_status(self.split_curve_data(), ShapeExtendStatus::Done3);
            return Ok(multiplicity == 0);
        }
        self.split_values()
            .change_sequence(|s| s.insert_before(*interval, spline.knot(knot)?).map(|_| ()))?;
        *interval += 1;
        self.split_curve_data().with_mut(|f| f.nb_curves += 1);
        Ok(false)
    }
}
fn remove_for_continuity(
    spline: &GeomBSplineCurve,
    knot: i32,
    multiplicity: i32,
    settings: ContinuitySettings,
) -> Result<bool, Failure> {
    remove_knot_and_recheck(
        KnotRemoval {
            degree: spline.degree(),
            knot,
            multiplicity,
            tolerance: settings.tolerance,
            order: settings.order,
        },
        |index, mult, tolerance| spline.remove_knot(index, mult, tolerance),
        |index| spline.multiplicity(index),
    )
}
