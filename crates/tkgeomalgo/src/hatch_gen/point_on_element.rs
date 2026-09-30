// Rust translation of HatchGen_PointOnElement.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `HatchGen_PointOnElement`: an intersection point seen on an element.
use super::text::{index_title, write_fields};
use super::{IntersectionPoint, IntersectionPointDump, IntersectionType};
use crate::int_res2d::{self, Position, Situation, Transition, TypeTrans};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, State};
use std::fmt;
use std::ops::{Deref, DerefMut};

/// `HatchGen_PointOnElement`: an [`IntersectionPoint`] on an element of the hatcher,
/// with the kind of intersection there.
///
/// ```
/// use rustycad_tkgeomalgo::hatch_gen::{IntersectionType, PointOnElement};
/// let point = PointOnElement::new();
/// assert_eq!(point.intersection_type(), IntersectionType::Undetermined);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PointOnElement {
    base: IntersectionPoint,
    my_type: IntersectionType,
}

impl Deref for PointOnElement {
    type Target = IntersectionPoint;
    fn deref(&self) -> &IntersectionPoint {
        &self.base
    }
}

impl DerefMut for PointOnElement {
    fn deref_mut(&mut self) -> &mut IntersectionPoint {
        &mut self.base
    }
}

impl PointOnElement {
    /// `HatchGen_PointOnElement()`: an empty point of UNDETERMINED type;
    /// `PointOnElement::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `HatchGen_PointOnElement(Point)`: the point on the second curve (the element)
    /// of an intersection whose first curve is the hatching. The position comes from
    /// the element's transition, the states and the type from the hatching's; a
    /// TOUCH of the hatching reads `IsOpposite` of the element's transition, which
    /// raises Standard_DomainError when that one is not a TOUCH;
    /// `PointOnElement::from_intersection_point(&point)?`.
    pub fn from_intersection_point(point: &int_res2d::IntersectionPoint) -> Result<Self, Failure> {
        let trs_h: Transition = point.transition_of_first();
        let trs_e: Transition = point.transition_of_second();
        let mut result = Self::new();
        result.base.my_index = 0;
        result.base.my_param = point.param_on_second();
        result.base.my_posit = position_orientation(trs_e.position_on_curve());
        result.set_from_hatching_transition(&trs_h, &trs_e)?;
        result.base.my_seg_beg = false;
        result.base.my_seg_end = false;
        Ok(result)
    }

    // The switch on the hatching's transition type.
    fn set_from_hatching_transition(
        &mut self,
        trs_h: &Transition,
        trs_e: &Transition,
    ) -> Result<(), Failure> {
        let crossing: IntersectionType = if self.base.my_posit == Orientation::Internal {
            IntersectionType::True
        } else {
            IntersectionType::Touch
        };
        match trs_h.transition_type() {
            TypeTrans::In => self.set_states(State::Out, State::In, crossing),
            TypeTrans::Out => self.set_states(State::In, State::Out, crossing),
            TypeTrans::Touch => self.set_touch(trs_h.situation()?, trs_e)?,
            TypeTrans::Undecided => self.set_states(
                State::Unknown,
                State::Unknown,
                IntersectionType::Undetermined,
            ),
        }
        Ok(())
    }

    fn set_states(&mut self, before: State, after: State, kind: IntersectionType) {
        self.base.my_before = before;
        self.base.my_after = after;
        self.my_type = kind;
    }

    // Modified by Sergey KHROMOV - Fri Jan  5 2001: the tangent cases. The states of
    // an Inside touch are those of an Outside one swapped.
    fn set_touch(&mut self, situation: Situation, trs_e: &Transition) -> Result<(), Failure> {
        self.my_type = IntersectionType::Tangent;
        let (at_middle, crossing) = match situation {
            Situation::Inside => ((State::In, State::In), (State::In, State::Out)),
            Situation::Outside => ((State::Out, State::Out), (State::Out, State::In)),
            Situation::Unknown => {
                self.base.my_before = State::Unknown;
                self.base.my_after = State::Unknown;
                return Ok(());
            }
        };
        let (before, after) = match self.base.my_posit {
            Orientation::Forward if trs_e.is_opposite()? => crossing,
            Orientation::Forward => (crossing.1, crossing.0),
            Orientation::Internal => at_middle,
            Orientation::Reversed if trs_e.is_opposite()? => (crossing.1, crossing.0),
            Orientation::Reversed => crossing,
            Orientation::External => return Ok(()),
        };
        self.base.my_before = before;
        self.base.my_after = after;
        Ok(())
    }

    /// SetIntersectionType; `point.set_intersection_type(IntersectionType::True)`.
    pub fn set_intersection_type(&mut self, kind: IntersectionType) {
        self.my_type = kind;
    }

    /// IntersectionType; `point.intersection_type()`.
    pub fn intersection_type(&self) -> IntersectionType {
        self.my_type
    }

    /// IsIdentical(Point, Confusion): the parameters differ by at most `confusion`
    /// and every other field is equal; `a.is_identical(&b, 1e-9)`.
    pub fn is_identical(&self, point: &PointOnElement, confusion: f64) -> bool {
        let delta: f64 = (self.base.my_param - point.base.my_param).abs();
        (delta <= confusion) && self.same_fields(point)
    }

    /// IsDifferent(Point, Confusion): the parameters differ by more than `confusion`
    /// or another field differs; `a.is_different(&b, 1e-9)`.
    pub fn is_different(&self, point: &PointOnElement, confusion: f64) -> bool {
        let delta: f64 = (self.base.my_param - point.base.my_param).abs();
        (delta > confusion) || !self.same_fields(point)
    }

    // Every field but the parameter: index, position, type, states, segment flags.
    fn same_fields(&self, point: &PointOnElement) -> bool {
        let (a, b) = (&self.base, &point.base);
        a.my_index == b.my_index
            && a.my_posit == b.my_posit
            && self.my_type == point.my_type
            && a.my_before == b.my_before
            && a.my_after == b.my_after
            && a.my_seg_beg == b.my_seg_beg
            && a.my_seg_end == b.my_seg_end
    }
}

/// The FORWARD/INTERNAL/REVERSED position of an IntRes2d head/middle/end.
pub(crate) fn position_orientation(position: Position) -> Orientation {
    match position {
        Position::Head => Orientation::Forward,
        Position::Middle => Orientation::Internal,
        Position::End => Orientation::Reversed,
    }
}

fn type_text(kind: IntersectionType) -> &'static str {
    match kind {
        IntersectionType::True => "TRUE",
        IntersectionType::Touch => "TOUCH",
        IntersectionType::Tangent => "TANGENT",
        IntersectionType::Undetermined => "UNDETERMINED",
    }
}

impl IntersectionPointDump for PointOnElement {
    /// Dump(Index): the native text of the point on element.
    fn dump(&self, index: i32, out: &mut dyn fmt::Write) -> fmt::Result {
        let title = index_title(index, "------");
        writeln!(out, "    --- Point on element {title}---------------")?;
        let fields = self
            .base
            .dump_fields("element", Some(type_text(self.my_type)));
        write_fields(out, "        ", 20, &fields)?;
        writeln!(out, "    ------------------------------------------")
    }
}
