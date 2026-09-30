// Copyright (c) 1992-1999 Matra Datavision
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
// Rust translation of the walking-line part of IntPatch_ImpPrmIntersection::Perform of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpPrmIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The lines of the walking become walking lines: their quadric parameters
//! recomputed and reframed, their transitions, and their end vertices.
use super::super::iwline::IntPatchTheIWLineOfTheIWalking;
use super::super::point::IntPatchPoint;
use super::super::point_line::IntPatchPointLine;
use super::super::wline::{IntPatchWLType, IntPatchWLine};
use super::super::IntPatchTheIWalking;
use super::IntPatchImpPrmIntersection;
use super::perform::PrmInput;
use super::setup::recadre;
use crate::int_surf::{IntSurfLineOn2S, IntSurfPathPoint, IntSurfTransition, IntSurfTypeTrans, make_transition};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Direction3, Point3, Vector3};
use std::f64::consts::PI;

/// The V range of the quadric and the tolerance that snaps a V to it.
#[derive(Clone, Copy)]
struct VRange {
    vmin: f64,
    vmax: f64,
}

impl VRange {
    const TOL_V: f64 = 1.0e-14;
    fn snap(self, mut v: f64) -> f64 {
        if v < self.vmin && self.vmin - v < Self::TOL_V {
            v = self.vmin;
        }
        if v > self.vmax && v - self.vmax < Self::TOL_V {
            v = self.vmax;
        }
        v
    }
}

/// `u` within 1.5 PI of `r` by turns of 2 PI.
fn near(mut u: f64, r: f64) -> f64 {
    while u < (r - 1.5 * PI) {
        u += PI + PI;
    }
    while u > (r + 1.5 * PI) {
        u -= PI + PI;
    }
    u
}

fn is_revolved(t: SurfaceType) -> bool {
    matches!(t, SurfaceType::Cylinder | SurfaceType::Cone | SurfaceType::Sphere | SurfaceType::Torus)
}

/// The parameters of the previous point (quadric U, parametric U and V).
struct Previous {
    an_u1: f64,
    an_u2: f64,
    an_v2: f64,
}

impl IntPatchImpPrmIntersection {
    /// The walking lines of at least two points.
    pub(super) fn walking_lines(
        &mut self,
        input: &PrmInput<'_>,
        iwalk: &IntPatchTheIWalking,
        (seqpdep, destination): (&[IntSurfPathPoint], &[i32]),
    ) -> Result<(), Failure> {
        let q = input.qsurf();
        let vr = VRange { vmin: q.first_v_parameter()?, vmax: q.last_v_parameter()? };
        for j in 1..=iwalk.nb_lines()? {
            let iwline: Handle<IntPatchTheIWLineOfTheIWalking> = iwalk.value(j)?.clone();
            let iw: &IntPatchTheIWLineOfTheIWalking = iwline.get().expect("walking line");
            let thelin: Handle<IntSurfLineOn2S> = iw.line();
            let nbpts: i32 = thelin.nb_points();
            if nbpts < 2 {
                continue;
            }
            let (trans1, trans2) = self.line_transitions(input, iw, &thelin)?;
            self.adjust_line_parameters(input, &thelin, vr)?;
            let wline = IntPatchWLine::new(&thelin, false, trans1, trans2);
            wline.set_creating_way_info(IntPatchWLType::ImpPrm);
            let ends = LineEnds { iwline: iw, seqpdep, destination, vr, nbpts };
            self.first_vertices(input, &wline, &ends)?;
            self.last_vertices(input, &wline, &ends)?;
            self.slin.push(Handle::new(wline).cast());
        }
        Ok(())
    }

