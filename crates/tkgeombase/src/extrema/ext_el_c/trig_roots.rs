// Rust translation of the file-local ExtremaExtElC_TrigonometricRoots and RefineDir of
// Extrema_ExtElC.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The sorted, verified roots of a trigonometric polynomial that the line-conic
//! extrema solve, and the snapping of nearly axial directions.
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkmath::gp::Direction3;
use rustycad_tkmath::math::TrigonometricRoots;
use std::f64::consts::PI;

/// `ExtremaExtElC_TrigonometricRoots`: the roots in `[0, 2 PI]` of
/// `CC cos² + 2 SC cos sin + C cos + S sin + Cte`, sorted. A second attempt zeroes
/// the coefficients below 1e-10 when math_TrigonometricFunctionRoots fails; a root
/// whose residual (with the original coefficients) exceeds
/// `max(1e-8, 1e-12 max(CC, SC, C, S, Cte))` is dropped. (Code duplicated from
/// IntAna_IntQuadQuad.cxx natively, but not the same code.)
pub(super) struct SortedTrigRoots {
    roots: [f64; 4],
    done: bool,
    nb_roots: i32,
    infinite_roots: bool,
}

impl SortedTrigRoots {
    /// Solves `[CC, SC, C, S, Cte]` on `[binf, bsup]`.
    pub fn new(coefficients: [f64; 5], binf: f64, bsup: f64) -> Result<Self, Failure> {
        let mut result = Self {
            roots: [0.0; 4],
            done: false,
            nb_roots: 0,
            infinite_roots: false,
        };
        let mut attempt: [f64; 5] = coefficients;
        for _ in 1..=2 {
            let [cc, sc, c, s, cte] = attempt;
            let solver = TrigonometricRoots::new(cc, sc, c, s, cte, binf, bsup);
            if solver.is_done() {
                result.done = true;
                result.store(&solver, coefficients)?;
                return Ok(result);
            }
            // Try to set very small coefficients to zero.
            for (value, original) in attempt.iter_mut().zip(coefficients) {
                if original.abs() < 1e-10 {
                    *value = 0.0;
                }
            }
        }
        Ok(result)
    }

    // The native `if (MTFR.IsDone())` branch.
    fn store(
        &mut self,
        solver: &TrigonometricRoots,
        coefficients: [f64; 5],
    ) -> Result<(), Failure> {
        if solver.infinite_roots() {
            self.infinite_roots = true;
            return Ok(());
        }
        self.nb_roots = solver.solutions()?;
        let saved: usize = self.nb_roots as usize;
        for i in 0..saved {
            self.roots[i] = periodic_root(solver.value(i as i32 + 1)?);
        }
        // The direct search gives unreliable results.
        let [cc, sc, c, s, cte] = coefficients;
        let max_coef: f64 = std_max(std_max(std_max(std_max(cc, sc), c), s), cte);
        let precision: f64 = std_max(1.0e-8, 1.0e-12 * max_coef);
        for root in &mut self.roots[..saved] {
            if trig_residual(coefficients, *root).abs() > precision {
                self.nb_roots -= 1;
                *root = 1000.0;
            }
        }
        sort_roots(&mut self.roots[..saved]);
        self.infinite_roots = false;
        // Detect case Pol = Cte (1e-50).
        let small: bool = (cc.abs() + sc.abs() + c.abs() + s.abs()) < 1e-10;
        if self.nb_roots == 0 && small && cte.abs() < 1e-10 {
            self.infinite_roots = true;
        }
        Ok(())
    }

    /// IsDone().
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// InfiniteRoots(): StdFail_NotDone when not done.
    pub fn infinite_roots(&self) -> Result<bool, Failure> {
        if !self.done {
            return Err(self.not_done("InfiniteRoots"));
        }
        Ok(self.infinite_roots)
    }

    /// NbSolutions(): StdFail_NotDone when not done.
    pub fn nb_solutions(&self) -> Result<i32, Failure> {
        if !self.done {
            return Err(self.not_done("NbSolutions"));
        }
        Ok(self.nb_roots)
    }

    /// Value(n): StdFail_NotDone when not done or `n > NbSolutions()`.
    pub fn value(&self, n: i32) -> Result<f64, Failure> {
        if !self.done || n > self.nb_roots || n < 1 {
            return Err(self.not_done("Value"));
        }
        Ok(self.roots[(n - 1) as usize])
    }

    fn not_done(&self, member: &str) -> Failure {
        let message: String = format!(
            "ExtremaExtElC_TrigonometricRoots::{member}() - {} roots, done {}; expected a done solve",
            self.nb_roots, self.done
        );
        Failure::with_type("StdFail_NotDone", &message, "")
    }
}

// A root brought into `[0, 2 PI]` by one period.
fn periodic_root(root: f64) -> f64 {
    let two_pi: f64 = PI + PI;
    let mut root: f64 = root;
    if root < 0. {
        root += two_pi;
    }
    if root > two_pi {
        root -= two_pi;
    }
    root
}

// `cos (CC cos + 2 SC sin + C) + S sin + Cte` at `root`.
fn trig_residual([cc, sc, c, s, cte]: [f64; 5], root: f64) -> f64 {
    let (co, si) = (root.cos(), root.sin());
    co * (cc * co + (sc + sc) * si + c) + s * si + cte
}

// The native bubble sort over the saved count (the rejected roots, set to 1000,
// move to the end).
fn sort_roots(roots: &mut [f64]) {
    let mut sorted: bool = false;
    while !sorted {
        sorted = true;
        for i in 1..roots.len() {
            if roots[i] < roots[i - 1] {
                roots.swap(i, i - 1);
                sorted = false;
            }
        }
    }
}

/// RefineDir: a direction with a coordinate within RealEpsilon() of ±1 becomes that
/// axis exactly (checked in X, Y, Z order).
pub(super) fn refine_dir(direction: &mut Direction3) -> Result<(), Failure> {
    let mut coordinates: [f64; 3] = [direction.x(), direction.y(), direction.z()];
    for i in 0..3 {
        let one: f64 = if coordinates[i] > 0. { 1. } else { -1. };
        let (x1, x2) = (one - f64::EPSILON, one + f64::EPSILON);
        if coordinates[i] > x1 && coordinates[i] < x2 {
            coordinates = [0.0; 3];
            coordinates[i] = one;
            let [x, y, z] = coordinates;
            return direction.set_coordinates(x, y, z);
        }
    }
    Ok(())
}
