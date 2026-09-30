// Rust translation of the solution filtering of Extrema_GGenExtCC::Perform and of
// Extrema_GGenExtCC_ProjPOnC at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The solutions of Extrema_GGenExtCC: the sorted minima are grouped into runs whose
//! middles keep the minimal value; when every middle does and the runs move in one
//! direction, the curves are checked for parallelism by projecting the range ends.
use super::super::glob_opt_func_cc::GlobOptFuncCCC2;
use super::GGenExtCC;
use super::search::FoundMinima;
use crate::extrema::curve_tool::ExtremaCurveTool;
use crate::extrema::point_curve::GGExtPC;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkmath::math::Vector;

/// The run starts, the parallel flag and whether the runs grow in V with U.
struct Runs {
    solutions: Vec<usize>,
    is_parallel: bool,
    dirs_coinside: bool,
}

// `Extrema_GGenExtCC_comp`: X then Y.
fn compare(a: &[f64; 2], b: &[f64; 2]) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    if a[0] < b[0] || (a[0] == b[0] && a[1] < b[1]) {
        return Ordering::Less;
    }
    if b[0] < a[0] || (a[0] == b[0] && b[1] < a[1]) {
        return Ordering::Greater;
    }
    Ordering::Equal
}

/// `Extrema_GGenExtCC_ProjPOnC(P, ProjTool)`: the smallest square distance of the
/// projection of `point`, RealLast() when there is none.
fn project_on_curve<T: ExtremaCurveTool>(
    point: T::Point,
    tool: &mut GGExtPC<'_, T>,
) -> Result<f64, Failure> {
    let mut distance: f64 = f64::MAX;
    tool.perform(point)?;
    if tool.is_done() && tool.nb_ext()? != 0 {
        for i in 1..=tool.nb_ext()? {
            let d: f64 = tool.square_distance(i)?;
            if d < distance {
                distance = d;
            }
        }
    }
    Ok(distance)
}

impl<T: ExtremaCurveTool> GGenExtCC<'_, T> {
    /// Keeps one point per run, or the first point of parallel curves.
    pub(super) fn keep_solutions(
        &mut self,
        c1: &T::Curve,
        c2: &T::Curve,
        mut found: FoundMinima,
    ) -> Result<(), Failure> {
        if found.points.len() == 1 {
            self.push_solution(found.points[0]);
            return Ok(());
        }
        found.points.sort_by(compare);
        let runs: Runs = Self::group_runs(c1, c2, &found)?;
        let parallel: bool =
            runs.is_parallel && self.ends_project_at(c1, c2, found.value, runs.dirs_coinside)?;
        if parallel {
            self.push_solution(found.points[0]);
            self.parallel = true;
            return Ok(());
        }
        for index in runs.solutions {
            self.push_solution(found.points[index]);
        }
        Ok(())
    }

    fn push_solution(&mut self, point: [f64; 2]) {
        self.points1.push(point[0]);
        self.points2.push(point[1]);
    }

    // The runs of consecutive points whose middle keeps the minimal value.
    fn group_runs(c1: &T::Curve, c2: &T::Curve, found: &FoundMinima) -> Result<Runs, Failure> {
        let points: &[[f64; 2]] = &found.points;
        let function = GlobOptFuncCCC2::<T>::new(c1, c2);
        let mut runs = Runs {
            solutions: Vec::new(),
            is_parallel: true,
            dirs_coinside: true,
        };
        let (mut save_solution, mut different_solutions) = (true, false);
        let mut value: f64 = 0.0;
        let mut middle: Vector<'static> = Vector::filled(1, 2, 0.0)?;
        for index in 0..points.len() - 1 {
            let (current, next) = (points[index], points[index + 1]);
            middle[1] = (current[0] + next[0]) * 0.5;
            middle[2] = (current[1] + next[1]) * 0.5;
            function.value(&middle, &mut value)?;
            if (value - found.value).abs() < precision::confusion() {
                if save_solution {
                    runs.solutions.push(index);
                    save_solution = false;
                }
            } else {
                runs.is_parallel = false;
                runs.solutions.push(index);
                save_solution = true;
            }
            if !different_solutions && next[0] > current[0] {
                if next[1] > current[1] {
                    (different_solutions, runs.dirs_coinside) = (true, true);
                } else if next[1] < current[1] {
                    (different_solutions, runs.dirs_coinside) = (true, false);
                }
            }
        }
        runs.solutions.push(points.len() - 1);
        if !different_solutions {
            runs.is_parallel = false;
        }
        Ok(runs)
    }

    // Whether the range ends of each curve project on the other at the minimal
    // distance (the square distance within Precision::Confusion() of the square of the
    // minimal value).
    fn ends_project_at(
        &self,
        c1: &T::Curve,
        c2: &T::Curve,
        value: f64,
        dirs_coinside: bool,
    ) -> Result<bool, Failure> {
        let t1: [f64; 2] = [self.low_border[0], self.upp_border[0]];
        let t2: [f64; 2] = if dirs_coinside {
            [self.low_border[1], self.upp_border[1]]
        } else {
            [self.upp_border[1], self.low_border[1]]
        };
        let mut ext_pc1 = GGExtPC::<T>::new();
        let mut ext_pc2 = GGExtPC::<T>::new();
        ext_pc1.initialize(c1, self.low_border[0], self.upp_border[0], 1.0e-10)?;
        ext_pc2.initialize(c2, self.low_border[1], self.upp_border[1], 1.0e-10)?;
        for it in 0..2 {
            let dist1: f64 = project_on_curve(T::value(c1, t1[it])?, &mut ext_pc2)?;
            let dist2: f64 = project_on_curve(T::value(c2, t2[it])?, &mut ext_pc1)?;
            let is_parallel: bool =
                (std_min(dist1, dist2) - value * value).abs() < precision::confusion();
            if !is_parallel {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