    /// The transitions of the line from its tangent at the index it stores (or its
    /// middle point).
    fn line_transitions(
        &self,
        input: &PrmInput<'_>,
        iw: &IntPatchTheIWLineOfTheIWalking,
        thelin: &IntSurfLineOn2S,
    ) -> Result<(IntSurfTypeTrans, IntSurfTypeTrans), Failure> {
        let nbpts: i32 = thelin.nb_points();
        let (tgline, mut k) = iw.tangent_vector();
        if !(k >= 1 && k <= nbpts) {
            k = nbpts >> 1;
        }
        let p = thelin.value(k)?;
        let valpt: Point3 = p.value();
        let (norm1, norm2) = if !input.reversed {
            let (u2, v2) = p.parameters_on_s2();
            let d1 = input.surf2.d1(u2, v2)?;
            (input.quad.normale_at(valpt)?, d1.d1u.crossed(d1.d1v))
        } else {
            let (u2, v2) = p.parameters_on_s1();
            let d1 = input.surf1.d1(u2, v2)?;
            (d1.d1u.crossed(d1.d1v), input.quad.normale_at(valpt)?)
        };
        if tgline.dot_cross(norm2, norm1) > 0.0 {
            return Ok((IntSurfTypeTrans::Out, IntSurfTypeTrans::In));
        }
        Ok((IntSurfTypeTrans::In, IntSurfTypeTrans::Out))
    }

    /// The quadric parameters of the points recomputed from their 3d points (V snapped
    /// to the range, U continuous by turns of 2 PI, NIZNHY-PKV 28 Mar 2008), and the
    /// periodic parameters of the parametric surface made continuous too.
    fn adjust_line_parameters(&self, input: &PrmInput<'_>, thelin: &IntSurfLineOn2S, vr: VRange) -> Result<(), Failure> {
        let quad = &input.quad;
        let (an_u1, v1) = quad.parameters(thelin.value(1)?.value()).unwrap_or((0.0, 0.0));
        let v1 = vr.snap(v1);
        thelin.set_uv(1, !input.reversed, an_u1, v1)?;
        let (an_u2, an_v2) = thelin.value(1)?.parameters_on_surface(input.reversed);
        let mut prev = Previous { an_u1, an_u2, an_v2 };
        let arecadr: bool = matches!(quad.type_quadric(), SurfaceType::Cylinder | SurfaceType::Cone | SurfaceType::Sphere);
        for k in 2..=thelin.nb_points() {
            let (mut u1, v1) = quad.parameters(thelin.value(k)?.value()).unwrap_or((0.0, 0.0));
            let v1 = vr.snap(v1);
            if arecadr {
                u1 = recadr_u(u1, prev.an_u1);
            }
            thelin.set_uv(k, !input.reversed, u1, v1)?;
            let (mut u2, mut v2) = thelin.value(k)?.parameters_on_surface(input.reversed);
            // The periodic type of the other surface (typeS1 when reversed, as natively).
            let t_u: SurfaceType = if input.reversed { input.types.0 } else { input.types.1 };
            if is_revolved(t_u) {
                u2 = near(u2, prev.an_u2);
            }
            if input.types.1 == SurfaceType::Torus {
                v2 = near(v2, prev.an_v2);
            }
            thelin.set_uv(k, input.reversed, u2, v2)?;
            prev = Previous { an_u1: u1, an_u2: u2, an_v2: v2 };
        }
        Ok(())
    }
}

/// `U1` of the quadric brought within 1.5 PI of the previous one by whole turns.
fn recadr_u(u1: f64, an_u1: f64) -> f64 {
    let two_pi: f64 = PI + PI;
    let mut cf: f64 = 0.0;
    if (u1 - an_u1) > 1.5 * PI {
        while (u1 - an_u1) > (1.5 * PI + cf * two_pi) {
            cf += 1.0;
        }
        return u1 - cf * two_pi;
    }
    while (u1 - an_u1) < (-1.5 * PI - cf * two_pi) {
        cf += 1.0;
    }
    u1 + cf * two_pi
}

/// The data of the end vertices of one line.
struct LineEnds<'a> {
    iwline: &'a IntPatchTheIWLineOfTheIWalking,
    seqpdep: &'a [IntSurfPathPoint],
    destination: &'a [i32],
    vr: VRange,
    nbpts: i32,
}

impl IntPatchImpPrmIntersection {
    /// A vertex at a line end without path point: tangent or not.
    fn plain_end(input: &PrmInput<'_>, wline: &IntPatchWLine, thelin: &IntSurfLineOn2S, (index, tangent): (i32, bool)) -> Result<(), Failure> {
        let p = thelin.value(index)?;
        let (u1, v1, u2, v2) = p.parameters();
        let mut pt = IntPatchPoint::new();
        pt.set_value_tangent(p.value(), input.tol_arc, tangent);
        pt.set_parameters(u1, v1, u2, v2);
        pt.set_parameter(f64::from(index));
        wline.add_vertex(&pt, false);
        Ok(())
    }

