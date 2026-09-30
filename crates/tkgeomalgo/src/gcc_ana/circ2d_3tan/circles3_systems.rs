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
// Rust translation of the first two steps of the three-circle constructor of
// GccAna_Circ2d3Tan.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Steps 1 and 2 of the circle tangent to three circles: the eight systems
//! (X - Xi)^2 + (Y - Yi)^2 = (R +- Ri)^2, reduced by eliminating R to two conics
//! a x^2 + 2 b x y + c y^2 + 2 d x + 2 e y + f = 0 (one for circles 1-2, one for
//! circles 1-3), and the (x, y) roots of each pair of conics.
use crate::gcc_ana::solutions::Slots;
use crate::gcc_ent::QualifiedCirc;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::DirectPolynomialRoots;

/// Centres and radii of the three circles.
#[derive(Clone, Copy)]
pub(super) struct Three {
    pub(super) x: [f64; 3],
    pub(super) y: [f64; 3],
    pub(super) r: [f64; 3],
}

/// One conic a x^2 + 2 b x y + c y^2 + 2 d x + 2 e y + f and the linear
/// relation R +- Ri = beta X + gamma Y + delta it came from.
#[derive(Clone, Copy, Default)]
pub(super) struct Conic {
    pub(super) a: f64,
    pub(super) b: f64,
    pub(super) c: f64,
    pub(super) d: f64,
    pub(super) e: f64,
    pub(super) f: f64,
    pub(super) beta: f64,
    pub(super) gamma: f64,
    pub(super) delta: f64,
}

// System i uses R - R2 for i in {1, 4, 5, 8} (else R + R2) and R - R3 for i in
// {1, 3, 6, 8} (else R + R3), as the native table of the eight systems writes.
pub(super) fn minus2(i: usize) -> bool {
    matches!(i, 1 | 4 | 5 | 8)
}
pub(super) fn minus3(i: usize) -> bool {
    matches!(i, 1 | 3 | 6 | 8)
}

// The conic of circles 1 and `k` (index 1 or 2 in `t`) for system i. When the radii
// are equal within the tolerance and the system subtracts them, the conic is the
// radical line and beta, gamma, delta stay unset (0 here, uninitialized natively,
// never read).
fn conic(t: &Three, k: usize, minus: bool, tolerance: f64) -> Conic {
    let (x1, y1, r1) = (t.x[0], t.y[0], t.r[0]);
    let (xk, yk, rk) = (t.x[k], t.y[k], t.r[k]);
    let mut q = Conic::default();
    if minus {
        if (r1 - rk).abs() > tolerance {
            q.beta = (x1 - xk) / (r1 - rk);
            q.gamma = (y1 - yk) / (r1 - rk);
            q.delta =
                (xk * xk - x1 * x1 + yk * yk - y1 * y1 + (r1 - rk) * (r1 - rk)) / (2.0 * (r1 - rk));
        }
    } else {
        q.beta = (x1 - xk) / (r1 + rk);
        q.gamma = (y1 - yk) / (r1 + rk);
        q.delta =
            (xk * xk - x1 * x1 + yk * yk - y1 * y1 + (r1 + rk) * (r1 + rk)) / (2.0 * (r1 + rk));
    }
    if minus && (r1 - rk).abs() <= tolerance {
        (q.a, q.b, q.c) = (0.0, 0.0, 0.0);
        q.d = xk - x1;
        q.e = yk - y1;
        q.f = x1 * x1 - xk * xk + y1 * y1 - yk * yk;
        return q;
    }
    q.a = q.beta * q.beta - 1.0;
    q.b = q.beta * q.gamma;
    q.c = q.gamma * q.gamma - 1.0;
    q.d = q.beta * q.delta + xk;
    q.e = q.gamma * q.delta + yk;
    q.f = q.delta * q.delta - xk * xk - yk * yk;
    q
}

