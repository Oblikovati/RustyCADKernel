// Rust translation of IntAna2d_Outils.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! IntAna2d_Outils: the polynomial solver and the point utilities shared by the
//! conic intersections of IntAna2d.
use super::IntPoint;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::gp::Axis2d;
use rustycad_tkmath::math::DirectPolynomialRoots;

/// The capacity of the native `sol` and `val` arrays.
const SLOTS: usize = 16;

/// MyDirectPolynomialRoots: the real roots of a quadratic or a quartic,
/// completed for the quartic by the roots of the truncated polynomials when the
/// direct roots look unreliable.
///
/// `let roots = MyDirectPolynomialRoots::quadratic(1.0, -3.0, 2.0)?;`
#[derive(Clone, Copy, Debug)]
pub struct MyDirectPolynomialRoots {
    sol: [f64; SLOTS],
    val: [f64; SLOTS],
    nbsol: i32,
    same: bool,
}

impl MyDirectPolynomialRoots {
    /// The solver before any root: the native arrays filled with RealLast().
    fn cleared(same: bool) -> Self {
        Self {
            sol: [f64::MAX; SLOTS],
            val: [f64::MAX; SLOTS],
            nbsol: 0,
            same,
        }
    }

    /// MyDirectPolynomialRoots(A4, A3, A2, A1, A0): the roots of
    /// `A4 x^4 + A3 x^3 + A2 x^2 + A1 x + A0`. When a root does not cancel the
    /// polynomial within Epsilon(100), or the number of roots is odd, the roots of
    /// `A4..A1`, `A3..A0` and `A3 x^2 + A2 x + A1` (the native "MATH_A210") are
    /// added, sorted by increasing residual, and the first ones are kept. More than
    /// four roots mean identical elements. `MyDirectPolynomialRoots::quartic(1.0, 0.0, -5.0, 0.0, 4.0)?`.
    pub fn quartic(a4: f64, a3: f64, a2: f64, a1: f64, a0: f64) -> Result<Self, Failure> {
        let magnitudes: [f64; 5] = [a0.abs(), a1.abs(), a2.abs(), a3.abs(), a4.abs()];
        let sum: f64 =
            magnitudes[0] + magnitudes[1] + magnitudes[2] + magnitudes[3] + magnitudes[4];
        if sum < epsilon(10000.0) {
            return Ok(Self::cleared(true));
        }
        let coefficients: [f64; 5] = [a4, a3, a2, a1, a0];
        let mut roots: Self = Self::cleared(false);
        let complete = DirectPolynomialRoots::quartic(a4, a3, a2, a1, a0);
        let (count, suspicious): (i32, bool) = roots.add_complete(&complete, coefficients)?;
        if suspicious {
            roots.add_truncated(coefficients, magnitudes)?;
            roots.keep_best(count);
        }
        roots.finish();
        Ok(roots)
    }

    /// Stores every root of the complete quartic; returns their count and whether
    /// the result is suspicious (not done, a residual above Epsilon(100) or an
    /// odd count).
    fn add_complete(
        &mut self,
        complete: &DirectPolynomialRoots,
        coefficients: [f64; 5],
    ) -> Result<(i32, bool), Failure> {
        if !complete.is_done() {
            return Ok((0, true));
        }
        let tolerance: f64 = epsilon(100.0);
        let count: i32 = complete.solutions()?;
        let mut suspicious: bool = false;
        for index in 1..=count {
            let x: f64 = complete.value(index)?;
            let residual: f64 = quartic_residual(x, coefficients);
            self.push(x, residual);
            if residual > tolerance || residual < -tolerance {
                suspicious = true;
            }
        }
        Ok((count, suspicious || (count & 1) != 0))
    }

