use super::*;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use crate::shape_upgrade::curve_continuity::continuity_order;
use crate::shape_upgrade::{ShapeUpgradeSplitCurve3dContinuity, SplitCurve, SplitSurface};
use rustycad_tkg3d::geom::{GeomOffsetSurface, GeomRectangularTrimmedSurface, GeomSweptSurface};

impl ShapeUpgradeSplitSurfaceContinuity {
    pub(super) fn compute_wrapper(
        &self,
        surface: &Handle<dyn GeomSurface>,
        bounds: (f64, f64, f64, f64),
        settings: ContinuitySettings,
    ) -> Result<bool, Failure> {
        if let Some(surface) = surface.cast::<GeomSurfaceOfRevolution>().get() {
            self.compute_revolution(surface, bounds, settings)?;
            return Ok(true);
        }
        if let Some(surface) = surface.cast::<GeomSurfaceOfLinearExtrusion>().get() {
            self.compute_extrusion(surface, bounds, settings)?;
            return Ok(true);
        }
        if self.compute_trimmed(surface, bounds, settings)? {
            return Ok(true);
        }
        self.compute_offset(surface, bounds, settings)
    }

    fn compute_revolution(
        &self,
        surface: &GeomSurfaceOfRevolution,
        bounds: (f64, f64, f64, f64),
        settings: ContinuitySettings,
    ) -> Result<(), Failure> {
        if surface.continuity() >= settings.criterion && self.has_only_endpoints() {
            return Ok(());
        }
        let basis = surface.basis_curve();
        let splitter = self.split_basis_curve(
            &basis,
            (bounds.2, bounds.3),
            &self.v_split_values(),
            settings,
        )?;
        self.apply_curve_results(&self.v_split_values(), &splitter);
        Ok(())
    }

    fn compute_extrusion(
        &self,
        surface: &GeomSurfaceOfLinearExtrusion,
        bounds: (f64, f64, f64, f64),
        settings: ContinuitySettings,
    ) -> Result<(), Failure> {
        if surface.continuity() >= settings.criterion && self.has_only_endpoints() {
            return Ok(());
        }
        let basis = surface.basis_curve();
        let splitter = self.split_basis_curve(
            &basis,
            (bounds.0, bounds.1),
            &self.u_split_values(),
            settings,
        )?;
        self.apply_extrusion_results(surface, &splitter)
    }

    fn apply_extrusion_results(
        &self,
        surface: &GeomSurfaceOfLinearExtrusion,
        splitter: &ShapeUpgradeSplitCurve3dContinuity,
    ) -> Result<(), Failure> {
        self.apply_curve_results(&self.u_split_values(), splitter);
        self.replace_extrusion_basis(surface, splitter)
    }

    fn apply_curve_results(
        &self,
        values: &SurfaceSplitValues,
        splitter: &ShapeUpgradeSplitCurve3dContinuity,
    ) {
        self.replace_values(values, &splitter.split_values());
        self.merge_curve_status(splitter);
    }

    fn replace_extrusion_basis(
        &self,
        surface: &GeomSurfaceOfLinearExtrusion,
        splitter: &ShapeUpgradeSplitCurve3dContinuity,
    ) -> Result<(), Failure> {
        if splitter.status_flag(ShapeExtendStatus::Done3) {
            surface.set_basis_curve(&splitter.curve())?;
        }
        Ok(())
    }

    fn split_basis_curve(
        &self,
        basis: &Handle<dyn rustycad_tkg3d::geom::GeomCurve>,
        range: (f64, f64),
        values: &SurfaceSplitValues,
        settings: ContinuitySettings,
    ) -> Result<ShapeUpgradeSplitCurve3dContinuity, Failure> {
        let splitter = ShapeUpgradeSplitCurve3dContinuity::new();
        splitter.init_curve_range(basis, range.0, range.1)?;
        splitter.set_criterion(settings.criterion);
        splitter.set_tolerance(settings.tolerance);
        splitter.set_split_values(values)?;
        splitter.compute()?;
        Ok(splitter)
    }

