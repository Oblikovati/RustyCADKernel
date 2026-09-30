// Rust translation of VertexFunctor and BRepExtrema_DistShapeShape::DistanceVertVert
// (ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_DistShapeShape.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The vertex-vertex step of `BRepExtrema_DistShapeShape`.
use super::BRepExtremaDistShapeShape;
use super::tasks::{TaskSolution, index_bands, lock, run_tasks, task_ranges};
use crate::brep_extrema::{BRepExtremaSolutionElem, BRepExtremaSupportType};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Vertex};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;
use std::sync::Mutex;

/// The data a `VertexFunctor` task reads.
struct VertexTask<'a> {
    map1: &'a IndexedMapOfShape,
    map2: &'a IndexedMapOfShape,
    eps: f64,
    start_dist: f64,
}

/// A vertex of a map with its point.
fn vertex_point(map: &IndexedMapOfShape, index: i32) -> Result<(Vertex, Point3), Failure> {
    let vertex: Vertex = topods::vertex(map.find_key(index as usize)?)?;
    let point: Point3 = BRepTool::pnt(&vertex)?;
    Ok((vertex, point))
}

impl VertexTask<'_> {
    /// `VertexFunctor::operator()`: the vertices of the band against every vertex of the
    /// second map; a smaller distance (by more than the deflection) replaces the task
    /// solutions, an equal one (within it) is appended.
    fn run(
        &self,
        (first, last): (i32, i32),
        range: &ProgressRange<'_>,
        solution: &mut TaskSolution,
    ) -> Result<(), Failure> {
        let count2: i32 = self.map2.extent() as i32;
        solution.dist = self.start_dist;
        let scope = ProgressScope::unnamed(range, f64::from(last) - f64::from(first), false);
        for index1 in first..=last {
            if !scope.more() {
                break;
            }
            drop(scope.next(1.0));
            let (vertex1, point1) = vertex_point(self.map1, index1)?;
            for index2 in 1..=count2 {
                let (vertex2, point2) = vertex_point(self.map2, index2)?;
                let dist: f64 = point1.distance(&point2);
                let sol1 = || {
                    BRepExtremaSolutionElem::with_vertex(
                        dist,
                        point1,
                        BRepExtremaSupportType::IsVertex,
                        &vertex1,
                    )
                };
                let sol2 = || {
                    BRepExtremaSolutionElem::with_vertex(
                        dist,
                        point2,
                        BRepExtremaSupportType::IsVertex,
                        &vertex2,
                    )
                };
                if dist < solution.dist - self.eps {
                    solution.shape1 = vec![sol1()];
                    solution.shape2 = vec![sol2()];
                    solution.dist = dist;
                } else if (dist - solution.dist).abs() < self.eps {
                    solution.shape1.push(sol1());
                    solution.shape2.push(sol2());
                    if solution.dist > dist {
                        solution.dist = dist;
                    }
                }
            }
        }
        Ok(())
    }
}

impl BRepExtremaDistShapeShape {
    /// `DistanceVertVert(theMap1, theMap2, theRange)`: the vertex pairs of both shapes in
    /// bands of the first map (at least 10 vertices a band), one task per band; then the
    /// task results in order lower the reference (an equal one within the deflection is
    /// appended and replaces the reference). False when the progress is stopped.
    pub(super) fn distance_vert_vert(
        &mut self,
        range: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let bands: Vec<(i32, i32)> = index_bands(self.maps1.vertices.extent() as i32, 10);
        let dist_scope = ProgressScope::unnamed(range, 1.0, false);
        let functor_range = dist_scope.next(1.0);
        let scope = ProgressScope::with_literal(
            &functor_range,
            "Vertices distances calculating",
            bands.len() as f64,
            false,
        );
        let ranges = task_ranges(&scope, bands.len());
        let solutions: Vec<Mutex<TaskSolution>> = bands.iter().map(|_| Mutex::default()).collect();
        let task = VertexTask {
            map1: &self.maps1.vertices,
            map2: &self.maps2.vertices,
            eps: self.my_eps,
            start_dist: self.my_dist_ref,
        };
        run_tasks(bands.len() as i32, self.my_is_multi_thread, |index| {
            let i = index as usize;
            task.run(bands[i], &lock(&ranges[i]), &mut lock(&solutions[i]))
        })?;
        drop(ranges);
        drop(scope);
        drop(functor_range);
        if !dist_scope.more() {
            return Ok(false);
        }
        for slot in solutions {
            let solution: TaskSolution = slot
                .into_inner()
                .unwrap_or_else(|poison| poison.into_inner());
            if solution.dist < self.my_dist_ref - self.my_eps {
                self.my_solutions_shape1 = solution.shape1;
                self.my_solutions_shape2 = solution.shape2;
                self.my_dist_ref = solution.dist;
            } else if (solution.dist - self.my_dist_ref).abs() < self.my_eps {
                self.my_solutions_shape1.extend(solution.shape1);
                self.my_solutions_shape2.extend(solution.shape2);
                self.my_dist_ref = solution.dist;
            }
        }
        Ok(true)
    }
}