/// First step: the two conics of each of the eight systems (index 0..8 for system
/// 1..8).
pub(super) fn conics(t: &Three, tolerance: f64) -> [(Conic, Conic); 8] {
    std::array::from_fn(|n| {
        let i: usize = n + 1;
        (
            conic(t, 1, minus2(i), tolerance),
            conic(t, 2, minus3(i), tolerance),
        )
    })
}

// Whether the qualifiers exclude system i: an enclosed or enclosing circle needs
// R - Ri, an outside one R + Ri.
fn excluded(i: usize, q: [QualifiedCirc; 3]) -> bool {
    let plus1 = matches!(i, 2 | 5 | 6 | 8);
    let plus2 = matches!(i, 3 | 5 | 7 | 8);
    let plus3 = matches!(i, 4 | 6 | 7 | 8);
    [(plus1, q[0]), (plus2, q[1]), (plus3, q[2])]
        .into_iter()
        .any(|(plus, qualified)| {
            (plus && (qualified.is_enclosed() || qualified.is_enclosing()))
                || (!plus && qualified.is_outside())
        })
}

/// The (x, y) candidates of step 2, grouped by system: `first[i]` is the first
/// candidate of system i (1-based, `FirstSol`), candidates in `xsol`/`ysol`.
pub(super) struct Candidates {
    pub(super) xsol: Slots<f64>,
    pub(super) ysol: Slots<f64>,
    pub(super) first: Slots<i32>,
    cur_sol: i32,
}

impl Candidates {
    fn push(&mut self, x: f64, y: f64) -> Result<(), Failure> {
        self.xsol.set(self.cur_sol, x)?;
        self.ysol.set(self.cur_sol, y)?;
        self.cur_sol += 1;
        Ok(())
    }

    // Whether a y root repeats one already stored for this system (native starts
    // the scan at 1 for the first system).
    fn repeats(&self, i: usize, y: f64, tolerance: f64) -> Result<bool, Failure> {
        let first_index: i32 = if i == 1 { 1 } else { self.first.get(i as i32)? };
        let mut is_same = false;
        for l in first_index..self.cur_sol {
            if (y - self.ysol.get(l)?).abs() <= 10.0 * tolerance {
                is_same = true;
            }
        }
        Ok(is_same)
    }
}

// The usable roots of a polynomial solver (done and finite), else none.
fn roots(solver: &DirectPolynomialRoots) -> Result<Vec<f64>, Failure> {
    if !solver.is_done() || solver.infinite_roots() {
        return Ok(Vec::new());
    }
    (1..=solver.solutions()?).map(|k| solver.value(k)).collect()
}

// Push (x, y) for each root x of a x^2 + b x + c, skipping a second root within
// 10 Tolerance of the first.
fn push_x_roots(
    out: &mut Candidates,
    (a, b, c): (f64, f64, f64),
    y: f64,
    tolerance: f64,
) -> Result<(), Failure> {
    let xs: Vec<f64> = roots(&DirectPolynomialRoots::quadratic(a, b, c))?;
    for (j, &x) in xs.iter().enumerate() {
        if j != 1 || (x - xs[0]).abs() > 10.0 * tolerance {
            out.push(x, y)?;
        }
    }
    Ok(())
}

/// Second step: the (x, y) roots of every system the qualifiers allow.
pub(super) fn solve_systems(
    t: &Three,
    systems: &[(Conic, Conic); 8],
    q: [QualifiedCirc; 3],
    tolerance: f64,
) -> Result<Candidates, Failure> {
    let mut out = Candidates {
        xsol: Slots::new(64),
        ysol: Slots::new(64),
        first: Slots::new(9),
        cur_sol: 1,
    };
    let touch: bool = is_touch(t, tolerance);
    for i in 1..=8usize {
        let (p, s) = systems[i - 1];
        out.first.set(i as i32, out.cur_sol)?;
        if excluded(i, q) {
            continue;
        }
        let same = |u: f64, v: f64| (u - v).abs() <= tolerance;
        if same(p.a, s.a)
            && same(p.b, s.b)
            && same(p.c, s.c)
            && same(p.d, s.d)
            && same(p.e, s.e)
            && same(p.f, s.f)
        {
            // The two equations are equal: circle 1 itself solves the system.
            out.push(t.x[0], t.y[0])?;
            continue;
        }
        if p.a.abs() <= tolerance {
            degenerate_first(&mut out, i, p, s, touch, tolerance)?;
        } else {
            regular_first(&mut out, i, p, s, touch, tolerance)?;
        }
    }
    out.first.set(9, out.cur_sol)?;
    Ok(out)
}

