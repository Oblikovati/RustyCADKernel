// Created on: 1992-11-19
// Created by: Remi LEQUETTE
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
// Rust translation of BRepClass_Intersector.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepClass_Intersector`: the intersection of a classification segment with an
//! edge of a face, and the local geometry of the edge.
use super::edge::{BRepClassEdge, BndBoxState};
use super::intersector_checks::{
    SkipContext, check_on, check_skip, is_inter, max_tol_2d_cur_edge, tangent_as_chord,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeomalgo::geom2d_int::{GInter, IntConicCurveOfGInter};
use rustycad_tkgeomalgo::int_res2d::{Domain, Intersection};
use rustycad_tkgeomalgo::topclass::Classifier2dIntersector;
use rustycad_tkgeombase::bnd_lib::Add2dCurve;
use rustycad_tkgeombase::geom_lprop::CLProps2d;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Direction2, Line2, Point2};
use std::ops::{Deref, DerefMut};

/// `BRepClass_Intersector` (docs/topalgo-design.md section 2): a
/// `Geom2dInt_IntConicCurveOfGInter` (reached through `Deref`, with its
/// `IntRes2d_Intersection` results) that intersects a classification segment with an
/// edge. A point within the tolerance of the edge gives one ON point appended to the
/// results of the previous Perform, as natively.
///
/// ```
/// use rustycad_tktopalgo::brep_class::BRepClassIntersector;
/// let intersector = BRepClassIntersector::new();
/// assert!(!intersector.is_done());
/// ```
#[derive(Default)]
pub struct BRepClassIntersector {
    base: IntConicCurveOfGInter,
}

impl Deref for BRepClassIntersector {
    type Target = IntConicCurveOfGInter;
    fn deref(&self) -> &IntConicCurveOfGInter {
        &self.base
    }
}

impl DerefMut for BRepClassIntersector {
    fn deref_mut(&mut self) -> &mut IntConicCurveOfGInter {
        &mut self.base
    }
}

/// The pcurve the intersector works on, with its range.
struct EdgeCurve {
    curve: Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
}

impl BRepClassIntersector {
    /// `BRepClass_Intersector()`: not done; `BRepClassIntersector::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    // The results of the IntRes2d base.
    fn results_mut(&mut self) -> &mut Intersection {
        &mut self.base
    }

    /// `Perform(L, P, Tol, E)`: intersects the segment `[0, param]` of `lin` (a
    /// half-line when `param` is RealLast) with the pcurve of `edge` (its cached one,
    /// else its curve on the face). A point of `lin`'s origin within `tolerance` of
    /// the pcurve (with the bounding box, when used, around the origin) is appended
    /// as an ON point. Otherwise the curve-curve intersection replaces the results,
    /// retried on the gap at the last vertex when empty; with boxes, a box that misses
    /// the segment ends it not done. No pcurve leaves it not done. The exceptions of
    /// the geometry are returned; `intersector.perform(&line, 10.0, 1e-7, &edge)?`.
    pub fn perform(
        &mut self,
        lin: &Line2,
        param: f64,
        tolerance: f64,
        edge: &BRepClassEdge,
    ) -> Result<(), Failure> {
        let Some(curve) = edge_curve(edge)? else {
            self.results_mut().done = false;
            return Ok(());
        };
        let mut tol_z: f64 = tolerance;
        let bounds: Option<BoundingBox2> = edge_bounds(edge, &curve, tol_z)?;
        let adaptor = Geom2dAdaptorCurve::with_range(&curve.curve, curve.first, curve.last)?;
        let origin: Point2 = lin.location();
        if bounds
            .as_ref()
            .is_none_or(|bounds| !bounds.is_out_point(origin))
            && self.perform_on(lin, edge, &curve, &mut tol_z)?
        {
            return Ok(());
        }
        if let Some(mut bounds) = bounds {
            let (first_vertex, last_vertex) = topexp::vertices(edge.edge(), false);
            tol_z = max_tol_2d_cur_edge(&first_vertex, &last_vertex, edge.face(), tolerance)?;
            bounds.set_gap(tol_z);
            if !is_inter(&bounds, lin, param) {
                self.results_mut().done = false;
                return Ok(());
            }
        }
        self.perform_inter(lin, param, edge, (&curve, &adaptor))
    }

