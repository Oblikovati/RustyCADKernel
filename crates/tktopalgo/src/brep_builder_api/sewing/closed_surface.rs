// Rust translation of the static IsClosedByIsos and of IsUClosedSurface,
// IsVClosedSurface and IsMergedClosed (BRepBuilderAPI_Sewing.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The closedness of a face's surface along an edge, and whether two sections on a
//! closed surface may be merged across its seam.
use super::BRepBuilderAPISewing;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::geom::{GeomOffsetSurface, GeomRectangularTrimmedSurface, GeomSurface};
use rustycad_tkgeombase::bnd_lib::Add2dCurve;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::toploc::Location;

/// The direction of a closedness test: `IsUClosedSurface` compares V-isolines,
/// `IsVClosedSurface` U-isolines.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ClosedDirection {
    U,
    V,
}

/// The first and last points of the p-curve, the ends clamped to the curve range when
/// it is not periodic.
fn pcurve_ends(c: &dyn Geom2dCurve, f2d: f64, l2d: f64) -> Result<(Point2, Point2), Failure> {
    if c.is_periodic() {
        return Ok((c.value(f2d)?, c.value(l2d)?));
    }
    let first = c.value(std_max(f2d, c.first_parameter()))?;
    Ok((first, c.value(std_min(l2d, c.last_parameter()))?))
}

/// Whether an isoline returns to its start: its end points are closer than its start
/// is to its middle point, by more than Precision::Confusion().
fn isoline_closed(iso: &Handle<dyn rustycad_tkg3d::geom::GeomCurve>) -> Result<bool, Failure> {
    let c = iso
        .get()
        .ok_or_else(|| super::failures::null_object("IsClosedByIsos: null isoline"))?;
    let (af, al) = (c.first_parameter(), c.last_parameter());
    let p1: Point3 = c.d0(af)?;
    let pm: Point3 = c.d0((af + al) * 0.5)?;
    let p2: Point3 = c.d0(al)?;
    Ok((p1.xyz() - p2.xyz()).modulus() < (p1.xyz() - pm.xyz()).modulus() - precision::confusion())
}

/// The static `IsClosedByIsos(thesurf, acrv2d, f2d, l2d, isUIsos)`: the isolines of the
/// surface through both ends of the p-curve are both closed.
fn is_closed_by_isos(
    surface: &dyn GeomSurface,
    c: &dyn Geom2dCurve,
    (f2d, l2d): (f64, f64),
    u_isos: bool,
) -> Result<bool, Failure> {
    let (psurf1, psurf2) = pcurve_ends(c, f2d, l2d)?;
    let (c1, c2) = if u_isos {
        (surface.u_iso(psurf1.x())?, surface.u_iso(psurf2.x())?)
    } else {
        (surface.v_iso(psurf1.y())?, surface.v_iso(psurf2.y())?)
    };
    let closed1 = isoline_closed(&c1)?;
    let closed2 = isoline_closed(&c2)?;
    Ok(closed1 && closed2)
}

/// The basis of a rectangular trimmed or offset surface, which the tests recurse on.
fn basis_of(surface: &Handle<dyn GeomSurface>) -> Result<Option<Handle<dyn GeomSurface>>, Failure> {
    if let Some(trimmed) = surface.cast::<GeomRectangularTrimmedSurface>().get() {
        return Ok(Some(trimmed.basis_surface()));
    }
    if let Some(offset) = surface.cast::<GeomOffsetSurface>().get() {
        return Ok(Some(offset.basis_surface()));
    }
    Ok(None)
}

impl BRepBuilderAPISewing {
    /// `IsUClosedSurface(surf, theEdge, theloc)`: a rectangular trimmed or offset surface
    /// answers for its basis; another surface is U-closed, or has closed V-isolines at
    /// both ends of the edge's p-curve on it.
    pub(crate) fn is_u_closed_surface(
        &self,
        surf: &Handle<dyn GeomSurface>,
        edge: &Shape,
        location: &Location,
    ) -> Result<bool, Failure> {
        closed_surface(surf, edge, location, ClosedDirection::U)
    }

    /// `IsVClosedSurface(surf, theEdge, theloc)`: as `IsUClosedSurface` for V, with the
    /// U-isolines.
    pub(crate) fn is_v_closed_surface(
        &self,
        surf: &Handle<dyn GeomSurface>,
        edge: &Shape,
        location: &Location,
    ) -> Result<bool, Failure> {
        closed_surface(surf, edge, location, ClosedDirection::V)
    }

    /// Either closedness test on the surface of a face.
    pub(crate) fn is_closed_surface(
        &self,
        surf: &Handle<dyn GeomSurface>,
        edge: &Shape,
        location: &Location,
    ) -> Result<bool, Failure> {
        Ok(self.is_u_closed_surface(surf, edge, location)?
            || self.is_v_closed_surface(surf, edge, location)?)
    }

