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
// Rust translation of the vertex part of IntPatch_RstInt::PutVertexOnLine of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_RstInt.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! A crossing of the line and an arc becomes a new vertex of the line, or is merged
//! into the existing vertex at its place.
use super::super::point::IntPatchPoint;
use super::arcs::ArcRun;
use super::helpers::{PointLineRef, coincide_on_arc, tol3d_vertex, verify_tgline};
use super::{RstInput, RstState};
use crate::int_start::{TheArc, TheVertex};
use crate::int_surf::{IntSurfTypeTrans, make_transition};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Direction3, Vector3, resolution};

/// Whether two arcs are the same handle.
fn same_arc(a: &TheArc, b: &TheArc) -> bool {
    a.object_address() == b.object_address()
}

impl RstInput<'_> {
    /// `Replace(k, pnt)` on the line.
    fn replace_vertex(&self, k: i32, pnt: &IntPatchPoint) -> Result<(), Failure> {
        match &self.line {
            PointLineRef::Walking(w) => w.replace(k, pnt),
            PointLineRef::Restriction(r) => r.replace(k, pnt),
        }
    }

    /// `AddVertex(pnt)` on the line; the vertex count follows.
    fn add_vertex(&self, st: &mut RstState, pnt: &IntPatchPoint) {
        self.line.line().add_vertex(pnt, false);
        st.nbvtx += 1;
    }

    /// Whether the vertex is on the arc of its domain (for this side).
    fn on_this_arc(&self, p: &IntPatchPoint, arc: &TheArc) -> Result<bool, Failure> {
        if self.on_first && p.is_on_dom_s1() {
            return Ok(same_arc(&p.arc_on_s1()?, arc));
        }
        if !self.on_first && p.is_on_dom_s2() {
            return Ok(same_arc(&p.arc_on_s2()?, arc));
        }
        Ok(false)
    }

    /// The vertex of the domain at the point, and the vertex tolerance.
    fn vertex_on_arc(
        &self,
        st: &mut RstState,
        run: &ArcRun<'_>,
        toler: f64,
    ) -> Result<bool, Failure> {
        let mut vtx: TheVertex = st.vtxarc.clone();
        let found = coincide_on_arc(
            st.ptsommet,
            run.arc,
            self.surf,
            toler,
            self.domain,
            &mut vtx,
        )?;
        st.vtxarc = vtx;
        Ok(found)
    }

    /// The crossing put on the line: a new vertex, or merged into the nearest vertex
    /// not already on this arc within the tolerances (none on a cone).
    pub(super) fn put_vertex(&self, st: &mut RstState, run: &ArcRun<'_>) -> Result<(), Failure> {
        let mut vtx_on_arc: bool = self.vertex_on_arc(st, run, run.edge_tol)?;
        let mut vtx_tol: f64 = if vtx_on_arc {
            let t: f64 = tol3d_vertex(&st.vtxarc, self.domain, 0.0)?;
            if run.edge_tol > t { run.edge_tol } else { t }
        } else {
            run.edge_tol
        };
        let ivtx: i32 = self.nearest_vertex(st, run.arc, vtx_tol)?;
        if ivtx != 0 && st.ptline.tolerance() > vtx_tol {
            vtx_tol = st.ptline.tolerance();
            if !vtx_on_arc {
                // A new attempt to coincide on a bound of the arc.
                vtx_on_arc = self.vertex_on_arc(st, run, vtx_tol)?;
                if vtx_on_arc {
                    let t: f64 = tol3d_vertex(&st.vtxarc, self.domain, 0.0)?;
                    if t > vtx_tol {
                        vtx_tol = t;
                    }
                }
            }
        }
        self.line_arc_transitions(st, run.edge_tol)?;
        let ctx = VertexCtx {
            arc: run.arc,
            vtx_on_arc,
            vtx_tol,
        };
        if ivtx == 0 {
            return self.new_vertex(st, &ctx);
        }
        self.merge_vertex(st, &ctx, ivtx)
    }

    /// The index of the vertex the point merges into (0 for none): the nearest vertex
    /// not on this arc; a nearer one out of its tolerance cancels the choice.
    fn nearest_vertex(
        &self,
        st: &mut RstState,
        arc: &TheArc,
        vtx_tol: f64,
    ) -> Result<i32, Failure> {
        let mut ivtx: i32 = 0;
        let mut dmin: f64 = f64::MAX;
        for j in 1..=st.nbvtx {
            let rptline: IntPatchPoint = self.line.line().vertex(j)?;
            if self.on_this_arc(&rptline, arc)? || !self.closed_match(st, &rptline) {
                continue;
            }
            let dist: f64 = st.ptsommet.distance(&rptline.value());
            let dt: f64 = std_max(vtx_tol, rptline.tolerance());
            if dist < dmin {
                if dist <= dt {
                    st.ptline = rptline;
                    ivtx = if self.surfacetype == SurfaceType::Cone {
                        0
                    } else {
                        j
                    };
                } else {
                    // This point is better, but its tolerance is not large enough.
                    ivtx = 0;
                }
                dmin = dist;
            }
        }
        Ok(ivtx)
    }

    /// On closed surfaces, whether the 2d parameters of the vertex match the point.
    fn closed_match(&self, st: &mut RstState, rptline: &IntPatchPoint) -> bool {
        if !self.possibly_closed() {
            return true;
        }
        if self.closed[0] || self.closed[1] {
            (st.u, st.v) = if self.on_first {
                rptline.parameters_on_s1()
            } else {
                rptline.parameters_on_s2()
            };
            if (self.closed[0] && (st.u - st.u1).abs() > self.tol_closed[0])
                || (self.closed[1] && (st.v - st.v1).abs() > self.tol_closed[1])
            {
                return false;
            }
        }
        if self.closed[2] || self.closed[3] {
            (st.u, st.v) = if self.on_first {
                rptline.parameters_on_s2()
            } else {
                rptline.parameters_on_s1()
            };
            if (self.closed[2] && (st.u - st.u2).abs() > self.tol_closed[2])
                || (self.closed[3] && (st.v - st.v2).abs() > self.tol_closed[3])
            {
                return false;
            }
        }
        true
    }
}

