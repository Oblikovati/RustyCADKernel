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
// Rust translation of Geom2dHatch_Hatcher.cxx (category Computation - Trimming) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The trimming of the hatchings by the elements: the intersection points of each
//! hatching and their global transitions.
use super::{Hatcher, Hatching, Intersector};
use crate::hatch_gen::{ErrorStatus, IntersectionType, PointOnElement, PointOnHatching};
use crate::int_res2d::{IntersectionPoint, IntersectionSegment, Position, TypeTrans};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::top_abs::{Orientation, State};

impl Hatcher {
    /// Trim(): trims every hatching by every element; `hatcher.trim()?`.
    pub fn trim(&mut self) -> Result<(), Failure> {
        for ind_h in 1..=self.my_nb_hatchings {
            if self.my_hatchings.is_bound(&ind_h) {
                self.trim_hatching(ind_h)?;
            }
        }
        Ok(())
    }

    /// Trim(Curve): adds a hatching, trims it by the elements and returns its
    /// index; `hatcher.trim_curve(adaptor)?`.
    pub fn trim_curve(&mut self, curve: Geom2dAdaptorCurve) -> Result<i32, Failure> {
        let ind_h: i32 = self.add_hatching(curve);
        self.trim_hatching(ind_h)?;
        Ok(ind_h)
    }

    /// Trim(IndH): recomputes the intersection points of the hatching with every
    /// element, then (when every intersection is done) the global transition at
    /// each point: the status is NoProblem, TrimFailure or TransitionFailure;
    /// Standard_NoSuchObject for an unbound index; `hatcher.trim_hatching(1)?`.
    pub fn trim_hatching(&mut self, ind_h: i32) -> Result<(), Failure> {
        self.hatching_mut(ind_h)?.clr_points();
        let mut all_ok: bool = true;
        for ind_e in 1..=self.my_nb_elements {
            if self.my_elements.is_bound(ind_e) {
                let ok: bool = self.trim_with(ind_h, ind_e)?;
                all_ok = all_ok && ok;
            }
        }
        let hatching = self.hatching_mut(ind_h)?;
        hatching.set_trim_done(true);
        hatching.set_trim_failed(!all_ok);
        if all_ok {
            self.set_global_transitions(ind_h)?;
        }
        Ok(())
    }

    // The global transition of every point of the hatching, then its status.
    fn set_global_transitions(&mut self, ind_h: i32) -> Result<(), Failure> {
        let mut all_ok: bool = true;
        for ipnt in 1..=self.hatching(ind_h)?.nb_points() {
            let transition = self.global_transition(self.point(ind_h, ipnt)?)?;
            let point: &mut PointOnHatching = self.hatching_mut(ind_h)?.change_point(ipnt)?;
            let ok: bool = transition.is_some_and(|states| states.apply(point));
            all_ok = all_ok && ok;
        }
        let status = if all_ok {
            ErrorStatus::NoProblem
        } else {
            ErrorStatus::TransitionFailure
        };
        self.hatching_mut(ind_h)?.set_status(status);
        Ok(())
    }

    // Trim(IndH, IndE): the intersection points of the hatching with one element;
    // false when the intersection is not done (natively reported on std::cout).
    fn trim_with(&mut self, ind_h: i32, ind_e: i32) -> Result<bool, Failure> {
        let hatching_curve: &Geom2dAdaptorCurve = self.my_hatchings.find(&ind_h)?.curve();
        let element_curve: &Geom2dAdaptorCurve = self.my_elements.find(ind_e)?.curve();
        self.my_intersector
            .intersect(hatching_curve, element_curve)?;
        if !self.my_intersector.is_done() {
            return Ok(false);
        }
        if self.my_intersector.is_empty()? {
            return Ok(true);
        }
        let (points, segments) = intersection_results(&self.my_intersector)?;
        let marks = Marks {
            ind_h,
            ind_e,
            confusion2d: self.my_confusion2d,
        };
        let confusion3d: f64 = self.my_confusion3d;
        let hatching: &mut Hatching = self.hatching_mut(ind_h)?;
        for point in &points {
            marks.add_point(hatching, point, (false, false))?;
        }
        for segment in &segments {
            marks.add_segment(hatching, segment, confusion3d)?;
        }
        Ok(true)
    }
}

// The points and the segments of a done intersection.
fn intersection_results(
    intersector: &Intersector,
) -> Result<(Vec<IntersectionPoint>, Vec<IntersectionSegment>), Failure> {
    let points = (1..=intersector.nb_points()?)
        .map(|index| intersector.point(index))
        .collect::<Result<_, _>>()?;
    let segments = (1..=intersector.nb_segments()?)
        .map(|index| intersector.segment(index))
        .collect::<Result<_, _>>()?;
    Ok((points, segments))
}

// The indices of the hatching and of the element of one trimming, and the 2d
// confusion of the point merges.
struct Marks {
    ind_h: i32,
    ind_e: i32,
    confusion2d: f64,
}

