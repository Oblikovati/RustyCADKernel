// Rust translation of TreatmentFunctor and BRepExtrema_DistShapeShape::SolidTreatment
// (ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_DistShapeShape.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The inner-solid treatment of `BRepExtrema_DistShapeShape`: a vertex of one shape
//! classified IN a solid of the other one is an inner solution at distance 0.
use super::BRepExtremaDistShapeShape;
use super::tasks::{lock, nb_threads, run_tasks, task_ranges};
use crate::brep_class3d::BRepClass3dSolidClassifier;
use crate::brep_extrema::{BRepExtremaSolutionElem, BRepExtremaSupportType};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::gp::Point3;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// The classification tolerance of the treatment.
const TREATMENT_TOLERANCE: f64 = 0.001; // tol:classification, the native aTolerance

/// The shared state of the `TreatmentFunctor` tasks: the found flags and the solutions
/// (the member lists and `myDistRef` natively, written under the mutex).
struct Treatment {
    inner_sol: AtomicBool,
    is_done: AtomicBool,
    found: Mutex<Vec<BRepExtremaSolutionElem>>,
}

/// The vertices of the map in tasks of at least 3 vertices (ten tasks a thread at most),
/// the last task cut at the map size.
fn vertex_tasks(map: &IndexedMapOfShape) -> Result<Vec<Vec<Shape>>, Failure> {
    let map_size: i32 = map.extent() as i32;
    let mut nb_tasks: i32 = nb_threads() * 10;
    let mut task_size: i32 = (f64::from(map_size) / f64::from(nb_tasks)).ceil() as i32;
    if task_size < 3 {
        task_size = 3;
        nb_tasks = (f64::from(map_size) / f64::from(task_size)).ceil() as i32;
    }
    let mut tasks: Vec<Vec<Shape>> = vec![Vec::new(); nb_tasks.max(0) as usize];
    for i in 1..=map_size {
        tasks[((i - 1) / task_size) as usize].push(map.find_key(i as usize)?.clone());
    }
    Ok(tasks)
}

impl Treatment {
    /// `TreatmentFunctor::operator()`: the vertices of one task classified in the solid
    /// until one is IN (or another task found one).
    fn run(
        &self,
        solid: &Shape,
        vertices: &[Shape],
        range: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let scope = ProgressScope::unnamed(range, vertices.len() as f64, false);
        let mut classifier = BRepClass3dSolidClassifier::with_shape(solid)?;
        for vertex in vertices {
            if !scope.more() {
                break;
            }
            drop(scope.next(1.0));
            if self.is_done.load(Ordering::Acquire) {
                break;
            }
            let vertex = topods::vertex(vertex)?;
            let point: Point3 = BRepTool::pnt(&vertex)?;
            classifier.perform(&point, TREATMENT_TOLERANCE)?;
            if classifier.state() == State::In {
                let mut found = lock(&self.found);
                self.inner_sol.store(true, Ordering::Release);
                self.is_done.store(true, Ordering::Release);
                found.push(BRepExtremaSolutionElem::with_vertex(
                    0.0,
                    point,
                    BRepExtremaSupportType::IsVertex,
                    &vertex,
                ));
                break;
            }
        }
        Ok(())
    }
}

impl BRepExtremaDistShapeShape {
    /// `SolidTreatment(theShape, theVertexMap, theRange)`: the vertices of the other
    /// shape classified in `solid` (tolerance 0.001) task by task; the first vertex IN
    /// sets the reference to 0, the inner solution and done flags, and is the solution
    /// on both shapes. In a parallel run, tasks that find a vertex IN before seeing the
    /// flag of another add their own solution (their order follows the thread timing).
    /// Returns false when the progress is stopped.
    pub(super) fn solid_treatment(
        &mut self,
        solid: &Shape,
        vertex_map: &IndexedMapOfShape,
        range: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let tasks: Vec<Vec<Shape>> = vertex_tasks(vertex_map)?;
        let treatment = Treatment {
            inner_sol: AtomicBool::new(self.my_inner_sol),
            is_done: AtomicBool::new(self.my_is_done),
            found: Mutex::new(Vec::new()),
        };
        let scope =
            ProgressScope::with_literal(range, "Solid treatment", tasks.len() as f64, false);
        let functor_range = scope.next(1.0);
        let functor_scope = ProgressScope::with_literal(
            &functor_range,
            "Search for the inner solid",
            tasks.len() as f64,
            false,
        );
        let ranges = task_ranges(&functor_scope, tasks.len());
        let run = run_tasks(tasks.len() as i32, self.my_is_multi_thread, |index| {
            let i = index as usize;
            treatment.run(solid, &tasks[i], &lock(&ranges[i]))
        });
        // The tasks write the member solutions and reference directly: they are kept
        // when a task raises; the flags are copied back only after the loop.
        let found: Vec<BRepExtremaSolutionElem> = treatment
            .found
            .into_inner()
            .unwrap_or_else(|p| p.into_inner());
        if !found.is_empty() {
            self.my_dist_ref = 0.0;
        }
        self.my_solutions_shape1.extend(found.iter().cloned());
        self.my_solutions_shape2.extend(found);
        run?;
        self.my_inner_sol = treatment.inner_sol.load(Ordering::Acquire);
        self.my_is_done = treatment.is_done.load(Ordering::Acquire);
        Ok(scope.more())
    }
}
