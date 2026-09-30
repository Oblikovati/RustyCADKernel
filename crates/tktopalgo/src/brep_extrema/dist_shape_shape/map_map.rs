// Rust translation of DistancePairFunctor, DistanceFunctor and
// BRepExtrema_DistShapeShape::DistanceMapMap
// (ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_DistShapeShape.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The map-to-map steps of `BRepExtrema_DistShapeShape`: the box pairs within the
//! reference distance, sorted by box distance, then the sub-shape distances.
use super::BRepExtremaDistShapeShape;
use super::tasks::{CheckPair, MapPair, TaskSolution, index_bands, lock, nb_threads, run_tasks, task_ranges};
use crate::brep_extrema::BRepExtremaDistanceSS;
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag};
use std::cmp::Ordering;
use std::sync::Mutex;

/// `DistancePairFunctor::operator()`: the pairs of the band whose non-void boxes are
/// within the reference distance (plus the deflection).
fn pair_task(step: &MapPair<'_>, (first, last): (i32, i32), (dist_ref, eps): (f64, f64), range: &ProgressRange<'_>) -> Vec<CheckPair> {
    let mut pairs: Vec<CheckPair> = Vec::new();
    let scope = ProgressScope::unnamed(range, f64::from(last) - f64::from(first), false);
    for index1 in first..=last {
        if !scope.more() {
            break;
        }
        drop(scope.next(1.0));
        for index2 in 1..=step.boxes2.len() as i32 {
            let box1 = &step.boxes1[index1 as usize - 1];
            let box2 = &step.boxes2[index2 as usize - 1];
            if box1.is_void() || box2.is_void() {
                continue;
            }
            let distance: f64 = box1.distance(box2);
            if distance - dist_ref < eps {
                pairs.push(CheckPair { index1, index2, distance });
            }
        }
    }
    pairs
}

/// `std::stable_sort` with the comparator `theLeft.Distance < theRight.Distance`.
fn by_box_distance(left: &CheckPair, right: &CheckPair) -> Ordering {
    if left.distance < right.distance {
        Ordering::Less
    } else if right.distance < left.distance {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

/// The pairs dealt one by one to `nb_tasks` tasks, from the nearest boxes to the
/// farthest (task `j` gets the pairs `j`, `j + nb_tasks`, ...).
fn deal(pairs: &[CheckPair], nb_tasks: usize) -> Vec<Vec<CheckPair>> {
    (0..nb_tasks)
        .map(|j| pairs.iter().skip(j).step_by(nb_tasks).copied().collect())
        .collect()
}

/// The data a `DistanceFunctor` task reads.
struct DistanceTask<'a> {
    step: &'a MapPair<'a>,
    eps: f64,
    start_dist: f64,
}

impl DistanceTask<'_> {
    /// `DistanceFunctor::operator()`: the sub-shape distances of the task pairs, until a
    /// pair whose box distance exceeds the task distance plus the deflection.
    fn run(&self, pairs: &[CheckPair], range: &ProgressRange<'_>, solution: &mut TaskSolution) -> Result<(), Failure> {
        let scope = ProgressScope::unnamed(range, pairs.len() as f64, false);
        solution.dist = self.start_dist;
        for pair in pairs {
            if !scope.more() {
                return Ok(());
            }
            drop(scope.next(1.0));
            if pair.distance > solution.dist + self.eps {
                break; // early search termination
            }
            self.pair_distance(pair, solution)?;
        }
        Ok(())
    }

    /// The distance of one pair of sub-shapes, merged into the task solution.
    fn pair_distance(&self, pair: &CheckPair, solution: &mut TaskSolution) -> Result<(), Failure> {
        let (i1, i2) = (pair.index1 as usize, pair.index2 as usize);
        let boxes = (&self.step.boxes1[i1 - 1], &self.step.boxes2[i2 - 1]);
        let shape1 = self.step.map1.find_key(i1)?;
        let shape2 = self.step.map2.find_key(i2)?;
        let options = (ExtFlag::MinMax, ExtAlgo::Grad);
        let tool = BRepExtremaDistanceSS::new(shape1, shape2, boxes, solution.dist, self.eps, options)?;
        let dist: f64 = tool.dist_value();
        if !tool.is_done() {
            return Ok(());
        }
        if dist < solution.dist - self.eps {
            solution.shape1 = tool.seq1_value().to_vec();
            solution.shape2 = tool.seq2_value().to_vec();
            solution.dist = tool.dist_value();
        } else if (dist - solution.dist).abs() < self.eps {
            solution.shape1.extend_from_slice(tool.seq1_value());
            solution.shape2.extend_from_slice(tool.seq2_value());
            if solution.dist > dist {
                solution.dist = dist;
            }
        }
        Ok(())
    }
}

