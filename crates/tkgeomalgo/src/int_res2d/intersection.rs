// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of IntRes2d_Intersection.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntRes2d_Intersection`: the result lists shared by every 2D curve intersection.
use super::failures::not_done;
use super::{IntersectionPoint, IntersectionSegment, Position, Transition, TypeTrans};
use rustycad_tkernel::standard::Failure;

// `PARAMEQUAL(a, b)`: parameters closer than 1e-8.
fn param_equal(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-8
}

/// `IntRes2d_Intersection`: the intersection points and segments of two 2D curves.
/// The algorithms own one and expose it through `Deref`; the protected native
/// members are public for algorithms of other crates.
///
/// `Clone` is the native copy constructor: the copy has the points and segments but
/// is not done and not reversed (IntRes2d_Intersection.lxx); [`Intersection::assign`]
/// is the native assignment, which copies the flags too.
///
/// ```
/// use rustycad_tkgeomalgo::int_res2d::Intersection;
/// let inter = Intersection::new();
/// assert!(!inter.is_done() && inter.nb_points().is_err());
/// ```
#[derive(Debug, Default)]
pub struct Intersection {
    lpnt: Vec<IntersectionPoint>,
    lseg: Vec<IntersectionSegment>,
    /// `done`: the computation succeeded (protected native field).
    pub done: bool,
    reverse: bool,
}

impl Clone for Intersection {
    fn clone(&self) -> Self {
        Self {
            lpnt: self.lpnt.clone(),
            lseg: self.lseg.clone(),
            done: false,
            reverse: false,
        }
    }
}

impl Intersection {
    /// `IntRes2d_Intersection()`: not done, not reversed. `Intersection::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `operator=`: copies the lists and both flags. `target.assign(&source)`.
    pub fn assign(&mut self, other: &Intersection) {
        self.lpnt.clone_from(&other.lpnt);
        self.lseg.clone_from(&other.lseg);
        self.done = other.done;
        self.reverse = other.reverse;
    }
    /// `IsDone()`: the computation succeeded. `inter.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// `IsEmpty()`: no point and no segment; StdFail_NotDone when not done.
    /// `inter.is_empty()?`.
    pub fn is_empty(&self) -> Result<bool, Failure> {
        self.check_done("IntRes2d_Intersection::IsEmpty")?;
        Ok(self.lpnt.is_empty() && self.lseg.is_empty())
    }
    /// `NbPoints()`; StdFail_NotDone when not done. `inter.nb_points()?`.
    pub fn nb_points(&self) -> Result<i32, Failure> {
        self.check_done("IntRes2d_Intersection::NbPoints")?;
        Ok(self.lpnt.len() as i32)
    }
    /// `Point(N)`: the point of rank `n` (1-based); StdFail_NotDone when not done,
    /// Standard_OutOfRange outside `1..=NbPoints()`. `inter.point(1)?`.
    pub fn point(&self, n: i32) -> Result<IntersectionPoint, Failure> {
        self.check_done("IntRes2d_Intersection::Point")?;
        rank(&self.lpnt, n, "IntRes2d_Intersection::Point").copied()
    }
    /// `NbSegments()`; StdFail_NotDone when not done. `inter.nb_segments()?`.
    pub fn nb_segments(&self) -> Result<i32, Failure> {
        self.check_done("IntRes2d_Intersection::NbSegments")?;
        Ok(self.lseg.len() as i32)
    }
    /// `Segment(N)`: the segment of rank `n` (1-based); StdFail_NotDone when not
    /// done, Standard_OutOfRange outside `1..=NbSegments()`. `inter.segment(1)?`.
    pub fn segment(&self, n: i32) -> Result<IntersectionSegment, Failure> {
        self.check_done("IntRes2d_Intersection::Segment")?;
        rank(&self.lseg, n, "IntRes2d_Intersection::Segment").copied()
    }
    /// `SetReversedParameters(Reverseflag)`: the next computation swaps the roles of
    /// the curves in its results. `inter.set_reversed_parameters(true)`.
    pub fn set_reversed_parameters(&mut self, flag: bool) {
        self.reverse = flag;
    }
    /// `ReversedParameters()` (protected). `inter.reversed_parameters()`.
    pub fn reversed_parameters(&self) -> bool {
        self.reverse
    }
    /// `Append(Seg)` (protected): adds a segment at the end. `inter.append_segment(seg)`.
    pub fn append_segment(&mut self, segment: IntersectionSegment) {
        self.lseg.push(segment);
    }
    /// `Append(Pnt)` (protected): adds a point at the end. `inter.append_point(point)`.
    pub fn append_point(&mut self, point: IntersectionPoint) {
        self.lpnt.push(point);
    }
    /// `ResetFields()` (protected): empties a done result and marks it not done.
    /// `inter.reset_fields()`.
    pub fn reset_fields(&mut self) {
        if self.done {
            self.lseg.clear();
            self.lpnt.clear();
            self.done = false;
        }
    }
    /// `SetValues(Inter)` (protected): copies the result of a done intersection, or
    /// marks this one not done. `inter.set_values(&other)`.
    pub fn set_values(&mut self, other: &Intersection) {
        if !other.done {
            self.done = false;
            return;
        }
        self.lseg.clear();
        self.lpnt.clear();
        self.lpnt.extend_from_slice(&other.lpnt);
        self.lseg.extend_from_slice(&other.lseg);
        self.done = true;
    }

    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if !self.done {
            return Err(not_done(member));
        }
        Ok(())
    }
}