    /// The first vertices: one per restriction point of the first path point (the
    /// multiplicities of the path point), or the first point of the line.
    fn first_vertices(&self, input: &PrmInput<'_>, wline: &IntPatchWLine, ends: &LineEnds<'_>) -> Result<(), Failure> {
        let iw = ends.iwline;
        let thelin = iw.line();
        if iw.has_first_point() && !iw.is_tangent_at_begining() {
            let indfirst: i32 = iw.first_point_index()?;
            let ppoint: &IntSurfPathPoint = path_point(ends.seqpdep, indfirst)?;
            let tgline: Vector3 = ppoint.direction3d()?;
            return self.path_point_vertices(input, wline, ends, (indfirst, ppoint, tgline), true);
        }
        Self::plain_end(input, wline, &thelin, (1, iw.is_tangent_at_begining()))?;
        wline.set_first_point(wline.nb_vertex());
        Ok(())
    }

    /// The last vertices, as the first ones with the reversed tangent.
    fn last_vertices(&self, input: &PrmInput<'_>, wline: &IntPatchWLine, ends: &LineEnds<'_>) -> Result<(), Failure> {
        let iw = ends.iwline;
        let thelin = iw.line();
        if iw.has_last_point() && !iw.is_tangent_at_end() {
            let indlast: i32 = iw.last_point_index()?;
            let ppoint: &IntSurfPathPoint = path_point(ends.seqpdep, indlast)?;
            let tgline: Vector3 = ppoint.direction3d()?.reversed();
            return self.path_point_vertices(input, wline, ends, (indlast, ppoint, tgline), false);
        }
        Self::plain_end(input, wline, &thelin, (ends.nbpts, iw.is_tangent_at_end()))?;
        wline.set_last_point(wline.nb_vertex());
        Ok(())
    }

    /// The vertices of the path point `ind` at an end: one per restriction point of
    /// rank `ind`, from the last one, with the parameters of the multiplicity and the
    /// transitions against the arc.
    fn path_point_vertices(
        &self,
        input: &PrmInput<'_>,
        wline: &IntPatchWLine,
        ends: &LineEnds<'_>,
        (ind, ppoint, tgline): (i32, &IntSurfPathPoint, Vector3),
        is_first: bool,
    ) -> Result<(), Failure> {
        let mut themult: i32 = ppoint.multiplicity()?;
        let nb_point_rst: i32 = self.solrst.nb_points()?;
        for i in (1..=nb_point_rst).rev() {
            if ends.destination[i as usize] != ind {
                continue;
            }
            let pt = self.end_vertex(input, ends, (ppoint, themult, tgline), i, is_first)?;
            wline.add_vertex(&pt, false);
            if themult == 0 {
                if is_first {
                    wline.set_first_point(wline.nb_vertex());
                } else {
                    wline.set_last_point(wline.nb_vertex());
                }
            }
            themult -= 1;
        }
        Ok(())
    }

