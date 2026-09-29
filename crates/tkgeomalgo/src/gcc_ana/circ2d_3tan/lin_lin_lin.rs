// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of GccAna_Circ2d3Tan_3.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The circles tangent to three lines: the intersections of the bisectors of
//! (L1, L2) and (L1, L3), with the mean distance as radius, then a filter by the
//! side each qualifier asks for.
//!
//! Native quirks kept: the filtered solutions are compacted in place but their
//! qualifiers are not moved; an outside first line with an unqualified second one
//! takes the enclosed side; two unqualified-line combinations leave the second
//! side slot of the third line unset (0 here, uninitialized natively).
use super::{Circ2d3Tan, check_line_qualifier, line_qualifier, normal_of};
use crate::gcc_ana::Lin2dBisec;
use crate::gcc_ent::QualifiedLin;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana2d::AnaIntersection;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Direction2, Line2, Point2, Xy};

// The sides (cote) a solution centre may take for each line and their count.
#[derive(Default)]
struct Sides {
    cote: [[f64; 2]; 3],
    ncote: [i32; 3],
    all_unqualified: bool,
}

// E enclosed (+1), O outside (-1), U unqualified.
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    E,
    O,
    U,
}

fn kind(q: QualifiedLin) -> Kind {
    if q.is_enclosed() {
        Kind::E
    } else if q.is_outside() {
        Kind::O
    } else {
        Kind::U
    }
}

// The single side of a qualified (enclosed or outside) line.
fn single(k: Kind) -> f64 {
    if k == Kind::E { 1.0 } else { -1.0 }
}

impl Sides {
    fn set(&mut self, line: usize, values: &[f64]) {
        self.ncote[line] = values.len() as i32;
        self.cote[line][..values.len()].copy_from_slice(values);
    }
}

// The native selection of the sides, including its slips.
fn sides(k: [Kind; 3]) -> Sides {
    let mut s = Sides::default();
    if !k.contains(&Kind::U) {
        for (line, &kind) in k.iter().enumerate() {
            s.set(line, &[single(kind)]);
        }
        return s;
    }
    let both = [1.0, -1.0];
    if k[0] == Kind::U {
        s.set(0, &both);
        match (k[1], k[2]) {
            (Kind::U, Kind::U) => {
                // Native sets cote2 again instead of cote3 and returns every centre.
                s.set(1, &both);
                s.ncote[2] = 2;
                s.all_unqualified = true;
            }
            (Kind::U, third) => {
                s.set(1, &both);
                s.set(2, &[single(third)]);
            }
            (second, Kind::U) => {
                s.set(1, &[single(second)]);
                if second == Kind::E {
                    // cote3(1) = 1 then cote3(1) = -1: the second slot stays unset.
                    s.ncote[2] = 2;
                    s.cote[2][0] = -1.0;
                } else {
                    s.set(2, &both);
                }
            }
            (second, third) => {
                s.set(1, &[single(second)]);
                s.set(2, &[single(third)]);
            }
        }
        return s;
    }
    if k[1] == Kind::U {
        s.set(1, &both);
        // An outside first line gets cote1 = 1 natively (not -1).
        s.set(0, &[1.0]);
        match (k[0], k[2]) {
            (Kind::E, Kind::U) => {
                s.ncote[2] = 2;
                s.cote[2][0] = -1.0;
            }
            (_, Kind::U) => s.set(2, &both),
            (_, third) => s.set(2, &[single(third)]),
        }
        return s;
    }
    s.set(2, &both);
    s.set(0, &[single(k[0])]);
    s.set(1, &[single(k[1])]);
    s
}

// The side of a centre relative to a line, as the filter writes it:
// (cx - ox) * (-dy) + (cy - oy) * dx.
fn centre_side(line: Line2, center: Point2) -> f64 {
    let (origin, dir) = (line.location(), line.direction());
    (center.x() - origin.x()) * (-dir.y()) + (center.y() - origin.y()) * (dir.x())
}

