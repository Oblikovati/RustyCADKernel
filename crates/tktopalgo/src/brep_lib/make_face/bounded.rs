// Rust translation of BRepLib_MakeFace::Init(S, UMin, UMax, VMin, VMax, TolDegen)
// (BRepLib_MakeFace.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The bounded `Init` of `BRepLib_MakeFace`: the face of a surface over a parameter
//! box, with one edge per finite side (degenerated when its iso curve is a point).
use super::{BRepLibMakeFace, sides};
use crate::brep_lib::errors::BRepLibFaceError;
use crate::brep_lib::same_parameter::same_parameter_shape;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomOffsetSurface, GeomRectangularTrimmedSurface, GeomSurface, GeomSurfaceOfLinearExtrusion,
    GeomSurfaceOfRevolution, SurfaceBounds,
};
use rustycad_tkmath::elclib;

impl BRepLibMakeFace {
    /// `Init(S, UMin, UMax, VMin, VMax, TolDegen)`: the face of the surface over the
    /// box: periodic bounds adjusted into the period, swapped bounds reordered (then
    /// `BRepLib_ParametersOutOfRange` beyond the natural bounds), one edge per finite
    /// side, closed sides sharing one seam edge, iso curves within `tol_degen` of a
    /// point degenerated; `maker.init_surface_bounds(&s, (0.0, 1.0, 0.0, 1.0), 1e-7)?`.
    pub fn init_surface_bounds(
        &mut self,
        surface: &Handle<dyn GeomSurface>,
        bounds: SurfaceBounds,
        tol_degen: f64,
    ) -> Result<(), Failure> {
        self.my_error = BRepLibFaceError::FaceDone;
        let basis = basis_surface(surface)?;
        let offset: bool = basis.cast::<GeomOffsetSurface>().get().is_some();
        let natural: SurfaceBounds = basis.get().map(|b| b.bounds()).unwrap_or_default();
        let s = if offset {
            offset_trimmed(surface, &basis, natural, bounds)?
        } else {
            surface.clone()
        };
        let Some(bounds) = adjusted_bounds(&s, natural, bounds)? else {
            self.my_error = BRepLibFaceError::ParametersOutOfRange;
            return Ok(());
        };
        let face = build_face(&s, natural, bounds, tol_degen)?;
        self.base.my_shape = face.clone().into_shape();
        if offset {
            // Les Isos sont Approximees a Precision::Approximation()
            // et on code Precision::Confusion() dans l'arete.
            // ==> Un petit passage dans SamePrameter pour regler les tolerances.
            same_parameter_shape(&self.base.my_shape, precision::confusion(), true)?;
        }
        self.set_done();
        Ok(())
    }
}

/// The surface, or the basis of a rectangular trimmed surface (Standard_NullObject for
/// a null surface, a crash natively).
fn basis_surface(surface: &Handle<dyn GeomSurface>) -> Result<Handle<dyn GeomSurface>, Failure> {
    if surface.is_null() {
        return Err(crate::brep_lib::errors::null_object(
            "BRepLib_MakeFace: null surface",
        ));
    }
    Ok(
        match surface.cast::<GeomRectangularTrimmedSurface>().get() {
            Some(trimmed) => trimmed.basis_surface(),
            None => surface.clone(),
        },
    )
}

/// An offset surface on an extrusion or a revolution trimmed to the requested bounds in
/// the infinite direction (both directions for an extrusion infinite in U), as natively.
fn offset_trimmed(
    surface: &Handle<dyn GeomSurface>,
    basis: &Handle<dyn GeomSurface>,
    natural: SurfaceBounds,
    (umin, umax, vmin, vmax): SurfaceBounds,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let offset = basis.cast::<GeomOffsetSurface>();
    let base = offset.get().expect("an offset surface").basis_surface();
    let (nu1, nu2, nv1, nv2) = natural;
    let trimmed = if base.cast::<GeomSurfaceOfLinearExtrusion>().get().is_some() {
        if precision::is_infinite(nu1) || precision::is_infinite(nu2) {
            GeomRectangularTrimmedSurface::new(basis, (umin, umax), (vmin, vmax), (true, true))?
        } else {
            GeomRectangularTrimmedSurface::with_one_direction(basis, vmin, vmax, false, true)?
        }
    } else if base.cast::<GeomSurfaceOfRevolution>().get().is_some()
        && (precision::is_infinite(nv1) || precision::is_infinite(nv2))
    {
        GeomRectangularTrimmedSurface::with_one_direction(basis, vmin, vmax, false, true)?
    } else {
        return Ok(surface.clone());
    };
    Ok(Handle::new(trimmed).cast())
}

/// "adjust periodical surface or reordonate": the bounds in the period of a periodic
/// direction, or swapped when reversed; None when a swapped pair leaves the natural
/// bounds by more than Precision::PConfusion().
fn adjusted_bounds(
    s: &Handle<dyn GeomSurface>,
    (nu1, nu2, nv1, nv2): SurfaceBounds,
    (umin, umax, vmin, vmax): SurfaceBounds,
) -> Result<Option<SurfaceBounds>, Failure> {
    let surface = s
        .get()
        .ok_or_else(|| crate::brep_lib::errors::null_object("BRepLib_MakeFace: null surface"))?;
    let Some((umin, umax)) = adjusted_pair(surface.is_u_periodic(), (nu1, nu2), (umin, umax))
    else {
        return Ok(None);
    };
    let Some((vmin, vmax)) = adjusted_pair(surface.is_v_periodic(), (nv1, nv2), (vmin, vmax))
    else {
        return Ok(None);
    };
    Ok(Some((umin, umax, vmin, vmax)))
}

/// One direction of [`adjusted_bounds`].
fn adjusted_pair(
    periodic: bool,
    (first, last): (f64, f64),
    (p1, p2): (f64, f64),
) -> Option<(f64, f64)> {
    let epsilon: f64 = precision::p_confusion();
    if periodic {
        return Some(elclib::adjust_periodic(first, last, epsilon, p1, p2));
    }
    if p1 <= p2 {
        return Some((p1, p2));
    }
    let (p1, p2) = (p2, p1);
    if first - p1 > epsilon || p2 - last > epsilon {
        return None;
    }
    Some((p1, p2))
}

/// The face on the surface with the edges of the box sides in one wire, two wires (a
/// band infinite in one direction) or none.
fn build_face(
    surface: &Handle<dyn GeomSurface>,
    natural: SurfaceBounds,
    bounds: SurfaceBounds,
    tol_degen: f64,
) -> Result<Face, Failure> {
    let s: &dyn GeomSurface = surface.get().expect("checked non-null");
    let sides = sides::sides(s, bounds, tol_degen)?;
    let closed = sides::closed_flags(s, natural, bounds)?;
    let vertices = sides::corners(s, &sides, bounds, closed)?;
    // make the face
    let mut face = Face::default();
    BRepBuilder::default().make_face_on_surface(&mut face, surface, precision::confusion())?;
    let edges = super::edges::side_edges(&face, &sides, &vertices, bounds, closed)?;
    super::edges::add_wires(&mut face, &sides, edges, closed)?;
    Ok(face)
}
