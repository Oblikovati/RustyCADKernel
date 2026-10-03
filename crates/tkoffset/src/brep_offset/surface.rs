// Translation of BRepOffset::Surface at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::surface_analytic;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomOffsetSurface, GeomRectangularTrimmedSurface, GeomSurface};

/// Native status of an offset surface; `BRepOffsetStatus::Good as i32 == 0`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum BRepOffsetStatus {
    /// The analytic offset retains its orientation.
    Good = 0,
    /// An analytic radius crossed zero.
    Reversed = 1,
    /// An analytic radius is within the confusion tolerance of zero.
    Degenerated = 2,
    /// Public native status retained even though `Surface` never assigns it.
    Unknown = 3,
}

pub(super) type SurfaceOffset = (Handle<dyn GeomSurface>, BRepOffsetStatus);

/// Construct the native analytic offset or its general offset-surface fallback.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
/// use rustycad_tkmath::gp::Axis3;
/// use rustycad_tkoffset::brep_offset::{surface, BRepOffsetStatus};
/// let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
/// assert_eq!(surface(&plane, 2.0, false).unwrap().1, BRepOffsetStatus::Good);
/// ```
pub fn surface(
    input: &Handle<dyn GeomSurface>,
    offset: f64,
    allow_c0: bool,
) -> Result<SurfaceOffset, Failure> {
    let (candidate, status) = surface_analytic::analytic_surface(input, offset, allow_c0)?;
    if !candidate.is_null() {
        return Ok((candidate, status));
    }
    let fallback = GeomOffsetSurface::new(input, offset, allow_c0)?;
    Ok((Handle::new(fallback).cast(), status))
}

pub(super) fn trimmed_surface(
    input: &GeomRectangularTrimmedSurface,
    offset: f64,
    allow_c0: bool,
) -> Result<SurfaceOffset, Failure> {
    let (u1, u2, v1, v2) = input.bounds();
    let (basis, status) = surface(&input.basis_surface(), offset, allow_c0)?;
    let trimmed = GeomRectangularTrimmedSurface::new(&basis, (u1, u2), (v1, v2), (true, true))?;
    Ok((Handle::new(trimmed).cast(), status))
}
