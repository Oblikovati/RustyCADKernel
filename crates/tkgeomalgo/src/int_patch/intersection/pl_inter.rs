// Copyright (c) 1993-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of the static helpers FUN_GetUiso, FUN_GetViso and FUN_PL_Intersection
// of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_Intersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `FUN_PL_Intersection`: two infinite surfaces whose isolines of one direction are
//! parallel straight lines (extrusions, planes, cylinders...) intersect along lines of
//! that direction, through the intersection points of their other isolines projected
//! on a plane normal to it.
use crate::geom2d_api::InterCurveCurve;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom::{
    GeomCurve, GeomOffsetSurface, GeomPlane, GeomRectangularTrimmedSurface, GeomSurface,
    GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeombase::geom_proj_lib;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Axis2, Direction3, Plane3, Point3, Vector3};
use std::f64::consts::PI;

/// The bounds, closure and periodicity of a surface in the direction of an isoline.
#[derive(Clone, Copy)]
pub(super) struct IsoRange {
    pub(super) first: f64,
    pub(super) last: f64,
    pub(super) closed: bool,
    pub(super) periodic: bool,
}

/// The Geom surface of an adaptor that is a `GeomAdaptor_Surface` (the native code casts
/// the adaptor; another adaptor raises Standard_NullObject).
pub(super) fn geom_surface(s: &Adaptor3dSurfaceHandle) -> Result<Handle<dyn GeomSurface>, Failure> {
    let gas: Handle<GeomAdaptorSurface> = s.cast::<GeomAdaptorSurface>();
    match gas.get() {
        Some(adaptor) => Ok(adaptor.surface()),
        None => Err(Failure::with_message(
            "Standard_NullObject",
            "IntPatch_Intersection - the surface is not a GeomAdaptor_Surface; expected one",
        )),
    }
}

/// The isoline of the surface (U iso when `u_iso`) or of the basis of an offset.
pub(super) fn iso_of(
    gs: &Handle<dyn GeomSurface>,
    typ: SurfaceType,
    u_iso: bool,
    prm: f64,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let surface: Handle<dyn GeomSurface> = match typ {
        SurfaceType::OffsetSurface => gs.cast::<GeomOffsetSurface>().basis_surface(),
        _ => gs.clone(),
    };
    if u_iso {
        surface.u_iso(prm)
    } else {
        surface.v_iso(prm)
    }
}

/// The type of a Geom curve.
pub(super) fn curve_type(c: &Handle<dyn GeomCurve>) -> Result<CurveType, Failure> {
    GeomAdaptorCurve::with_curve(c)?.get_type()
}

/// The isoline trimmed to the range, unless the surface is periodic over [0, 2 Pi].
fn trimmed_iso(
    gc: Handle<dyn GeomCurve>,
    range: IsoRange,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    if range.periodic && (range.first == 0.0 && range.last == 2.0 * PI) {
        return Ok(gc);
    }
    Ok(Handle::new(GeomTrimmedCurve::new(
        &gc,
        range.first,
        range.last,
        true,
        true,
    )?)
    .cast())
}

/// The trim range of the offset of a line or a parabola (1e4 at most) or of a hyperbola
/// (4 at most); `None` for a hyperbola entirely beyond 4.
fn offset_trim(gact: CurveType, first: f64, last: f64) -> Option<(f64, f64)> {
    if gact != CurveType::Hyperbola {
        if first >= 0.0 && last >= 0.0 {
            return Some((
                first,
                if (last - first) > 1.0e+4 {
                    first + 1.0e+4
                } else {
                    last
                },
            ));
        } else if first < 0.0 && last < 0.0 {
            return Some((
                if (first - last) < -1.0e+4 {
                    last - 1.0e+4
                } else {
                    first
                },
                last,
            ));
        }
        return Some((
            if first < -1.0e+4 { -1.0e+4 } else { first },
            if last > 1.0e+4 { 1.0e+4 } else { last },
        ));
    }
    if first >= 0.0 && last >= 0.0 {
        if first > 4.0 {
            return None;
        }
        return Some((first, if last > 4.0 { 4.0 } else { last }));
    } else if first < 0.0 && last < 0.0 {
        if last < -4.0 {
            return None;
        }
        return Some((if first < -4.0 { -4.0 } else { first }, last));
    }
    Some((
        if first < -4.0 { -4.0 } else { first },
        if last > 4.0 { 4.0 } else { last },
    ))
}

