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
// Rust translation of IntCurve_IntPolyPolyGen.gxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurve_IntPolyPolyGen`: the intersection of two parametric curves, or of one
//! curve with itself, through the interference of their polygons.
use super::int_poly_poly_zone::{CurveOf, PointPair, point_transitions};
use super::{PCurveTool, Polygon2dGen};
use crate::int_imp_par_gen::{IntersectorTypes, ParTool};
use crate::int_res2d::{Domain, Intersection, IntersectionPoint, Position, Transition};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point2;
use std::ops::{Deref, DerefMut};

/// `NBITER_MAX_POLYGON`: the deepest refinement of a tangent zone.
pub(super) const NBITER_MAX_POLYGON: i32 = 10;
/// `TOL_CONF_MINI`: the smallest confusion tolerance.
const TOL_CONF_MINI: f64 = 0.0000000001;
/// `TOL_MINI`: the smallest tolerance.
const TOL_MINI: f64 = 0.0000000001;

/// `IntCurve_IntPolyPolyGen` (`Geom2dInt_TheIntPCurvePCurveOfGInter`): an
/// [`Intersection`] reached through `Deref`. The polygons of both curves
/// ([`Polygon2dGen`]) are intersected by Intf_InterferencePolygon2d; each section
/// point is refined by IntCurve_ExactIntersectionPoint (or, when that fails, by a
/// subdivision search) and each tangent zone becomes a segment, or is intersected
/// again on finer polygons while the deflection exceeds the confusion tolerance.
/// `let mut inter = TheIntPCurvePCurveOfGInter::new();
/// inter.perform(&a1, &d1, &a2, &d2, 1e-9, 1e-9)?;`.
pub struct IntPolyPolyGen<T: IntersectorTypes>
where
    T::Par: PCurveTool,
{
    pub(super) base: Intersection,
    pub(super) domain_on_curve1: Domain,
    pub(super) domain_on_curve2: Domain,
    my_min_pnt_nb: i32,
    tool: std::marker::PhantomData<fn() -> T>,
}

impl<T: IntersectorTypes> Default for IntPolyPolyGen<T>
where
    T::Par: PCurveTool,
{
    /// `IntCurve_IntPolyPolyGen()`: at least 20 samples, not done.
    fn default() -> Self {
        // Minimum number of samples.
        let a_min_pnt_nb: i32 = 20;
        Self {
            base: Intersection::new(),
            domain_on_curve1: Domain::new(),
            domain_on_curve2: Domain::new(),
            my_min_pnt_nb: a_min_pnt_nb,
            tool: std::marker::PhantomData,
        }
    }
}

impl<T: IntersectorTypes> Clone for IntPolyPolyGen<T>
where
    T::Par: PCurveTool,
{
    /// The native copy constructor: the IntRes2d_Intersection part is copied as
    /// neither done nor reversed.
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
            domain_on_curve1: self.domain_on_curve1,
            domain_on_curve2: self.domain_on_curve2,
            my_min_pnt_nb: self.my_min_pnt_nb,
            tool: std::marker::PhantomData,
        }
    }
}

impl<T: IntersectorTypes> Deref for IntPolyPolyGen<T>
where
    T::Par: PCurveTool,
{
    type Target = Intersection;
    fn deref(&self) -> &Intersection {
        &self.base
    }
}

impl<T: IntersectorTypes> DerefMut for IntPolyPolyGen<T>
where
    T::Par: PCurveTool,
{
    fn deref_mut(&mut self) -> &mut Intersection {
        &mut self.base
    }
}

/// The end flags of HeadOrEndPoint: which ends of each curve already carry a
/// point, and `PosSegment` (1 Head-Head, 2 Head-End, 4 End-Head, 8 End-End).
#[derive(Clone, Copy, Default)]
struct EndFlags {
    head_on1: bool,
    head_on2: bool,
    end_on1: bool,
    end_on2: bool,
    pos_segment: i32,
}

