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
// Rust translation of the IntPatch_PrmPrmIntersection::Perform overloads on one surface
// (ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_PrmPrmIntersection.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The self-intersection of a surface from the self-interference of its polyhedron:
//! the walkings start away from the diagonal (the points whose parameters on both
//! sides differ) and their ends on the diagonal are trimmed.
use super::IntPatchPrmPrmIntersection;
use super::helpers::section_point_to_parameters;
use super::perform_poly::IntPatchPrmPrmTolerances;
use super::start_params;
use super::walk_steps::{
    EndsCheck, WalkInputs, add_end_vertices, duplicated_line, keep_rejected, put_vertices,
    transitions,
};
use crate::int_patch::{
    IntPatchInterferencePolyhedron, IntPatchPoint, IntPatchPointLine, IntPatchPolyhedron,
    IntPatchWLType, IntPatchWLine,
};
use crate::int_start::TheTopolTool;
use crate::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S, IntSurfTypeTrans};
use crate::int_walk::IntWalkPWalking;
use crate::intf::{SectionLine, SectionPoint};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::gp::Point2;

/// The walking of the self-intersection; `uv` keeps the parameters of the last section
/// point read, as the native locals do.
struct SelfWalk<'a> {
    inputs: WalkInputs<'a>,
    poly: &'a IntPatchPolyhedron,
    pw: IntWalkPWalking,
    uv: [f64; 4],
}

impl SelfWalk<'_> {
    fn parameters(&mut self, sp: &SectionPoint) -> Result<[f64; 4], Failure> {
        section_point_to_parameters(sp, self.poly, self.poly, &mut self.uv)?;
        Ok(self.uv)
    }
}

/// Whether the parameters `(pu, pv)` are in the parameter box of the segment
/// `[a, b]` on the first surface, enlarged by 1e-7.
fn in_segment_box((pu, pv): (f64, f64), a: (f64, f64), b: (f64, f64)) -> bool {
    let (mut au1, mut av1, mut bu1, mut bv1) = (a.0, a.1, b.0, b.1);
    if au1 > bu1 {
        std::mem::swap(&mut au1, &mut bu1);
    }
    if av1 > bv1 {
        std::mem::swap(&mut av1, &mut bv1);
    }
    au1 -= 1.0e-7;
    av1 -= 1.0e-7;
    bu1 += 1.0e-7;
    bv1 += 1.0e-7;
    pu >= au1 && pu <= bu1 && pv >= av1 && pv <= bv1
}

/// Whether the start point is far from the lines found: none of its parameter pairs
/// is in the parameter box of a segment of a line (the threshold test of the native
/// loops).
// The negated comparisons send the NaN operands where the native ones do.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn is_new_self_start(
    inputs: &WalkInputs<'_>,
    start: &IntSurfPntOn2S,
    lines: &[Handle<dyn crate::int_patch::IntPatchLine>],
) -> Result<bool, Failure> {
    let seuil: f64 = inputs.seuil();
    let mut dmini: f64 = seuil + seuil;
    let (pu1, pv1, pu2, pv2) = start.parameters();
    for line in lines {
        if !(dmini >= seuil) {
            break;
        }
        let testwline: Handle<IntPatchWLine> = line.cast::<IntPatchWLine>();
        if testwline.is_out_surf1_box(Point2::new(pu1, pv1))?
            || testwline.is_out_surf2_box(Point2::new(pu2, pv2))?
            || testwline.is_out_box(start.value())?
        {
            continue;
        }
        let nb: i32 = testwline.nb_pnts()?;
        let mut ll: i32 = 1;
        while ll < nb && dmini >= seuil {
            let a = testwline.point(ll)?.parameters_on_s1();
            let b = testwline.point(ll + 1)?.parameters_on_s1();
            if in_segment_box((pu1, pv1), a, b) || in_segment_box((pu2, pv2), a, b) {
                dmini = 0.0;
            }
            ll += 1;
        }
    }
    Ok(dmini > seuil)
}

/// Whether the parameters of the point on both sides are equal within 1e-6.
fn on_diagonal(p: &IntSurfPntOn2S) -> bool {
    let (u1, v1, u2, v2) = p.parameters();
    (u1 - u2).abs() <= 1e-6 && (v1 - v2).abs() <= 1e-6
}

