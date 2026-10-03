use super::*;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkernel::precision;
use rustycad_tkg3d::geom::GeomBSplineSurface;

struct SurfaceKnotScan<'a> {
    surface: &'a GeomBSplineSurface,
    u_direction: bool,
    values: SurfaceSplitValues,
    first: f64,
    knot_index: i32,
    final_knot: i32,
    interval: usize,
}

impl ShapeUpgradeSplitSurfaceContinuity {
    pub(super) fn scan_direction(
        &self,
        surface: &GeomBSplineSurface,
        u_direction: bool,
        range: (f64, f64),
        settings: ContinuitySettings,
    ) -> Result<(), Failure> {
        let values = if u_direction {
            self.u_split_values()
        } else {
            self.v_split_values()
        };
        let mut scan = SurfaceKnotScan::new(surface, u_direction, range, values)?;
        scan.scan_intervals(self, settings)
    }

    pub(super) fn retain_correction(&self, corrected: Handle<GeomBSplineSurface>) {
        let status = self.split_surface_data().with(|fields| fields.status);
        if ShapeExtend::decode_status(status, ShapeExtendStatus::Done3) {
            let surface: Handle<dyn GeomSurface> = corrected.cast();
            self.split_surface_data()
                .with_mut(|fields| fields.surface = surface);
        }
    }

    pub(super) fn finalize_split_status(&self) {
        if self.u_split_values().sequence(|values| values.len()) > 2
            || self.v_split_values().sequence(|values| values.len()) > 2
        {
            let encoded = ShapeExtend::encode_status(ShapeExtendStatus::Done1);
            self.split_surface_data()
                .with_mut(|fields| fields.status |= encoded);
        }
    }
}

impl<'a> SurfaceKnotScan<'a> {
    fn new(
        surface: &'a GeomBSplineSurface,
        u_direction: bool,
        range: (f64, f64),
        values: SurfaceSplitValues,
    ) -> Result<Self, Failure> {
        Ok(Self {
            surface,
            u_direction,
            values,
            first: range.0,
            knot_index: first_knot_index(surface, u_direction)? + 1,
            final_knot: last_knot_index(surface, u_direction)? - 1,
            interval: 2,
        })
    }

    fn scan_intervals(
        &mut self,
        splitter: &ShapeUpgradeSplitSurfaceContinuity,
        settings: ContinuitySettings,
    ) -> Result<(), Failure> {
        while self.interval <= self.values.sequence(|sequence| sequence.len()) {
            self.scan_current_interval(splitter, settings)?;
        }
        Ok(())
    }

    fn scan_current_interval(
        &mut self,
        splitter: &ShapeUpgradeSplitSurfaceContinuity,
        settings: ContinuitySettings,
    ) -> Result<(), Failure> {
        let last = self
            .values
            .sequence(|sequence| sequence.value(self.interval).copied())?;
        while self.knot_index <= self.final_knot {
            if self.scan_current_knot(splitter, (self.first, last), settings)? {
                break;
            }
        }
        self.first = last;
        self.interval += 1;
        Ok(())
    }

    fn scan_current_knot(
        &mut self,
        splitter: &ShapeUpgradeSplitSurfaceContinuity,
        range: (f64, f64),
        settings: ContinuitySettings,
    ) -> Result<bool, Failure> {
        let value = direction_knot(self.surface, self.u_direction, self.knot_index)?;
        if value <= range.0 + precision::confusion() {
            self.knot_index += 1;
            return Ok(false);
        }
        if value >= range.1 - precision::confusion() {
            return Ok(true);
        }
        self.process_knot(splitter, value, settings)?;
        self.knot_index += 1;
        Ok(false)
    }

    fn process_knot(
        &mut self,
        splitter: &ShapeUpgradeSplitSurfaceContinuity,
        value: f64,
        settings: ContinuitySettings,
    ) -> Result<(), Failure> {
        let degree = direction_degree(self.surface, self.u_direction);
        let mult = direction_multiplicity(self.surface, self.u_direction, self.knot_index)?;
        if degree - mult >= settings.order {
            return Ok(());
        }
        let target = degree - settings.order;
        if self.remove_and_check(target, degree, settings)? {
            self.record_removal(splitter, target);
            return Ok(());
        }
        self.record_split(splitter, value)
    }

    fn remove_and_check(
        &self,
        target: i32,
        degree: i32,
        settings: ContinuitySettings,
    ) -> Result<bool, Failure> {
        if !self.remove_to_target(target, settings.tolerance)? {
            return Ok(false);
        }
        removed_order_satisfies(self, target, degree, settings.order)
    }

    fn remove_to_target(&self, target: i32, tolerance: f64) -> Result<bool, Failure> {
        if target < 0 {
            return Ok(false);
        }
        remove_direction_knot(
            self.surface,
            self.u_direction,
            self.knot_index,
            target,
            tolerance,
        )
    }

    fn record_removal(&mut self, splitter: &ShapeUpgradeSplitSurfaceContinuity, target: i32) {
        if target == 0 {
            self.knot_index -= 1;
            self.final_knot -= 1;
        }
        let encoded = ShapeExtend::encode_status(ShapeExtendStatus::Done3);
        splitter
            .split_surface_data()
            .with_mut(|fields| fields.status |= encoded);
    }

    fn record_split(
        &mut self,
        splitter: &ShapeUpgradeSplitSurfaceContinuity,
        value: f64,
    ) -> Result<(), Failure> {
        self.values
            .change_sequence(|sequence| sequence.insert_before(self.interval, value).map(|_| ()))?;
        splitter.split_surface_data().with_mut(|fields| {
            if self.u_direction {
                fields.rows += 1;
            } else {
                fields.columns += 1;
            }
        });
        self.interval += 1;
        Ok(())
    }
}

fn removed_order_satisfies(
    scan: &SurfaceKnotScan<'_>,
    target: i32,
    degree: i32,
    order: i32,
) -> Result<bool, Failure> {
    if target == 0 {
        return Ok(true);
    }
    let multiplicity = direction_multiplicity(scan.surface, scan.u_direction, scan.knot_index)?;
    Ok(degree - multiplicity >= order)
}

fn first_knot_index(surface: &GeomBSplineSurface, u: bool) -> Result<i32, Failure> {
    if u {
        surface.first_u_knot_index()
    } else {
        surface.first_v_knot_index()
    }
}
fn last_knot_index(surface: &GeomBSplineSurface, u: bool) -> Result<i32, Failure> {
    if u {
        surface.last_u_knot_index()
    } else {
        surface.last_v_knot_index()
    }
}
fn direction_degree(surface: &GeomBSplineSurface, u: bool) -> i32 {
    if u {
        surface.u_degree()
    } else {
        surface.v_degree()
    }
}
fn direction_knot(surface: &GeomBSplineSurface, u: bool, index: i32) -> Result<f64, Failure> {
    if u {
        surface.u_knot(index)
    } else {
        surface.v_knot(index)
    }
}
fn direction_multiplicity(
    surface: &GeomBSplineSurface,
    u: bool,
    index: i32,
) -> Result<i32, Failure> {
    if u {
        surface.u_multiplicity(index)
    } else {
        surface.v_multiplicity(index)
    }
}
fn remove_direction_knot(
    surface: &GeomBSplineSurface,
    u: bool,
    index: i32,
    target: i32,
    tolerance: f64,
) -> Result<bool, Failure> {
    if u {
        surface.remove_u_knot(index, target, tolerance)
    } else {
        surface.remove_v_knot(index, target, tolerance)
    }
}
