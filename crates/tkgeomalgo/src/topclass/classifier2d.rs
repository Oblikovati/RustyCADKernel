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
// Rust translation of TopClass_Classifier2d.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopClass_Classifier2d`: the state of a point from the closest intersection of a
//! segment starting at the point with the edges of a boundary.
use super::Classifier2dIntersector;
use crate::int_res2d::{IntersectionPoint, Position, Situation, TypeTrans};
use crate::toptrans::CurveTransition;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::{real_epsilon, real_last};
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkmath::gp::{Direction3, Line2};

/// `TopClass_Classifier2d` (the fields of its instances, such as
/// `Geom2dHatch_FClass2dOfClassifier`, and the template functions as methods): the
/// point is the origin of the line of [`Classifier2d::reset`]; each
/// [`Classifier2d::compare`] intersects the segment `[0, Parameter]` with an edge and
/// keeps the state given by the closest intersection, shortening the segment to it.
/// At an end of an edge the state comes from the complex transition of the edges
/// meeting there.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
/// use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
/// use rustycad_tkg3d::top_abs::{Orientation, State};
/// use rustycad_tkgeomalgo::geom2d_hatch::FClass2dOfClassifier;
/// use rustycad_tkmath::gp::{Direction2, Line2, Point2};
/// let up = Direction2::new(0.0, 1.0).unwrap();
/// let edge: Handle<dyn Geom2dCurve> =
///     Handle::new(Geom2dLine::from_point(Point2::new(1.0, -1.0), up)).cast();
/// let edge = Geom2dAdaptorCurve::with_range(&edge, 0.0, 2.0).unwrap();
/// let mut classifier = FClass2dOfClassifier::new();
/// let x_axis = Direction2::new(1.0, 0.0).unwrap();
/// classifier.reset(Line2::new(Point2::new(0.0, 0.0), x_axis), 10.0, 1e-7);
/// classifier.compare(&edge, Orientation::Forward).unwrap();
/// assert_eq!(classifier.state(), State::In);
/// assert_eq!(classifier.parameter(), 1.0);
/// ```
pub struct Classifier2d<I: Classifier2dIntersector> {
    // Written by Reset and never read, as natively.
    #[allow(dead_code)]
    my_is_set: bool,
    my_first_compare: bool,
    my_first_trans: bool,
    my_lin: Line2,
    my_param: f64,
    my_tolerance: f64,
    my_trans: CurveTransition,
    my_intersector: I,
    my_closest: i32,
    my_state: State,
    my_is_head_or_end: bool,
}

// The closest intersection along the segment: the point is ON the boundary, or the
// closest point (with its parameter on the segment), or nothing.
enum Closest {
    On,
    At(IntersectionPoint, f64),
    Nothing,
}

impl<I: Classifier2dIntersector> Default for Classifier2d<I> {
    /// The undefined classifier: UNKNOWN, no closest intersection, a default line.
    fn default() -> Self {
        Self {
            my_is_set: false,
            my_first_compare: true,
            my_first_trans: true,
            my_lin: Line2::default(),
            my_param: 0.0,
            my_tolerance: 0.0,
            my_trans: CurveTransition::new(),
            my_intersector: I::default(),
            my_closest: 0,
            my_state: State::Unknown,
            my_is_head_or_end: false,
        }
    }
}

impl<I: Classifier2dIntersector> Classifier2d<I> {
    /// The undefined classifier (`Geom2dHatch_FClass2dOfClassifier()`);
    /// `FClass2dOfClassifier::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Reset(L, P, Tol)`: starts the classification of the origin of `lin` with the
    /// segment `[0, param]` and the segment tolerance `tolerance`; the state is
    /// UNKNOWN; `classifier.reset(line, 10.0, 1e-7)`.
    pub fn reset(&mut self, lin: Line2, param: f64, tolerance: f64) {
        self.my_lin = lin;
        self.my_param = param;
        self.my_tolerance = tolerance;
        self.my_state = State::Unknown;
        self.my_first_compare = true;
        self.my_first_trans = true;
        self.my_closest = 0;
        self.my_is_set = true;
        self.my_is_head_or_end = false;
    }

    /// `Compare(E, Or)`: updates the state with the edge `edge` of orientation
    /// `orientation`; the intersector's exceptions are returned;
    /// `classifier.compare(&edge, Orientation::Forward)?`.
    pub fn compare(&mut self, edge: &I::Edge, orientation: Orientation) -> Result<(), Failure> {
        self.my_closest = 0;
        let (lin, param, tolerance) = (self.my_lin, self.my_param, self.my_tolerance);
        self.my_intersector
            .perform_segment(&lin, param, tolerance, edge)?;
        let results = self.my_intersector.results();
        if !results.is_done() || (results.nb_points()? == 0 && results.nb_segments()? == 0) {
            return Ok(());
        }
        let (closest, d_min) = match self.find_closest()? {
            Closest::On => return Ok(()),
            Closest::Nothing => return Ok(()),
            Closest::At(point, d_min) => (point, d_min),
        };
        if orientation == Orientation::Internal {
            self.my_state = State::In;
            return Ok(());
        }
        if orientation == Orientation::External {
            self.my_state = State::Out;
            return Ok(());
        }
        if !self.my_first_compare && d_min > self.my_param {
            return Ok(());
        }
        self.process_closest(edge, orientation, &closest, d_min)
    }

    // The closest point of the results: a point at the head of the segment makes
    // the state ON; for the segments only the first point counts. The index of the
    // closest intersection counts the points, then two per segment.
    fn find_closest(&mut self) -> Result<Closest, Failure> {
        let mut d_min: f64 = real_last();
        let mut found: Option<(IntersectionPoint, f64)> = None;
        for (point, index) in self.candidates()? {
            if self.seen_on_or_closer(&point, index, &mut d_min, &mut found) {
                return Ok(Closest::On);
            }
        }
        if self.my_closest == 0 {
            return Ok(Closest::Nothing);
        }
        Ok(found.map_or(Closest::Nothing, |(point, d)| Closest::At(point, d)))
    }