/// The range of the points of the walking without its ends on the diagonal (one
/// diagonal point kept at each end).
fn trimmed_range(line: &IntSurfLineOn2S) -> Result<(i32, i32), Failure> {
    let nbpw: i32 = line.nb_points();
    let mut i: i32 = 0;
    let mut imin: i32;
    loop {
        i += 1;
        imin = i;
        if !(i < nbpw && on_diagonal(&line.value(i)?)) {
            break;
        }
    }
    if imin > 2 {
        imin -= 1;
    }
    i = nbpw + 1;
    let mut imax: i32;
    loop {
        i -= 1;
        imax = i;
        if !(i > 2 && on_diagonal(&line.value(i)?)) {
            break;
        }
    }
    if imax < nbpw {
        imax += 1;
    }
    Ok((imin, imax))
}

impl IntPatchPrmPrmIntersection {
    /// `Perform(Caro1, Domain1, TolTangency, Epsilon, Deflection, Increment)`: the
    /// self-intersection of the surface, on its polyhedron of `NbSamplesU x NbSamplesV`
    /// cells of the domain.
    ///
    /// ```ignore
    /// inter.perform_self(&s1, &*d1, tolerances)?;
    /// ```
    pub fn perform_self(
        &mut self,
        surf1: &Adaptor3dSurfaceHandle,
        d1: &(dyn TheTopolTool + 'static),
        tolerances: IntPatchPrmPrmTolerances,
    ) -> Result<(), Failure> {
        let poly1 =
            IntPatchPolyhedron::with_subdivisions(surf1, d1.nb_samples_u()?, d1.nb_samples_v()?)?;
        self.perform_self_polyhedron((surf1, &poly1, d1), tolerances)
    }

    /// `Perform(Caro1, Polyhedron1, Domain1, TolTangency, Epsilon, Deflection,
    /// Increment)`: the self-intersection of the surface from the self-interference of
    /// its polyhedron.
    ///
    /// ```ignore
    /// inter.perform_self_polyhedron((&s1, &p1, &*d1), tolerances)?;
    /// ```
    pub fn perform_self_polyhedron(
        &mut self,
        (surf1, poly1, d1): (
            &Adaptor3dSurfaceHandle,
            &IntPatchPolyhedron,
            &(dyn TheTopolTool + 'static),
        ),
        tolerances: IntPatchPrmPrmTolerances,
    ) -> Result<(), Failure> {
        let interference = IntPatchInterferencePolyhedron::of_self(poly1)?;
        self.empt = true;
        self.done = true;
        self.slin.clear();
        let t = tolerances;
        let pw = IntWalkPWalking::new(
            surf1,
            surf1,
            t.tol_tangency,
            t.epsilon,
            t.deflection,
            t.increment,
        )?;
        let inputs = WalkInputs {
            surf1,
            d1,
            surf2: surf1,
            d2: None,
            tol_tangency: t.tol_tangency,
            epsilon: t.epsilon,
            deflection: t.deflection,
            increment: t.increment,
        };
        let mut walk = SelfWalk {
            inputs,
            poly: poly1,
            pw,
            uv: [0.0; 4],
        };
        for ls in 1..=interference.nb_section_lines() {
            self.walk_self_section(&mut walk, interference.line_value(ls)?)?;
        }
        for z in 1..=interference.nb_tangent_zones() {
            let zone = interference.zone_value(z)?;
            for pz in 1..=zone.number_of_points() {
                let uv: [f64; 4] = walk.parameters(zone.get_point(pz)?)?;
                self.walk_self_start(&mut walk, uv)?;
            }
        }
        Ok(())
    }

    /// The marching from the start points of one section line: the point farthest from
    /// the diagonal, then the first (or middle) one, then the last one.
    fn walk_self_section(
        &mut self,
        walk: &mut SelfWalk<'_>,
        line: &SectionLine,
    ) -> Result<(), Failure> {
        let nbp: i32 = line.number_of_points();
        let mut nbps2: i32 = if nbp > 3 { nbp / 2 } else { 1 };
        let (mut n, mut first, mut second) = (0, 0, 0);
        loop {
            n += 1;
            if n == 1 {
                nbps2 = farthest_from_diagonal(walk, line, nbps2)?;
                if nbp < 3 {
                    n = 3;
                }
                first = nbps2;
            } else if n == 2 {
                nbps2 = if first == 1 { nbp / 2 } else { 1 };
                second = nbps2;
            } else {
                nbps2 = if first == nbp {
                    (first + second) / 2
                } else {
                    nbp
                };
            }
            let uv: [f64; 4] = walk.parameters(line.get_point(nbps2)?)?;
            self.walk_self_start(walk, uv)?;
            if !(nbp > 5 && n < 3) {
                return Ok(());
            }
        }
    }

    /// The walking from the start parameters `uv`, when its first point is off the
    /// diagonal.
    fn walk_self_start(&mut self, walk: &mut SelfWalk<'_>, uv: [f64; 4]) -> Result<(), Failure> {
        let start = start_params(uv)?;
        let Some(start_pon2s) = walk.pw.perform_first_point(&start)? else {
            return Ok(());
        };
        let (pu1, pv1, pu2, pv2) = start_pon2s.parameters();
        if !((pu1 - pu2).abs() > 1e-7 || (pv1 - pv2).abs() > 1e-7) {
            return Ok(());
        }
        let nb_lig_calculee: i32 = self.slin.len() as i32;
        if !is_new_self_start(&walk.inputs, &start_pon2s, &self.slin)? {
            return Ok(());
        }
        walk.pw.perform(&start)?;
        if !walk.pw.is_done() || walk.pw.nb_points()? <= 2 {
            return Ok(());
        }
        let pw: &IntWalkPWalking = &walk.pw;
        let ends = (pw.value(1)?.value(), pw.value(pw.nb_points()?)?.value());
        let duplicated = duplicated_line(
            &walk.inputs,
            pw,
            &self.slin,
            (nb_lig_calculee, false, EndsCheck::Either),
        )?;
        if keep_rejected(pw, duplicated, &mut self.slin)? {
            return Ok(());
        }
        self.accept_self_walking(&walk.inputs, pw, ends)
    }

    /// The line of an accepted self walking: the walking without its diagonal ends, a
    /// closing vertex when it is closed, the vertices of the restrictions and the end
    /// vertices; appended to the lines.
    fn accept_self_walking(
        &mut self,
        inputs: &WalkInputs<'_>,
        pw: &IntWalkPWalking,
        ends: (rustycad_tkmath::gp::Point3, rustycad_tkmath::gp::Point3),
    ) -> Result<(), Failure> {
        let (trans1, trans2): (IntSurfTypeTrans, IntSurfTypeTrans) =
            transitions(inputs, pw, false)?;
        let (imin, imax) = trimmed_range(pw.line()?)?;
        if imin >= imax {
            return Ok(());
        }
        let pw_line: Handle<IntSurfLineOn2S> = Handle::new(IntSurfLineOn2S::new(None));
        for i in imin..=imax {
            pw_line.add(&pw.line()?.value(i)?);
        }
        let tol_tang: f64 = inputs.tol_tangency;
        let wline: Handle<IntPatchWLine> =
            Handle::new(IntPatchWLine::new(&pw_line, false, trans1, trans2));
        wline.set_creating_way_info(IntPatchWLType::PrmPrm);
        close_self_line(&wline, inputs.tol_tangency, tol_tang)?;
        // The method PutVertexOnLine can reduce the number of points in <wline>.
        if !put_vertices(inputs, &wline, tol_tang)? {
            return Ok(());
        }
        add_end_vertices(pw, &wline, ends, tol_tang)?;
        self.slin.push(wline.cast());
        self.empt = false;
        Ok(())
    }
}

/// The point of the section line farthest from the diagonal (the largest difference of
/// the parameters on both sides), from the middle toward the first point, then toward
/// the last one.
fn farthest_from_diagonal(
    walk: &mut SelfWalk<'_>,
    line: &SectionLine,
    nbps2: i32,
) -> Result<i32, Failure> {
    let nbp: i32 = line.number_of_points();
    let mut best: i32 = nbps2;
    let mut incidence: f64 = 0.0;
    let ranks = (1..=nbp / 2).rev().chain(nbp / 2..=nbp);
    for nsp1 in ranks {
        let [u1, v1, u2, v2] = walk.parameters(line.get_point(nsp1)?)?;
        let current: f64 = (u1 - u2).abs() + (v1 - v2).abs();
        if current > incidence {
            best = nsp1;
            incidence = current;
        }
    }
    Ok(best)
}

/// A closed walking line (its end points within the tangency tolerance) gets its last
/// point replaced by its first point, with the parameters of both sides exchanged.
// The negated comparison sends the NaN operands where the native one does.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn close_self_line(wline: &IntPatchWLine, tol_tangency: f64, tol_tang: f64) -> Result<(), Failure> {
    let nb: i32 = wline.nb_pnts()?;
    let (deb, fin) = (wline.point(1)?, wline.point(nb)?);
    if !(deb.value().distance(&fin.value()) <= tol_tangency) {
        return Ok(());
    }
    let (u1t, v1t, u2t, v2t) = deb.parameters();
    let mut vtx = IntPatchPoint::new();
    vtx.set_value_tangent(deb.value(), tol_tang, false);
    vtx.set_parameters(u2t, v2t, u1t, v1t);
    vtx.set_parameter(f64::from(nb));
    wline.set_point(nb, &vtx)
}