// `NCollection_Sequence::operator()(N)`: the item of 1-based rank `n`.
fn rank<'a, T>(items: &'a [T], n: i32, member: &str) -> Result<&'a T, Failure> {
    if n < 1 || n as usize > items.len() {
        return Err(Failure::out_of_range(&format!(
            "{member}() - index {n} outside [1, {}]",
            items.len()
        )));
    }
    Ok(&items[(n - 1) as usize])
}

// `TransitionEqual`: same position, type and (for TOUCH) tangency, side and
// orientation.
fn transition_equal(t1: &Transition, t2: &Transition) -> Result<bool, Failure> {
    if t1.position_on_curve() != t2.position_on_curve()
        || t1.transition_type() != t2.transition_type()
    {
        return Ok(false);
    }
    if t1.transition_type() != TypeTrans::Touch {
        return Ok(true);
    }
    Ok(t1.is_tangent()? == t2.is_tangent()?
        && t1.situation()? == t2.situation()?
        && t1.is_opposite()? == t2.is_opposite()?)
}

impl Intersection {
    /// `Insert(Pnt)` (protected): inserts the point before the first one of greater
    /// or equal first parameter, unless an equal point (parameters within 1e-8 and
    /// equal transitions) comes first. `inter.insert(point)?`.
    pub fn insert(&mut self, pnt: IntersectionPoint) -> Result<(), Failure> {
        let n: usize = self.lpnt.len();
        let u: f64 = pnt.param_on_first();
        let mut b: usize = n + 1;
        let mut i: usize = 1;
        while i <= n {
            let pnti: IntersectionPoint = self.lpnt[i - 1];
            let ui: f64 = pnti.param_on_first();
            if ui >= u {
                b = i;
                i = n;
            }
            if param_equal(ui, u) && same_point(&pnt, &pnti)? {
                b = 0;
                i = n;
            }
            i += 1;
        }
        if b > n {
            self.lpnt.push(pnt);
        } else if b > 0 {
            self.lpnt.insert(b - 1, pnt);
        }
        Ok(())
    }
}

// The rest of the duplicate test of Insert once the first parameters are equal.
fn same_point(pnt: &IntersectionPoint, pnti: &IntersectionPoint) -> Result<bool, Failure> {
    if !param_equal(pnt.param_on_second(), pnti.param_on_second()) {
        return Ok(false);
    }
    Ok(
        transition_equal(&pnt.transition_of_first(), &pnti.transition_of_first())?
            && transition_equal(&pnt.transition_of_second(), &pnti.transition_of_second())?,
    )
}

/// The parameter ranges of the two composite curves that `Append` merges into.
#[derive(Clone, Copy, Debug)]
struct CompositeBounds {
    first1: f64,
    last1: f64,
    first2: f64,
    last2: f64,
}

// `InternalVerifyPosition`: a bound position strictly inside the composite range
// (and not at one of its ends) becomes Middle.
fn verify_position(point: &IntersectionPoint, bounds: &CompositeBounds) -> IntersectionPoint {
    let (u1, u2) = (point.param_on_first(), point.param_on_second());
    let mut t1: Transition = point.transition_of_first();
    let mut t2: Transition = point.transition_of_second();
    if is_inner(t1, u1, bounds.first1, bounds.last1) {
        t1.set_position(Position::Middle);
    }
    if is_inner(t2, u2, bounds.first2, bounds.last2) {
        t2.set_position(Position::Middle);
    }
    IntersectionPoint::new(point.value(), u1, u2, t1, t2, false)
}

fn is_inner(t: Transition, u: f64, first: f64, last: f64) -> bool {
    t.position_on_curve() != Position::Middle
        && !(param_equal(u, first) || param_equal(u, last))
        && u > first
        && u < last
}