/// The arc and the vertex data of one crossing.
pub(super) struct VertexCtx<'a> {
    pub(super) arc: &'a TheArc,
    pub(super) vtx_on_arc: bool,
    pub(super) vtx_tol: f64,
}

impl RstInput<'_> {
    /// The line tangent (lengthened on a walking line) and the transitions against the
    /// arc tangent on the surface; undecided on a null surface normal.
    pub(super) fn line_arc_transitions(
        &self,
        st: &mut RstState,
        edge_tol: f64,
    ) -> Result<(), Failure> {
        if let PointLineRef::Walking(w) = &self.line {
            verify_tgline(w, st.paramline as i32, edge_tol, &mut st.tgline)?;
        }
        let d1 = self.surf.d1(st.u1, st.v1)?;
        let mut tgrst = Vector3::default();
        tgrst.set_linear_form(st.d2d.x(), d1.d1u, st.d2d.y(), d1.d1v);
        let normsurf: Vector3 = d1.d1u.crossed(d1.d1v);
        if normsurf.magnitude() < resolution() {
            st.transline.set_type(true, IntSurfTypeTrans::Undecided);
            st.transarc.set_type(true, IntSurfTypeTrans::Undecided);
            return Ok(());
        }
        let n = Direction3::from_vector(normsurf)?;
        make_transition(st.tgline, tgrst, n, &mut st.transline, &mut st.transarc);
        Ok(())
    }

    /// A new vertex (the point replaces the line point of index `paramline`).
    fn new_vertex(&self, st: &mut RstState, ctx: &VertexCtx<'_>) -> Result<(), Failure> {
        st.sommet.set_value_tangent(st.ptsommet, ctx.vtx_tol, false);
        if self.on_first {
            st.sommet.set_parameters(st.u1, st.v1, st.u2, st.v2);
        } else {
            st.sommet.set_parameters(st.u2, st.v2, st.u1, st.v1);
        }
        if ctx.vtx_on_arc {
            st.sommet.set_vertex(self.on_first, &st.vtxarc);
        }
        st.sommet.set_parameter(st.paramline);
        st.sommet.set_arc(
            self.on_first,
            ctx.arc,
            st.paramarc,
            &st.transline,
            &st.transarc,
        );
        let sommet = st.sommet.clone();
        self.add_vertex(st, &sommet);
        Ok(())
    }

    /// The arc and vertex of the crossing set on a copy of `pnt`.
    fn with_arc(
        &self,
        st: &RstState,
        pnt: &mut IntPatchPoint,
        ctx: &VertexCtx<'_>,
        with_vertex: bool,
    ) {
        pnt.set_arc(
            self.on_first,
            ctx.arc,
            st.paramarc,
            &st.transline,
            &st.transarc,
        );
        if with_vertex && ctx.vtx_on_arc {
            pnt.set_vertex(self.on_first, &st.vtxarc);
        }
    }

    /// Whether the vertex is a vertex of the domain on this side.
    fn is_vertex_here(&self, p: &IntPatchPoint) -> bool {
        if self.on_first {
            p.is_vertex_on_s1()
        } else {
            p.is_vertex_on_s2()
        }
    }

    /// Whether the vertex is on the domain of the other side.
    fn is_on_other_dom(&self, p: &IntPatchPoint) -> bool {
        if self.on_first {
            p.is_on_dom_s2()
        } else {
            p.is_on_dom_s1()
        }
    }

    /// Whether the vertex is a vertex of the domain of the other side.
    fn is_vertex_other(&self, p: &IntPatchPoint) -> bool {
        if self.on_first {
            p.is_vertex_on_s2()
        } else {
            p.is_vertex_on_s1()
        }
    }

    /// The crossing merged into the vertex `ivtx` (`ptline`): see the cases of the
    /// native comment (a point on S1 found on S2, with or without vertices).
    fn merge_vertex(
        &self,
        st: &mut RstState,
        ctx: &VertexCtx<'_>,
        ivtx: i32,
    ) -> Result<(), Failure> {
        let on_different_rst: bool = {
            let p = &st.ptline;
            (self.on_first && p.is_on_dom_s1() && !same_arc(&p.arc_on_s1()?, ctx.arc))
                || (!self.on_first && p.is_on_dom_s2() && !same_arc(&p.arc_on_s2()?, ctx.arc))
        };
        st.ptline.set_tolerance(ctx.vtx_tol);
        if !self.is_vertex_here(&st.ptline) || on_different_rst {
            return self.merge_point(st, ctx, ivtx, on_different_rst);
        }
        self.merge_domain_vertex(st, ctx, ivtx)
    }

    /// The vertex is no vertex of this domain (or is on another restriction).
    fn merge_point(
        &self,
        st: &mut RstState,
        ctx: &VertexCtx<'_>,
        ivtx: i32,
        on_different_rst: bool,
    ) -> Result<(), Failure> {
        let own_dom: bool = if self.on_first {
            st.ptline.is_on_dom_s1()
        } else {
            st.ptline.is_on_dom_s2()
        };
        if !own_dom || on_different_rst {
            let mut p = st.ptline.clone();
            self.with_arc(st, &mut p, ctx, true);
            st.ptline = p.clone();
            if on_different_rst {
                self.add_vertex(st, &p);
            } else {
                self.replace_vertex(ivtx, &p)?;
            }
        } else if self.is_vertex_other(&st.ptline) {
            let mut sommet = st.ptline.clone();
            self.with_arc(st, &mut sommet, ctx, true);
            st.sommet = sommet.clone();
            self.add_vertex(st, &sommet);
        }
        Ok(())
    }

    /// The vertex is a vertex of this domain: the other vertices of the same domain
    /// vertex get the arc too.
    fn merge_domain_vertex(
        &self,
        st: &mut RstState,
        ctx: &VertexCtx<'_>,
        ivtx: i32,
    ) -> Result<(), Failure> {
        let vtxref: TheVertex = if self.on_first {
            st.ptline.vertex_on_s1()?
        } else {
            st.ptline.vertex_on_s2()?
        };
        if !self.is_on_other_dom(&st.ptline) {
            let mut p = st.ptline.clone();
            self.with_arc(st, &mut p, ctx, true);
            st.ptline = p.clone();
            self.replace_vertex(ivtx, &p)?;
            return self.same_domain_vertices(st, ctx, (ivtx, &vtxref), false);
        }
        if self.is_vertex_other(&st.ptline) {
            // The other vertex should be the vertex of the arc: no check.
            let mut sommet = st.ptline.clone();
            self.with_arc(st, &mut sommet, ctx, false);
            st.sommet = sommet.clone();
            self.add_vertex(st, &sommet);
            return self.same_domain_vertices(st, ctx, (ivtx, &vtxref), true);
        }
        Ok(())
    }

    /// The vertices `k != ivtx` of the domain vertex `vtxref` get the arc: replaced in
    /// place, or (`duplicate`) kept and added again with the arc. The vertex count is
    /// read again at each step, so the added vertices are visited too.
    fn same_domain_vertices(
        &self,
        st: &mut RstState,
        ctx: &VertexCtx<'_>,
        (ivtx, vtxref): (i32, &TheVertex),
        duplicate: bool,
    ) -> Result<(), Failure> {
        let mut k: i32 = 1;
        while k <= st.nbvtx {
            if k != ivtx {
                st.ptline = self.line.line().vertex(k)?;
                if self.is_vertex_here(&st.ptline) {
                    let other: TheVertex = if self.on_first {
                        st.ptline.vertex_on_s1()?
                    } else {
                        st.ptline.vertex_on_s2()?
                    };
                    if self.domain.identical(vtxref, &other)? {
                        self.same_vertex(st, ctx, k, duplicate)?;
                    }
                }
            }
            k += 1;
        }
        Ok(())
    }

    fn same_vertex(
        &self,
        st: &mut RstState,
        ctx: &VertexCtx<'_>,
        k: i32,
        duplicate: bool,
    ) -> Result<(), Failure> {
        if st.ptline.tolerance() < ctx.vtx_tol {
            st.ptline.set_tolerance(ctx.vtx_tol);
        }
        if !duplicate {
            let mut p = st.ptline.clone();
            self.with_arc(st, &mut p, ctx, true);
            st.ptline = p.clone();
            return self.replace_vertex(k, &p);
        }
        let mut sommet = st.ptline.clone();
        self.with_arc(st, &mut sommet, ctx, false);
        st.sommet = sommet.clone();
        let ptline = st.ptline.clone();
        self.replace_vertex(k, &ptline)?;
        self.add_vertex(st, &sommet);
        Ok(())
    }
}
