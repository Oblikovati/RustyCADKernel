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
// Rust translation of the three-circle constructor of GccAna_Circ2d3Tan.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The circle tangent to three circles: the degenerate configurations, then the
//! radius of each (x, y) candidate (step 3), the check of the third tangency
//! (step 4) and the solutions (step 5).
use super::circles3_systems::{Candidates, Conic, Three, conics, minus2, minus3, solve_systems};
use super::{Argument, Circ2d3Tan};
use crate::gcc_ana::check_qualifier;
use crate::gcc_ana::solutions::{Slots, qualify};
use crate::gcc_ent::QualifiedCirc;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Circle2, Direction2, Point2, Xy};

// (x, y, r) triplets grouped by system, `first[i]` the first of system i.
struct Triplets {
    x: Slots<f64>,
    y: Slots<f64>,
    r: Slots<f64>,
    first: Slots<i32>,
    cur_sol: i32,
}

impl Triplets {
    fn new(xy_size: usize, r_size: usize) -> Self {
        Self {
            x: Slots::new(xy_size),
            y: Slots::new(xy_size),
            r: Slots::new(r_size),
            first: Slots::new(9),
            cur_sol: 1,
        }
    }

    fn push(&mut self, x: f64, y: f64, r: f64) -> Result<(), Failure> {
        self.x.set(self.cur_sol, x)?;
        self.y.set(self.cur_sol, y)?;
        self.r.set(self.cur_sol, r)?;
        self.cur_sol += 1;
        Ok(())
    }
}

impl Circ2d3Tan {
    /// GccAna_Circ2d3Tan(Qualified1, Qualified2, Qualified3, Tolerance): the
    /// circles tangent to three qualified circles, up to 16. The eight sign systems
    /// are solved in the native order and each solution keeps its system's order;
    /// `Circ2d3Tan::circ_circ_circ(q1, q2, q3, tol)?`.
    pub fn circ_circ_circ(
        qualified1: QualifiedCirc,
        qualified2: QualifiedCirc,
        qualified3: QualifiedCirc,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(16);
        for qualified in [qualified1, qualified2, qualified3] {
            check_qualifier(qualified.qualifier(), super::NAME)?;
        }
        let circles: [Circle2; 3] = [
            qualified1.qualified(),
            qualified2.qualified(),
            qualified3.qualified(),
        ];
        let three = Three {
            x: circles.map(|c| c.location().x()),
            y: circles.map(|c| c.location().y()),
            r: circles.map(|c| c.radius()),
        };
        if degenerate(&circles, tolerance) {
            return Ok(circ);
        }
        let systems: [(Conic, Conic); 8] = conics(&three, tolerance);
        let q = [qualified1, qualified2, qualified3];
        let candidates: Candidates = solve_systems(&three, &systems, q, tolerance)?;
        let radii: Triplets = radii(&three, &systems, &candidates, tolerance)?;
        let checked: Triplets = third_tangency(&three, &systems, &radii, tolerance)?;
        circ.store_solutions(&circles, q, &checked, tolerance)?;
        circ.well_done = true;
        Ok(circ)
    }

    // Fifth step: keep the triplets the qualifiers accept and store them. Past 16
    // solutions native overwrites the last slot (RLE: avoid out of range).
    fn store_solutions(
        &mut self,
        circles: &[Circle2; 3],
        q: [QualifiedCirc; 3],
        t: &Triplets,
        tolerance: f64,
    ) -> Result<(), Failure> {
        let tol: f64 = tolerance.abs();
        let radii: [f64; 3] = circles.map(|c| c.radius());
        for i in 1..=8usize {
            for j in t.first.get(i as i32)?..t.first.get(i as i32 + 1)? {
                let r: f64 = t.r.get(j)?;
                let rejected = (0..3).any(|k| {
                    (q[k].is_enclosed() && r > radii[k]) || (q[k].is_enclosing() && r < radii[k])
                });
                if rejected {
                    continue;
                }
                self.nbr_sol += 1;
                if self.nbr_sol > self.cirsol.upper() {
                    self.nbr_sol = self.cirsol.upper();
                }
                let index: i32 = self.nbr_sol;
                let center = Point2::new(t.x.get(j)?, t.y.get(j)?);
                self.cirsol
                    .set(index, crate::gcc_ana::solutions::circle_at(center, r)?)?;
                // Native measures the three distances from the centre of circle 1.
                let distcc: f64 = center.distance(circles[0].location());
                for k in 0..3 {
                    let position = qualify(q[k].qualifier(), distcc, r, radii[k], tol, true);
                    self.set_qualifier(k + 1, index, position)?;
                }
                for (k, &circle) in circles.iter().enumerate() {
                    self.circle_tangency(k, i, index, circle, tolerance)?;
                }
            }
        }
        Ok(())
    }

