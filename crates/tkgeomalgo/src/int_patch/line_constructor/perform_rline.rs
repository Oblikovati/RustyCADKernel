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
// Rust translation of the restriction branch of IntPatch_LineConstructor::Perform
// (ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_LineConstructor.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_LineConstructor::Perform` on a restriction line: a part between two
//! vertices is kept when it lies on both restrictions, or when a point near its start,
//! moved on the other surface, is inside the other domain.
use super::helpers::compute_parametric_tolerance;
use super::perform_curves::Split;
use super::wline_rline::Domains;
use crate::int_patch::failures::null_object;
use crate::int_patch::{IntPatchPoint, IntPatchPointLine, IntPatchRLine};
use crate::int_start::TheArc;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::gp::{Point2, Point3, Vector3};

/// The side of the restriction (`on_first`: the arc is on the first surface, the part is
/// classified in the domain of the second one).
struct RestrictionSide<'a> {
    domains: &'a Domains<'a>,
    arc: TheArc,
    on_first: bool,
}

impl RestrictionSide<'_> {
    /// The parameters of a vertex on the other surface.
    fn other_params(&self, vtx: &IntPatchPoint) -> (f64, f64) {
        let (u1, v1, u2, v2) = vtx.pnt_on_2s().parameters();
        if self.on_first { (u2, v2) } else { (u1, v1) }
    }

    /// `Classify` in the domain of the other surface.
    fn classify(&self, (u, v): (f64, f64), tol: f64) -> Result<State, Failure> {
        self.domains
            .side(!self.on_first)
            .1
            .classify(Point2::new(u, v), tol, false)
    }

    /// The point of the arc near the start of the part, on the surface of the arc.
    /// A null arc (a restriction on neither surface) raises Standard_NullObject where the
    /// native code dereferences the null handle.
    fn arc_point(&self, u: f64) -> Result<Point3, Failure> {
        let Some(arc) = self.arc.get() else {
            return Err(null_object(
                "IntPatch_LineConstructor::Perform",
                "Adaptor2d_Curve2d",
            ));
        };
        let px2d: Point2 = arc.value(u)?;
        self.domains.side(self.on_first).0.value(px2d.x(), px2d.y())
    }
}

/// The steps along the derivatives of the other surface that move it from `p0` to `px`
/// (a tiny step when the derivatives are parallel):
/// `P0Px . D1u = deltau * D1u.D1u + deltav * D1u.D1v` and
/// `P0Px . D1v = deltau * D1u.D1v + deltav * D1v.D1v`.
fn parametric_steps(p0px: Vector3, d1u: Vector3, d1v: Vector3) -> (f64, f64) {
    let d1ud1u: f64 = d1u.dot(d1u);
    let d1vd1v: f64 = d1v.dot(d1v);
    let d1ud1v: f64 = d1u.dot(d1v);
    let tgd1u: f64 = p0px.dot(d1u);
    let tgd1v: f64 = p0px.dot(d1v);
    let dis: f64 = d1ud1u * d1vd1v - d1ud1v * d1ud1v;
    if dis < -1e-10 || dis > 1e-10 {
        let deltau: f64 = (tgd1u * d1vd1v - tgd1v * d1ud1v) / dis;
        let deltav: f64 = (tgd1v * d1ud1u - tgd1u * d1ud1v) / dis;
        return (deltau, deltav);
    }
    (1e-10, 1e-10)
}

/// Whether the part between `vtx1` and `vtx2` of a restriction on one surface only is
/// inside the domain of the other surface (`tol` persists across the parts).
// The negated comparison sends the NaN operands where the native one does.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn is_inside_other(
    side: &RestrictionSide<'_>,
    (vtx1, vtx2): (&IntPatchPoint, &IntPatchPoint),
    tol: &mut f64,
) -> Result<bool, Failure> {
    let (u0, u1) = (vtx1.parameter_on_line(), vtx2.parameter_on_line());
    if !((u1 - u0).abs() > precision::p_confusion()) {
        return Ok(false);
    }
    let u: f64 = (999.0 * u0 + u1) * 0.001;
    let p0px: Vector3 = Vector3::between(vtx1.value(), side.arc_point(u)?);
    let (mut uo, mut vo) = side.other_params(vtx1);
    let (other_surf, other_dom) = side.domains.side(!side.on_first);
    let d1 = other_surf.d1(uo, vo)?;
    other_dom.init();
    // The native code always tests the second domain, also when it initialized the
    // first one (`myDom1->Init(); if (myDom2->More())` when the arc is on S2).
    if side.domains.d2.more() {
        *tol = compute_parametric_tolerance(other_dom.tol3d(&other_dom.value()?)?, d1.d1u, d1.d1v);
    }
    // Le 23 mars 1999.
    let mut bornin: State = side.classify((uo, vo), *tol)?;
    if bornin != State::Out {
        bornin = side.classify(side.other_params(vtx2), *tol)?;
    }
    if bornin == State::Out {
        return Ok(false);
    }
    let (mut deltau, mut deltav) = parametric_steps(p0px, d1.d1u, d1.d1v);
    uo += deltau;
    vo += deltav;
    let mut in2: State = side.classify((uo, vo), *tol)?;
    deltau *= 0.05;
    deltav *= 0.05;
    let around: [(f64, f64); 4] = [
        (uo + deltau, vo),
        (uo - deltau, vo),
        (uo, vo + deltav),
        (uo, vo - deltav),
    ];
    for uv in around {
        if in2 == State::Out {
            in2 = side.classify(uv, *tol)?;
        }
    }
    Ok(in2 != State::Out)
}

/// `Perform` on a restriction line.
pub(super) fn perform_rline(split: &mut Split<'_>, domains: &Domains<'_>) -> Result<(), Failure> {
    let mut tol: f64 = precision::p_confusion() * 100.0;
    let rline: Handle<IntPatchRLine> = split.l.cast::<IntPatchRLine>();
    let nb_vtx: i32 = rline.nb_vertex();
    let (rest_on_s1, rest_on_s2) = (rline.is_arc_on_s1(), rline.is_arc_on_s2());
    for i in 1..nb_vtx {
        let (vtx1, vtx2) = (rline.vertex(i)?, rline.vertex(i + 1)?);
        if rest_on_s1 && rest_on_s2 {
            split.add_part_untagged(i, i + 1)?;
            continue;
        }
        // On na classifie pas sur 1.
        let arc: TheArc = if rest_on_s1 {
            rline.arc_on_s1()
        } else {
            rline.arc_on_s2()
        };
        let side = RestrictionSide {
            domains,
            arc,
            on_first: rest_on_s1,
        };
        if is_inside_other(&side, (&vtx1, &vtx2), &mut tol)? {
            split.add_part_untagged(i, i + 1)?;
        }
    }
    Ok(())
}