impl EndFlags {
    // Records the positions of one result point.
    fn record(&mut self, point: &IntersectionPoint) {
        let pos1: Position = point.transition_of_first().position_on_curve();
        let pos2: Position = point.transition_of_second().position_on_curve();
        match pos1 {
            Position::Head => self.head_on1 = true,
            Position::End => self.end_on1 = true,
            Position::Middle => {}
        }
        match pos2 {
            Position::Head => self.head_on2 = true,
            Position::End => self.end_on2 = true,
            Position::Middle => {}
        }
        self.pos_segment |= match (pos1, pos2) {
            (Position::Head, Position::Head) => 1,
            (Position::Head, Position::End) => 2,
            (Position::End, Position::Head) => 4,
            (Position::End, Position::End) => 8,
            _ => 0,
        };
    }
}

impl<T: IntersectorTypes> IntPolyPolyGen<T>
where
    T::Par: PCurveTool,
{
    /// `IntCurve_IntPolyPolyGen()`. `TheIntPCurvePCurveOfGInter::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Perform(Curve1, Domain1, Curve2, Domain2, TolConf, Tol)`: the intersection on
    /// the bounded domains (Standard_DomainError otherwise), both tolerances at
    /// least 1e-10, then the points at the domain ends that no result carries yet.
    /// `inter.perform(&a1, &d1, &a2, &d2, 1e-9, 1e-9)?`.
    pub fn perform(
        &mut self,
        c1: &CurveOf<T>,
        d1: &Domain,
        c2: &CurveOf<T>,
        d2: &Domain,
        the_tol_conf: f64,
        the_tol: f64,
    ) -> Result<(), Failure> {
        self.base.reset_fields();
        self.domain_on_curve1 = *d1;
        self.domain_on_curve2 = *d2;
        let du: f64 = d1.last_parameter()? - d1.first_parameter()?;
        let dv: f64 = d2.last_parameter()? - d2.first_parameter()?;
        let tl: f64 = if the_tol < TOL_MINI {
            TOL_MINI
        } else {
            the_tol
        };
        let tl_conf: f64 = if the_tol_conf < TOL_CONF_MINI {
            TOL_CONF_MINI
        } else {
            the_tol_conf
        };
        self.perform_iter(c1, d1, c2, d2, tl_conf, tl, 0, du, dv)?;
        let mut flags: EndFlags = self.end_flags()?;
        let (u0, u1) = (d1.first_parameter()?, d1.last_parameter()?);
        let (v0, v1) = (d2.first_parameter()?, d2.last_parameter()?);
        // (first end on curve 1, first end on curve 2), in the native order.
        for (first1, first2) in [(true, true), (true, false), (false, true), (false, false)] {
            let tested: bool =
                end_tolerance(d1, first1)? != 0.0 || end_tolerance(d2, first2)? != 0.0;
            if !tested {
                continue;
            }
            let u: f64 = if first1 { u0 } else { u1 };
            let v: f64 = if first2 { v0 } else { v1 };
            if let Some(int_pt) =
                head_or_end_point::<T>((d1, c1, u), (d2, c2, v), the_tol_conf, &mut flags)?
            {
                self.base.insert(int_pt)?;
            }
        }
        Ok(())
    }

    // The positions of every result point and segment end (the scan before the
    // HeadOrEndPoint calls).
    fn end_flags(&self) -> Result<EndFlags, Failure> {
        let mut flags: EndFlags = EndFlags::default();
        for i in 1..=self.base.nb_points()? {
            flags.record(&self.base.point(i)?);
        }
        for i in 1..=self.base.nb_segments()? {
            let segment = self.base.segment(i)?;
            flags.record(&segment.first_point()?);
            flags.record(&segment.last_point()?);
        }
        Ok(flags)
    }

    /// `Perform(Curve1, Domain1, TolConf, Tol)`: the self intersection of the curve
    /// on the bounded domain, both tolerances at least 1e-10.
    /// `inter.perform_self(&adaptor, &domain, 1e-9, 1e-9)?`.
    pub fn perform_self(
        &mut self,
        c1: &CurveOf<T>,
        d1: &Domain,
        the_tol_conf: f64,
        the_tol: f64,
    ) -> Result<(), Failure> {
        self.base.reset_fields();
        self.domain_on_curve1 = *d1;
        self.domain_on_curve2 = *d1;
        let du: f64 = d1.last_parameter()? - d1.first_parameter()?;
        let tl: f64 = if the_tol < TOL_MINI {
            TOL_MINI
        } else {
            the_tol
        };
        let tl_conf: f64 = if the_tol_conf < TOL_CONF_MINI {
            TOL_CONF_MINI
        } else {
            the_tol_conf
        };
        self.perform_self_iter(c1, d1, tl_conf, tl, 0, du, du)?;
        // The native code computes PosSegment here and discards it.
        self.end_flags()?;
        Ok(())
    }

    /// `Perform(Curve1, Domain1, Curve2, Domain2, TolConf, Tol, NbIter, DeltaU,
    /// DeltaV)` (protected): one polygon pass of iteration `nb_iter` (none past 10);
    /// the polygons have at least the minimal sample count on the first pass and
    /// 5/4 of the samples times the iteration after; when both deflections exceed
    /// the confusion tolerance each polygon is refined in the other's box. When
    /// refined polygons miss a root the pass runs again on the full ones.
    /// `inter.perform_iter(&a1, &d1, &a2, &d2, 1e-9, 1e-9, 0, 1.0, 1.0)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn perform_iter(
        &mut self,
        c1: &CurveOf<T>,
        d1: &Domain,
        c2: &CurveOf<T>,
        d2: &Domain,
        tol_conf: f64,
        tol: f64,
        nb_iter: i32,
        delta_u: f64,
        delta_v: f64,
    ) -> Result<(), Failure> {
        self.base.done = false;
        if nb_iter > NBITER_MAX_POLYGON {
            return Ok(());
        }
        // Number of samples running.
        let mut nbsamples_on_c1: i32 =
            T::Par::nb_samples(c1, d1.first_parameter()?, d1.last_parameter()?)?;
        let mut nbsamples_on_c2: i32 =
            T::Par::nb_samples(c2, d2.first_parameter()?, d2.last_parameter()?)?;
        if nb_iter == 0 {
            // Minimal number of points.
            nbsamples_on_c1 = nbsamples_on_c1.max(self.my_min_pnt_nb);
            nbsamples_on_c2 = nbsamples_on_c2.max(self.my_min_pnt_nb);
        } else {
            // Increase number of samples in second and next iterations.
            nbsamples_on_c1 = (5 * (nbsamples_on_c1 * nb_iter)) / 4;
            nbsamples_on_c2 = (5 * (nbsamples_on_c2 * nb_iter)) / 4;
        }
        let mut a_poly1: Polygon2dGen<T::Par> = Polygon2dGen::new(c1, nbsamples_on_c1, d1, tol)?;
        let mut a_poly2: Polygon2dGen<T::Par> = Polygon2dGen::new(c2, nbsamples_on_c2, d2, tol)?;
        refine_polygons::<T>(
            (c1, &mut a_poly1, nbsamples_on_c1),
            (c2, &mut a_poly2, nbsamples_on_c2),
            tol_conf,
        )?;
        // For a polygon whose points were replaced by chords, a missed root is
        // searched again on the polygon of all the initial points.
        let is_full_representation: bool =
            a_poly1.nb_segments() == nbsamples_on_c1 && a_poly2.nb_segments() == nbsamples_on_c2;
        let deltas = (nb_iter, delta_u, delta_v);
        if !self.find_intersect(
            (c1, d1, &a_poly1),
            (c2, d2, &a_poly2),
            (tol_conf, tol),
            deltas,
            is_full_representation,
        )? && !is_full_representation
        {
            if a_poly1.nb_segments() < nbsamples_on_c1 {
                a_poly1 = Polygon2dGen::new(c1, nbsamples_on_c1, d1, tol)?;
            }
            if a_poly2.nb_segments() < nbsamples_on_c2 {
                a_poly2 = Polygon2dGen::new(c2, nbsamples_on_c2, d2, tol)?;
            }
            self.find_intersect(
                (c1, d1, &a_poly1),
                (c2, d2, &a_poly2),
                (tol_conf, tol),
                deltas,
                true,
            )?;
        }
        self.base.done = true;
        Ok(())
    }

    /// `SetMinNbSamples(theMinNbSamples)`. `inter.set_min_nb_samples(40)`.
    pub fn set_min_nb_samples(&mut self, the_min_nb_samples: i32) {
        self.my_min_pnt_nb = the_min_nb_samples;
    }

    /// `GetMinNbSamples()`. `inter.get_min_nb_samples()`.
    pub fn get_min_nb_samples(&self) -> i32 {
        self.my_min_pnt_nb
    }
}