    // The tangency on circle k (0-based) of the solution `index` of system i: the
    // solution is the circle itself when their centres match, else the point at
    // the solution radius towards (or away from) the argument centre.
    fn circle_tangency(
        &mut self,
        k: usize,
        i: usize,
        index: i32,
        circle: Circle2,
        tolerance: f64,
    ) -> Result<(), Failure> {
        let solution: Circle2 = self.cirsol.get(index)?;
        let (center, r) = (solution.location(), solution.radius());
        if center.distance(circle.location()) <= tolerance {
            return self.mark_same(k + 1, index);
        }
        let plus: bool = match k {
            0 => matches!(i, 2 | 5 | 6 | 8),
            1 => matches!(i, 3 | 5 | 7 | 8),
            _ => matches!(i, 4 | 6 | 7 | 8),
        };
        let towards: Xy = if plus || r > circle.radius() {
            circle.location().xy() - center.xy()
        } else {
            center.xy() - circle.location().xy()
        };
        let dc = Direction2::from_xy(towards)?;
        let point = Point2::from(center.xy() + r * dc.xy());
        self.touch(k + 1, index, point, Argument::Circle(circle))
    }
}

// The configurations native refuses (WellDone stays false): two identical
// circles, or three aligned centres with mutually tangent circles.
fn degenerate(circles: &[Circle2; 3], tolerance: f64) -> bool {
    let [c1, c2, c3] = circles.map(|c| c.location());
    let [r1, r2, r3] = circles.map(|c| c.radius());
    let same = |ra: f64, rb: f64, a: Point2, b: Point2| {
        (ra - rb).abs() <= tolerance && a.is_equal(b, tolerance)
    };
    if same(r1, r2, c1, c2) || same(r1, r3, c1, c3) || same(r2, r3, c2, c3) {
        return true;
    }
    let dir2: Xy = c1.xy() - c2.xy();
    let dir3: Xy = c1.xy() - c3.xy();
    if (dir2 ^ dir3).abs() <= tolerance {
        return aligned_and_tangent([c1, c2, c3], [r1, r2, r3], tolerance);
    }
    false
}

// Aligned centres: refused when the circles are mutually tangent in the native
// combinations.
fn aligned_and_tangent([c1, c2, c3]: [Point2; 3], [r1, r2, r3]: [f64; 3], tolerance: f64) -> bool {
    let (dist1, dist2, dist3) = (c1.distance(c2), c1.distance(c3), c2.distance(c3));
    let near = |value: f64| value.abs() <= tolerance;
    if near((r1 - r2).abs() - dist1) {
        if near((r1 - r3).abs() - dist2) {
            return near((r2 - r3).abs() - dist3);
        }
        return near(r1 + r3 - dist2) && near(r2 + r3 - dist3);
    }
    if near(r1 + r2 - dist1) {
        let first = near((r1 - r3).abs() - dist2) && near(r2 + r3 - dist3);
        return !first && near((r2 - r3).abs() - dist3) && near(r1 + r3 - dist2);
    }
    false
}