impl Intersection {
    /// `Append(Inter, FirstParam1, LastParam1, FirstParam2, LastParam2)`
    /// (protected): merges the result of one part of composite curves into this
    /// one, relabelling inner bounds as Middle and joining segments that meet end
    /// to end; Standard_DomainError on an unbounded segment (composite curves are
    /// always bounded). `inter.append_composite(&part, 0.0, 1.0, 0.0, 1.0)?`.
    pub fn append_composite(
        &mut self,
        other: &Intersection,
        first_param1: f64,
        last_param1: f64,
        first_param2: f64,
        last_param2: f64,
    ) -> Result<(), Failure> {
        if !other.done {
            self.done = false;
            return Ok(());
        }
        let bounds = CompositeBounds {
            first1: first_param1,
            last1: last_param1,
            first2: first_param2,
            last2: last_param2,
        };
        for point in &other.lpnt {
            self.insert(verify_position(point, &bounds))?;
        }
        for segment in &other.lseg {
            self.merge_segment(segment, &bounds)?;
        }
        self.done = true;
        Ok(())
    }

    // One segment of Append: joined to an existing segment it continues, whose
    // inner points are then removed, or appended.
    fn merge_segment(
        &mut self,
        segment: &IntersectionSegment,
        bounds: &CompositeBounds,
    ) -> Result<(), Failure> {
        let (p1, p2) = (segment.first_point()?, segment.last_point()?);
        let opposite: bool = segment.is_opposite();
        // A joined segment takes the points of `other` as they are; only an appended
        // one takes their relabelled positions, as the native code does.
        match self.join_segment(p1, p2, opposite)? {
            Some(range) => self.remove_points_within(range),
            None => self.append_segment(IntersectionSegment::bounded(
                verify_position(&p1, bounds),
                verify_position(&p2, bounds),
                opposite,
                false,
            )),
        }
        Ok(())
    }

    // The loop over the previous segments of Append. The native loop holds
    // references into `lseg(j)`, so a later branch of the same iteration reads the
    // segment written by an earlier one; `lseg[j]` is re-read at each use for that.
    // Returns the parameters of the modified segment, `[P1First, P1Second,
    // P2First, P2Second]`.
    fn join_segment(
        &mut self,
        p1: IntersectionPoint,
        p2: IntersectionPoint,
        opposite: bool,
    ) -> Result<Option<[f64; 4]>, Failure> {
        let (p1f, p1s) = (p1.param_on_first(), p1.param_on_second());
        let (p2f, p2s) = (p2.param_on_first(), p2.param_on_second());
        let mut modified: Option<[f64; 4]> = None;
        for j in 0..self.lseg.len() {
            if modified.is_some() {
                break;
            }
            if opposite != self.lseg[j].is_opposite() {
                continue;
            }
            let an1: IntersectionPoint = self.lseg[j].first_point()?;
            let an2: IntersectionPoint = self.lseg[j].last_point()?;
            let (a1f, a1s) = (an1.param_on_first(), an1.param_on_second());
            let (a2f, a2s) = (an2.param_on_first(), an2.param_on_second());
            if param_equal(p1f, a2f) && param_equal(p1s, a2s) {
                let first: IntersectionPoint = self.lseg[j].first_point()?;
                self.lseg[j] = IntersectionSegment::bounded(first, p2, opposite, false);
                modified = Some([a1f, a1s, p2f, p2s]);
            } else if param_equal(p2f, a1f) && param_equal(p2s, a1s) {
                let last: IntersectionPoint = self.lseg[j].last_point()?;
                self.lseg[j] = IntersectionSegment::bounded(p1, last, opposite, false);
                modified = Some([p1f, p1s, a2f, a2s]);
            }
            if param_equal(p1f, a1f) && param_equal(p1s, a1s) {
                let last: IntersectionPoint = self.lseg[j].last_point()?;
                self.lseg[j] = IntersectionSegment::bounded(last, p2, opposite, false);
                modified = Some([p2f, p2s, a2f, a2s]);
            } else if param_equal(p2f, a2f) && param_equal(p2s, a2s) {
                let first: IntersectionPoint = self.lseg[j].first_point()?;
                self.lseg[j] = IntersectionSegment::bounded(p1, first, opposite, false);
                modified = Some([p1f, p1s, a1f, a1s]);
            }
        }
        Ok(modified)
    }

    // Removes the points whose both parameters lie in the modified segment.
    fn remove_points_within(&mut self, [f1, s1, f2, s2]: [f64; 4]) {
        self.lpnt.retain(|point| {
            let (on_first, on_second) = (point.param_on_first(), point.param_on_second());
            let inside_first: bool =
                (on_first >= f1 && on_first <= f2) || (on_first <= f1 && on_first >= f2);
            let inside_second: bool =
                (on_second >= s1 && on_second <= s2) || (on_second <= s1 && on_second >= s2);
            !(inside_first && inside_second)
        });
    }
}