// The box refinement of the polygons when both deflections exceed the confusion
// tolerance (the polygon of more samples first), then each deflection raised to
// the confusion tolerance to detect tangent zones.
fn refine_polygons<T: IntersectorTypes>(
    (c1, poly1, n1): (&CurveOf<T>, &mut Polygon2dGen<T::Par>, i32),
    (c2, poly2, n2): (&CurveOf<T>, &mut Polygon2dGen<T::Par>, i32),
    tol_conf: f64,
) -> Result<(), Failure>
where
    T::Par: PCurveTool,
{
    use crate::intf::Polygon2d;
    if (poly1.deflection_over_estimation() > tol_conf)
        && (poly2.deflection_over_estimation() > tol_conf)
    {
        let a_deflection_sum: f64 = native_max(poly1.deflection_over_estimation(), tol_conf)
            + native_max(poly2.deflection_over_estimation(), tol_conf);
        if n2 > n1 {
            poly2.compute_with_box(c2, poly1.bounding())?;
            poly1.set_deflection_over_estimation(a_deflection_sum);
            poly1.compute_with_box(c1, poly2.bounding())?;
        } else {
            poly1.compute_with_box(c1, poly2.bounding())?;
            poly2.set_deflection_over_estimation(a_deflection_sum);
            poly2.compute_with_box(c2, poly1.bounding())?;
        }
    }
    if poly1.deflection_over_estimation() < tol_conf {
        poly1.set_deflection_over_estimation(tol_conf);
    }
    if poly2.deflection_over_estimation() < tol_conf {
        poly2.set_deflection_over_estimation(tol_conf);
    }
    Ok(())
}