    // The "ON" case: a direct check of the origin against the pcurve extended by the
    // tolerance; true when the ON point was appended.
    fn perform_on(
        &mut self,
        lin: &Line2,
        edge: &BRepClassEdge,
        curve: &EdgeCurve,
        tol_z: &mut f64,
    ) -> Result<bool, Failure> {
        let (mut deb_tol, mut fin_tol) = (curve.first, curve.last);
        if *tol_z > precision::confusion() {
            deb_tol = curve.first - *tol_z;
            fin_tol = curve.last + *tol_z;
        }
        let adaptor = Geom2dAdaptorCurve::with_range(&curve.curve, deb_tol, fin_tol)?;
        let Some(point) = check_on(edge.face(), lin, &adaptor, tol_z, curve.last, curve.first)?
        else {
            return Ok(false);
        };
        let results = self.results_mut();
        results.append_point(point);
        results.done = true;
        Ok(true)
    }

    // The curve-curve intersection of the segment and the pcurve, retried on the gap
    // at a high-tolerance last vertex when it is empty.
    fn perform_inter(
        &mut self,
        lin: &Line2,
        param: f64,
        edge: &BRepClassEdge,
        (curve, adaptor): (&EdgeCurve, &Geom2dAdaptorCurve),
    ) -> Result<(), Failure> {
        let (mut deb, mut fin) = (curve.first, curve.last);
        let mut pdeb: Point2 = adaptor.d0(deb)?;
        let mut pfin: Point2 = adaptor.d0(fin)?;
        let dl: Domain = segment_domain(lin, param);
        let de: Domain = edge_domain(adaptor, (pdeb, deb), (pfin, fin))?;
        let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_lin2d(*lin)).cast();
        let cga = Geom2dAdaptorCurve::with_curve(&line)?;
        let tolerances = (precision::p_confusion(), precision::p_intersection());
        let mut inter = GInter::with_domains(&cga, &dl, adaptor, &de, tolerances.0, tolerances.1)?;
        // The check is for hitting the intersector to a vertex with high tolerance.
        if inter.is_empty()? {
            let context = SkipContext {
                lin,
                edge,
                curve: &curve.curve,
                dl: &dl,
                cga: &cga,
            };
            check_skip(
                &mut inter,
                &context,
                adaptor,
                (&mut fin, &mut deb),
                (&mut pdeb, &mut pfin),
            )?;
        }
        self.results_mut().set_values(&inter);
        Ok(())
    }

    /// `LocalGeometry(E, U, T, N, C)`: the tangent, normal and curvature of the
    /// pcurve of `edge` (cached, else its curve on the face) at `u`, by
    /// GeomLProp_CLProps2d. An undefined tangent is the chord to a nearby point
    /// ((1, 0) when the chord is too short) with a zero curvature; a curvature not
    /// above PConfusion or infinite gives the normal (T.Y, -T.X);
    /// `let (tangent, normal, curvature) = intersector.local_geometry(&edge, 0.5)?;`.
    pub fn local_geometry(
        &self,
        edge: &BRepClassEdge,
        u: f64,
    ) -> Result<(Direction2, Direction2, f64), Failure> {
        let (mut fpar, mut lpar) = (edge.first_parameter(), edge.last_parameter());
        let mut pcurve: Handle<dyn Geom2dCurve> = edge.curve().clone();
        if pcurve.is_null() {
            let found = BRepTool::curve_on_face(edge.edge(), edge.face())?;
            (pcurve, fpar, lpar) = (found.pcurve, found.first, found.last);
        }
        let mut prop = CLProps2d::with_parameter(pcurve.clone(), u, 2, precision::p_confusion())?;
        // The native output directions are default-constructed by the caller: (1, 0).
        let mut tang = Direction2::new(1.0, 0.0)?;
        let mut curvature: f64 = 0.0;
        if prop.is_tangent_defined()? {
            tang = prop.tangent()?;
            curvature = prop.curvature()?;
        } else {
            tangent_as_chord(&pcurve, &mut tang, u, fpar, lpar)?;
        }
        let norm: Direction2 =
            if curvature > precision::p_confusion() && !precision::is_infinite(curvature) {
                prop.normal()?
            } else {
                Direction2::new(tang.y(), -tang.x())?
            };
        Ok((tang, norm, curvature))
    }
}

