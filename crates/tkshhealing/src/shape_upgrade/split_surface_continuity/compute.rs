use super::*;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkernel::precision;
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomGeometry};

impl ShapeUpgradeSplitSurfaceContinuity {
    pub(super) fn compute_continuity(&self, segment: bool) -> Result<(), Failure> {
        let surface = self.surface();
        if !segment {
            self.extend_split_bounds(&surface);
        }
        let values = self.split_bounds()?;
        let settings = *self.settings.read();
        self.initialize_status(&surface, settings.criterion);
        if self.compute_wrapper(&surface, values, settings)? {
            return Ok(());
        }
        self.compute_spline(&surface, settings)
    }

    fn surface(&self) -> Handle<dyn GeomSurface> {
        self.split_surface_data()
            .with(|fields| fields.surface.clone())
    }
    fn split_bounds(&self) -> Result<(f64, f64, f64, f64), Failure> {
        let (u_first, u_last) = surface_endpoints(&self.u_split_values())?;
        let (v_first, v_last) = surface_endpoints(&self.v_split_values())?;
        Ok((u_first, u_last, v_first, v_last))
    }
    fn extend_split_bounds(&self, surface: &Handle<dyn GeomSurface>) {
        let Some(surface) = surface.get() else {
            return;
        };
        let (uf, ul, vf, vl) = surface.bounds();
        update_endpoints(&self.u_split_values(), (uf, ul));
        update_endpoints(&self.v_split_values(), (vf, vl));
    }
    fn initialize_status(&self, surface: &Handle<dyn GeomSurface>, criterion: Shape) {
        let Some(surface) = surface.get() else {
            return;
        };
        if surface.continuity() < criterion {
            self.set_status(ShapeExtendStatus::Done2);
        }
        if self.u_split_values().sequence(|s| s.len()) > 2
            || self.v_split_values().sequence(|s| s.len()) > 2
        {
            self.set_status(ShapeExtendStatus::Done1);
        }
    }
    fn compute_spline(
        &self,
        surface: &Handle<dyn GeomSurface>,
        settings: ContinuitySettings,
    ) -> Result<(), Failure> {
        let Some(copied) = self.copy_spline_for_correction(surface, settings.criterion)? else {
            return Ok(());
        };
        let (u_first, u_last, v_first, v_last) = self.split_bounds()?;
        if copied.nb_u_knots() > 2 {
            self.scan_direction(&copied, true, (u_first, u_last), settings)?;
        }
        if copied.nb_v_knots() > 2 {
            self.scan_direction(&copied, false, (v_first, v_last), settings)?;
        }
        self.retain_correction(copied);
        self.finalize_split_status();
        Ok(())
    }

    fn copy_spline_for_correction(
        &self,
        surface: &Handle<dyn GeomSurface>,
        criterion: Shape,
    ) -> Result<Option<Handle<GeomBSplineSurface>>, Failure> {
        let original = surface.get().ok_or_else(null_surface)?;
        if original.continuity() >= criterion {
            return Ok(None);
        }
        let spline = surface.cast::<GeomBSplineSurface>();
        if spline.is_null() {
            return Ok(None);
        }
        Ok(Some(spline.copy()?.cast()))
    }
    fn set_status(&self, status: ShapeExtendStatus) {
        let encoded = ShapeExtend::encode_status(status);
        self.split_surface_data()
            .with_mut(|fields| fields.status = encoded);
    }
}

fn surface_endpoints(values: &SurfaceSplitValues) -> Result<(f64, f64), Failure> {
    values.sequence(|seq| Ok((*seq.value(1)?, *seq.value(seq.len())?)))
}
fn update_endpoints(values: &SurfaceSplitValues, bounds: (f64, f64)) {
    values.change_sequence(|seq| {
        if !precision::is_infinite(bounds.0) {
            let _ = seq.set_value(1, bounds.0);
        }
        if !precision::is_infinite(bounds.1) {
            let _ = seq.set_value(seq.len(), bounds.1);
        }
    });
}
fn null_surface() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "ShapeUpgrade_SplitSurfaceContinuity: null input surface; expected initialized surface",
        "",
    )
}