/// `FUN_GetUiso` (`u_iso`) and `FUN_GetViso`: the isoline of the surface at `prm`,
/// trimmed to the range of the other direction; the isolines of an infinite offset of a
/// line, a parabola or a hyperbola are taken on the offset trimmed first (in U, with the
/// bounds of the other direction, as natively). `None` for a hyperbola out of range.
pub(super) fn get_iso(
    gs: &Handle<dyn GeomSurface>,
    typ: SurfaceType,
    range: IsoRange,
    (u_iso, prm): (bool, f64),
) -> Result<Option<Handle<dyn GeomCurve>>, Failure> {
    let iso = |s: &Handle<dyn GeomSurface>| if u_iso { s.u_iso(prm) } else { s.v_iso(prm) };
    if typ != SurfaceType::OffsetSurface {
        return Ok(Some(trimmed_iso(iso(gs)?, range)?));
    }
    let gos: Handle<GeomOffsetSurface> = gs.cast::<GeomOffsetSurface>();
    let gcbs: Handle<dyn GeomCurve> = iso(&gos.basis_surface())?;
    let gact: CurveType = curve_type(&gcbs)?;
    if range.periodic
        || range.closed
        || gact == CurveType::BSplineCurve
        || gact == CurveType::BezierCurve
        || (range.last - range.first).abs() < 1.0e+5
    {
        return Ok(Some(trimmed_iso(iso(&gos.cast())?, range)?));
    }
    let Some((m_tr, cap_tr)) = offset_trim(gact, range.first, range.last) else {
        return Ok(None);
    };
    // Make trimmed surface (trimmed in U with the bounds of the other direction).
    let rts =
        GeomRectangularTrimmedSurface::with_one_direction(&gos.cast(), m_tr, cap_tr, true, true)?;
    let rts: Handle<dyn GeomSurface> = Handle::new(rts).cast();
    Ok(Some(iso(&rts)?))
}

/// The surface of a side of `FUN_PL_Intersection`: its Geom surface, type, middle
/// parameters and ranges.
pub(super) struct PlSide<'a> {
    pub(super) s: &'a Adaptor3dSurfaceHandle,
    pub(super) typ: SurfaceType,
}

impl PlSide<'_> {
    fn middle(&self) -> Result<[f64; 2], Failure> {
        let s = self.s;
        Ok([
            0.5 * (s.last_u_parameter()? + s.first_u_parameter()?),
            0.5 * (s.last_v_parameter()? + s.first_v_parameter()?),
        ])
    }

    /// Whether its U isolines (`[0]`) and V isolines (`[1]`) are straight lines.
    fn iso_lines(&self, gs: &Handle<dyn GeomSurface>, ms: [f64; 2]) -> Result<[bool; 2], Failure> {
        let s = self.s;
        let mut is_line = [false, false];
        if self.typ == SurfaceType::SurfaceOfExtrusion {
            is_line[0] = true;
        } else if !s.is_v_periodic()? && !s.is_v_closed()? {
            is_line[0] = curve_type(&iso_of(gs, self.typ, true, ms[0])?)? == CurveType::Line;
        }
        if !s.is_u_periodic()? && !s.is_u_closed()? {
            is_line[1] = curve_type(&iso_of(gs, self.typ, false, ms[1])?)? == CurveType::Line;
        }
        Ok(is_line)
    }

    /// The range of the direction along a U isoline (V) or a V isoline (U).
    fn range(&self, of_u_iso: bool) -> Result<IsoRange, Failure> {
        let s = self.s;
        Ok(match of_u_iso {
            true => IsoRange {
                first: s.first_v_parameter()?,
                last: s.last_v_parameter()?,
                closed: s.is_v_closed()?,
                periodic: s.is_v_periodic()?,
            },
            false => IsoRange {
                first: s.first_u_parameter()?,
                last: s.last_u_parameter()?,
                closed: s.is_u_closed()?,
                periodic: s.is_u_periodic()?,
            },
        })
    }
}

/// The result of `FUN_PL_Intersection`: whether the case applies, the points of the
/// lines and their direction.
pub(super) struct PlIntersection {
    pub(super) is_ok: bool,
    pub(super) points: Vec<Point3>,
    pub(super) direction: Vector3,
}