/// Whether two of the circles touch each other (the IsTouch flag), which makes
/// the quartic resolvent have double roots, found from its derivative.
pub(super) fn is_touch(t: &Three, tolerance: f64) -> bool {
    let touch = |a: usize, b: usize| {
        let (dx, dy) = (t.x[a] - t.x[b], t.y[a] - t.y[b]);
        let (ra, rb) = (t.r[a], t.r[b]);
        (dx * dx + dy * dy - (ra - rb) * (ra - rb)).abs() <= tolerance
            || (dx * dx + dy * dy - (ra + rb) * (ra + rb)).abs() <= tolerance
    };
    touch(0, 1) || touch(0, 2) || touch(1, 2)
}

// The quartic resolvent of case 1 (a2 = 0) in y.
fn quartic_a2_zero(p: Conic, s: Conic) -> [f64; 5] {
    let (a3, b2, b3, c2, c3, d2, d3, e2, e3, f2, f3) =
        (s.a, p.b, s.b, p.c, s.c, p.d, s.d, p.e, s.e, p.f, s.f);
    let a = a3 * c2 * c2 - 4.0 * b2 * (b3 * c2 - b2 * c3);
    let b = 4.0 * a3 * c2 * e2 - 4.0 * b3 * (c2 * d2 + 2.0 * b2 * e2)
        + 4.0 * b2 * (2.0 * c3 * d2 - c2 * d3 + 2.0 * b2 * e3);
    let c = 2.0 * a3 * (c2 * f2 + 2.0 * e2 * e2) - 4.0 * b3 * (b2 * f2 + 2.0 * e2 * d2)
        + 4.0 * c3 * d2 * d2
        - 4.0 * d3 * (c2 * d2 + 2.0 * b2 * e2)
        + 16.0 * b2 * e3 * d2
        + 4.0 * b2 * b2 * f3;
    let d = 4.0 * a3 * e2 * f2 - 4.0 * b3 * d2 * f2 - 4.0 * d3 * (b2 * f2 + 2.0 * d2 * e2)
        + 8.0 * d2 * d2 * e3
        + 8.0 * b2 * d2 * f3;
    let e = a3 * f2 * f2 - 4.0 * d2 * d3 * f2 + 4.0 * d2 * d2 * f3;
    [a, b, c, d, e]
}

// Epsilon of the derivative test: (|(|(|4 A y| + |3 B|) y| + |2 C|) y|) + |D|.
fn derivative_epsilon([a, b, c, _, _]: [f64; 5], y: f64) -> f64 {
    ((((4.0 * a * y).abs() + (3.0 * b).abs()) * y).abs() + (2.0 * c).abs()) * y
}

fn quartic_value([a, b, c, d, e]: [f64; 5], y: f64) -> f64 {
    (((a * y + b) * y + c) * y + d) * y + e
}

