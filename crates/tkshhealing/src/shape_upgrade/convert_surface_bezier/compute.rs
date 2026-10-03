use super::*;
use crate::shape_extend::ShapeExtendStatus;
use crate::shape_upgrade::SurfaceSplitValues;
use rustycad_tkernel::{
    ncollection::{Array1, Array2, HArray2},
    precision,
};
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomOffsetSurface, GeomPlane,
    GeomRectangularTrimmedSurface,
};
use rustycad_tkgeombase::geom_convert::BSplineSurfaceToBezierSurface;

impl ShapeUpgradeConvertSurfaceToBezierBasis {
    pub(super) fn compute_conversion(&self, segment: bool) -> Result<(), Failure> {
        let (surface, bounds) = self.input_bounds(segment)?;
        if let Some(trimmed) = surface.cast::<GeomRectangularTrimmedSurface>().get() {
            return self.compute_nested(&trimmed.basis_surface(), bounds);
        }
        if let Some(offset) = surface.cast::<GeomOffsetSurface>().get() {
            return self.compute_nested(&offset.basis_surface(), bounds);
        }
        self.compute_regular(&surface, bounds)
    }
    fn input_bounds(
        &self,
        segment: bool,
    ) -> Result<(Handle<dyn GeomSurface>, ParameterBounds), Failure> {
        let surface = self.state().with(|fields| fields.surface.clone());
        let values = (self.u_split_values(), self.v_split_values());
        if !segment {
            replace_finite_endpoints(&values.0, surface.bounds().0, surface.bounds().1)?;
            replace_finite_endpoints(&values.1, surface.bounds().2, surface.bounds().3)?;
        }
        let bounds = (
            sequence_value(&values.0, 1)?,
            sequence_value(&values.0, values.0.sequence(|s| s.len()) as i32)?,
            sequence_value(&values.1, 1)?,
            sequence_value(&values.1, values.1.sequence(|s| s.len()) as i32)?,
        );
        Ok((surface, bounds))
    }
    fn compute_regular(
        &self,
        surface: &Handle<dyn GeomSurface>,
        bounds: (f64, f64, f64, f64),
    ) -> Result<(), Failure> {
        if self.compute_primary(surface, bounds)? {
            return Ok(());
        }
        if let Some(done) = self.compute_swept(surface, bounds)? {
            return Ok(done);
        }
        self.compute_fallback(surface, bounds)
    }
    fn compute_primary(
        &self,
        surface: &Handle<dyn GeomSurface>,
        bounds: ParameterBounds,
    ) -> Result<bool, Failure> {
        if self.compute_plane_if(surface, bounds)? {
            return Ok(true);
        }
        if self.compute_bezier_if(surface, bounds)? {
            return Ok(true);
        }
        if self.compute_bspline_if(surface)? {
            return Ok(true);
        }
        Ok(false)
    }
    fn compute_plane_if(
        &self,
        surface: &Handle<dyn GeomSurface>,
        bounds: ParameterBounds,
    ) -> Result<bool, Failure> {
        let plane_handle = surface.cast::<GeomPlane>();
        let Some(plane) = plane_handle.get().filter(|_| self.modes().plane) else {
            return Ok(false);
        };
        self.compute_plane(plane, bounds)?;
        Ok(true)
    }
    fn compute_bezier_if(
        &self,
        surface: &Handle<dyn GeomSurface>,
        bounds: ParameterBounds,
    ) -> Result<bool, Failure> {
        let bezier_handle = surface.cast::<GeomBezierSurface>();
        let Some(bezier) = bezier_handle.get() else {
            return Ok(false);
        };
        self.compute_bezier(bezier, bounds)?;
        Ok(true)
    }
    fn compute_bspline_if(&self, surface: &Handle<dyn GeomSurface>) -> Result<bool, Failure> {
        let Some(_spline) = surface
            .cast::<GeomBSplineSurface>()
            .get()
            .filter(|_| self.modes().bspline)
        else {
            return Ok(false);
        };
        self.compute_bspline()?;
        Ok(true)
    }
    fn compute_plane(
        &self,
        plane: &GeomPlane,
        bounds: (f64, f64, f64, f64),
    ) -> Result<(), Failure> {
        let (u1, u2, v1, v2) = bounds;
        let points = [
            plane.value(u1, v1)?,
            plane.value(u1, v2)?,
            plane.value(u2, v1)?,
            plane.value(u2, v2)?,
        ];
        let poles = Array2::from_shared(&points, 1, 2, 1, 2)
            .map_err(|e| e.into_failure("ShapeUpgrade_ConvertSurfaceToBezierBasis::Compute"))?;
        let bezier: Handle<dyn GeomSurface> = Handle::new(GeomBezierSurface::new(&poles)?).cast();
        self.store_single(bezier, bounds, ShapeExtendStatus::Done1)
    }
    fn compute_bezier(
        &self,
        bezier: &GeomBezierSurface,
        bounds: (f64, f64, f64, f64),
    ) -> Result<(), Failure> {
        let full = full_bezier_bounds(bounds);
        let patch = bezier_patch(self, bezier, bounds, full)?;
        self.store_single(
            patch,
            bounds,
            if full {
                ShapeExtendStatus::Ok
            } else {
                ShapeExtendStatus::Done2
            },
        )
    }
    fn compute_bspline(&self) -> Result<(), Failure> {
        let spline_handle: Handle<GeomBSplineSurface> = self.state().with(|f| f.surface.cast());
        let converter = BSplineSurfaceToBezierSurface::new(&spline_handle)?;
        let (u_raw, v_raw) = raw_knots(&converter)?;
        let u = retained_knots(&u_raw);
        let v = retained_knots(&v_raw);
        insert_interior_knots(&self.u_split_values(), &u_raw)?;
        insert_interior_knots(&self.v_split_values(), &v_raw)?;
        let composite = bspline_segments(&converter, &u_raw, &v_raw, &u, &v)?;
        self.set_segments(composite);
        self.set_status(ShapeExtendStatus::Done1);
        Ok(())
    }
    fn compute_fallback(
        &self,
        surface: &Handle<dyn GeomSurface>,
        bounds: (f64, f64, f64, f64),
    ) -> Result<(), Failure> {
        let patch = fallback_patch(surface, bounds)?;
        self.store_single(patch, bounds, ShapeExtendStatus::Ok)
    }
    fn compute_nested(
        &self,
        basis: &Handle<dyn GeomSurface>,
        bounds: (f64, f64, f64, f64),
    ) -> Result<(), Failure> {
        let nested = ShapeUpgradeConvertSurfaceToBezierBasis::new();
        nested.init_surface_bounds(basis, bounds)?;
        nested.set_u_split_values(&self.u_split_values())?;
        nested.set_v_split_values(&self.v_split_values())?;
        nested.compute_conversion(true)?;
        replace_sequence(&self.u_split_values(), &nested.u_split_values())?;
        replace_sequence(&self.v_split_values(), &nested.v_split_values())?;
        self.state()
            .with_mut(|f| f.status |= nested.state().with(|n| n.status));
        *self.segments.write() = nested.segments();
        Ok(())
    }
    fn store_single(
        &self,
        patch: Handle<dyn GeomSurface>,
        bounds: (f64, f64, f64, f64),
        status: ShapeExtendStatus,
    ) -> Result<(), Failure> {
        self.set_segments(single_segment(patch, bounds)?);
        self.set_status(status);
        Ok(())
    }
    pub(super) fn set_status(&self, status: ShapeExtendStatus) {
        self.state()
            .with_mut(|f| f.status = ShapeExtend::encode_status(status));
    }
}

