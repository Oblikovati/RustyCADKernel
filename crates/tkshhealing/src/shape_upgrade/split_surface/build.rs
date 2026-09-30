//! Dispatch wrappers before ordinary surface patch construction.
use super::swept::SweepKind;
use super::*;
use rustycad_tkg3d::geom::{
    GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution, GeomSweptSurface,
};
impl SplitSurfaceData {
    pub(super) fn build_surface(&self, segment: bool) -> Result<(), Failure> {
        let u = split_snapshot(&self.values(true))?;
        let v = split_snapshot(&self.values(false))?;
        let ub = split_endpoints(&self.values(true))?;
        let vb = split_endpoints(&self.values(false))?;
        let bounds = (ub.0, ub.1, vb.0, vb.1);
        if u.len() > 2 || v.len() > 2 {
            self.with_mut(|fields| {
                fields.status = ShapeExtend::encode_status(ShapeExtendStatus::Done1)
            });
        }
        let surface = self.with(|fields| fields.surface.clone());
        let _natural = surface.bounds();
        self.dispatch_surface(&surface, bounds, segment)
    }
    fn dispatch_surface(
        &self,
        surface: &Handle<dyn GeomSurface>,
        bounds: (f64, f64, f64, f64),
        segment: bool,
    ) -> Result<(), Failure> {
        if let Some(revolution) = surface.cast::<GeomSurfaceOfRevolution>().get() {
            return self.build_swept(
                SweepKind::Revolution(revolution.axis()),
                &revolution.basis_curve(),
                segment,
            );
        }
        if let Some(extrusion) = surface.cast::<GeomSurfaceOfLinearExtrusion>().get() {
            return self.build_swept(
                SweepKind::Extrusion(extrusion.direction()),
                &extrusion.basis_curve(),
                segment,
            );
        }
        self.dispatch_wrapper(surface, bounds, segment)
    }
    fn dispatch_wrapper(
        &self,
        surface: &Handle<dyn GeomSurface>,
        bounds: (f64, f64, f64, f64),
        segment: bool,
    ) -> Result<(), Failure> {
        if let Some(trimmed) = surface.cast::<GeomRectangularTrimmedSurface>().get() {
            return self.build_trimmed(&trimmed.basis_surface(), bounds, segment);
        }
        if let Some(offset) = surface.cast::<GeomOffsetSurface>().get() {
            return self.build_offset(offset, bounds, segment);
        }
        self.build_regular(surface, bounds, segment)
    }
    fn nested_splitter(
        &self,
        basis: &Handle<dyn GeomSurface>,
        bounds: (f64, f64, f64, f64),
        segment: bool,
    ) -> Result<ShapeUpgradeSplitSurface, Failure> {
        let nested = ShapeUpgradeSplitSurface::new();
        nested.init_surface_bounds(basis, bounds, 0.)?;
        nested.set_u_split_values(&self.values(true))?;
        nested.set_v_split_values(&self.values(false))?;
        nested
            .state
            .with_mut(|fields| fields.status = self.with(|own| own.status));
        nested.build(segment)?;
        Ok(nested)
    }
    fn build_trimmed(
        &self,
        basis: &Handle<dyn GeomSurface>,
        bounds: (f64, f64, f64, f64),
        segment: bool,
    ) -> Result<(), Failure> {
        let nested = self.nested_splitter(basis, bounds, segment)?;
        self.with_mut(|fields| {
            fields.status |= nested.state.with(|other| other.status);
            fields.result = nested.result_surfaces();
        });
        Ok(())
    }
    fn build_offset(
        &self,
        offset: &GeomOffsetSurface,
        bounds: (f64, f64, f64, f64),
        segment: bool,
    ) -> Result<(), Failure> {
        let nested = self.nested_splitter(&offset.basis_surface(), bounds, segment)?;
        let result = nested.result_surfaces();
        let grid = offset_grid(&result, offset.offset())?;
        self.adopt_natural_grid(&grid)?;
        self.with_mut(|fields| fields.status |= nested.state.with(|other| other.status));
        Ok(())
    }
    pub(super) fn adopt_natural_grid(&self, grid: &SurfaceGrid) -> Result<(), Failure> {
        let result = self.result();
        result.init(grid, ShapeExtendParametrisation::Natural)?;
        result.set_u_first_value(self.values(true).sequence(|s| s.value(1).copied())?)?;
        result.set_v_first_value(self.values(false).sequence(|s| s.value(1).copied())?)?;
        Ok(())
    }
    fn build_regular(
        &self,
        surface: &Handle<dyn GeomSurface>,
        bounds: (f64, f64, f64, f64),
        segment: bool,
    ) -> Result<(), Failure> {
        let rows = self.values(true).sequence(|s| s.len()) as i32 - 1;
        let columns = self.values(false).sequence(|s| s.len()) as i32 - 1;
        self.with_mut(|fields| {
            fields.rows = rows;
            fields.columns = columns;
        });
        let grid = new_grid(rows, columns)?;
        if rows == 1 && columns == 1 && self.fill_single(&grid, surface, bounds, segment)? {
            return self.adopt_natural_grid(&grid);
        }
        self.snap_knots(surface)?;
        self.fill_grid(&grid, surface)?;
        self.adopt_explicit_grid(&grid)
    }
    fn adopt_explicit_grid(&self, grid: &SurfaceGrid) -> Result<(), Failure> {
        let u = split_snapshot(&self.values(true))?;
        let v = split_snapshot(&self.values(false))?;
        self.result().init_with_joints(
            grid,
            &Array1::from_shared(&u, 1),
            &Array1::from_shared(&v, 1),
        )?;
        Ok(())
    }
}

fn offset_grid(result: &ShapeExtendCompositeSurface, offset: f64) -> Result<SurfaceGrid, Failure> {
    let rows = result.nb_u_patches()?;
    let columns = result.nb_v_patches()?;
    let grid = new_grid(rows, columns)?;
    for row in 1..=rows {
        for column in 1..=columns {
            let patch = Handle::new(GeomOffsetSurface::new(
                &result.patch(row, column)?,
                offset,
                false,
            )?)
            .cast();
            grid_put(&grid, row, column, patch)?;
        }
    }
    Ok(grid)
}