impl Marks {
    // An intersection point as a point on the hatching holding its point on the
    // element, with the segment beginning and end flags of the element point.
    fn add_point(
        &self,
        hatching: &mut Hatching,
        point: &IntersectionPoint,
        (seg_beg, seg_end): (bool, bool),
    ) -> Result<(), Failure> {
        let mut pnt_e = PointOnElement::from_intersection_point(point)?;
        pnt_e.set_index(self.ind_e);
        if seg_beg || seg_end {
            pnt_e.set_segment_beginning(seg_beg);
            pnt_e.set_segment_end(seg_end);
        }
        let mut pnt_h = PointOnHatching::from_intersection_point(point);
        pnt_h.set_index(self.ind_h);
        self.merge(hatching, pnt_h, &pnt_e);
        Ok(())
    }

    fn merge(&self, hatching: &mut Hatching, mut pnt_h: PointOnHatching, pnt_e: &PointOnElement) {
        pnt_h.add_point(pnt_e, self.confusion2d);
        hatching.add_point(&pnt_h, self.confusion2d);
    }

    // An intersection segment: one point when its ends are confused (in the
    // parametric space of the hatching, or in the plane for two crossing ends),
    // else its two ends. A segment missing an end is ignored: natively both end
    // treatments are inside the test that both ends exist.
    fn add_segment(
        &self,
        hatching: &mut Hatching,
        segment: &IntersectionSegment,
        confusion3d: f64,
    ) -> Result<(), Failure> {
        if !(segment.has_first_point() && segment.has_last_point()) {
            return Ok(());
        }
        let (pnt1, pnt2) = (segment.first_point()?, segment.last_point()?);
        if self.ends_confused(&pnt1, &pnt2, confusion3d) {
            let (pnt_h, pnt_e) = confused_points(&pnt1, &pnt2, self.ind_h, self.ind_e);
            self.merge(hatching, pnt_h, &pnt_e);
            return Ok(());
        }
        self.add_point(hatching, &pnt1, (true, false))?;
        self.add_point(hatching, &pnt2, (false, true))
    }

    // Conf2d: the parameters on the hatching are within Confusion2d; Conf3d: both
    // ends are crossings of the same types, within Confusion3d in the plane.
    fn ends_confused(
        &self,
        pnt1: &IntersectionPoint,
        pnt2: &IntersectionPoint,
        confusion3d: f64,
    ) -> bool {
        let conf2d: bool =
            (pnt1.param_on_first() - pnt2.param_on_first()).abs() <= self.confusion2d;
        if conf2d {
            return true;
        }
        let types = [
            pnt1.transition_of_first().transition_type(),
            pnt1.transition_of_second().transition_type(),
            pnt2.transition_of_first().transition_type(),
            pnt2.transition_of_second().transition_type(),
        ];
        types
            .iter()
            .all(|kind| *kind != TypeTrans::Touch && *kind != TypeTrans::Undecided)
            && types[0] == types[2]
            && types[1] == types[3]
            && pnt1.value().distance(pnt2.value()) <= confusion3d
    }
}

// The single point of a segment whose ends are confused: the mean parameters, the
// position from the ends' positions, the states from the hatching's types.
fn confused_points(
    pnt1: &IntersectionPoint,
    pnt2: &IntersectionPoint,
    ind_h: i32,
    ind_e: i32,
) -> (PointOnHatching, PointOnElement) {
    let pnt_e = confused_element_point(pnt1, pnt2, ind_e);
    let (trs1_h, trs2_h) = (pnt1.transition_of_first(), pnt2.transition_of_first());
    let mut pnt_h = PointOnHatching::new();
    pnt_h.set_index(ind_h);
    pnt_h.set_parameter((pnt1.param_on_first() + pnt2.param_on_first()) / 2.0);
    let (first, second) = (trs1_h.position_on_curve(), trs2_h.position_on_curve());
    set_merged_position(&mut pnt_h, first, second);
    (pnt_h, pnt_e)
}

// The point on element of a confused segment: TRUE inside the element, TOUCH at
// its ends.
fn confused_element_point(
    pnt1: &IntersectionPoint,
    pnt2: &IntersectionPoint,
    ind_e: i32,
) -> PointOnElement {
    let mut pnt_e = PointOnElement::new();
    pnt_e.set_index(ind_e);
    pnt_e.set_parameter((pnt1.param_on_second() + pnt2.param_on_second()) / 2.0);
    let first = pnt1.transition_of_second().position_on_curve();
    set_merged_position(
        &mut pnt_e,
        first,
        pnt2.transition_of_second().position_on_curve(),
    );
    let kind = if pnt_e.position() == Orientation::Internal {
        IntersectionType::True
    } else {
        IntersectionType::Touch
    };
    pnt_e.set_intersection_type(kind);
    pnt_e.set_state_before(state_of(pnt1.transition_of_first().transition_type()));
    pnt_e.set_state_after(state_of(pnt2.transition_of_first().transition_type()));
    pnt_e
}

// OUT for an IN type, IN otherwise (the native ternaries of the confused segment).
fn state_of(kind: TypeTrans) -> State {
    if kind == TypeTrans::In {
        State::Out
    } else {
        State::In
    }
}

// The position of a confused segment: the first end's head or end, else the
// second end's position.
fn set_merged_position(
    point: &mut crate::hatch_gen::IntersectionPoint,
    first: Position,
    second: Position,
) {
    let position = match (first, second) {
        (Position::Head, _) | (Position::Middle, Position::Head) => Orientation::Forward,
        (Position::Middle, Position::Middle) => Orientation::Internal,
        (Position::End, _) | (Position::Middle, Position::End) => Orientation::Reversed,
    };
    point.set_position(position);
}
