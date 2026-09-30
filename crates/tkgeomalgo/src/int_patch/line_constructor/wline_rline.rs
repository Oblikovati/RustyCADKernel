// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of the static helpers TestWLineIsARLine, TestIfWLineIsRestriction and
// ProjectOnArc of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_LineConstructor.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The tests of a walking line against the restrictions: whether it lies on an arc of
//! a domain (then it is a restriction line, see `rline_coding.rs`).
use super::helpers::surface_tolerance;
use crate::geom2d_int::TheProjPCurOfGInter;
use crate::int_patch::{
    IntPatchIType, IntPatchLine, IntPatchPointLine, IntPatchRLine, IntPatchWLine,
};
use crate::int_start::{TheArc, TheTopolTool};
use crate::int_surf::IntSurfPntOn2S;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::gp::Point2;

/// The parameters of a point on the first (`on_first`) or the second surface.
pub(super) fn params_on(p: &IntSurfPntOn2S, on_first: bool) -> (f64, f64) {
    if on_first {
        p.parameters_on_s1()
    } else {
        p.parameters_on_s2()
    }
}

/// The projection of `(u, v)` on the arc: its parameter and the distance to it.
fn project(arc: &TheArc, u: f64, v: f64) -> Result<(f64, f64), Failure> {
    let p2d = Point2::new(u, v);
    let par: f64 = TheProjPCurOfGInter::find_parameter(&**arc, p2d, 1e-7)?;
    let pobt: Point2 = arc.value(par)?;
    Ok((par, pobt.distance(p2d)))
}

/// `TestWLineIsARLine`: whether the middle of the walking line lies on an arc of a
/// restriction line of `slinref`, within `tol2d`.
fn test_wline_is_a_rline(
    slinref: &[Handle<dyn IntPatchLine>],
    wlin: &IntPatchWLine,
    tol2d: f64,
) -> Result<bool, Failure> {
    let indicepnt: i32 = wlin.nb_pnts()? / 2;
    if indicepnt < 1 {
        return Ok(false);
    }
    let (pon2s, pon2s1) = (wlin.point(indicepnt)?, wlin.point(indicepnt + 1)?);
    for line in slinref
        .iter()
        .filter(|l| l.arc_type() == IntPatchIType::Restriction)
    {
        let rlin: Handle<IntPatchRLine> = line.cast::<IntPatchRLine>();
        for on_first in [true, false] {
            let arc: TheArc = match on_first {
                true if rlin.is_arc_on_s1() => rlin.arc_on_s1(),
                false if rlin.is_arc_on_s2() => rlin.arc_on_s2(),
                _ => continue,
            };
            let ((mut u, mut v), (u1, v1)) =
                (params_on(&pon2s, on_first), params_on(&pon2s1, on_first));
            if indicepnt == 1 {
                u = (u + u1) * 0.5;
                v = (v + v1) * 0.5;
            }
            if project(&arc, u, v)?.1 < tol2d {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// The surfaces and domains of `IntPatch_LineConstructor::Perform`.
pub(super) struct Domains<'a> {
    pub(super) s1: &'a Adaptor3dSurfaceHandle,
    pub(super) d1: &'a dyn TheTopolTool,
    pub(super) s2: &'a Adaptor3dSurfaceHandle,
    pub(super) d2: &'a dyn TheTopolTool,
}

impl Domains<'_> {
    /// The surface and the domain of the first (`on_first`) or the second surface.
    pub(super) fn side(&self, on_first: bool) -> (&Adaptor3dSurfaceHandle, &dyn TheTopolTool) {
        if on_first {
            (self.s1, self.d1)
        } else {
            (self.s2, self.d2)
        }
    }
}

/// The count of the first points of a line on one boundary and the largest
/// parametric tolerance met.
#[derive(Default)]
struct BoundaryRun {
    allon: i32,
    tol2d: f64,
}

impl BoundaryRun {
    /// The point `i` at `(u, v)`: the tolerance grows, and the run goes on while the
    /// points are on the boundary.
    fn step(
        &mut self,
        (s, d): (&Adaptor3dSurfaceHandle, &dyn TheTopolTool),
        (u, v): (f64, f64),
        (i, tol_arc): (i32, f64),
    ) -> Result<(), Failure> {
        // An upper bound of the parametric tolerance from the 3d one.
        let tol: f64 = surface_tolerance(s, tol_arc, u, v)?;
        if tol > self.tol2d {
            self.tol2d = tol;
        }
        if self.allon + 1 == i && d.is_the_point_on(Point2::new(u, v), tol, true)? {
            self.allon += 1;
        }
        Ok(())
    }
}

/// `TestIfWLineIsRestriction`: whether all the points of the walking line are on the
/// boundary of one domain, and its middle on an arc of a restriction line of `slinref`.
pub(super) fn test_if_wline_is_restriction(
    slinref: &[Handle<dyn IntPatchLine>],
    wlin: &IntPatchWLine,
    domains: &Domains<'_>,
    tol_arc: f64,
) -> Result<bool, Failure> {
    let nb_pnts: i32 = wlin.nb_pnts()?;
    let (mut run1, mut run2) = (BoundaryRun::default(), BoundaryRun::default());
    for i in 1..=nb_pnts {
        let (u1, v1, u2, v2) = wlin.point(i)?.parameters();
        run1.step(domains.side(true), (u1, v1), (i, tol_arc))?;
        run2.step(domains.side(false), (u2, v2), (i, tol_arc))?;
        if run1.allon != i && run2.allon != i {
            break;
        }
    }
    if run1.allon == nb_pnts || run2.allon == nb_pnts {
        let tol2d: f64 = crate::intf::std_max(run1.tol2d, run2.tol2d);
        return test_wline_is_a_rline(slinref, wlin, tol2d);
    }
    Ok(false)
}

/// `ProjectOnArc`: the projection of `(u, v)` on the arc, its parameter and distance,
/// and whether the distance is within the parametric tolerance of `tol_arc` there.
pub(super) fn project_on_arc(
    (u, v): (f64, f64),
    arc: &TheArc,
    surf: &Adaptor3dSurfaceHandle,
    tol_arc: f64,
) -> Result<(bool, f64, f64), Failure> {
    let tol2d: f64 = surface_tolerance(surf, tol_arc, u, v)?;
    let (par, dist) = project(arc, u, v)?;
    Ok((dist <= tol2d, par, dist))
}