// Case 1: a2 = 0; the first conic is linear in x: x = -(c2 y^2 + 2 e2 y + f2) / (2 (b2 y + d2)).
fn degenerate_first(
    out: &mut Candidates,
    i: usize,
    p: Conic,
    s: Conic,
    touch: bool,
    tolerance: f64,
) -> Result<(), Failure> {
    // 1.1) b2 y + d2 = 0: y from c2 y^2 + 2 e2 y + f2 = 0, then x from the second conic.
    let ys: Vec<f64> = roots(&DirectPolynomialRoots::quadratic(p.c, 2.0 * p.e, p.f))?;
    for (k, &y) in ys.iter().enumerate() {
        if (k != 1 || (y - ys[0]).abs() > 10.0 * tolerance)
            && (p.b * y + p.d).abs() <= p.b * tolerance
        {
            let coefficients = (
                s.a,
                2.0 * (s.b * y + s.d),
                s.c * (y * y) + 2.0 * s.e * y + s.f,
            );
            push_x_roots(out, coefficients, y, tolerance)?;
        }
    }
    // 1.2) b2 y + d2 != 0: the quartic resolvent in y.
    let quartic: [f64; 5] = quartic_a2_zero(p, s);
    let solve_x = |y: f64| -(p.c * (y * y) + 2.0 * p.e * y + p.f) / (2.0 * (p.b * y + p.d));
    if touch {
        let [a, b, c, d, _] = quartic;
        let ys1: Vec<f64> = roots(&DirectPolynomialRoots::cubic(4.0 * a, 3.0 * b, 2.0 * c, d))?;
        for (k, &y) in ys1.iter().enumerate() {
            let is_same = ys1[..k]
                .iter()
                .any(|&other| (y - other).abs() <= 10.0 * tolerance);
            let epsilon: f64 = derivative_epsilon(quartic, y).abs() + d.abs();
            if quartic_value(quartic, y).abs() <= epsilon * tolerance
                && !is_same
                && (p.b * y + p.d).abs() > p.b * tolerance
            {
                out.push(solve_x(y), y)?;
            }
        }
    }
    let [a, b, c, d, e] = quartic;
    let ys1: Vec<f64> = roots(&DirectPolynomialRoots::quartic(a, b, c, d, e))?;
    for &y in &ys1 {
        if !out.repeats(i, y, tolerance)? && (p.b * y + p.d).abs() > p.b * tolerance {
            out.push(solve_x(y), y)?;
        }
    }
    Ok(())
}

// The coefficients m, n, t, s, v of (s y + v) sqrt(p^2 - q) = m y^2 + n y + t.
fn mntsv(p: Conic, s: Conic) -> [f64; 5] {
    let (a2, a3, b2, b3, c2, c3, d2, d3, e2, e3, f2, f3) =
        (p.a, s.a, p.b, s.b, p.c, s.c, p.d, s.d, p.e, s.e, p.f, s.f);
    let m = 2.0 * a3 * b2 * b2 / (a2 * a2) - 2.0 * b2 * b3 / a2 - a3 * c2 / a2 + c3;
    let n = 4.0 * a3 * b2 * d2 / (a2 * a2)
        - 2.0 * b3 * d2 / a2
        - 2.0 * b2 * d3 / a2
        - 2.0 * a3 * e2 / a2
        + 2.0 * e3;
    let t = 2.0 * a3 * d2 * d2 / (a2 * a2) - 2.0 * d2 * d3 / a2 - a3 * f2 / a2 + f3;
    let sv = 2.0 * b3 - 2.0 * a3 * b2 / a2;
    let v = 2.0 * d3 - 2.0 * d2 * a3 / a2;
    [m, n, t, sv, v]
}

// The quartic in y of case 2 when (s y + v) != 0.
fn quartic_regular(p: Conic, [m, n, t, s, v]: [f64; 5]) -> [f64; 5] {
    let (a2, b2, c2, d2, e2, f2) = (p.a, p.b, p.c, p.d, p.e, p.f);
    let a = s * s * (b2 * b2 - a2 * c2) - m * m * a2 * a2;
    let b = 2.0 * s * v * (b2 * b2 - a2 * c2) + 2.0 * s * s * (b2 * d2 - a2 * e2)
        - 2.0 * m * n * a2 * a2;
    let c = v * v * (b2 * b2 - a2 * c2)
        + 4.0 * s * v * (b2 * d2 - a2 * e2)
        + s * s * (d2 * d2 - a2 * f2)
        - a2 * a2 * (2.0 * m * t + n * n);
    let d = 2.0 * v * v * (b2 * d2 - a2 * e2) + 2.0 * s * v * (d2 * d2 - a2 * f2)
        - 2.0 * n * t * a2 * a2;
    let e = v * v * (d2 * d2 - a2 * f2) - t * t * a2 * a2;
    [a, b, c, d, e]
}