// The first or last tolerance of a domain.
fn end_tolerance(d: &Domain, first: bool) -> Result<f64, Failure> {
    if first {
        d.first_tolerance()
    } else {
        d.last_tolerance()
    }
}

// `std::max(a, b)`: `b` only when `a < b`.
fn native_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

// `HeadOrEndPoint(D1, C1, u, D2, C2, v, TolConf, IntPt, HeadOn1, HeadOn2, EndOn1,
// EndOn2, PosSegment)`: a point where the other curve's point at `(u, v)` lies on a
// domain end, unless a result already joins the same pair of ends.
fn head_or_end_point<T: IntersectorTypes>(
    (d1, c1, tu): (&Domain, &CurveOf<T>, f64),
    (d2, c2, tv): (&Domain, &CurveOf<T>, f64),
    tol_conf: f64,
    flags: &mut EndFlags,
) -> Result<Option<IntersectionPoint>, Failure>
where
    T::Par: PCurveTool,
{
    let (mut u, mut v) = (tu, tv);
    let (svu, svv) = (u, v);
    let mut pair: PointPair = PointPair::default();
    T::Par::d1(c1, u, &mut pair.p1, &mut pair.tan1)?;
    T::Par::d1(c2, v, &mut pair.p2, &mut pair.tan2)?;
    let (mut sp1, mut sp2) = (Point2::default(), Point2::default());
    let (mut pos1, mut pos2) = (Position::Middle, Position::Middle);
    // Head On 1: Head1 <-> P2; End On 1: End1 <-> P2; Head On 2: Head2 <-> P1;
    // End On 2: End2 <-> P1.
    if pair.p2.distance(d1.first_point()?) <= d1.first_tolerance()? {
        pos1 = Position::Head;
        flags.head_on1 = true;
        sp1 = d1.first_point()?;
        u = d1.first_parameter()?;
    } else if pair.p2.distance(d1.last_point()?) <= d1.last_tolerance()? {
        pos1 = Position::End;
        flags.end_on1 = true;
        sp1 = d1.last_point()?;
        u = d1.last_parameter()?;
    } else if pair.p1.distance(d2.first_point()?) <= d2.first_tolerance()? {
        pos2 = Position::Head;
        flags.head_on2 = true;
        sp2 = d2.first_point()?;
        v = d2.first_parameter()?;
    } else if pair.p1.distance(d2.last_point()?) <= d2.last_tolerance()? {
        pos2 = Position::End;
        flags.end_on2 = true;
        sp2 = d2.last_point()?;
        v = d2.last_parameter()?;
    }
    let eps_x1: f64 = T::Par::eps_x(c1);
    let eps_x2: f64 = T::Par::eps_x(c2);
    if pos1 == Position::Middle && pos2 == Position::Middle {
        return Ok(None);
    }
    if pos1 == Position::Middle {
        if (u - d1.first_parameter()?).abs() <= eps_x1 {
            pos1 = Position::Head;
            pair.p1 = d1.first_point()?;
            flags.head_on1 = true;
        } else if (u - d1.last_parameter()?).abs() <= eps_x1 {
            pos1 = Position::End;
            pair.p1 = d1.last_point()?;
            flags.end_on1 = true;
        }
    } else if u != tu {
        pair.p1 = sp1;
    }
    if pos2 == Position::Middle {
        pos2 = snap_second_end(d2, v, eps_x2, (pos1, &mut pair), flags)?;
    }
    // SP2 is written but never read natively.
    let _ = sp2;
    if joined_already(pos1, pos2, flags.pos_segment) {
        return Ok(None);
    }
    let (mut trans1, mut trans2) = (Transition::new(), Transition::new());
    // The second-order test evaluates at the original parameters.
    point_transitions::<T>(
        (c1, svu),
        (c2, svv),
        (pos1, pos2),
        &mut pair,
        (&mut trans1, &mut trans2),
        tol_conf,
    )?;
    let mut int_pt = IntersectionPoint::default();
    int_pt.set_values(pair.p1, u, v, trans1, trans2, false);
    Ok(Some(int_pt))
}

