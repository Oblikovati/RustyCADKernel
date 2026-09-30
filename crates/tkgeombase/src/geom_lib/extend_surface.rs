// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib.cxx (ExtendKPart,
// ExtendSurfByLength) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomLib::ExtendSurfByLength`: the extension of a bounded surface along one of
//! its boundaries.
use super::extend_surface_poles::{BoundaryTangents, ExtensionPoles, SurfaceRows};
use super::live;
use crate::geom_convert::{ApproxSurface, surface_to_bspline_surface};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomBoundedSurface, GeomRectangularTrimmedSurface, GeomSurface,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::geom_abs::{Shape, SurfaceType};

const THE_MAX_EXTENSION_CONTINUITY: i32 = 3;
// Surface conversion and extension.
const THE_SURFACE_APPROX_MAX_DEGREE: i32 = 14;
const THE_SURFACE_APPROX_MAX_SEGMENTS: i32 = 16;
const THE_SURFACE_APPROX_PRECISION: i32 = 1;
const THE_MAX_SURFACE_DEGREE_RETRIES: i32 = 2;
const THE_MAX_SURFACE_WEIGHT_ATTEMPTS: i32 = 5;

/// `GeomLib::ExtendSurfByLength(Surf, Length, Cont, InU, After)`: replaces `surface`
/// by the surface extended by `length` along its U (`in_u`) or V boundary, after its
/// last parameter (`after`) or before its first one, with continuity `cont`
/// (0..=3, else nothing is done).
///
/// A trimmed plane (in U or V), cylinder or extrusion (in V) is re-trimmed on its
/// basis. Any other surface is converted to a B-spline (a B-spline input is modified
/// in place: degree raised, periodicity removed, as natively), its boundary tangents
/// scaled so that the extension chord is `length` (the transverse degree is raised
/// by 2, up to twice, when adjacent tangents reverse), then extended by
/// BSplCLib::TangExtendToConstraint (retried with a third of the scale, up to five
/// times, when a rational weight vanishes);
/// `geom_lib::extend_surf_by_length(&mut surface, 0.4, 1, true, true)?`.
pub fn extend_surf_by_length(
    surface: &mut Handle<dyn GeomBoundedSurface>,
    length: f64,
    continuity: i32,
    in_u: bool,
    after: bool,
) -> Result<(), Failure> {
    if !(0..=THE_MAX_EXTENSION_CONTINUITY).contains(&continuity) {
        return Ok(());
    }
    // Kpart ?
    let mut ts: Handle<GeomRectangularTrimmedSurface> = surface.cast();
    if extend_k_part(&mut ts, length, in_u, after)? {
        *surface = ts.cast();
        return Ok(());
    }
    // BSplineSurface format with a degree sufficient for the desired continuity.
    let bs_handle: Handle<GeomBSplineSurface> = bspline_of(surface)?;
    let bs: &GeomBSplineSurface = live(&bs_handle, "GeomLib::ExtendSurfByLength")?;
    prepare(bs, continuity, in_u)?;
    let rational: bool = bs.is_u_rational() || bs.is_v_rational();
    let (rows, tangents) = boundary_tangents(
        bs,
        BoundarySide {
            in_u,
            after,
            continuity,
            rational,
            length,
        },
    )?;
    let extension = ExtensionPoles::extend(&rows, &tangents, (continuity, after), in_u, bs)?;
    *surface = extension.surface(bs, in_u)?.cast();
    Ok(())
}

// The native `occ::down_cast<Geom_BSplineSurface>(Surface)`, or its conversion: the
// C1 approximation within Precision::Confusion(), else SurfaceToBSplineSurface.
fn bspline_of(
    surface: &Handle<dyn GeomBoundedSurface>,
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    let bs: Handle<GeomBSplineSurface> = surface.cast();
    if !bs.is_null() {
        return Ok(bs);
    }
    let a_surf: Handle<dyn GeomSurface> = surface.cast();
    let the_approx = ApproxSurface::from_surface(
        &a_surf,
        precision::confusion(),
        (Shape::C1, Shape::C1),
        (THE_SURFACE_APPROX_MAX_DEGREE, THE_SURFACE_APPROX_MAX_DEGREE),
        THE_SURFACE_APPROX_MAX_SEGMENTS,
        THE_SURFACE_APPROX_PRECISION,
    )?;
    if the_approx.has_result() {
        return Ok(the_approx.surface());
    }
    surface_to_bspline_surface(&a_surf)
}

