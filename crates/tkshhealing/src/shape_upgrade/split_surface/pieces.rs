//! The general grid segments spline patches even when Segment is false, as OCCT does.
use super::*;
impl SplitSurfaceData {
    pub(super) fn fill_single(
        &self,
        grid: &SurfaceGrid,
        surface: &Handle<dyn GeomSurface>,
        bounds: (f64, f64, f64, f64),
        segment: bool,
    ) -> Result<bool, Failure> {
        if same_surface_bounds(surface.bounds(), bounds) {
            grid_put(grid, 1, 1, surface.clone())?;
            return Ok(true);
        }
        let done2 =
            self.with(|fields| ShapeExtend::decode_status(fields.status, ShapeExtendStatus::Done2));
        if segment && !surface.cast::<GeomBSplineSurface>().is_null() && done2 {
            return Ok(false);
        }
        let copy: Handle<dyn GeomSurface> = surface.copy()?.cast();
        grid_put(grid, 1, 1, trimmed_surface(&copy, bounds)?)?;
        Ok(true)
    }
    pub(super) fn fill_grid(
        &self,
        grid: &SurfaceGrid,
        surface: &Handle<dyn GeomSurface>,
    ) -> Result<(), Failure> {
        let u = split_snapshot(&self.values(true))?;
        let v = split_snapshot(&self.values(false))?;
        for (row, u) in u.windows(2).enumerate() {
            for (column, v) in v.windows(2).enumerate() {
                let patch = self.build_patch(surface, (u[0], u[1], v[0], v[1]))?;
                grid_put(grid, row as i32 + 1, column as i32 + 1, patch)?;
            }
        }
        Ok(())
    }
    fn build_patch(
        &self,
        surface: &Handle<dyn GeomSurface>,
        bounds: (f64, f64, f64, f64),
    ) -> Result<Handle<dyn GeomSurface>, Failure> {
        let copy: Handle<dyn GeomSurface> = surface.copy()?.cast();
        let spline = !surface.cast::<GeomBSplineSurface>().is_null()
            || !surface.cast::<GeomBezierSurface>().is_null();
        if !spline {
            return trimmed_surface(&copy, bounds);
        }
        if segment_patch(&copy, bounds).is_err() {
            // Native fallback trims the possibly mutated copy, not a fresh supporting surface.
            return trimmed_surface(&copy, bounds);
        }
        self.add_status(ShapeExtendStatus::Done3);
        Ok(copy)
    }
}
fn segment_patch(
    surface: &Handle<dyn GeomSurface>,
    bounds: (f64, f64, f64, f64),
) -> Result<(), Failure> {
    let (u1, u2, v1, v2) = bounds;
    if let Some(spline) = surface.cast::<GeomBSplineSurface>().get() {
        return spline.segment(
            (u1, u2),
            (v1, v2),
            (precision::p_confusion(), precision::p_confusion()),
        );
    }
    surface.cast::<GeomBezierSurface>().segment(u1, u2, v1, v2)
}

fn same_surface_bounds(natural: (f64, f64, f64, f64), bounds: (f64, f64, f64, f64)) -> bool {
    let differences = [
        natural.0 - bounds.0,
        natural.1 - bounds.1,
        natural.2 - bounds.2,
        natural.3 - bounds.3,
    ];
    differences
        .iter()
        .all(|difference| difference.abs() < precision::p_confusion())
}