    fn compute_trimmed(
        &self,
        surface: &Handle<dyn GeomSurface>,
        bounds: (f64, f64, f64, f64),
        settings: ContinuitySettings,
    ) -> Result<bool, Failure> {
        let handle = surface.cast::<GeomRectangularTrimmedSurface>();
        let Some(trimmed) = handle.get() else {
            return Ok(false);
        };
        if trimmed.continuity() >= settings.criterion && self.has_only_endpoints() {
            return Ok(true);
        }
        let trim_bounds = trimmed.bounds();
        let basis = trimmed.basis_surface();
        let clipped = clip_bounds(trim_bounds, bounds);
        self.compute_nested_surface(&basis, clipped, settings)?;
        Ok(true)
    }

    fn compute_offset(
        &self,
        surface: &Handle<dyn GeomSurface>,
        bounds: (f64, f64, f64, f64),
        settings: ContinuitySettings,
    ) -> Result<bool, Failure> {
        let handle = surface.cast::<GeomOffsetSurface>();
        let Some(offset) = handle.get() else {
            return Ok(false);
        };
        let criterion = offset_surface_criterion(settings.criterion);
        let basis = offset.basis_surface();
        if basis.continuity() >= criterion && self.has_only_endpoints() {
            return Ok(true);
        }
        let nested_settings = offset_settings(settings, criterion);
        self.compute_nested_surface(&basis, bounds, nested_settings)?;
        Ok(true)
    }

    fn compute_nested_surface(
        &self,
        basis: &Handle<dyn GeomSurface>,
        bounds: (f64, f64, f64, f64),
        settings: ContinuitySettings,
    ) -> Result<(), Failure> {
        let nested = Self::new();
        nested.init_surface_bounds(basis, bounds, 0.0)?;
        nested.set_u_split_values(&self.u_split_values())?;
        nested.set_v_split_values(&self.v_split_values())?;
        nested.set_tolerance(settings.tolerance);
        nested.set_criterion(settings.criterion);
        nested.compute(true)?;
        self.replace_values(&self.u_split_values(), &nested.u_split_values());
        self.replace_values(&self.v_split_values(), &nested.v_split_values());
        let status = nested.split_surface_data().with(|fields| fields.status);
        self.split_surface_data()
            .with_mut(|fields| fields.status |= status);
        Ok(())
    }

    fn has_only_endpoints(&self) -> bool {
        self.u_split_values().sequence(|values| values.len()) == 2
            && self.v_split_values().sequence(|values| values.len()) == 2
    }

    fn replace_values(&self, target: &SurfaceSplitValues, source: &SurfaceSplitValues) {
        let sequence = source.sequence(Clone::clone);
        target.change_sequence(|values| *values = sequence);
    }

    fn merge_curve_status(&self, splitter: &ShapeUpgradeSplitCurve3dContinuity) {
        let mut status = 0;
        for flag in [
            ShapeExtendStatus::Done1,
            ShapeExtendStatus::Done2,
            ShapeExtendStatus::Done3,
        ] {
            if splitter.status_flag(flag) {
                status |= ShapeExtend::encode_status(flag);
            }
        }
        self.split_surface_data()
            .with_mut(|fields| fields.status |= status);
    }
}

fn clip_bounds(trim: (f64, f64, f64, f64), split: (f64, f64, f64, f64)) -> (f64, f64, f64, f64) {
    (
        trim.0.max(split.0),
        trim.1.min(split.1),
        trim.2.max(split.2),
        trim.3.min(split.3),
    )
}

fn offset_surface_criterion(criterion: Shape) -> Shape {
    match criterion {
        Shape::C1 => Shape::C2,
        Shape::C2 => Shape::C3,
        Shape::C3 | Shape::CN => Shape::CN,
        _ => Shape::C2,
    }
}

fn offset_settings(settings: ContinuitySettings, criterion: Shape) -> ContinuitySettings {
    ContinuitySettings {
        criterion,
        order: continuity_order(criterion),
        ..settings
    }
}