    /// Adds the roots of the three truncated polynomials that differ by at least
    /// `min(1e-4, Epsilon(1000 * max|Ai| / min|Ai|))` from the stored ones.
    fn add_truncated(
        &mut self,
        coefficients: [f64; 5],
        magnitudes: [f64; 5],
    ) -> Result<(), Failure> {
        let [a4, a3, a2, a1, a0] = coefficients;
        let separation: f64 = root_separation(magnitudes);
        let truncated: [DirectPolynomialRoots; 3] = [
            DirectPolynomialRoots::cubic(a4, a3, a2, a1),
            DirectPolynomialRoots::cubic(a3, a2, a1, a0),
            DirectPolynomialRoots::quadratic(a3, a2, a1),
        ];
        for solver in &truncated {
            if !solver.is_done() {
                continue;
            }
            for index in 1..=solver.solutions()? {
                let x: f64 = solver.value(index)?;
                if self.is_new(x, separation) {
                    self.push(x, quartic_residual(x, coefficients));
                }
            }
        }
        Ok(())
    }

    /// False when `x` is closer than `separation` to a stored root (a NaN distance
    /// never rejects, as the native `<` test).
    fn is_new(&self, x: f64, separation: f64) -> bool {
        !self.sol[..self.nbsol as usize]
            .iter()
            .any(|&root| (root - x).abs() < separation)
    }

    /// Stores the root `x` and its residual.
    fn push(&mut self, x: f64, residual: f64) {
        let slot: usize = self.nbsol as usize;
        self.val[slot] = residual;
        self.sol[slot] = x;
        self.nbsol += 1;
    }

    /// Sorts the roots by increasing |residual| (the native bubble sort, stable)
    /// and keeps at least `complete_count` of them, plus every following root
    /// whose residual is below Epsilon(10000).
    fn keep_best(&mut self, complete_count: i32) {
        let count: usize = self.nbsol as usize;
        let mut sorted: bool = false;
        while !sorted {
            sorted = true;
            for i in 1..count {
                if self.val[i].abs() < self.val[i - 1].abs() {
                    self.val.swap(i, i - 1);
                    self.sol.swap(i, i - 1);
                    sorted = false;
                }
            }
        }
        let limit: f64 = epsilon(10000.0);
        let mut kept: i32 = 0;
        while (kept as usize) < SLOTS
            && (kept < complete_count || self.val[kept as usize].abs() < limit)
        {
            kept += 1;
        }
        self.nbsol = kept;
    }

    /// The native epilogue: no root means not done (-1), more than four mean
    /// identical elements.
    fn finish(&mut self) {
        if self.nbsol == 0 {
            self.nbsol = -1;
        }
        if self.nbsol > 4 {
            self.same = true;
            self.nbsol = 0;
        }
    }

    /// MyDirectPolynomialRoots(A2, A1, A0): the roots of `A2 x^2 + A1 x + A0`;
    /// identical elements when `|A2| + |A1| + |A0| < Epsilon(10000)`.
    /// `MyDirectPolynomialRoots::quadratic(1.0, -3.0, 2.0)?`.
    pub fn quadratic(a2: f64, a1: f64, a0: f64) -> Result<Self, Failure> {
        if (a2.abs() + a1.abs() + a0.abs()) < epsilon(10000.0) {
            return Ok(Self::cleared(true));
        }
        let mut roots: Self = Self::cleared(false);
        let solver = DirectPolynomialRoots::quadratic(a2, a1, a0);
        if !solver.is_done() {
            roots.nbsol = -1;
            return Ok(roots);
        }
        for index in 1..=solver.solutions()? {
            let x: f64 = solver.value(index)?;
            roots.push(x, a0 + x * (a1 + x * a2));
        }
        Ok(roots)
    }

    /// NbSolutions(): the number of roots (-1 when not done).
    /// `roots.nb_solutions()`.
    pub fn nb_solutions(&self) -> i32 {
        self.nbsol
    }