// Whether p^2 - q >= -Epsilon Tolerance at y (x real for the first conic).
fn real_x(p: Conic, y: f64, tolerance: f64) -> bool {
    let (a2, b2, c2, d2, e2, f2) = (p.a, p.b, p.c, p.d, p.e, p.f);
    let pp: f64 = -(b2 * y + d2) / a2;
    let q: f64 = (c2 * (y * y) + 2.0 * e2 * y + f2) / a2;
    let epsilon: f64 = 2.0
        * (((b2 * b2 + (a2 * c2).abs()) * y).abs() + (b2 * d2).abs() + (a2 * e2).abs())
        / (a2 * a2);
    pp * pp - q >= -epsilon * tolerance
}

// x from the first conic at y: a2 x^2 + 2 (b2 y + d2) x + (c2 y y + 2 e2 y + f2).
fn first_conic_x(p: Conic, y: f64) -> (f64, f64, f64) {
    (
        p.a,
        2.0 * (p.b * y + p.d),
        p.c * y * y + 2.0 * p.e * y + p.f,
    )
}

// Case 2: a2 != 0.
fn regular_first(
    out: &mut Candidates,
    i: usize,
    p: Conic,
    s: Conic,
    touch: bool,
    tolerance: f64,
) -> Result<(), Failure> {
    let coefficients: [f64; 5] = mntsv(p, s);
    let [m, n, t, sv, v] = coefficients;
    if sv.abs() <= tolerance && v.abs() <= tolerance {
        // s = v = 0: m y^2 + n y + t = 0.
        let ys: Vec<f64> = roots(&DirectPolynomialRoots::quadratic(m, n, t))?;
        for (k, &y) in ys.iter().enumerate() {
            if (k != 1 || (y - ys[0]).abs() > 10.0 * tolerance) && real_x(p, y, tolerance) {
                push_x_roots(out, first_conic_x(p, y), y, tolerance)?;
            }
        }
        return Ok(());
    }
    // Native keeps the resolvent in the locals A, B, C, D, E and reuses A, B, C for
    // the x quadratic of each accepted double root: the later derivative tests and
    // the quartic solved afterwards read the overwritten A, B, C.
    let mut quartic: [f64; 5] = quartic_regular(p, coefficients);
    if touch {
        let [a, b, c, d, _] = quartic;
        let ys1: Vec<f64> = roots(&DirectPolynomialRoots::cubic(4.0 * a, 3.0 * b, 2.0 * c, d))?;
        for &y in &ys1 {
            let is_same: bool = out.repeats(i, y, tolerance)?;
            let epsilon: f64 = derivative_epsilon(quartic, y).abs() + quartic[3].abs();
            if quartic_value(quartic, y).abs() <= epsilon * tolerance
                && !is_same
                && real_x(p, y, tolerance)
            {
                let x_quadratic: (f64, f64, f64) = first_conic_x(p, y);
                (quartic[0], quartic[1], quartic[2]) = x_quadratic;
                push_x_roots(out, x_quadratic, y, tolerance)?;
            }
        }
    }
    let [a, b, c, d, e] = quartic;
    let ys: Vec<f64> = roots(&DirectPolynomialRoots::quartic(a, b, c, d, e))?;
    for (k, &y) in ys.iter().enumerate() {
        let is_same = ys[..k]
            .iter()
            .any(|&other| (y - other).abs() <= 10.0 * tolerance);
        if !is_same && real_x(p, y, tolerance) {
            push_x_roots(out, first_conic_x(p, y), y, tolerance)?;
        }
    }
    Ok(())
}