/// The isolines to intersect: the V isoline (`false`) or the U isoline (`true`) of each
/// surface and the direction index of the lines, for the first pair of parallel
/// straight isolines in the native order.
fn parallel_isos(
    lines: ([bool; 2], [bool; 2]),
    ders: ([Vector3; 2], [Vector3; 2]),
) -> Result<Option<(bool, bool, usize)>, Failure> {
    let (l1, l2) = lines;
    let (d1, d2) = ders;
    let angular: f64 = precision::angular();
    if l1[0] && l2[0] && d1[1].is_parallel(d2[1], angular)? {
        return Ok(Some((false, false, 1)));
    }
    if l1[0] && l2[1] && d1[1].is_parallel(d2[0], angular)? {
        return Ok(Some((false, true, 1)));
    }
    if l1[1] && l2[0] && d1[0].is_parallel(d2[1], angular)? {
        return Ok(Some((true, false, 0)));
    }
    if l1[1] && l2[1] && d1[0].is_parallel(d2[0], angular)? {
        return Ok(Some((true, true, 0)));
    }
    Ok(None)
}

/// The points of the lines: the intersections of the isolines projected along the
/// direction on the plane through `puv_s1`.
fn projected_intersections(
    c1: &Handle<dyn GeomCurve>,
    c2: &Handle<dyn GeomCurve>,
    puv_s1: Point3,
    dv: Vector3,
) -> Result<Vec<Point3>, Failure> {
    let dir = Direction3::from_vector(dv)?;
    let gpln: Handle<GeomPlane> = Handle::new(GeomPlane::from_pln(Plane3::new(puv_s1, dir)?));
    let c1_prj = geom_proj_lib::project_on_plane(c1, &gpln, dir, true)?;
    let c2_prj = geom_proj_lib::project_on_plane(c2, &gpln, dir, true)?;
    if c1_prj.is_null() || c2_prj.is_null() {
        return Ok(Vec::new());
    }
    let plane: Handle<dyn GeomSurface> = gpln.cast();
    let c1_prj2d: Handle<dyn Geom2dCurve> = geom_proj_lib::curve2d(&c1_prj, &plane)?;
    let c2_prj2d: Handle<dyn Geom2dCurve> = geom_proj_lib::curve2d(&c2_prj, &plane)?;
    let icc = InterCurveCurve::between(&c1_prj2d, &c2_prj2d, 1.0e-7)?;
    let ax = Axis2::new(puv_s1, dir)?;
    let mut points: Vec<Point3> = Vec::new();
    for ip in 1..=icc.nb_points() {
        points.push(elclib::to3d_point(ax, icc.point(ip)?));
    }
    Ok(points)
}

/// `FUN_PL_Intersection(S1, T1, S2, T2, IsOk, SP, DV)`.
pub(super) fn pl_intersection(
    side1: &PlSide<'_>,
    side2: &PlSide<'_>,
) -> Result<PlIntersection, Failure> {
    let mut result = PlIntersection {
        is_ok: false,
        points: Vec::new(),
        direction: Vector3::new(0.0, 0.0, 1.0),
    };
    let (gs1, gs2) = (geom_surface(side1.s)?, geom_surface(side2.s)?);
    let (ms1, ms2) = (side1.middle()?, side2.middle()?);
    let (l1, l2) = (side1.iso_lines(&gs1, ms1)?, side2.iso_lines(&gs2, ms2)?);
    // 1. Check: both surfaces have U(V)isos - lines.
    if !((l1[0] || l1[1]) && (l2[0] || l2[1])) {
        return Ok(result);
    }
    // 2. Check: Uiso lines of both surfaces are collinear.
    let d1 = side1.s.d1(ms1[0], ms1[1])?;
    let d2 = side2.s.d1(ms2[0], ms2[1])?;
    let ders = ([d1.d1u, d1.d1v], [d2.d1u, d2.d1v]);
    let Some((u_iso1, u_iso2, iso)) = parallel_isos((l1, l2), ders)? else {
        return Ok(result);
    };
    let c1 = get_iso(
        &gs1,
        side1.typ,
        side1.range(u_iso1)?,
        (u_iso1, ms1[usize::from(!u_iso1)]),
    )?;
    let c2 = get_iso(
        &gs2,
        side2.typ,
        side2.range(u_iso2)?,
        (u_iso2, ms2[usize::from(!u_iso2)]),
    )?;
    result.is_ok = true;
    // 3. Make intersections of V(U)isos.
    let (Some(c1), Some(c2)) = (c1, c2) else {
        return Ok(result);
    };
    result.direction = ders.0[iso];
    result.points = projected_intersections(&c1, &c2, d1.point, result.direction)?;
    Ok(result)
}
