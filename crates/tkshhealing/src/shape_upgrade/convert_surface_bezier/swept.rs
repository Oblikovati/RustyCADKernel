use super::*;
use crate::shape_extend::{ShapeExtendCompositeSurface, ShapeExtendStatus};
use crate::shape_upgrade::{ShapeUpgradeConvertCurve3dToBezier, SplitCurve, SurfaceSplitValues};
use rustycad_tkernel::{
    ncollection::{Array1, Array2, HArray2},
    precision,
};
use rustycad_tkg3d::geom::{
    GeomBezierCurve, GeomBezierSurface, GeomCurve, GeomOffsetCurve, GeomRectangularTrimmedSurface,
    GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution, GeomSweptSurface, GeomTrimmedCurve,
};
use rustycad_tkmath::gp::{Point3, Transform3};
type RevolutionCurves = (Vec<Handle<dyn GeomCurve>>, Vec<f64>, Vec<f64>, bool);

impl ShapeUpgradeConvertSurfaceToBezierBasis {
    pub(super) fn compute_swept(
        &self,
        surface: &Handle<dyn GeomSurface>,
        bounds: (f64, f64, f64, f64),
    ) -> Result<Option<()>, Failure> {
        if self.compute_revolution_if(surface, bounds)? {
            return Ok(Some(()));
        }
        if self.compute_extrusion_if(surface, bounds)? {
            return Ok(Some(()));
        }
        Ok(None)
    }
    fn compute_revolution_if(
        &self,
        surface: &Handle<dyn GeomSurface>,
        bounds: ParameterBounds,
    ) -> Result<bool, Failure> {
        let converter = surface.cast::<GeomSurfaceOfRevolution>();
        let Some(revolution) = converter.get().filter(|_| self.modes().revolution) else {
            return Ok(false);
        };
        self.compute_revolution(surface, revolution, bounds)?;
        Ok(true)
    }
    fn compute_extrusion_if(
        &self,
        surface: &Handle<dyn GeomSurface>,
        bounds: ParameterBounds,
    ) -> Result<bool, Failure> {
        let converter = surface.cast::<GeomSurfaceOfLinearExtrusion>();
        let Some(extrusion) = converter.get().filter(|_| self.modes().extrusion) else {
            return Ok(false);
        };
        self.compute_extrusion(extrusion, bounds)?;
        Ok(true)
    }
    fn compute_revolution(
        &self,
        surface: &Handle<dyn GeomSurface>,
        revolution: &GeomSurfaceOfRevolution,
        bounds: (f64, f64, f64, f64),
    ) -> Result<(), Failure> {
        let source = unwrap_trimmed_curve(&revolution.basis_curve());
        let (curves, params, split_values, done) =
            converted_revolution_curves(&source, (bounds.2, bounds.3))?;
        let patches =
            build_revolution_patches(&curves, revolution.axis(), surface.bounds(), bounds)?;
        self.set_segments(make_composite(&patches, &[bounds.0, bounds.1], &params)?);
        insert_split_knots(&self.v_split_values(), &split_values, false, bounds.2)?;
        self.set_status(if done {
            ShapeExtendStatus::Done1
        } else {
            ShapeExtendStatus::Ok
        });
        Ok(())
    }
    fn compute_extrusion(
        &self,
        extrusion: &GeomSurfaceOfLinearExtrusion,
        bounds: (f64, f64, f64, f64),
    ) -> Result<(), Failure> {
        let source = extrusion.basis_curve();
        let converter = curve_converter(&source, (bounds.0, bounds.1))?;
        let curves = curve_outputs(&converter.curves())?;
        let params = sequence_values(&converter.split_params());
        let patches = extrusion_patches(&curves, extrusion, bounds)?;
        self.set_segments(make_composite(&patches, &params, &[bounds.2, bounds.3])?);
        let split = sequence_values(&converter.split_values());
        insert_split_knots(&self.u_split_values(), &split, true, bounds.0)?;
        self.set_status(ShapeExtendStatus::Done1);
        Ok(())
    }
}