// The degree raised for the continuity and the periodicity removed in the extension
// direction (if BS was periodic in that direction, it will no longer be).
fn prepare(bs: &GeomBSplineSurface, continuity: i32, in_u: bool) -> Result<(), Failure> {
    if in_u && bs.u_degree() < continuity + 1 {
        bs.increase_degree(continuity + 1, bs.v_degree())?;
    }
    if !in_u && bs.v_degree() < continuity + 1 {
        bs.increase_degree(bs.u_degree(), continuity + 1)?;
    }
    if (in_u && bs.is_u_periodic()) || (!in_u && bs.is_v_periodic()) {
        let (u0, u1, v0, v1) = bs.bounds();
        bs.segment(
            (u0, u1),
            (v0, v1),
            (precision::p_confusion(), precision::p_confusion()),
        )?;
    }
    Ok(())
}

/// The side of the extension and what the boundary tangents depend on.
#[derive(Clone, Copy)]
pub(super) struct BoundarySide {
    pub in_u: bool,
    pub after: bool,
    pub continuity: i32,
    pub rational: bool,
    pub length: f64,
}

// The surface as a non-rational B-spline curve of dimension 3 or 4 times the pole
// count across (a rational pole (x, y, z) of weight w becomes (x, y, z, w)), and its
// point and tangents at the boundary; the transverse degree is raised by 2 and the
// whole retried (up to twice) when adjacent boundary tangents reverse.
fn boundary_tangents(
    bs: &GeomBSplineSurface,
    side: BoundarySide,
) -> Result<(SurfaceRows, BoundaryTangents), Failure> {
    let mut kount: i32 = 0;
    loop {
        let rows = SurfaceRows::of(bs, side.in_u, side.rational)?;
        let tangents = BoundaryTangents::at(&rows, side)?;
        kount += 1;
        if tangents.ok || kount > THE_MAX_SURFACE_DEGREE_RETRIES {
            return Ok((rows, tangents));
        }
        // Increase the degree of the border iso to bring the surface poles closer, and
        // retry.
        if side.in_u {
            bs.increase_degree(bs.u_degree(), bs.v_degree() + 2)?;
        } else {
            bs.increase_degree(bs.u_degree() + 2, bs.v_degree())?;
        }
    }
}

// The retry count of the rational extension, shared with extend_surface_poles.
pub(super) const fn weight_attempts() -> i32 {
    THE_MAX_SURFACE_WEIGHT_ATTEMPTS
}

// `ExtendKPart`: a trimmed plane extended in U or V, or a trimmed cylinder or
// extrusion in V, is the basis re-trimmed; false for any other surface.
fn extend_k_part(
    surface: &mut Handle<GeomRectangularTrimmedSurface>,
    length: f64,
    in_u: bool,
    after: bool,
) -> Result<bool, Failure> {
    let Some(trimmed) = surface.get() else {
        return Ok(false);
    };
    let support: Handle<dyn GeomSurface> = trimmed.basis_surface();
    let (mut uf, mut ul, mut vf, mut vl) = trimmed.bounds();
    let adaptor = GeomAdaptorSurface::with_surface(&surface.cast())?;
    let kind: SurfaceType = adaptor.get_type()?;
    let extensible: bool = if in_u {
        kind == SurfaceType::Plane
    } else {
        matches!(
            kind,
            SurfaceType::Plane | SurfaceType::Cylinder | SurfaceType::SurfaceOfExtrusion
        )
    };
    if !extensible {
        return Ok(false);
    }
    match (in_u, after) {
        (true, true) => ul += length,
        (true, false) => uf -= length,
        (false, true) => vl += length,
        (false, false) => vf -= length,
    }
    *surface = Handle::new(GeomRectangularTrimmedSurface::new(
        &support,
        (uf, ul),
        (vf, vl),
        (true, true),
    )?);
    Ok(true)
}

/// Copy flat knots as knots with their multiplicities: the knots and multiplicities
/// of the extended direction (`Usize` or `Vsize` one more than the surface's), the
/// last multiplicity forced to Degree + 1 ("petite verrue" useful when the continuity
/// is not reached).
pub(super) fn extended_knots(
    flat: &[f64],
    size: i32,
    degree: i32,
) -> Result<(Array1<'static, f64>, Array1<'static, i32>), Failure> {
    let mut fk_res: Array1<'static, f64> = one_based(flat.len() as i32)?;
    fk_res.as_mut_slice().copy_from_slice(flat);
    let mut knots: Array1<'static, f64> = one_based(size)?;
    let mut mults: Array1<'static, i32> = one_based(size)?;
    bsplclib::knots(&fk_res, &mut knots, &mut mults, false)?;
    mults[size] = degree + 1;
    Ok((knots, mults))
}