// Third step: the radius of each (x, y) candidate from the relation with circle 2,
// kept when positive (and, for equal radii, when it satisfies circle 2).
fn radii(
    t: &Three,
    systems: &[(Conic, Conic); 8],
    c: &Candidates,
    tolerance: f64,
) -> Result<Triplets, Failure> {
    let mut out = Triplets::new(32, 32);
    let (x1, y1, r1, x2, y2, r2) = (t.x[0], t.y[0], t.r[0], t.x[1], t.y[1], t.r[1]);
    for i in 1..=8usize {
        out.first.set(i as i32, out.cur_sol)?;
        let q: Conic = systems[i - 1].0;
        for j in c.first.get(i as i32)?..c.first.get(i as i32 + 1)? {
            let (x, y) = (c.xsol.get(j)?, c.ysol.get(j)?);
            if minus2(i) && (r1 - r2).abs() <= tolerance {
                let root: f64 = ((x - x1) * (x - x1) + (y - y1) * (y - y1)).sqrt();
                if i == 1 || i == 4 {
                    let on_circle2 = |r: f64| {
                        let epsilon: f64 =
                            10.0 * (2.0 * (r - r2).abs() + (x - x2).abs() + (y - y2).abs());
                        ((r - r2) * (r - r2) - (x - x2) * (x - x2) - (y - y2) * (y - y2)).abs()
                            <= epsilon * tolerance
                    };
                    let r: f64 = r1 + root;
                    if on_circle2(r) {
                        out.push(x, y, r)?;
                    }
                    let r: f64 = r1 - root;
                    if (r > tolerance) && on_circle2(r) {
                        out.push(x, y, r)?;
                    }
                } else {
                    let r: f64 = -r1 + root;
                    if r > tolerance {
                        out.push(x, y, r)?;
                    }
                }
                continue;
            }
            let r: f64 = match i {
                1 | 4 => r2 + q.beta * x + q.gamma * y + q.delta,
                5 | 8 => -r2 - q.beta * x - q.gamma * y - q.delta,
                3 | 7 => -r2 + q.beta * x + q.gamma * y + q.delta,
                _ => r2 - q.beta * x - q.gamma * y - q.delta,
            };
            if r > tolerance {
                out.push(x, y, r)?;
            }
        }
    }
    out.first.set(9, out.cur_sol)?;
    Ok(out)
}

// Fourth step: keep the triplets whose circle also touches circle 3.
fn third_tangency(
    t: &Three,
    systems: &[(Conic, Conic); 8],
    c: &Triplets,
    tolerance: f64,
) -> Result<Triplets, Failure> {
    let mut out = Triplets::new(64, 16);
    let (r1, x3, y3, r3) = (t.r[0], t.x[2], t.y[2], t.r[2]);
    for i in 1..=8usize {
        out.first.set(i as i32, out.cur_sol)?;
        let q: Conic = systems[i - 1].1;
        for j in c.first.get(i as i32)?..c.first.get(i as i32 + 1)? {
            let (x, y, r) = (c.x.get(j)?, c.y.get(j)?, c.r.get(j)?);
            let keep: bool = if minus3(i) && (r1 - r3).abs() <= tolerance {
                if i == 1 || i == 3 {
                    let epsilon: f64 =
                        10.0 * (2.0 * (r - r3).abs() + (x - x3).abs() + (y - y3).abs());
                    ((r - r3) * (r - r3) - (x - x3) * (x - x3) - (y - y3) * (y - y3)).abs()
                        <= epsilon * tolerance
                } else {
                    let epsilon: f64 = 10.0 * (2.0 * (r + r3) + (x - x3).abs() + (y - y3).abs());
                    ((r + r3) * (r + r3) - (x - x3) * (x - x3) - (y - y3) * (y - y3)).abs()
                        <= epsilon * tolerance
                }
            } else {
                let epsilon: f64 = 10.0 * (q.beta.abs() + q.gamma.abs() + 1.0);
                let residual: f64 = match i {
                    1 | 3 => r3 + q.beta * x + q.gamma * y + q.delta - r,
                    6 | 8 => r3 + q.beta * x + q.gamma * y + q.delta + r,
                    4 | 7 => q.beta * x + q.gamma * y + q.delta - r - r3,
                    _ => r - r3 + q.beta * x + q.gamma * y + q.delta,
                };
                residual.abs() <= epsilon * tolerance
            };
            if keep {
                out.push(x, y, r)?;
            }
        }
    }
    out.first.set(9, out.cur_sol)?;
    Ok(out)
}
