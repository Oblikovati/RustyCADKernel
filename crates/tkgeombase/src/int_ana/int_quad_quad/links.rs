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
// Rust translation of IntAna_IntQuadQuad::InternalSetNextAndPrevious, HasNextCurve,
// NextCurve, HasPreviousCurve and PreviousCurve at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The end-to-end links of the curves of an [`IntQuadQuad`]: `next[i]` and
//! `previous[i]` hold `j + 1` (same orientation) or `-(j + 1)` (opposite).
use super::IntQuadQuad;
use crate::int_ana::Curve;
use rustycad_tkernel::standard::Failure;

/// The parameter gap (1e-7) within which two curve ends may coincide.
const PARAMETER_GAP: f64 = 0.0000001;
/// The distance (1e-10) below which two curve ends coincide.
const EPSILON_DISTANCE: f64 = 0.0000000001;

/// One end of a curve: its domain and whether each end is open (infinite).
struct CurveEnds {
    inf: f64,
    sup: f64,
    first_open: bool,
    last_open: bool,
}

impl CurveEnds {
    fn of(curve: &Curve) -> Result<Self, Failure> {
        let (inf, sup) = curve.domain()?;
        Ok(Self {
            inf,
            sup,
            first_open: curve.is_first_open(),
            last_open: curve.is_last_open(),
        })
    }
}

impl IntQuadQuad {
    /// InternalSetNextAndPrevious: links each curve `c1` to the curves `c2` of
    /// lower index (the native loop stops at `c2 == c1`) whose closed ends coincide
    /// in parameter (1e-7) and in space (1e-10): start on end (previous/next),
    /// start on start, end on end, end on start.
    pub(super) fn internal_set_next_and_previous(&mut self) -> Result<(), Failure> {
        let count: usize = self.nb_curves as usize;
        self.nextcurve[..count].fill(0);
        self.previouscurve[..count].fill(0);
        for c1 in 0..count {
            let first = CurveEnds::of(&self.the_curve[c1])?;
            for c2 in 0..c1 {
                let second = CurveEnds::of(&self.the_curve[c2])?;
                self.link_first_end(c1, c2, &first, &second)?;
                self.link_last_end(c1, c2, &first, &second)?;
            }
        }
        Ok(())
    }

    /// The links of the first end of `c1` (when closed).
    fn link_first_end(
        &mut self,
        c1: usize,
        c2: usize,
        first: &CurveEnds,
        second: &CurveEnds,
    ) -> Result<(), Failure> {
        if first.first_open {
            return Ok(());
        }
        let (one, two) = (c1 as i32 + 1, c2 as i32 + 1);
        if !second.last_open && self.ends_meet(c1, first.inf, c2, second.sup)? {
            self.previouscurve[c1] = two;
            self.nextcurve[c2] = one;
        }
        if !second.first_open && self.ends_meet(c1, first.inf, c2, second.inf)? {
            self.previouscurve[c1] = -two;
            self.previouscurve[c2] = -one;
        }
        Ok(())
    }

    /// The links of the last end of `c1` (when closed).
    fn link_last_end(
        &mut self,
        c1: usize,
        c2: usize,
        first: &CurveEnds,
        second: &CurveEnds,
    ) -> Result<(), Failure> {
        if first.last_open {
            return Ok(());
        }
        let (one, two) = (c1 as i32 + 1, c2 as i32 + 1);
        if !second.last_open && self.ends_meet(c1, first.sup, c2, second.sup)? {
            self.nextcurve[c1] = -two;
            self.nextcurve[c2] = -one;
        }
        if !second.first_open && self.ends_meet(c1, first.sup, c2, second.inf)? {
            self.nextcurve[c1] = two;
            self.previouscurve[c2] = one;
        }
        Ok(())
    }

    /// The parameters `t1` on `c1` and `t2` on `c2` are within 1e-7 and give
    /// points closer than 1e-10 (the points are only computed for close parameters).
    fn ends_meet(&self, c1: usize, t1: f64, c2: usize, t2: f64) -> Result<bool, Failure> {
        if (t1 - t2).abs() > PARAMETER_GAP || (t1 - t2).is_nan() {
            return Ok(false);
        }
        let p1 = self.the_curve[c1].value(t1)?;
        let p2 = self.the_curve[c2].value(t2)?;
        Ok(p1.distance(&p2) < EPSILON_DISTANCE)
    }

    /// HasPreviousCurve(I): the curve `i` shares its first bound with another
    /// curve; StdFail_NotDone, Standard_DomainError for identical elements,
    /// Standard_OutOfRange outside `1..=NbCurve()`. `inter.has_previous_curve(1)?`.
    pub fn has_previous_curve(&self, i: i32) -> Result<bool, Failure> {
        let slot: usize = self.check_curve(i, "Incorrect Curve Number 'HasPrevious Curve'")?;
        Ok(self.previouscurve[slot] != 0)
    }

    /// HasNextCurve(I): the curve `i` shares its last bound with another curve,
    /// with the errors of [`Self::has_previous_curve`]. `inter.has_next_curve(1)?`.
    pub fn has_next_curve(&self, i: i32) -> Result<bool, Failure> {
        let slot: usize = self.check_curve(i, "Incorrect Curve Number 'HasNextCurve'")?;
        Ok(self.nextcurve[slot] != 0)
    }

    /// PreviousCurve(I, theOpposite): the curve sharing the first bound of curve
    /// `i` and whether its first (true) or last (false) bound is the common point;
    /// Standard_DomainError when there is none. `let (j, opposite) = inter.previous_curve(1)?;`
    pub fn previous_curve(&self, i: i32) -> Result<(i32, bool), Failure> {
        if !self.has_previous_curve(i)? {
            return Err(no_link("Incorrect Curve Number 'PreviousCurve'", i));
        }
        let link: i32 = self.previouscurve[(i - 1) as usize];
        Ok(if link > 0 {
            (link, false)
        } else {
            (-link, true)
        })
    }

    /// NextCurve(I, theOpposite): the curve sharing the last bound of curve `i`
    /// and whether its last (true) or first (false) bound is the common point;
    /// Standard_DomainError when there is none. `let (j, opposite) = inter.next_curve(1)?;`
    pub fn next_curve(&self, i: i32) -> Result<(i32, bool), Failure> {
        if !self.has_next_curve(i)? {
            return Err(no_link("Incorrect Curve Number 'NextCurve'", i));
        }
        let link: i32 = self.nextcurve[(i - 1) as usize];
        Ok((link.abs(), link < 0))
    }
}

/// The Standard_DomainError of a curve without the requested link.
fn no_link(message: &str, i: i32) -> Failure {
    Failure::with_type(
        "Standard_DomainError",
        &format!("{message}: curve {i} has no such link"),
        "",
    )
}