    /// The vertex at the restriction point `i` of the path point.
    fn end_vertex(
        &self,
        input: &PrmInput<'_>,
        ends: &LineEnds<'_>,
        (ppoint, themult, tgline): (&IntSurfPathPoint, i32, Vector3),
        i: i32,
        is_first: bool,
    ) -> Result<IntPatchPoint, Failure> {
        let (qu, qv) = input.quad.parameters(ppoint.value()).unwrap_or((0.0, 0.0));
        let qv = ends.vr.snap(qv);
        let (pu, pv) = ppoint.parameters(themult)?;
        let psurf = input.psurf().0;
        let d1 = psurf.d1(pu, pv)?;
        let vec_normale: Vector3 = d1.d1u.crossed(d1.d1v);
        let [u1, v1, u2, v2] = if input.reversed { [pu, pv, qu, qv] } else { [qu, qv, pu, pv] };
        let mut pt = IntPatchPoint::new();
        pt.set_value_tangent(ppoint.value(), input.tol_arc, false);
        pt.set_parameters(u1, v1, u2, v2);
        let param: i32 = if is_first { 1 } else { ends.nbpts - 1 };
        pt.set_parameter(if is_first { 1.0 } else { f64::from(ends.nbpts) });
        recadre(input.types, &mut pt, ends.iwline, param, [u1, v1, u2, v2])?;
        let rst = self.solrst.point(i)?;
        let (currentarc, currentparam) = (rst.arc(), rst.parameter());
        let (_, d2d) = currentarc.d1(currentparam)?;
        let mut tgrst = Vector3::default();
        tgrst.set_linear_form(d2d.x(), d1.d1u, d2d.y(), d1.d1v);
        let (mut tline, mut tarc) = (IntSurfTransition::default(), IntSurfTransition::default());
        if vec_normale.square_magnitude() > 1e-13 {
            make_transition(tgline, tgrst, Direction3::from_vector(vec_normale)?, &mut tline, &mut tarc);
        } else {
            tline.set_type(true, IntSurfTypeTrans::Undecided);
            tarc.set_type(true, IntSurfTypeTrans::Undecided);
        }
        pt.set_arc(input.reversed, &currentarc, currentparam, &tline, &tarc);
        if !rst.is_new() {
            pt.set_vertex(input.reversed, &rst.vertex()?);
        }
        Ok(pt)
    }
}

/// `seqpdep(index)`, with the Standard_OutOfRange of NCollection_Sequence.
fn path_point(seqpdep: &[IntSurfPathPoint], index: i32) -> Result<&IntSurfPathPoint, Failure> {
    crate::int_start::failures::sequence_value(seqpdep, index, "NCollection_Sequence::Value")
}

/// The first or last vertex of a walking line when it is a tangency point.
fn tangent_end(w: &IntPatchWLine, first: bool) -> Result<Option<(IntPatchPoint, i32)>, Failure> {
    if first && !w.has_first_point() || !first && !w.has_last_point() {
        return Ok(None);
    }
    let (p, ind) = if first { w.first_point_index()? } else { w.last_point_index()? };
    Ok(p.is_tangency_point().then_some((p, ind)))
}

impl IntPatchImpPrmIntersection {
    /// The junctions of the lines: their ends that are the same tangency point become
    /// multiple points (the lines can join at tangency points only).
    pub(super) fn tangency_joins(&mut self, tol_arc: f64) -> Result<(), Failure> {
        let nblines: usize = self.slin.len();
        for j in 0..nblines.saturating_sub(1) {
            let wlin1: Handle<IntPatchWLine> = self.slin[j].cast();
            let w1: &IntPatchWLine = wlin1.get().expect("walking line");
            let mut deb = tangent_end(w1, true)?;
            let mut fin = tangent_end(w1, false)?;
            if deb.is_none() && fin.is_none() {
                continue;
            }
            for k in j + 1..nblines {
                let wlin2: Handle<IntPatchWLine> = self.slin[k].cast();
                let w2: &IntPatchWLine = wlin2.get().expect("walking line");
                for first in [true, false] {
                    if let Some((ptbis, ind2)) = tangent_end(w2, first)? {
                        join(w2, (ptbis, ind2), (&mut deb, &mut fin), tol_arc)?;
                    }
                }
            }
            if let Some((p, ind)) = &deb {
                w1.replace(*ind, p)?;
            }
            if let Some((p, ind)) = &fin {
                w1.replace(*ind, p)?;
            }
        }
        Ok(())
    }
}

/// An end vertex of the first line and its index, when it is a tangency point.
type TangentEnd = Option<(IntPatchPoint, i32)>;

/// The tangency end `ptbis` of another line near the ends of the first one: both are
/// made multiple.
fn join(
    w2: &IntPatchWLine,
    (mut ptbis, ind2): (IntPatchPoint, i32),
    (deb, fin): (&mut TangentEnd, &mut TangentEnd),
    tol_arc: f64,
) -> Result<(), Failure> {
    for end in [deb, fin] {
        let Some((p, _)) = end.as_mut() else {
            continue;
        };
        if p.value().distance(&ptbis.value()) <= tol_arc {
            p.set_multiple(true);
            if !ptbis.is_multiple() {
                ptbis.set_multiple(true);
                w2.replace(ind2, &ptbis)?;
            }
        }
    }
    Ok(())
}