// The cached pcurve of the edge, else its curve on the face; None without one.
fn edge_curve(edge: &BRepClassEdge) -> Result<Option<EdgeCurve>, Failure> {
    let (curve, first, last) = if edge.curve().is_null() {
        let found = BRepTool::curve_on_face(edge.edge(), edge.face())?;
        (found.pcurve, found.first, found.last)
    } else {
        (
            edge.curve().clone(),
            edge.first_parameter(),
            edge.last_parameter(),
        )
    };
    if curve.is_null() {
        return Ok(None);
    }
    Ok(Some(EdgeCurve { curve, first, last }))
}

// The bounding box of the pcurve, with the gap `tol_z`, when the edge uses boxes and
// its box is available: the cached one, or one computed without writing the cache.
// A void box means no box.
fn edge_bounds(
    edge: &BRepClassEdge,
    curve: &EdgeCurve,
    tol_z: f64,
) -> Result<Option<BoundingBox2>, Failure> {
    if !edge.use_bnd_box() || edge.bounding_box_state() == BndBoxState::Unavailable {
        return Ok(None);
    }
    let mut bounds = BoundingBox2::default();
    if edge.bounding_box_state() == BndBoxState::Ready {
        bounds = edge.bounding_box().clone();
    } else {
        Add2dCurve::add_curve_range(&curve.curve, curve.first, curve.last, 0.0, &mut bounds)?;
    }
    if bounds.is_void() {
        return Ok(None);
    }
    bounds.set_gap(tol_z);
    Ok(Some(bounds))
}

// The domain of the segment: [0, P] with PConfusion at the ends, or the half-line
// from the origin when P is RealLast.
fn segment_domain(lin: &Line2, param: f64) -> Domain {
    let mut dl = Domain::new();
    let origin: Point2 = lin.location();
    if param != real_last() {
        let end: Point2 = elclib::value(param, lin);
        dl.set_values(
            origin,
            0.0,
            precision::p_confusion(),
            end,
            param,
            precision::p_confusion(),
        );
    } else {
        dl.set_semi_infinite_values(origin, 0.0, precision::p_confusion(), true);
    }
    dl
}

// The domain of the pcurve on its range with 1e-5 at the ends, periodic on the
// period of the curve when the curve is periodic ("temporary periodic domain").
pub(super) fn edge_domain(
    adaptor: &Geom2dAdaptorCurve,
    (pdeb, deb): (Point2, f64),
    (pfin, fin): (Point2, f64),
) -> Result<Domain, Failure> {
    let (toldeb, tolfin) = (1.0e-5, 1.0e-5);
    let mut de = Domain::bounded(pdeb, deb, toldeb, pfin, fin, tolfin);
    let curve: Handle<dyn Geom2dCurve> = adaptor.curve();
    if curve.is_periodic() {
        let first: f64 = adaptor.first_parameter()?;
        de.set_equivalent_parameters(
            first,
            first + curve.last_parameter() - curve.first_parameter(),
        )?;
    }
    Ok(de)
}

impl Classifier2dIntersector for BRepClassIntersector {
    type Edge = BRepClassEdge;
    fn perform_segment(
        &mut self,
        lin: &Line2,
        param: f64,
        tolerance: f64,
        edge: &BRepClassEdge,
    ) -> Result<(), Failure> {
        self.perform(lin, param, tolerance, edge)
    }
    fn results(&self) -> &Intersection {
        &self.base
    }
    fn local_geometry(
        &self,
        edge: &BRepClassEdge,
        u: f64,
    ) -> Result<(Direction2, Direction2, f64), Failure> {
        BRepClassIntersector::local_geometry(self, edge, u)
    }
}
