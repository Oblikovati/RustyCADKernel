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
// Rust translation of Geom2dHatch_Intersector.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dHatch_Intersector`: the curve-curve intersector of the hatcher and of its
//! classifier.
use crate::geom2d_int::GInter;
use crate::int_res2d::{Domain, Intersection};
use crate::topclass::Classifier2dIntersector;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeombase::geom_lprop::CLProps2d;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Direction2, Line2};
use std::ops::{Deref, DerefMut};

/// `Geom2dHatch_Intersector`: a [`GInter`] (reached through `Deref`) with the
/// confusion and tangency tolerances of the hatcher. `Clone` is the native copy
/// constructor (the IntRes2d results copied as neither done nor reversed) and
/// [`Intersector::assign`] the assignment.
///
/// ```
/// use rustycad_tkgeomalgo::geom2d_hatch::Intersector;
/// let intersector = Intersector::new(1e-7, 1e-7);
/// assert_eq!(intersector.confusion_tolerance(), 1e-7);
/// ```
#[derive(Clone, Default)]
pub struct Intersector {
    base: GInter,
    my_confusion_tolerance: f64,
    my_tangency_tolerance: f64,
}

impl Deref for Intersector {
    type Target = GInter;
    fn deref(&self) -> &GInter {
        &self.base
    }
}

impl DerefMut for Intersector {
    fn deref_mut(&mut self) -> &mut GInter {
        &mut self.base
    }
}

impl Intersector {
    /// `Geom2dHatch_Intersector(Confusion, Tangency)`; `Intersector::new(1e-7, 1e-7)`.
    pub fn new(confusion: f64, tangency: f64) -> Self {
        Self {
            base: GInter::new(),
            my_confusion_tolerance: confusion,
            my_tangency_tolerance: tangency,
        }
    }

    /// The assignment operator: the GInter by its assignment, then the tolerances;
    /// `target.assign(&source)`.
    pub fn assign(&mut self, other: &Intersector) {
        self.base.assign(&other.base);
        self.my_confusion_tolerance = other.my_confusion_tolerance;
        self.my_tangency_tolerance = other.my_tangency_tolerance;
    }

    /// ConfusionTolerance(); `intersector.confusion_tolerance()`.
    pub fn confusion_tolerance(&self) -> f64 {
        self.my_confusion_tolerance
    }

    /// SetConfusionTolerance(Confusion); `intersector.set_confusion_tolerance(1e-6)`.
    pub fn set_confusion_tolerance(&mut self, confusion: f64) {
        self.my_confusion_tolerance = confusion;
    }

    /// TangencyTolerance(); `intersector.tangency_tolerance()`.
    pub fn tangency_tolerance(&self) -> f64 {
        self.my_tangency_tolerance
    }

    /// SetTangencyTolerance(Tangency); `intersector.set_tangency_tolerance(1e-6)`.
    pub fn set_tangency_tolerance(&mut self, tangency: f64) {
        self.my_tangency_tolerance = tangency;
    }

    /// Intersect(C1, C2): `Geom2dInt_GInter::Perform(C1, C2, Confusion, Tangency)`;
    /// the results are read through IntRes2d_Intersection;
    /// `intersector.intersect(&hatching, &element)?`.
    pub fn intersect(
        &mut self,
        c1: &Geom2dAdaptorCurve,
        c2: &Geom2dAdaptorCurve,
    ) -> Result<(), Failure> {
        let (confusion, tangency) = (self.my_confusion_tolerance, self.my_tangency_tolerance);
        self.base.perform_between(c1, c2, confusion, tangency)
    }

    /// Perform(L, P, Tol, E): intersects the segment `[0, param]` of `lin` (a
    /// half-line when `param` is RealLast), with tolerance `tolerance` at its ends,
    /// and the curve `edge` on its parameter range; the segment is the first curve.
    /// The intersection runs on a fresh Geom2dInt_GInter with the parametric
    /// tolerances whose results are copied here (SetValues);
    /// `intersector.perform(&line, 10.0, 1e-7, &edge)?`.
    pub fn perform(
        &mut self,
        lin: &Line2,
        param: f64,
        tolerance: f64,
        edge: &Geom2dAdaptorCurve,
    ) -> Result<(), Failure> {
        let dl: Domain = segment_domain(lin, param, tolerance);
        let de: Domain = edge_domain(edge)?;
        let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_lin2d(*lin)).cast();
        let cga = Geom2dAdaptorCurve::with_curve(&line)?;
        let inter = GInter::with_domains(
            &cga,
            &dl,
            edge,
            &de,
            precision::p_confusion(),
            precision::p_intersection(),
        )?;
        self.base.set_values(&inter);
        Ok(())
    }

    /// LocalGeometry(E, U, T, N, C): the tangent, normal and curvature of the curve
    /// of `edge` (its Geom2d curve, whatever the adaptor's bounds) at `u`, by
    /// GeomLProp_CLProps2d. An undefined tangent gives (1, 0) and a zero curvature;
    /// a curvature not in ]PConfusion, RealLast[ gives the normal (T.Y, -T.X);
    /// `let (t, n, c) = intersector.local_geometry(&edge, 0.5)?;`.
    pub fn local_geometry(
        &self,
        edge: &Geom2dAdaptorCurve,
        u: f64,
    ) -> Result<(Direction2, Direction2, f64), Failure> {
        let mut prop = CLProps2d::with_parameter(edge.curve(), u, 2, precision::p_confusion())?;
        let mut curvature: f64 = 0.0;
        let tang: Direction2 = if prop.is_tangent_defined()? {
            let tang = prop.tangent()?;
            curvature = prop.curvature()?;
            tang
        } else {
            Direction2::new(1.0, 0.0)?
        };
        let norm: Direction2 = if curvature > precision::p_confusion() && curvature < real_last() {
            prop.normal()?
        } else {
            Direction2::new(tang.y(), -tang.x())?
        };
        Ok((tang, norm, curvature))
    }
}

// The domain of the segment [0, P] of L: bounded, or semi-infinite from the
// origin when P is RealLast.
fn segment_domain(lin: &Line2, param: f64, tolerance: f64) -> Domain {
    let mut dl = Domain::new();
    if param != real_last() {
        let end = elclib::value(param, lin);
        dl.set_values(lin.location(), 0.0, tolerance, end, param, tolerance);
    } else {
        dl.set_semi_infinite_values(lin.location(), 0.0, tolerance, true);
    }
    dl
}

// The domain of the edge on its parameter range, with PIntersection at its ends.
fn edge_domain(edge: &Geom2dAdaptorCurve) -> Result<Domain, Failure> {
    let (first, last) = (edge.first_parameter()?, edge.last_parameter()?);
    Ok(Domain::bounded(
        edge.value(first)?,
        first,
        precision::p_intersection(),
        edge.value(last)?,
        last,
        precision::p_intersection(),
    ))
}

impl Classifier2dIntersector for Intersector {
    type Edge = Geom2dAdaptorCurve;
    fn perform_segment(
        &mut self,
        lin: &Line2,
        param: f64,
        tolerance: f64,
        edge: &Geom2dAdaptorCurve,
    ) -> Result<(), Failure> {
        self.perform(lin, param, tolerance, edge)
    }
    fn results(&self) -> &Intersection {
        &self.base
    }
    fn local_geometry(
        &self,
        edge: &Geom2dAdaptorCurve,
        u: f64,
    ) -> Result<(Direction2, Direction2, f64), Failure> {
        Intersector::local_geometry(self, edge, u)
    }
}
