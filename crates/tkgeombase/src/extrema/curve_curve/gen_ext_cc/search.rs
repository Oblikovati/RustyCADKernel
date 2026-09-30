// Rust translation of the math_GlobOptMin loop of Extrema_GGenExtCC::Perform and of
// Extrema_GGenExtCC_PointsInspector at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The global search of Extrema_GGenExtCC: math_GlobOptMin on each pair of
//! intervals; the points of the best value (within `SameTol * 1e-2`) are kept once,
//! a cell filter merging points closer than the cell size.
use super::super::glob_opt_func_cc::GlobOptFuncCCC2;
use super::GGenExtCC;
use super::intervals::IntervalLayout;
use super::lipschitz::Lipschitz;
use crate::extrema::curve_tool::ExtremaCurveTool;
use rustycad_tkernel::ncollection::{CellAction, CellFilter, CellInspector};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::math::{GlobalObjective, GlobalOptimum, Vector};
use std::f64::consts::SQRT_2;

const DISC_TOL: f64 = 1.0e-2;
const VALUE_TOL: f64 = 1.0e-2;

/// The minima found: the best value and the kept parameter pairs, in finding order.
pub(super) struct FoundMinima {
    pub value: f64,
    pub points: Vec<[f64; 2]>,
}

/// `Extrema_GGenExtCC_PointsInspector`: finds a kept point within the cell size.
struct PointsInspector {
    tol: f64,
    current: [f64; 2],
    is_find: bool,
}

impl CellInspector for PointsInspector {
    type Target = [f64; 2];
    type Point = [f64; 2];
    const DIMENSION: usize = 2;
    fn coord(axis: usize, point: &[f64; 2]) -> f64 {
        point[axis]
    }
    fn is_equal(first: &[f64; 2], second: &[f64; 2]) -> bool {
        first == second
    }
    fn inspect(&mut self, target: &[f64; 2]) -> CellAction {
        let (dx, dy) = (self.current[0] - target[0], self.current[1] - target[1]);
        if dx * dx + dy * dy < self.tol {
            self.is_find = true;
        }
        CellAction::Keep
    }
}

// `Shift(thePnt, theTol)`.
fn shift(point: [f64; 2], tol: f64) -> [f64; 2] {
    [point[0] + tol, point[1] + tol]
}

// A 1-based math_Vector of two values.
fn pair(first: f64, second: f64) -> Result<Vector<'static>, Failure> {
    let mut vector: Vector<'static> = Vector::with_bounds(1, 2)?;
    (vector[1], vector[2]) = (first, second);
    Ok(vector)
}

impl<T: ExtremaCurveTool> GGenExtCC<'_, T> {
    /// The loop over the interval pairs.
    pub(super) fn search_minima(
        &self,
        c1: &T::Curve,
        c2: &T::Curve,
        layout: &IntervalLayout,
        lipschitz: Lipschitz,
    ) -> Result<FoundMinima, Failure> {
        let mut function = GlobOptFuncCCC2::<T>::new(c1, c2);
        let (low, upp) = (
            pair(self.low_border[0], self.low_border[1])?,
            pair(self.upp_border[0], self.upp_border[1])?,
        );
        let objective = GlobalObjective::Hessian(&mut function);
        let mut finder =
            GlobalOptimum::with_options(objective, &low, &upp, lipschitz.constant, 1.0e-2, 1.0e-7)?;
        finder.set_lipschitz_locked(lipschitz.locked);
        finder.set_continuity(if layout.continuity == Shape::C2 { 2 } else { 1 });
        let same_tol: f64 = self.curve_min_tol / DISC_TOL;
        finder.set_tolerances(DISC_TOL, same_tol);
        finder.set_functional_minimal_value(0.0);
        let found = self.interval_pairs(&mut finder, layout, same_tol)?;
        drop(finder);
        function.take_failure()?;
        Ok(found)
    }

    // Every pair of intervals, keeping the points of the best value.
    fn interval_pairs(
        &self,
        finder: &mut GlobalOptimum<'_>,
        layout: &IntervalLayout,
        same_tol: f64,
    ) -> Result<FoundMinima, Failure> {
        let (i1, i2) = (&layout.intervals1, &layout.intervals2);
        let cell_size: f64 = std_max(
            std_max(i1[i1.len() - 1] - i1[0], i2[i2.len() - 1] - i2[0]) * precision::p_confusion()
                / (2.0 * SQRT_2),
            precision::p_confusion(),
        );
        let mut inspector = PointsInspector {
            tol: cell_size * cell_size,
            current: [0.0; 2],
            is_find: false,
        };
        let mut filter: CellFilter<PointsInspector> = CellFilter::new(cell_size);
        let mut found = FoundMinima {
            value: f64::MAX,
            points: Vec::new(),
        };
        for i in 0..i1.len() - 1 {
            for j in 0..i2.len() - 1 {
                finder.set_local_params(&pair(i1[i], i2[j])?, &pair(i1[i + 1], i2[j + 1])?)?;
                finder.perform(self.get_single_solution_flag())?;
                let current: f64 = finder.minimum();
                if current >= found.value + same_tol * VALUE_TOL {
                    continue;
                }
                if current > found.value - same_tol * VALUE_TOL {
                    if current < found.value {
                        found.value = current;
                    }
                } else {
                    found.value = current;
                    filter.reset(cell_size, None)?;
                    found.points.clear();
                }
                Self::keep_new_points(
                    finder,
                    &mut filter,
                    &mut inspector,
                    &mut found.points,
                    cell_size,
                )?;
            }
        }
        Ok(found)
    }

    // The extrema of the last search that no kept point is within the cell size of.
    fn keep_new_points(
        finder: &GlobalOptimum<'_>,
        filter: &mut CellFilter<PointsInspector>,
        inspector: &mut PointsInspector,
        points: &mut Vec<[f64; 2]>,
        cell_size: f64,
    ) -> Result<(), Failure> {
        let mut sol: Vector<'static> = Vector::with_bounds(1, 2)?;
        for k in 1..=finder.number_of_extrema() {
            finder.points(k, &mut sol)?;
            let pnt2d: [f64; 2] = [sol[1], sol[2]];
            let (xy_min, xy_max) = (shift(pnt2d, -cell_size), shift(pnt2d, cell_size));
            inspector.is_find = false;
            inspector.current = pnt2d;
            filter.inspect_range(&xy_min, &xy_max, inspector);
            if !inspector.is_find {
                filter.add(&pnt2d, &pnt2d);
                points.push([sol[1], sol[2]]);
            }
        }
        Ok(())
    }
}
