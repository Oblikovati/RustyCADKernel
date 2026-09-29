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
// Rust translation of Geom2dHatch_Hatcher::GlobalTransition at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The global transition of a hatching point: the states before and after it from
//! the complex transition of the elements meeting there.
use super::{Element, Hatcher};
use crate::hatch_gen::{IntersectionType, PointOnElement, PointOnHatching};
use crate::toptrans::CurveTransition;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkmath::gp::Direction3;

/// The result of a successful GlobalTransition: the states and the segment flags
/// to set on the point.
pub(super) struct GlobalStates {
    before: State,
    after: State,
    segment_begin: bool,
    segment_end: bool,
}

impl GlobalStates {
    // Writes the states and flags on the point; always true (the success).
    pub(super) fn apply(self, point: &mut PointOnHatching) -> bool {
        point.set_state_before(self.before);
        point.set_state_after(self.after);
        point.set_segment_beginning(self.segment_begin);
        point.set_segment_end(self.segment_end);
        true
    }
}

impl Hatcher {
    // GlobalTransition(Point): the complex transition of the hatching at the point,
    // compared with each element meeting it; None when a state is ON or UNKNOWN
    // (the native false, the point unchanged).
    pub(super) fn global_transition(
        &self,
        point: &PointOnHatching,
    ) -> Result<Option<GlobalStates>, Failure> {
        let mut complex = self.hatching_transition(point)?;
        let (mut segment_begin, mut segment_end) = (false, false);
        for pnt_e in point.element_points() {
            segment_begin = segment_begin || pnt_e.segment_beginning();
            segment_end = segment_end || pnt_e.segment_end();
            self.compare_element(&mut complex, pnt_e)?;
        }
        Ok(defined(complex.state_before())
            .zip(defined(complex.state_after()))
            .map(|(before, after)| GlobalStates {
                before,
                after,
                segment_begin,
                segment_end,
            }))
    }

    // The complex transition reset with the hatching's tangent, normal (the tangent
    // turned left under a Confusion curvature) and curvature at the point.
    fn hatching_transition(&self, point: &PointOnHatching) -> Result<CurveTransition, Failure> {
        let curve_h = self.hatching_curve(point.index())?;
        let (tang2d, norm2d, curv) = self
            .my_intersector
            .local_geometry(curve_h, point.parameter())?;
        let tangent = Direction3::new(tang2d.x(), tang2d.y(), 0.0)?;
        let normal = if curv < precision::confusion() {
            Direction3::new(-tang2d.y(), tang2d.x(), 0.0)?
        } else {
            Direction3::new(norm2d.x(), norm2d.y(), 0.0)?
        };
        let mut complex = CurveTransition::new();
        complex.reset(tangent, normal, curv);
        Ok(complex)
    }

    // One element meeting the point, compared in the complex transition.
    fn compare_element(
        &self,
        complex: &mut CurveTransition,
        pnt_e: &PointOnElement,
    ) -> Result<(), Failure> {
        let element: &Element = self.my_elements.find(pnt_e.index())?;
        let to_reverse: bool = element.orientation() == Orientation::Reversed;
        // TODO: finding #25 upstream: the native code first takes the element's end
        // parameter for a FORWARD or REVERSED position, then overwrites it with the
        // point's parameter (removing the overwrite caused blend regressions).
        let param: f64 = pnt_e.parameter();
        let (tang2d, norm2d, curv) = self.my_intersector.local_geometry(element.curve(), param)?;
        let local = local_transition(element.orientation(), pnt_e, to_reverse);
        let tangent_orientation = tangent_orientation(pnt_e.position(), to_reverse);
        let tangent = if to_reverse {
            Direction3::new(-tang2d.x(), -tang2d.y(), 0.0)?
        } else {
            Direction3::new(tang2d.x(), tang2d.y(), 0.0)?
        };
        let normal = Direction3::new(norm2d.x(), norm2d.y(), 0.0)?;
        complex.compare(
            precision::angular(),
            tangent,
            normal,
            curv,
            local,
            tangent_orientation,
        );
        Ok(())
    }
}

// IN or OUT; None for ON and UNKNOWN (the native returns of false).
fn defined(state: State) -> Option<State> {
    matches!(state, State::In | State::Out).then_some(state)
}

// The local transition of the hatching across the element: INTERNAL and EXTERNAL
// elements give their orientation; a tangency gives IN-IN or OUT-OUT
// (INTERNAL/EXTERNAL) inside the element and REVERSED/FORWARD at its ends; a crossing
// gives REVERSED/FORWARD; all with the element seen FORWARD. An UNKNOWN or ON state
// before keeps EXTERNAL.
fn local_transition(
    orientation: Orientation,
    pnt_e: &PointOnElement,
    to_reverse: bool,
) -> Orientation {
    if orientation == Orientation::Internal || orientation == Orientation::External {
        return orientation;
    }
    let pick = |if_reversed: Orientation, otherwise: Orientation| {
        if to_reverse { if_reversed } else { otherwise }
    };
    let tangent_inside: bool = pnt_e.intersection_type() == IntersectionType::Tangent
        && pnt_e.position() == Orientation::Internal;
    match (pnt_e.state_before(), tangent_inside) {
        (State::In, true) => pick(Orientation::External, Orientation::Internal),
        (State::Out, true) => pick(Orientation::Internal, Orientation::External),
        (State::In, false) => pick(Orientation::Forward, Orientation::Reversed),
        (State::Out, false) => pick(Orientation::Reversed, Orientation::Forward),
        _ => Orientation::External,
    }
}

// The orientation of the element's tangent at the point: FORWARD at its start,
// REVERSED at its end (both swapped for a REVERSED element), INTERNAL inside.
fn tangent_orientation(position: Orientation, to_reverse: bool) -> Orientation {
    match position {
        Orientation::Forward if to_reverse => Orientation::Reversed,
        Orientation::Reversed if to_reverse => Orientation::Forward,
        Orientation::External => Orientation::Forward,
        other => other,
    }
}