    // The points of the results, then the first point of each segment, with their
    // closest-intersection indices, in the native scan order.
    fn candidates(&self) -> Result<Vec<(IntersectionPoint, i32)>, Failure> {
        let results = self.my_intersector.results();
        let nb_points: i32 = results.nb_points()?;
        let mut candidates = Vec::new();
        for index in 1..=nb_points {
            candidates.push((results.point(index)?, index));
        }
        for segment in 1..=results.nb_segments()? {
            let first = results.segment(segment)?.first_point()?;
            candidates.push((first, nb_points + segment + segment - 1));
        }
        Ok(candidates)
    }

    // One candidate of `find_closest`: true when it is at the head of the segment
    // (the state is then ON), else it becomes the closest one when it is closer.
    fn seen_on_or_closer(
        &mut self,
        point: &IntersectionPoint,
        index: i32,
        d_min: &mut f64,
        found: &mut Option<(IntersectionPoint, f64)>,
    ) -> bool {
        if point.transition_of_first().position_on_curve() == Position::Head {
            self.my_closest = index;
            self.my_state = State::On;
            return true;
        }
        let param_first: f64 = point.param_on_first();
        if param_first < *d_min {
            self.my_closest = index;
            *found = Some((*point, param_first));
            *d_min = param_first;
        }
        false
    }

    // Processes the closest point, found at `d_min` on the line.
    fn process_closest(
        &mut self,
        edge: &I::Edge,
        orientation: Orientation,
        closest: &IntersectionPoint,
        d_min: f64,
    ) -> Result<(), Failure> {
        self.my_first_compare = false;
        if self.my_param > d_min {
            self.my_first_trans = true;
        }
        self.my_param = d_min;
        let position_on_edge: Position = closest.transition_of_second().position_on_curve();
        self.my_is_head_or_end = matches!(position_on_edge, Position::Head | Position::End);
        let Some(seg_trans) = segment_transition(closest, orientation)? else {
            return Ok(());
        };
        if !self.my_is_head_or_end {
            self.my_state = match seg_trans {
                Orientation::Forward | Orientation::External => State::Out,
                Orientation::Reversed | Orientation::Internal => State::In,
            };
            return Ok(());
        }
        self.compare_at_vertex(edge, closest, seg_trans, position_on_edge)
    }

    // The closest point is the head or the end of the edge: updates the complex
    // transition of the edges meeting there.
    fn compare_at_vertex(
        &mut self,
        edge: &I::Edge,
        closest: &IntersectionPoint,
        seg_trans: Orientation,
        position_on_edge: Position,
    ) -> Result<(), Failure> {
        let (tang2d, norm2d, curv) = self
            .my_intersector
            .local_geometry(edge, closest.param_on_second())?;
        let tang = Direction3::new(tang2d.x(), tang2d.y(), 0.0)?;
        let norm = Direction3::new(norm2d.x(), norm2d.y(), 0.0)?;
        if self.my_first_trans {
            let lin_dir = self.my_lin.direction();
            self.my_trans
                .reset_line(Direction3::new(lin_dir.x(), lin_dir.y(), 0.0)?);
            self.my_first_trans = false;
        }
        let ort = if position_on_edge == Position::Head {
            Orientation::Forward
        } else {
            Orientation::Reversed
        };
        self.my_trans
            .compare(real_epsilon(), tang, norm, curv, seg_trans, ort);
        self.my_state = self.my_trans.state_before();
        Ok(())
    }

    /// Parameter(): the current length of the segment; `classifier.parameter()`.
    pub fn parameter(&self) -> f64 {
        self.my_param
    }

    /// Intersector(): the intersection algorithm; `classifier.intersector()`.
    pub fn intersector(&mut self) -> &mut I {
        &mut self.my_intersector
    }

    /// ClosestIntersection(): 0 when the last compared edge had no relevant
    /// intersection, else the index of that intersection in the intersector (the
    /// points first, then two per segment); `classifier.closest_intersection()`.
    pub fn closest_intersection(&self) -> i32 {
        self.my_closest
    }

    /// State(): the current state of the point; `classifier.state()`.
    pub fn state(&self) -> State {
        self.my_state
    }

    /// IsHeadOrEnd(): the closest intersection is at the head or the end of the
    /// edge; `classifier.is_head_or_end()`.
    pub fn is_head_or_end(&self) -> bool {
        self.my_is_head_or_end
    }
}

// The transition of the segment at the closest point from the hatching side of its
// IntRes2d transition; None where the native code returns (an unknown touch, an
// undecided transition).
fn segment_transition(
    closest: &IntersectionPoint,
    orientation: Orientation,
) -> Result<Option<Orientation>, Failure> {
    let reversed: bool = orientation == Orientation::Reversed;
    let pick = |if_reversed: Orientation, otherwise: Orientation| {
        Some(if reversed { if_reversed } else { otherwise })
    };
    let t1 = closest.transition_of_first();
    Ok(match t1.transition_type() {
        TypeTrans::In => pick(Orientation::Reversed, Orientation::Forward),
        TypeTrans::Out => pick(Orientation::Forward, Orientation::Reversed),
        TypeTrans::Touch => match t1.situation()? {
            Situation::Inside => pick(Orientation::External, Orientation::Internal),
            Situation::Outside => pick(Orientation::Internal, Orientation::External),
            Situation::Unknown => None,
        },
        TypeTrans::Undecided => None,
    })
}