    /// Value(i): the root of index `i`. The native member reads its 16-slot array
    /// unchecked; an index outside `1..=16` is a Standard_OutOfRange here.
    /// `roots.value(1)?`.
    pub fn value(&self, index: i32) -> Result<f64, Failure> {
        if !(1..=SLOTS as i32).contains(&index) {
            return Err(Failure::out_of_range(&format!(
                "MyDirectPolynomialRoots::Value() - index {index} outside [1, {SLOTS}]"
            )));
        }
        Ok(self.sol[(index - 1) as usize])
    }

    /// IsDone(): true when the roots were computed (`NbSolutions() > -1`). The
    /// native member returns the comparison as a double (1.0 or 0.0).
    /// `roots.is_done()`.
    pub fn is_done(&self) -> bool {
        self.nbsol > -1
    }

    /// InfiniteRoots(): true when every coefficient vanishes (identical elements).
    /// `roots.infinite_roots()`.
    pub fn infinite_roots(&self) -> bool {
        self.same
    }
}

/// The residual of the quartic at `x`, in the native Horner form.
fn quartic_residual(x: f64, [a4, a3, a2, a1, a0]: [f64; 5]) -> f64 {
    a0 + x * (a1 + x * (a2 + x * (a3 + x * a4)))
}

/// The minimal separation between two kept roots:
/// `min(1e-4, Epsilon(1000 * AMax / AMin))`, each |Ai| clamped to RealEpsilon().
fn root_separation(magnitudes: [f64; 5]) -> f64 {
    let floor: f64 = f64::EPSILON;
    let mut smallest: f64 = f64::MAX;
    let mut largest: f64 = -1.0;
    for magnitude in magnitudes {
        smallest = std_min(smallest, std_max(magnitude, floor));
        largest = std_max(largest, std_max(magnitude, floor));
    }
    std_min(1.0e-4, epsilon(1000. * largest / smallest))
}

/// Points_Confondus: true when both coordinates of the points differ by less than
/// Epsilon of the first point's coordinate. `points_confondus(1.0, 2.0, 1.0, 2.0)`.
pub fn points_confondus(x1: f64, y1: f64, x2: f64, y2: f64) -> bool {
    (x1 - x2).abs() < epsilon(x1) && (y1 - y2).abs() < epsilon(y1)
}

/// Traitement_Points_Confondus: removes from the first `nb_pts` points each one
/// that coincides with an earlier point, shifting the following points down with
/// the implicit SetValue (the second parameter becomes RealLast()), and returns
/// the new count. `let count = traitement_points_confondus(2, &mut points);`
pub fn traitement_points_confondus(nb_pts: i32, pts: &mut [IntPoint]) -> i32 {
    let mut count: i32 = nb_pts;
    for i in (2..=nb_pts).rev() {
        let point = pts[(i - 1) as usize].value();
        let earlier = (1..i).rev().find(|&j| {
            let other = pts[(j - 1) as usize].value();
            points_confondus(point.x(), point.y(), other.x(), other.y())
        });
        if earlier.is_some() {
            shift_down(pts, i, count);
            count -= 1;
        }
    }
    count
}

/// Moves the points of 0-based index `from..count` one slot down.
fn shift_down(pts: &mut [IntPoint], from: i32, count: i32) {
    for k in from..count {
        let moved = pts[k as usize];
        let location = moved.value();
        pts[(k - 1) as usize].set_implicit_value(
            location.x(),
            location.y(),
            moved.param_on_first(),
        );
    }
}

/// Coord_Ancien_Repere: the absolute coordinates of the point `(x, y)` given in
/// the frame of `axis`. `let (x, y) = coord_ancien_repere(1.0, 0.0, axis);`
pub fn coord_ancien_repere(x: f64, y: f64, axis: Axis2d) -> (f64, f64) {
    let (t11, t21) = (axis.direction().x(), axis.direction().y());
    let (t13, t23) = (axis.location().x(), axis.location().y());
    let (t22, t12) = (t11, -t21);
    (t11 * x + t12 * y + t13, t21 * x + t22 * y + t23)
}