impl BRepExtremaDistShapeShape {
    /// `DistanceMapMap(Map1, Map2, LBox1, LBox2, theRange)`: nothing for an empty map;
    /// the box pairs within the reference (bands of the first map, one task a band),
    /// sorted by box distance and dealt to at most one task per thread; then the task
    /// results in order lower the reference. False when the progress is stopped.
    pub(super) fn distance_map_map(&mut self, step: &MapPair<'_>, range: &ProgressRange<'_>) -> Result<bool, Failure> {
        let count1: i32 = step.map1.extent() as i32;
        if count1 == 0 || step.map2.extent() == 0 {
            return Ok(true);
        }
        let twin_scope = ProgressScope::unnamed(range, 1.0, false);
        let bands: Vec<(i32, i32)> = index_bands(count1, 10);
        drop(twin_scope.next(0.15));
        let pairs: Option<Vec<CheckPair>> = self.box_pairs(step, &bands, &twin_scope)?;
        let Some(mut pairs) = pairs else {
            return Ok(false);
        };
        if pairs.is_empty() {
            return Ok(true);
        }
        pairs.sort_by(by_box_distance);
        let nb_tasks: usize = pairs.len().min(nb_threads() as usize);
        let dealt: Vec<Vec<CheckPair>> = deal(&pairs, nb_tasks);
        let solutions: Option<Vec<TaskSolution>> = self.pair_distances(step, &dealt, &twin_scope)?;
        let Some(solutions) = solutions else {
            return Ok(false);
        };
        self.merge_task_solutions(solutions);
        Ok(true)
    }

    /// The `DistancePairFunctor` run: the pairs of every band in band order, or `None`
    /// when the progress is stopped.
    fn box_pairs(&self, step: &MapPair<'_>, bands: &[(i32, i32)], twin_scope: &ProgressScope<'_>) -> Result<Option<Vec<CheckPair>>, Failure> {
        let functor_range = twin_scope.next(0.15);
        let scope = ProgressScope::with_literal(&functor_range, "Boxes distances calculating", bands.len() as f64, false);
        let ranges = task_ranges(&scope, bands.len());
        let lists: Vec<Mutex<Vec<CheckPair>>> = bands.iter().map(|_| Mutex::default()).collect();
        let limits = (self.my_dist_ref, self.my_eps);
        run_tasks(bands.len() as i32, self.my_is_multi_thread, |index| {
            let i = index as usize;
            *lock(&lists[i]) = pair_task(step, bands[i], limits, &lock(&ranges[i]));
            Ok(())
        })?;
        if !twin_scope.more() {
            return Ok(None);
        }
        let lists = lists.into_iter().map(|list| list.into_inner().unwrap_or_else(|p| p.into_inner()));
        Ok(Some(lists.flatten().collect()))
    }

    /// The `DistanceFunctor` run over the dealt pairs: the task solutions, or `None` when
    /// the progress is stopped.
    fn pair_distances(&self, step: &MapPair<'_>, dealt: &[Vec<CheckPair>], twin_scope: &ProgressScope<'_>) -> Result<Option<Vec<TaskSolution>>, Failure> {
        let functor_range = twin_scope.next(0.85);
        let scope = ProgressScope::with_literal(&functor_range, "Shapes distances calculating", dealt.len() as f64, false);
        let ranges = task_ranges(&scope, dealt.len());
        let solutions: Vec<Mutex<TaskSolution>> = dealt.iter().map(|_| Mutex::default()).collect();
        let task = DistanceTask {
            step,
            eps: self.my_eps,
            start_dist: self.my_dist_ref,
        };
        run_tasks(dealt.len() as i32, self.my_is_multi_thread, |index| {
            let i = index as usize;
            task.run(&dealt[i], &lock(&ranges[i]), &mut lock(&solutions[i]))
        })?;
        if !twin_scope.more() {
            return Ok(None);
        }
        Ok(Some(solutions.into_iter().map(|slot| slot.into_inner().unwrap_or_else(|p| p.into_inner())).collect()))
    }

    /// The merge of the task solutions of a map-to-map step, in task order.
    fn merge_task_solutions(&mut self, solutions: Vec<TaskSolution>) {
        for solution in solutions {
            if solution.dist < self.my_dist_ref - self.my_eps {
                self.my_solutions_shape1 = solution.shape1;
                self.my_solutions_shape2 = solution.shape2;
                self.my_dist_ref = solution.dist;
            } else if (solution.dist - self.my_dist_ref).abs() < self.my_eps {
                self.my_solutions_shape1.extend(solution.shape1);
                self.my_solutions_shape2.extend(solution.shape2);
                if self.my_dist_ref > solution.dist {
                    self.my_dist_ref = solution.dist;
                }
            }
        }
    }
}
