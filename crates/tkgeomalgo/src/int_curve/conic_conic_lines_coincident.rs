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
// Rust translation of IntCurve_IntConicConic_1.cxx (Perform for two lines, lines coincident
// within the tolerance) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Two lines coincident within the tolerance: the common part of their domains on
//! L1 (the reference for the positive sense), which may be unbounded.
use super::IntConicConic;
use super::conic_conic_lines::{Lines, position_ll};
use crate::int_res2d::{IntersectionPoint, IntersectionSegment, Situation, Transition};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use std::cmp::Ordering;

/// One end of the common part: its parameters on L1 and L2.
#[derive(Clone, Copy)]
struct CommonEnd {
    on1: f64,
    on2: f64,
}

impl Lines<'_> {
    // `ResHasFirstPoint`/`ResHasLastPoint`: 1 when L1 bounds that end, +2 when L2
    // does (its last point bounds the start of opposite lines).
    fn end_codes(&self) -> (i32, i32) {
        let mut first: i32 = i32::from(self.domain1.has_first_point());
        let mut last: i32 = i32::from(self.domain1.has_last_point());
        let (start2, end2) = if self.is_opposite {
            (
                self.domain2.has_last_point(),
                self.domain2.has_first_point(),
            )
        } else {
            (
                self.domain2.has_first_point(),
                self.domain2.has_last_point(),
            )
        };
        first += if start2 { 2 } else { 0 };
        last += if end2 { 2 } else { 0 };
        (first, last)
    }

    // The start of the common part for a code (0: unbounded on the left).
    fn common_start(&self, code: i32, org2_sur_l1: f64) -> Result<CommonEnd, Failure> {
        let (d1, d2, opp) = (self.domain1, self.domain2, self.is_opposite);
        let end: CommonEnd = match code {
            1 => {
                let on1: f64 = d1.first_parameter()?;
                CommonEnd {
                    on1,
                    on2: if opp {
                        org2_sur_l1 - on1
                    } else {
                        on1 - org2_sur_l1
                    },
                }
            }
            2 | 3 => {
                let on2: f64 = if opp {
                    d2.last_parameter()?
                } else {
                    d2.first_parameter()?
                };
                let mut end = CommonEnd {
                    on1: if opp {
                        org2_sur_l1 - on2
                    } else {
                        org2_sur_l1 + on2
                    },
                    on2,
                };
                if code == 3 && end.on1 < d1.first_parameter()? {
                    end.on1 = d1.first_parameter()?;
                    end.on2 = if opp {
                        org2_sur_l1 - end.on1
                    } else {
                        end.on1 - org2_sur_l1
                    };
                }
                end
            }
            _ => CommonEnd { on1: 0., on2: 0. },
        };
        Ok(end)
    }

    // The end of the common part for a code (0: unbounded on the right).
    fn common_end(&self, code: i32, org2_sur_l1: f64) -> Result<CommonEnd, Failure> {
        let (d1, d2, opp) = (self.domain1, self.domain2, self.is_opposite);
        let end: CommonEnd = match code {
            1 => {
                let on1: f64 = d1.last_parameter()?;
                CommonEnd {
                    on1,
                    on2: if opp {
                        org2_sur_l1 - on1
                    } else {
                        on1 - org2_sur_l1
                    },
                }
            }
            2 | 3 => {
                let on2: f64 = if opp {
                    d2.first_parameter()?
                } else {
                    d2.last_parameter()?
                };
                let mut end = CommonEnd {
                    on1: if opp {
                        org2_sur_l1 - on2
                    } else {
                        org2_sur_l1 + on2
                    },
                    on2,
                };
                if code == 3 && end.on1 > d1.last_parameter()? {
                    end.on1 = d1.last_parameter()?;
                    end.on2 = if opp {
                        org2_sur_l1 - end.on1
                    } else {
                        end.on1 - org2_sur_l1
                    };
                }
                end
            }
            _ => CommonEnd { on1: 0., on2: 0. },
        };
        Ok(end)
    }

    // The point of an end with its positions and tangent TOUCH transitions; the
    // parameters are snapped to the domain ends in place (FindPositionLL takes them
    // by reference natively).
    fn common_point(&self, end: &mut CommonEnd) -> Result<IntersectionPoint, Failure> {
        let pos1;
        let pos2;
        (end.on1, pos1) = position_ll(end.on1, self.domain1)?;
        (end.on2, pos2) = position_ll(end.on2, self.domain2)?;
        let (mut tinf, mut tsup) = (Transition::default(), Transition::default());
        tinf.set_touch_value(true, pos1, Situation::Unknown, self.is_opposite);
        tsup.set_touch_value(true, pos2, Situation::Unknown, self.is_opposite);
        let pnt = elclib::line_value_2d(end.on1, self.l1.position());
        Ok(IntersectionPoint::new(
            pnt, end.on1, end.on2, tinf, tsup, false,
        ))
    }
}

impl IntConicConic {
    // Coincident lines: an infinite segment, a half line, a segment from the first
    // to the last common end, or a point when shorter than Tol.
    pub(super) fn coincident_lines(&mut self, lines: &Lines<'_>) -> Result<(), Failure> {
        let org2_sur_l1: f64 = elclib::line_parameter_2d(lines.l1.position(), lines.l2.location());
        let (first_code, last_code) = lines.end_codes();
        if first_code == 0 && last_code == 0 {
            self.base
                .append_segment(IntersectionSegment::infinite(lines.is_opposite));
            return Ok(());
        }
        let mut start: CommonEnd = lines.common_start(first_code, org2_sur_l1)?;
        let mut end: CommonEnd = lines.common_end(last_code, org2_sur_l1)?;
        if first_code == 0 {
            // Half line <-----------|
            let p2: IntersectionPoint = lines.common_point(&mut end)?;
            self.base.append_segment(IntersectionSegment::half_bounded(
                p2,
                false,
                lines.is_opposite,
                false,
            ));
            return Ok(());
        }
        if last_code == 0 {
            // Half line |----------->
            let p: IntersectionPoint = lines.common_point(&mut start)?;
            self.base.append_segment(IntersectionSegment::half_bounded(
                p,
                true,
                lines.is_opposite,
                false,
            ));
            return Ok(());
        }
        let reaches: Option<Ordering> = end.on1.partial_cmp(&(start.on1 - lines.tol));
        if !matches!(reaches, Some(Ordering::Greater | Ordering::Equal)) {
            return Ok(());
        }
        let p1: IntersectionPoint = lines.common_point(&mut start)?;
        // The start is snapped to its domain end before this test.
        if end.on1 > (start.on1 + lines.tol) {
            let p2: IntersectionPoint = lines.common_point(&mut end)?;
            self.base.append_segment(IntersectionSegment::bounded(
                p1,
                p2,
                lines.is_opposite,
                false,
            ));
        } else {
            // The segment is shorter than Tol.
            self.base.append_point(p1);
        }
        Ok(())
    }
}