// The second curve at a parameter end within EpsX: the point is the domain end and
// the first point the midpoint of both when the first curve is at an end too.
fn snap_second_end(
    d2: &Domain,
    v: f64,
    eps_x2: f64,
    (pos1, pair): (Position, &mut PointPair),
    flags: &mut EndFlags,
) -> Result<Position, Failure> {
    let pos2: Position = if (v - d2.first_parameter()?).abs() <= eps_x2 {
        flags.head_on2 = true;
        pair.p2 = d2.first_point()?;
        Position::Head
    } else if (v - d2.last_parameter()?).abs() <= eps_x2 {
        flags.end_on2 = true;
        pair.p2 = d2.last_point()?;
        Position::End
    } else {
        return Ok(Position::Middle);
    };
    if pos1 != Position::Middle {
        pair.p1 = Point2::new(
            0.5 * (pair.p1.x() + pair.p2.x()),
            0.5 * (pair.p1.y() + pair.p2.y()),
        );
    } else {
        pair.p2 = pair.p1;
    }
    Ok(pos2)
}

// Whether a result already joins these ends (PosSegment).
fn joined_already(pos1: Position, pos2: Position, pos_segment: i32) -> bool {
    match (pos1, pos2) {
        (Position::Head, Position::Head) => pos_segment & 1 != 0,
        (Position::Head, Position::End) => pos_segment & 2 != 0,
        (Position::End, Position::Head) => pos_segment & 4 != 0,
        (Position::End, Position::End) => pos_segment & 8 != 0,
        _ => false,
    }
}