    /// `IsMergedClosed(Edge1, Edge2, face)`: on a closed surface, two sections that
    /// overlap along the closed direction's other parameter and are nearer across the
    /// seam than inside the domain may be merged.
    pub(crate) fn is_merged_closed(
        &self,
        edge1: &Edge,
        edge2: &Edge,
        face: &Face,
    ) -> Result<bool, Failure> {
        let (surf, loc) = BRepTool::surface_with_location(face)?;
        let u_closed = self.is_u_closed_surface(&surf, edge1, &loc)?;
        let v_closed = self.is_v_closed_surface(&surf, edge1, &loc)?;
        if !u_closed && !v_closed {
            return Ok(false);
        }
        let c1 = BRepTool::curve_on_face(edge1, face)?;
        let c2 = BRepTool::curve_on_face(edge2, face)?;
        if c1.pcurve.is_null() || c2.pcurve.is_null() {
            return Ok(false);
        }
        let b1 = pcurve_box(&c1.pcurve, c1.first, c1.last)?;
        let b2 = pcurve_box(&c2.pcurve, c2.first, c2.last)?;
        let s = surf
            .get()
            .ok_or_else(|| super::failures::null_object("IsMergedClosed: null surface"))?;
        let bounds = s.bounds();
        if u_closed && b1.v_long && b2.v_long && across_seam(&b1, &b2, (bounds.0, bounds.1), true) {
            return Ok(true);
        }
        Ok(
            v_closed
                && b1.u_long
                && b2.u_long
                && across_seam(&b1, &b2, (bounds.2, bounds.3), false),
        )
    }
}

/// The recursion of `IsUClosedSurface` and `IsVClosedSurface`.
fn closed_surface(
    surf: &Handle<dyn GeomSurface>,
    edge: &Shape,
    location: &Location,
    direction: ClosedDirection,
) -> Result<bool, Failure> {
    if let Some(basis) = basis_of(surf)? {
        return closed_surface(&basis, edge, location, direction);
    }
    let s = surf
        .get()
        .ok_or_else(|| super::failures::null_object("IsUClosedSurface: null surface"))?;
    let closed = match direction {
        ClosedDirection::U => s.is_u_closed()?,
        ClosedDirection::V => s.is_v_closed()?,
    };
    if closed {
        return Ok(true);
    }
    let found = BRepTool::curve_on_surface(&topods::edge(edge)?, surf, location)?;
    let Some(c) = found.pcurve.get() else {
        return Ok(false);
    };
    is_closed_by_isos(
        s,
        c,
        (found.first, found.last),
        direction == ClosedDirection::V,
    )
}

/// The 2d box of a p-curve and which of its sides is the longer.
struct PcurveBox {
    u_min: f64,
    v_min: f64,
    u_max: f64,
    v_max: f64,
    u_long: bool,
    v_long: bool,
}

/// `BndLib_Add2dCurve::Add(Geom2dAdaptor_Curve(C), first, last, PConfusion, B)`.
fn pcurve_box(c: &Handle<dyn Geom2dCurve>, first: f64, last: f64) -> Result<PcurveBox, Failure> {
    let mut bounds = BoundingBox2::default();
    let adaptor = Geom2dAdaptorCurve::with_curve(c)?;
    Add2dCurve::add_adaptor_range(&adaptor, first, last, precision::p_confusion(), &mut bounds)?;
    let [u_min, v_min, u_max, v_max] = bounds.get()?;
    let (du, dv) = (u_max - u_min, v_max - v_min);
    Ok(PcurveBox {
        u_min,
        v_min,
        u_max,
        v_max,
        u_long: dv <= du,
        v_long: du <= dv,
    })
}

/// The test of one closed direction: the boxes overlap along the other parameter, and
/// the gap across the seam (`period` bounds) is at most the gap inside the domain.
fn across_seam(b1: &PcurveBox, b2: &PcurveBox, (s_min, s_max): (f64, f64), u_closed: bool) -> bool {
    let (along1, along2, across1, across2) = if u_closed {
        (
            (b1.v_min, b1.v_max),
            (b2.v_min, b2.v_max),
            (b1.u_min, b1.u_max),
            (b2.u_min, b2.u_max),
        )
    } else {
        (
            (b1.u_min, b1.u_max),
            (b2.u_min, b2.u_max),
            (b1.v_min, b1.v_max),
            (b2.v_min, b2.v_max),
        )
    };
    let dist = std_max(along2.0 - along1.1, along1.0 - along2.1);
    if dist >= 0.0 {
        return false;
    }
    let dist_inner = std_max(across2.0 - across1.1, across1.0 - across2.1);
    let dist_outer = (s_max - s_min) - std_max(across2.1 - across1.0, across1.1 - across2.0);
    dist_outer <= dist_inner
}
