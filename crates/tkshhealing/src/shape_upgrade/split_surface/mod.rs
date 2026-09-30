//! Native surface splitting at OCCT 3d097a0328e71b8.
use crate::shape_extend::{
    ShapeExtend, ShapeExtendCompositeSurface, ShapeExtendParametrisation, ShapeExtendStatus,
    SurfaceGrid,
};
use rustycad_tkernel::{
    impl_transient, impl_transient_interface,
    ncollection::{Array1, HArray2, HSequence},
    precision,
    standard::{Failure, Handle, SharedState, StandardTransient, Transient},
};
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomGeometry, GeomOffsetSurface,
    GeomRectangularTrimmedSurface, GeomSurface,
};
mod build;
mod initialize;
mod knots;
mod pieces;
mod swept;
/// Shared native real sequence; `Handle::new(HSequence::<f64>::default())`.
pub type SurfaceSplitValues = Handle<HSequence<f64>>;
/// Protected state available to specialized split criteria.
#[derive(Default)]
pub struct SplitSurfaceFields {
    /// U splitting parameters, including bounds.
    pub u_values: SurfaceSplitValues,
    /// V splitting parameters, including bounds.
    pub v_values: SurfaceSplitValues,
    /// Native resulting row count.
    pub rows: i32,
    /// Native resulting column count.
    pub columns: i32,
    /// Supporting surface (retained, not copied by Init).
    pub surface: Handle<dyn GeomSurface>,
    /// ShapeExtend status bits.
    pub status: i32,
    /// Shared resulting composite surface.
    pub result: Handle<ShapeExtendCompositeSurface>,
    /// Requested area, retained by natural-bound Init.
    pub area: f64,
    /// Mid-isoparametric U length, set only for nonzero requested area.
    pub u_size: f64,
    /// Mid-isoparametric V length, set only for nonzero requested area.
    pub v_size: f64,
}
/// Protected state shared by surface splitter implementations; `SplitSurfaceData::default()`.
#[derive(Default)]
pub struct SplitSurfaceData {
    fields: SharedState<SplitSurfaceFields>,
}
impl SplitSurfaceData {
    /// Read protected state; `state.with(|fields|fields.rows)`.
    pub fn with<R>(&self, read: impl FnOnce(&SplitSurfaceFields) -> R) -> R {
        read(&self.fields.read())
    }
    /// Change protected state for a derived criterion; `state.with_mut(|fields|fields.status=0)`.
    pub fn with_mut<R>(&self, write: impl FnOnce(&mut SplitSurfaceFields) -> R) -> R {
        write(&mut self.fields.write())
    }
    fn add_status(&self, status: ShapeExtendStatus) {
        self.with_mut(|fields| fields.status |= ShapeExtend::encode_status(status));
    }
    fn result(&self) -> Handle<ShapeExtendCompositeSurface> {
        self.with(|fields| fields.result.clone())
    }
    fn values(&self, u: bool) -> SurfaceSplitValues {
        self.with(|fields| {
            if u {
                fields.u_values.clone()
            } else {
                fields.v_values.clone()
            }
        })
    }
}
/// Virtual splitting criterion and common state; `splitter.perform(true)?`.
pub trait SplitSurface: Transient {
    /// Protected native state; `splitter.split_surface_data()`.
    fn split_surface_data(&self) -> &SplitSurfaceData;
    /// Initialize to natural surface bounds; `splitter.init_surface(&surface)?`.
    fn init_surface(&self, surface: &Handle<dyn GeomSurface>) -> Result<(), Failure> {
        self.split_surface_data().init_natural(surface)
    }
    /// Initialize and clamp explicit bounds, measuring isoparametric lengths for nonzero area.
    /// Example: `splitter.init_surface_bounds(&surface,(0.,1.,0.,1.),0.)?`.
    fn init_surface_bounds(
        &self,
        surface: &Handle<dyn GeomSurface>,
        bounds: (f64, f64, f64, f64),
        area: f64,
    ) -> Result<(), Failure> {
        self.split_surface_data().init_bounds(surface, bounds, area)
    }
    /// Merge sorted U split parameters; `splitter.set_u_split_values(&values)?`.
    fn set_u_split_values(&self, values: &SurfaceSplitValues) -> Result<(), Failure> {
        self.split_surface_data().merge_values(true, values)
    }
    /// Merge sorted V split parameters; `splitter.set_v_split_values(&values)?`.
    fn set_v_split_values(&self, values: &SurfaceSplitValues) -> Result<(), Failure> {
        self.split_surface_data().merge_values(false, values)
    }
    /// Build geometric patches with native wrapper and segmentation behavior; `splitter.build(true)?`.
    fn build(&self, segment: bool) -> Result<(), Failure> {
        self.split_surface_data().build_surface(segment)
    }
    /// The base criterion resets status; derived criteria may add split values.
    /// Example: `splitter.compute(true)?`.
    fn compute(&self, _segment: bool) -> Result<(), Failure> {
        self.split_surface_data()
            .with_mut(|fields| fields.status = 0);
        Ok(())
    }
    /// Dispatch Compute and Build in order; `splitter.perform(true)?`.
    fn perform(&self, segment: bool) -> Result<(), Failure> {
        self.compute(segment)?;
        self.build(segment)
    }
    /// Shared U split values; `splitter.u_split_values()`.
    fn u_split_values(&self) -> SurfaceSplitValues {
        self.split_surface_data().values(true)
    }
    /// Shared V split values; `splitter.v_split_values()`.
    fn v_split_values(&self) -> SurfaceSplitValues {
        self.split_surface_data().values(false)
    }
    /// Decode native status bits; `splitter.status_flag(ShapeExtendStatus::Done)`.
    fn status_flag(&self, status: ShapeExtendStatus) -> bool {
        self.split_surface_data()
            .with(|fields| ShapeExtend::decode_status(fields.status, status))
    }
    /// Shared result composite; `splitter.result_surfaces()`.
    fn result_surfaces(&self) -> Handle<ShapeExtendCompositeSurface> {
        self.split_surface_data().result()
    }
}
impl_transient_interface!(
    dyn SplitSurface,
    "ShapeUpgrade_SplitSurface",
    StandardTransient
);
/// Base surface splitting implementation; `ShapeUpgradeSplitSurface::new()`.
#[derive(Default)]
pub struct ShapeUpgradeSplitSurface {
    parent: StandardTransient,
    state: SplitSurfaceData,
}
impl_transient!(ShapeUpgradeSplitSurface,"ShapeUpgrade_SplitSurface",StandardTransient,parent,interfaces:[dyn SplitSurface]);
impl SplitSurface for ShapeUpgradeSplitSurface {
    fn split_surface_data(&self) -> &SplitSurfaceData {
        &self.state
    }
}
impl ShapeUpgradeSplitSurface {
    /// Create an uninitialized surface splitter; `ShapeUpgradeSplitSurface::new()`.
    pub fn new() -> Self {
        Self::default()
    }
}
fn new_grid(rows: i32, columns: i32) -> Result<SurfaceGrid, Failure> {
    let grid = HArray2::with_value(1, rows, 1, columns, Handle::default())
        .map_err(|e| e.into_failure("ShapeUpgrade_SplitSurface::Build"))?;
    Ok(Handle::new(grid))
}
fn grid_put(
    grid: &SurfaceGrid,
    row: i32,
    column: i32,
    surface: Handle<dyn GeomSurface>,
) -> Result<(), Failure> {
    grid.change_array2()
        .set_value(row, column, surface)
        .map_err(|e| e.into_failure("ShapeUpgrade_SplitSurface::Build"))
}
fn split_snapshot(values: &SurfaceSplitValues) -> Result<Vec<f64>, Failure> {
    let values = values.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "ShapeUpgrade_SplitSurface: null values; expected initialized split sequence",
            "",
        )
    })?;
    Ok(values.sequence(|s| s.iter().copied().collect()))
}
fn trimmed_surface(
    surface: &Handle<dyn GeomSurface>,
    bounds: (f64, f64, f64, f64),
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let (u1, u2, v1, v2) = bounds;
    Ok(Handle::new(GeomRectangularTrimmedSurface::new(
        surface,
        (u1, u2),
        (v1, v2),
        (true, true),
    )?)
    .cast())
}

fn split_endpoints(values: &SurfaceSplitValues) -> Result<(f64, f64), Failure> {
    let values = values.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "ShapeUpgrade_SplitSurface: null sequence; expected initialized values",
            "",
        )
    })?;
    values.sequence(|s| Ok((*s.value(1)?, *s.value(s.len())?)))
}
