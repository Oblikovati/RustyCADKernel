use super::*;
use crate::shape_extend::{JointValues, SurfaceGrid};
use rustycad_tkernel::{
    ncollection::{Array1, HArray2},
    precision,
};
use rustycad_tkg3d::geom::{
    GeomBezierCurve, GeomBezierSurface, GeomOffsetCurve, GeomOffsetSurface,
    GeomRectangularTrimmedSurface, GeomSurfaceOfRevolution, GeomSweptSurface,
};
type JointSlices<'a> = (&'a [f64], &'a [f64]);
type RowSpan<'a> = (usize, &'a [f64]);

struct BuildMapping<'a> {
    segments: &'a Handle<ShapeExtendCompositeSurface>,
    grid: &'a SurfaceGrid,
    joints: JointSlices<'a>,
    offset: Option<f64>,
}
struct CellBounds<'a> {
    patch: &'a Handle<dyn GeomSurface>,
    ranges: (&'a [f64], &'a [f64]),
    joints: JointSlices<'a>,
    patch_index: (usize, usize),
}

impl ShapeUpgradeConvertSurfaceToBezierBasis {
    pub(super) fn build_conversion(&self, _segment: bool) -> Result<(), Failure> {
        let surface = self.state().with(|fields| fields.surface.clone());
        let offset = source_offset(&surface)?;
        let (u, v) = self.split_values()?;
        let segments = self.segments();
        let (ju, jv) = segment_joints(&segments)?;
        let grid = result_grid(&segments, &u, &v, &ju, &jv, offset)?;
        let composite = ShapeExtendCompositeSurface::with_joints(
            &grid,
            &Array1::from_shared(&u, 1),
            &Array1::from_shared(&v, 1),
        )?;
        self.state()
            .with_mut(|fields| fields.result = Handle::new(composite));
        Ok(())
    }
    fn split_values(&self) -> Result<(Vec<f64>, Vec<f64>), Failure> {
        Ok((
            self.u_split_values()
                .sequence(|s| s.iter().copied().collect()),
            self.v_split_values()
                .sequence(|s| s.iter().copied().collect()),
        ))
    }
}
fn source_offset(surface: &Handle<dyn GeomSurface>) -> Result<Option<f64>, Failure> {
    let source = if let Some(trimmed) = surface.cast::<GeomRectangularTrimmedSurface>().get() {
        trimmed.basis_surface()
    } else {
        surface.clone()
    };
    Ok(source
        .cast::<GeomOffsetSurface>()
        .get()
        .map(|offset| offset.offset()))
}
fn segment_joints(
    segments: &Handle<ShapeExtendCompositeSurface>,
) -> Result<(Vec<f64>, Vec<f64>), Failure> {
    if segments.is_null() {
        return Err(Failure::with_type(
            "Standard_NullObject",
            "ShapeUpgrade_ConvertSurfaceToBezierBasis::Build: null Segments; expected Compute first",
            "",
        ));
    }
    Ok((
        array_values(&segments.u_joint_values())?,
        array_values(&segments.v_joint_values())?,
    ))
}
fn array_values(values: &JointValues) -> Result<Vec<f64>, Failure> {
    let array=values.get().ok_or_else(||Failure::with_type("Standard_NullObject","ShapeUpgrade_ConvertSurfaceToBezierBasis::Build: null joint values; expected computed segment grid",""))?;
    let array = array.array1();
    (array.lower()..=array.upper())
        .map(|i| {
            array
                .value(i)
                .copied()
                .map_err(|e| e.into_failure("ShapeUpgrade_ConvertSurfaceToBezierBasis::Build"))
        })
        .collect()
}
fn map_cells(
    segments: &Handle<ShapeExtendCompositeSurface>,
    grid: &SurfaceGrid,
    u: &[f64],
    v: &[f64],
    ju: &[f64],
    jv: &[f64],
    offset: Option<f64>,
) -> Result<(), Failure> {
    let mapping = BuildMapping {
        segments,
        grid,
        joints: (ju, jv),
        offset,
    };
    map_u_rows(&mapping, u, v, ju)
}
fn map_u_rows(
    mapping: &BuildMapping<'_>,
    u: &[f64],
    v: &[f64],
    joints: &[f64],
) -> Result<(), Failure> {
    let mut u_patch = 1;
    for (row, ub) in u.windows(2).enumerate() {
        advance_patch(ub[1], joints, &mut u_patch);
        mapping.map_row((row, ub), v, u_patch)?;
    }
    Ok(())
}
impl BuildMapping<'_> {
    fn map_row(&self, row_span: RowSpan<'_>, v: &[f64], u_patch: usize) -> Result<(), Failure> {
        let (row, ub) = row_span;
        let mut v_patch = 1;
        for (col, vb) in v.windows(2).enumerate() {
            self.map_column((row, col), (ub, vb), vb[1], u_patch, &mut v_patch)?;
        }
        Ok(())
    }
    fn map_column(
        &self,
        index: (usize, usize),
        bounds: (&[f64], &[f64]),
        upper: f64,
        u_patch: usize,
        v_patch: &mut usize,
    ) -> Result<(), Failure> {
        advance_patch(upper, self.joints.1, v_patch);
        let patch_index = (u_patch, *v_patch);
        self.map_cell(index, bounds, patch_index)
    }
    fn map_cell(
        &self,
        index: (usize, usize),
        parameter_ranges: (&[f64], &[f64]),
        patch_index: (usize, usize),
    ) -> Result<(), Failure> {
        let patch = self
            .segments
            .patch(patch_index.0 as i32, patch_index.1 as i32)?;
        let mapped = map_bounds(CellBounds {
            patch: &patch,
            ranges: parameter_ranges,
            joints: self.joints,
            patch_index,
        })?;
        let piece = apply_offset(segment_cell(&patch, mapped)?, self.offset)?;
        self.store_piece(index, piece)
    }
    fn store_piece(
        &self,
        index: (usize, usize),
        piece: Handle<dyn GeomSurface>,
    ) -> Result<(), Failure> {
        self.grid
            .change_array2()
            .set_value(index.0 as i32 + 1, index.1 as i32 + 1, piece)
            .map_err(|e| e.into_failure("ShapeUpgrade_ConvertSurfaceToBezierBasis::Build"))
    }
}
fn advance_patch(upper: f64, joints: &[f64], index: &mut usize) {
    while *index < joints.len() - 1 && upper - joints[*index] >= precision::p_confusion() {
        *index += 1;
    }
}
fn result_grid(
    segments: &Handle<ShapeExtendCompositeSurface>,
    u: &[f64],
    v: &[f64],
    ju: &[f64],
    jv: &[f64],
    offset: Option<f64>,
) -> Result<SurfaceGrid, Failure> {
    let grid = empty_result_grid(u.len() - 1, v.len() - 1)?;
    map_cells(segments, &grid, u, v, ju, jv, offset)?;
    Ok(grid)
}
fn empty_result_grid(rows: usize, cols: usize) -> Result<SurfaceGrid, Failure> {
    let array = HArray2::with_value(
        1,
        rows as i32,
        1,
        cols as i32,
        Handle::<dyn GeomSurface>::default(),
    )
    .map_err(|e| e.into_failure("ShapeUpgrade_ConvertSurfaceToBezierBasis::Build"))?;
    Ok(Handle::new(array))
}
fn map_bounds(input: CellBounds<'_>) -> Result<ParameterBounds, Failure> {
    let b = input.patch.bounds();
    let (u, v) = input.ranges;
    let (iu, iv) = input.patch_index;
    let (ubounds, vbounds) = input.joints;
    let (u_map, v_map) = (
        axis_map(ubounds, iu, (b.0, b.1)),
        axis_map(vbounds, iv, (b.2, b.3)),
    );
    Ok((
        map_parameter(u[0], u_map),
        map_parameter(u[1], u_map),
        map_parameter(v[0], v_map),
        map_parameter(v[1], v_map),
    ))
}
fn axis_map(joints: &[f64], index: usize, patch_bounds: (f64, f64)) -> (f64, f64, f64) {
    let first = joints[index - 1];
    let last = joints[index];
    let scale = (patch_bounds.1 - patch_bounds.0) / (last - first);
    (patch_bounds.0, first, scale)
}
fn map_parameter(value: f64, mapping: (f64, f64, f64)) -> f64 {
    mapping.0 + (value - mapping.1) * mapping.2
}
fn segment_cell(
    patch: &Handle<dyn GeomSurface>,
    bounds: ParameterBounds,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    if let Some(bezier) = patch.cast::<GeomBezierSurface>().get() {
        return segment_bezier(bezier, bounds);
    }
    segment_non_bezier(patch, bounds)
}
fn segment_non_bezier(
    patch: &Handle<dyn GeomSurface>,
    bounds: ParameterBounds,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let surface = unwrap_trimmed(patch);
    if let Some(revolution) = surface.cast::<GeomSurfaceOfRevolution>().get() {
        return segment_revolution(revolution, bounds);
    }
    segment_regular(&surface, bounds)
}
fn segment_regular(
    surface: &Handle<dyn GeomSurface>,
    bounds: ParameterBounds,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let natural = surface.bounds();
    let (u1, u2, v1, v2) = bounds;
    let p = precision::p_confusion();
    if u1 - natural.0 < p && natural.1 - u2 < p && v1 - natural.2 < p && natural.3 - v2 < p {
        return Ok(surface.clone());
    }
    Ok(Handle::new(GeomRectangularTrimmedSurface::new(
        surface,
        (u1, u2),
        (v1, v2),
        (true, true),
    )?)
    .cast())
}
fn segment_bezier(
    bezier: &GeomBezierSurface,
    bounds: ParameterBounds,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let (u1, u2, v1, v2) = bounds;
    let copy = GeomBezierSurface::copy_of(bezier)?;
    if u1 < precision::p_confusion()
        && u2 > 1. - precision::p_confusion()
        && v1 < precision::p_confusion()
        && v2 > 1. - precision::p_confusion()
    {
        return Ok(Handle::new(copy).cast());
    }
    // OCCT keeps direct parameter bounds; the older 2*u-1 workaround stays disabled.
    copy.segment(u1, u2, v1, v2)?;
    Ok(Handle::new(copy).cast())
}
fn unwrap_trimmed(patch: &Handle<dyn GeomSurface>) -> Handle<dyn GeomSurface> {
    patch
        .cast::<GeomRectangularTrimmedSurface>()
        .get()
        .map(|trimmed| trimmed.basis_surface())
        .unwrap_or_else(|| patch.clone())
}
fn segment_revolution(
    revolution: &GeomSurfaceOfRevolution,
    bounds: ParameterBounds,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let copy = GeomSurfaceOfRevolution::copy_of(revolution)?;
    segment_revolution_basis(&copy, bounds)?;
    trim_revolution(copy, bounds)
}
fn segment_revolution_basis(
    copy: &GeomSurfaceOfRevolution,
    bounds: ParameterBounds,
) -> Result<(), Failure> {
    let mut basis = copy.basis_curve();
    if let Some(offset) = basis.cast::<GeomOffsetCurve>().get() {
        basis = offset.basis_curve();
    }
    if let Some(bezier) = basis.cast::<GeomBezierCurve>().get() {
        bezier.segment(bounds.2, bounds.3)?;
    }
    Ok(())
}
fn trim_revolution(
    copy: GeomSurfaceOfRevolution,
    bounds: ParameterBounds,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let natural = copy.bounds();
    let (u1, u2, _, _) = bounds;
    if revolution_u_is_full(u1, u2, natural) {
        return Ok(Handle::new(copy).cast());
    }
    Ok(
        Handle::new(GeomRectangularTrimmedSurface::with_one_direction(
            &Handle::new(copy).cast(),
            u1,
            u2,
            true,
            true,
        )?)
        .cast(),
    )
}
fn revolution_u_is_full(first: f64, last: f64, bounds: (f64, f64, f64, f64)) -> bool {
    let p = precision::p_confusion();
    (first - bounds.0).abs() < p && (last - bounds.1).abs() < p
}
fn apply_offset(
    surface: Handle<dyn GeomSurface>,
    offset: Option<f64>,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let Some(offset) = offset else {
        return Ok(surface);
    };
    Ok(Handle::new(GeomOffsetSurface::new(&surface, offset, false)?).cast())
}
