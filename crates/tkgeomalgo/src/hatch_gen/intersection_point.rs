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
// Rust translation of HatchGen_IntersectionPoint.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `HatchGen_IntersectionPoint`: the data shared by the points on a hatching and on
//! an element.
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkg3d::top_abs::{Orientation, State};
use std::fmt;

/// `HatchGen_IntersectionPoint`: the index of the supporting curve, the parameter on
/// it, the position of the point on it, the states before and after the point and
/// the segment flags. The native class is abstract with a protected constructor;
/// [`PointOnElement`](super::PointOnElement) and
/// [`PointOnHatching`](super::PointOnHatching) embed it.
///
/// ```
/// use rustycad_tkgeomalgo::hatch_gen::PointOnHatching;
/// let mut point = PointOnHatching::new();
/// point.set_parameter(0.5);
/// assert_eq!(point.parameter(), 0.5);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IntersectionPoint {
    pub(super) my_index: i32,
    pub(super) my_param: f64,
    pub(super) my_posit: Orientation,
    pub(super) my_before: State,
    pub(super) my_after: State,
    pub(super) my_seg_beg: bool,
    pub(super) my_seg_end: bool,
}

impl Default for IntersectionPoint {
    /// `HatchGen_IntersectionPoint()`: index 0, parameter RealLast, INTERNAL, both
    /// states UNKNOWN, no segment flag.
    fn default() -> Self {
        Self {
            my_index: 0,
            my_param: real_last(),
            my_posit: Orientation::Internal,
            my_before: State::Unknown,
            my_after: State::Unknown,
            my_seg_beg: false,
            my_seg_end: false,
        }
    }
}

impl IntersectionPoint {
    /// SetIndex: the index of the supporting curve; `point.set_index(2)`.
    pub fn set_index(&mut self, index: i32) {
        self.my_index = index;
    }
    /// Index: the index of the supporting curve; `point.index()`.
    pub fn index(&self) -> i32 {
        self.my_index
    }
    /// SetParameter: the parameter on the curve; `point.set_parameter(0.5)`.
    pub fn set_parameter(&mut self, parameter: f64) {
        self.my_param = parameter;
    }
    /// Parameter: the parameter on the curve; `point.parameter()`.
    pub fn parameter(&self) -> f64 {
        self.my_param
    }
    /// SetPosition: the position of the point on the curve (FORWARD at its start,
    /// INTERNAL inside, REVERSED at its end); `point.set_position(Orientation::Forward)`.
    pub fn set_position(&mut self, position: Orientation) {
        self.my_posit = position;
    }
    /// Position: the position of the point on the curve; `point.position()`.
    pub fn position(&self) -> Orientation {
        self.my_posit
    }
    /// SetStateBefore: the state before the intersection; `point.set_state_before(State::Out)`.
    pub fn set_state_before(&mut self, state: State) {
        self.my_before = state;
    }
    /// StateBefore: the state before the intersection; `point.state_before()`.
    pub fn state_before(&self) -> State {
        self.my_before
    }
    /// SetStateAfter: the state after the intersection; `point.set_state_after(State::In)`.
    pub fn set_state_after(&mut self, state: State) {
        self.my_after = state;
    }
    /// StateAfter: the state after the intersection; `point.state_after()`.
    pub fn state_after(&self) -> State {
        self.my_after
    }
    /// SetSegmentBeginning(State = true): the point begins a segment;
    /// `point.set_segment_beginning(true)`.
    pub fn set_segment_beginning(&mut self, state: bool) {
        self.my_seg_beg = state;
    }
    /// SegmentBeginning: the point begins a segment; `point.segment_beginning()`.
    pub fn segment_beginning(&self) -> bool {
        self.my_seg_beg
    }
    /// SetSegmentEnd(State = true): the point ends a segment; `point.set_segment_end(true)`.
    pub fn set_segment_end(&mut self, state: bool) {
        self.my_seg_end = state;
    }
    /// SegmentEnd: the point ends a segment; `point.segment_end()`.
    pub fn segment_end(&self) -> bool {
        self.my_seg_end
    }
}

/// The pure virtual `HatchGen_IntersectionPoint::Dump(Index = 0)`: writes the native
/// `std::cout` text of the point to `out`, with `index` in the title when positive.
///
/// ```
/// use rustycad_tkgeomalgo::hatch_gen::{IntersectionPointDump, PointOnElement};
/// let mut text = String::new();
/// PointOnElement::new().dump(1, &mut text).unwrap();
/// assert!(text.starts_with("    --- Point on element #   1 "));
/// ```
pub trait IntersectionPointDump {
    /// Dump(Index): the text of the point; `point.dump(0, &mut text)?`.
    fn dump(&self, index: i32, out: &mut dyn fmt::Write) -> fmt::Result;
}