impl Circ2d3Tan {
    /// GccAna_Circ2d3Tan(Qualified1, Qualified2, Qualified3, Tolerance) with three
    /// lines, up to 4 solutions; the tolerance is not used natively;
    /// `Circ2d3Tan::lin_lin_lin(l1, l2, l3, tol)?`.
    pub fn lin_lin_lin(
        qualified1: QualifiedLin,
        qualified2: QualifiedLin,
        qualified3: QualifiedLin,
        _tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(4);
        let q = [qualified1, qualified2, qualified3];
        for qualified in q {
            check_line_qualifier(qualified)?;
        }
        let lines: [Line2; 3] = q.map(|qualified| qualified.qualified());
        let bisec1 = Lin2dBisec::new(lines[0], lines[1])?;
        let bisec2 = Lin2dBisec::new(lines[0], lines[2])?;
        let mut nbsol: i32 = 0;
        if bisec1.is_done() && bisec2.is_done() {
            for i in 1..=bisec1.nb_solutions()? {
                for j in 1..=bisec2.nb_solutions()? {
                    let intp =
                        AnaIntersection::lines(bisec1.this_solution(i)?, bisec2.this_solution(j)?);
                    if !intp.is_done() {
                        continue;
                    }
                    if !intp.is_empty()? {
                        for k in 1..=intp.nb_points()? {
                            nbsol += 1;
                            circ.mean_circle(nbsol, q, intp.point(k)?.value())?;
                        }
                    }
                    circ.well_done = true;
                }
            }
        }
        let s: Sides = sides(q.map(kind));
        if s.all_unqualified {
            circ.nbr_sol = nbsol;
            circ.well_done = true;
        }
        if circ.nbr_sol > 0 {
            return Ok(circ);
        }
        for i in 1..=nbsol {
            circ.filter_side(i, &lines, &s)?;
        }
        Ok(circ)
    }

    // Candidate `index`: the circle of mean distance at `center`, its qualifiers
    // and the feet of the normals on the three lines.
    fn mean_circle(
        &mut self,
        index: i32,
        q: [QualifiedLin; 3],
        center: Point2,
    ) -> Result<(), Failure> {
        let lines: [Line2; 3] = q.map(|qualified| qualified.qualified());
        let radius: f64 =
            (lines[0].distance(center) + lines[1].distance(center) + lines[2].distance(center))
                / 3.0;
        let circle = crate::gcc_ana::solutions::circle_at(center, radius)?;
        self.cirsol.set(index, circle)?;
        for (n, &qualified) in q.iter().enumerate() {
            self.set_qualifier(
                n + 1,
                index,
                line_qualifier(qualified, center, normal_of(lines[n])?)?,
            )?;
        }
        let mut points = [Point2::default(); 3];
        for (n, line) in lines.iter().enumerate() {
            points[n] = normal_foot(*line, center, radius)?;
        }
        for (n, line) in lines.iter().enumerate() {
            let par_sol: f64 = elclib::parameter(&circle, points[n])?;
            let par_arg: f64 = elclib::parameter(line, points[n])?;
            self.tangency_mut(n + 1)
                .record(index, points[n], par_sol, par_arg)?;
        }
        Ok(())
    }

    // Keep candidate i when it lies on an accepted side of all three lines,
    // copying it to the next solution slot (its qualifiers stay where they were).
    fn filter_side(&mut self, i: i32, lines: &[Line2; 3], s: &Sides) -> Result<(), Failure> {
        for j1 in 0..s.ncote[0] as usize {
            for j2 in 0..s.ncote[1] as usize {
                for j3 in 0..s.ncote[2] as usize {
                    let center: Point2 = self.cirsol.get(i)?.location();
                    let accepted: bool = s.cote[1][j2] * centre_side(lines[1], center) > 0.0
                        && s.cote[2][j3] * centre_side(lines[2], center) > 0.0
                        && s.cote[0][j1] * centre_side(lines[0], center) > 0.0;
                    if !accepted {
                        continue;
                    }
                    self.nbr_sol += 1;
                    let index: i32 = self.nbr_sol;
                    self.cirsol.set(index, self.cirsol.get(i)?)?;
                    for (n, line) in lines.iter().enumerate() {
                        self.touch_line(n + 1, index, *line)?;
                    }
                }
            }
        }
        Ok(())
    }
}

// XY(cx, cy) + cross * radius * XY(-dy, dx), cross the unit side of the line
// origin relative to the centre.
fn normal_foot(line: Line2, center: Point2, radius: f64) -> Result<Point2, Failure> {
    let (origin, dir) = (line.location(), line.direction());
    let (cx, cy) = (center.x(), center.y());
    let normal = Direction2::new(-dir.y(), dir.x())?;
    let mut cross: f64 = normal.dot(Direction2::new(origin.x() - cx, origin.y() - cy)?);
    if cross != 0.0 {
        cross /= cross.abs();
    }
    Ok(Point2::from(
        Xy::new(cx, cy) + cross * radius * Xy::new(-dir.y(), dir.x()),
    ))
}