fn converted_revolution_curves(
    source: &Handle<dyn GeomCurve>,
    range: (f64, f64),
) -> Result<RevolutionCurves, Failure> {
    let offset_handle = source.cast::<GeomOffsetCurve>();
    let offset = offset_handle.get();
    let conversion_source = offset
        .as_ref()
        .map(|curve| curve.basis_curve())
        .unwrap_or_else(|| source.clone());
    let converter = curve_converter(&conversion_source, range)?;
    let done = converter.status_flag(ShapeExtendStatus::Done);
    let params = sequence_values(&converter.split_params());
    let split = sequence_values(&converter.split_values());
    let mut curves = curve_outputs(&converter.curves())?;
    if let Some(offset) = offset {
        curves = offset_curves(&curves, offset.offset(), offset.direction())?;
    }
    Ok((curves, params, split, done))
}
fn curve_converter(
    source: &Handle<dyn GeomCurve>,
    range: (f64, f64),
) -> Result<ShapeUpgradeConvertCurve3dToBezier, Failure> {
    let converter = ShapeUpgradeConvertCurve3dToBezier::new();
    converter.init_curve_range(source, range.0, range.1)?;
    converter.perform(true)?;
    Ok(converter)
}
fn unwrap_trimmed_curve(source: &Handle<dyn GeomCurve>) -> Handle<dyn GeomCurve> {
    source
        .cast::<GeomTrimmedCurve>()
        .get()
        .map(|curve| curve.basis_curve())
        .unwrap_or_else(|| source.clone())
}
fn curve_outputs(
    curves: &crate::shape_upgrade::SplitCurves3d,
) -> Result<Vec<Handle<dyn GeomCurve>>, Failure> {
    let array = curves.get().ok_or_else(null_curve_array)?.array1();
    (array.lower()..=array.upper())
        .map(|i| {
            array
                .value(i)
                .cloned()
                .map_err(|e| e.into_failure("ShapeUpgrade_ConvertSurfaceToBezierBasis::Compute"))
        })
        .collect()
}
fn offset_curves(
    curves: &[Handle<dyn GeomCurve>],
    offset: f64,
    direction: rustycad_tkmath::gp::Direction3,
) -> Result<Vec<Handle<dyn GeomCurve>>, Failure> {
    curves
        .iter()
        .map(|curve| Ok(Handle::new(GeomOffsetCurve::new(curve, offset, direction, false)?).cast()))
        .collect()
}
fn sequence_values(values: &Handle<rustycad_tkernel::ncollection::HSequence<f64>>) -> Vec<f64> {
    values.sequence(|sequence| sequence.iter().copied().collect())
}
fn build_revolution_patches(
    curves: &[Handle<dyn GeomCurve>],
    axis: rustycad_tkmath::gp::Axis1,
    natural: (f64, f64, f64, f64),
    bounds: (f64, f64, f64, f64),
) -> Result<Vec<Handle<dyn GeomSurface>>, Failure> {
    curves
        .iter()
        .map(|curve| revolution_patch(curve, axis, natural, bounds))
        .collect()
}
fn revolution_patch(
    curve: &Handle<dyn GeomCurve>,
    axis: rustycad_tkmath::gp::Axis1,
    natural: ParameterBounds,
    bounds: ParameterBounds,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let patch: Handle<dyn GeomSurface> =
        Handle::new(GeomSurfaceOfRevolution::new(curve, axis)?).cast();
    if bounds.0 - natural.0 < precision::p_confusion()
        && natural.1 - bounds.1 < precision::p_confusion()
    {
        return Ok(patch);
    }
    Ok(
        Handle::new(GeomRectangularTrimmedSurface::with_one_direction(
            &patch, bounds.0, bounds.1, true, true,
        )?)
        .cast(),
    )
}
fn extrusion_patches(
    curves: &[Handle<dyn GeomCurve>],
    extrusion: &GeomSurfaceOfLinearExtrusion,
    bounds: (f64, f64, f64, f64),
) -> Result<Vec<Handle<dyn GeomSurface>>, Failure> {
    let start = extrusion.value(bounds.0, 0.)?;
    let first = extrusion.value(bounds.0, bounds.2)?;
    let last = extrusion.value(bounds.0, bounds.3)?;
    curves
        .iter()
        .map(|curve| extrusion_patch(curve, start, first, last))
        .collect()
}
fn extrusion_patch(
    curve: &Handle<dyn GeomCurve>,
    start: Point3,
    first: Point3,
    last: Point3,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let bezier_handle = curve.cast::<GeomBezierCurve>();
    let bezier=bezier_handle.get().ok_or_else(||Failure::with_type("Standard_TypeMismatch","ShapeUpgrade_ConvertSurfaceToBezierBasis::Compute: converted extrusion curve is not Bezier; expected Geom_BezierCurve",""))?;
    let poles = bezier.poles();
    let values = extrusion_poles(&poles, start, first, last)?;
    let array = Array2::from_shared(&values, 1, poles.len() as i32, 1, 2)
        .map_err(|e| e.into_failure("ShapeUpgrade_ConvertSurfaceToBezierBasis::Compute"))?;
    Ok(Handle::new(GeomBezierSurface::new(&array)?).cast())
}
fn extrusion_poles(
    poles: &Array1<'_, Point3>,
    start: Point3,
    first: Point3,
    last: Point3,
) -> Result<Vec<Point3>, Failure> {
    let mut first_shift = Transform3::default();
    first_shift.set_translation_between(start, first);
    let mut last_shift = Transform3::default();
    last_shift.set_translation_between(start, last);
    let pairs = (poles.lower()..=poles.upper())
        .map(|i| pole_pair(poles, i, first_shift, last_shift))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(pairs.into_iter().flatten().collect())
}
fn pole_pair(
    poles: &Array1<'_, Point3>,
    index: i32,
    first: Transform3,
    last: Transform3,
) -> Result<[Point3; 2], Failure> {
    let pole = *poles
        .value(index)
        .map_err(|e| e.into_failure("ShapeUpgrade_ConvertSurfaceToBezierBasis::Compute"))?;
    Ok([pole.transformed(first), pole.transformed(last)])
}
fn make_composite(
    patches: &[Handle<dyn GeomSurface>],
    u: &[f64],
    v: &[f64],
) -> Result<ShapeExtendCompositeSurface, Failure> {
    let grid = patch_grid(patches, u.len() > 2, v.len() > 2)?;
    ShapeExtendCompositeSurface::with_joints(
        &grid,
        &Array1::from_shared(u, 1),
        &Array1::from_shared(v, 1),
    )
}
fn patch_grid(
    patches: &[Handle<dyn GeomSurface>],
    rows: bool,
    cols: bool,
) -> Result<crate::shape_extend::SurfaceGrid, Failure> {
    let grid = empty_patch_grid(patches, rows, cols)?;
    fill_patch_grid(&grid, patches, rows)?;
    Ok(grid)
}
fn empty_patch_grid(
    patches: &[Handle<dyn GeomSurface>],
    rows: bool,
    cols: bool,
) -> Result<crate::shape_extend::SurfaceGrid, Failure> {
    let (u, v) = patch_dimensions(patches.len(), rows, cols);
    let array = HArray2::with_value(1, u, 1, v, Handle::<dyn GeomSurface>::default())
        .map_err(|e| e.into_failure("ShapeUpgrade_ConvertSurfaceToBezierBasis::Compute"))?;
    Ok(Handle::new(array))
}
fn patch_dimensions(count: usize, rows: bool, cols: bool) -> (i32, i32) {
    if rows {
        return (count as i32, 1);
    }
    if cols {
        return (1, count as i32);
    }
    (1, 1)
}
fn fill_patch_grid(
    grid: &crate::shape_extend::SurfaceGrid,
    patches: &[Handle<dyn GeomSurface>],
    rows: bool,
) -> Result<(), Failure> {
    for (i, patch) in patches.iter().enumerate() {
        set_patch_cell(grid, i, patch, rows)?;
    }
    Ok(())
}
fn set_patch_cell(
    grid: &crate::shape_extend::SurfaceGrid,
    index: usize,
    patch: &Handle<dyn GeomSurface>,
    rows: bool,
) -> Result<(), Failure> {
    let (row, col) = patch_cell(index, rows);
    grid.change_array2()
        .set_value(row, col, patch.clone())
        .map_err(|e| e.into_failure("ShapeUpgrade_ConvertSurfaceToBezierBasis::Compute"))
}
fn patch_cell(index: usize, rows: bool) -> (i32, i32) {
    if rows {
        return (index as i32 + 1, 1);
    }
    (1, index as i32 + 1)
}
fn insert_split_knots(
    values: &SurfaceSplitValues,
    knots: &[f64],
    extrusion: bool,
    first: f64,
) -> Result<(), Failure> {
    let mut result = values.sequence(|sequence| sequence.iter().copied().collect::<Vec<_>>());
    let additions = split_insertions(&result, knots, extrusion, first);
    result.extend(additions);
    result.sort_by(f64::total_cmp);
    values.change_sequence(|sequence| *sequence = result.into_iter().collect());
    Ok(())
}
fn split_insertions(values: &[f64], knots: &[f64], extrusion: bool, first: f64) -> Vec<f64> {
    values
        .windows(2)
        .flat_map(|interval| {
            knots
                .iter()
                .copied()
                .filter(|knot| valid_insertion(*knot, interval, extrusion, first))
        })
        .collect()
}
fn valid_insertion(knot: f64, interval: &[f64], extrusion: bool, first: f64) -> bool {
    let skip = if extrusion {
        knot + first <= precision::p_confusion()
    } else {
        knot - interval[0] <= precision::p_confusion()
    };
    !skip && interval[1] - knot > precision::p_confusion()
}
fn null_curve_array() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "ShapeUpgrade_ConvertSurfaceToBezierBasis::Compute: null curve result; expected initialized curve conversion",
        "",
    )
}