fn single_segment(
    patch: Handle<dyn GeomSurface>,
    bounds: ParameterBounds,
) -> Result<ShapeExtendCompositeSurface, Failure> {
    let grid = Handle::new(
        HArray2::with_value(1, 1, 1, 1, Handle::<dyn GeomSurface>::default())
            .map_err(|e| e.into_failure("ShapeUpgrade_ConvertSurfaceToBezierBasis::Compute"))?,
    );
    grid.change_array2()
        .set_value(1, 1, patch)
        .map_err(|e| e.into_failure("ShapeUpgrade_ConvertSurfaceToBezierBasis::Compute"))?;
    ShapeExtendCompositeSurface::with_joints(
        &grid,
        &Array1::from_shared(&[bounds.0, bounds.1], 1),
        &Array1::from_shared(&[bounds.2, bounds.3], 1),
    )
}
fn full_bezier_bounds((u1, u2, v1, v2): ParameterBounds) -> bool {
    u1 < precision::p_confusion()
        && u2 > 1. - precision::p_confusion()
        && v1 < precision::p_confusion()
        && v2 > 1. - precision::p_confusion()
}
fn bezier_patch(
    tool: &ShapeUpgradeConvertSurfaceToBezierBasis,
    bezier: &GeomBezierSurface,
    bounds: ParameterBounds,
    full: bool,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    if full {
        return Ok(tool.state().with(|fields| fields.surface.clone()));
    }
    let copy = GeomBezierSurface::copy_of(bezier)?;
    copy.segment(bounds.0, bounds.1, bounds.2, bounds.3)?;
    Ok(Handle::new(copy).cast())
}
fn fallback_patch(
    surface: &Handle<dyn GeomSurface>,
    bounds: ParameterBounds,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let natural = surface.bounds();
    let p = precision::p_confusion();
    if bounds_cover_natural(natural, bounds, p) {
        return Ok(surface.clone());
    }
    Ok(Handle::new(GeomRectangularTrimmedSurface::new(
        surface,
        (bounds.0, bounds.1),
        (bounds.2, bounds.3),
        (true, true),
    )?)
    .cast())
}
fn bounds_cover_natural(
    natural: ParameterBounds,
    requested: ParameterBounds,
    tolerance: f64,
) -> bool {
    natural.0 - requested.0 < tolerance
        && requested.1 - natural.1 < tolerance
        && natural.3 - requested.2 < tolerance
        && requested.3 - natural.3 < tolerance
}
fn bspline_segments(
    converter: &BSplineSurfaceToBezierSurface,
    u_raw: &[f64],
    v_raw: &[f64],
    u: &[f64],
    v: &[f64],
) -> Result<ShapeExtendCompositeSurface, Failure> {
    let grid = bezier_patch_grid(converter, u_raw, v_raw, u.len() - 1, v.len() - 1)?;
    ShapeExtendCompositeSurface::with_joints(
        &grid,
        &Array1::from_shared(u, 1),
        &Array1::from_shared(v, 1),
    )
}
fn bezier_patch_grid(
    converter: &BSplineSurfaceToBezierSurface,
    u: &[f64],
    v: &[f64],
    rows: usize,
    cols: usize,
) -> Result<crate::shape_extend::SurfaceGrid, Failure> {
    let grid = Handle::new(
        HArray2::with_value(
            1,
            rows as i32,
            1,
            cols as i32,
            Handle::<dyn GeomSurface>::default(),
        )
        .map_err(|e| e.into_failure("ShapeUpgrade_ConvertSurfaceToBezierBasis::Compute"))?,
    );
    fill_bspline_grid(converter, &grid, u, v)?;
    Ok(grid)
}

fn sequence_value(values: &SurfaceSplitValues, index: i32) -> Result<f64, Failure> {
    values.sequence(|s| s.value(index as usize).copied())
}
fn replace_finite_endpoints(
    values: &SurfaceSplitValues,
    first: f64,
    last: f64,
) -> Result<(), Failure> {
    if !precision::is_infinite(first) {
        values.change_sequence(|s| s.set_value(1, first))?;
    }
    if !precision::is_infinite(last) {
        let len = values.sequence(|s| s.len());
        values.change_sequence(|s| s.set_value(len, last))?;
    }
    Ok(())
}
fn replace_sequence(
    target: &SurfaceSplitValues,
    source: &SurfaceSplitValues,
) -> Result<(), Failure> {
    let values = source.sequence(|s| s.iter().copied().collect());
    target.change_sequence(|s| *s = values);
    Ok(())
}
fn raw_knots(converter: &BSplineSurfaceToBezierSurface) -> Result<(Vec<f64>, Vec<f64>), Failure> {
    let mut u = vec![0.; (converter.nb_u_patches()? + 1) as usize];
    let mut v = vec![0.; (converter.nb_v_patches()? + 1) as usize];
    converter.u_knots(&mut Array1::from_buffer(&mut u, true))?;
    converter.v_knots(&mut Array1::from_buffer(&mut v, true))?;
    Ok((u, v))
}
fn retained_knots(values: &[f64]) -> Vec<f64> {
    let mut retained = vec![values[0]];
    retained.extend(
        values
            .windows(2)
            .filter(|pair| pair[1] - pair[0] >= precision::p_confusion())
            .map(|pair| pair[1]),
    );
    retained
}
fn fill_bspline_grid(
    converter: &BSplineSurfaceToBezierSurface,
    grid: &Handle<HArray2<Handle<dyn GeomSurface>>>,
    u: &[f64],
    v: &[f64],
) -> Result<(), Failure> {
    let mut row = 0;
    for (i, interval) in u.windows(2).enumerate() {
        if interval[1] - interval[0] < precision::p_confusion() {
            continue;
        }
        row += 1;
        fill_bspline_row(converter, grid, v, i, row)?;
    }
    Ok(())
}
fn fill_bspline_row(
    converter: &BSplineSurfaceToBezierSurface,
    grid: &Handle<HArray2<Handle<dyn GeomSurface>>>,
    v: &[f64],
    u_index: usize,
    row: i32,
) -> Result<(), Failure> {
    let mut col = 0;
    for (j, interval) in v.windows(2).enumerate() {
        if interval[1] - interval[0] < precision::p_confusion() {
            continue;
        }
        col += 1;
        let patch = converter.patch(u_index as i32 + 1, j as i32 + 1)?.cast();
        grid.change_array2()
            .set_value(row, col, patch)
            .map_err(|e| e.into_failure("ShapeUpgrade_ConvertSurfaceToBezierBasis::Compute"))?;
    }
    Ok(())
}
fn insert_interior_knots(values: &SurfaceSplitValues, knots: &[f64]) -> Result<(), Failure> {
    let mut current = values.sequence(|s| s.iter().copied().collect::<Vec<_>>());
    let mut additions = Vec::new();
    for bounds in current.windows(2) {
        additions.extend(knots.iter().copied().filter(|k| {
            *k - bounds[0] > precision::p_confusion() && bounds[1] - *k > precision::p_confusion()
        }));
    }
    current.extend(additions);
    current.sort_by(f64::total_cmp);
    values.change_sequence(|seq| *seq = current.into_iter().collect());
    Ok(())
}
