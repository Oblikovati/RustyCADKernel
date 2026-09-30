// Rust translation of the task structures of
// ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_DistShapeShape.cxx (IndexBand,
// ThreadSolution, BRepExtrema_CheckPair and the OSD_Parallel::For calls) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The task layout shared by the parallel steps of `BRepExtrema_DistShapeShape`: the
//! index bands, the per-task solutions and the runner of `OSD_Parallel::For`.
use super::ShapeMaps;
use crate::brep_extrema::BRepExtremaSolutionElem;
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::osd::{Parallel, ThreadPool};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::BoundingBox3;
use std::sync::{Mutex, MutexGuard};

/// One map-to-map step: the maps and the boxes of both sides.
pub(super) struct MapPair<'a> {
    pub(super) map1: &'a IndexedMapOfShape,
    pub(super) map2: &'a IndexedMapOfShape,
    pub(super) boxes1: &'a [BoundingBox3],
    pub(super) boxes2: &'a [BoundingBox3],
}

/// The constructor of one step from the chosen map and boxes of each side.
macro_rules! map_pair {
    ($name:ident, $map1:ident, $boxes1:ident, $map2:ident, $boxes2:ident) => {
        pub(super) fn $name(maps1: &'a ShapeMaps, maps2: &'a ShapeMaps) -> Self {
            Self {
                map1: &maps1.$map1,
                map2: &maps2.$map2,
                boxes1: &maps1.$boxes1,
                boxes2: &maps2.$boxes2,
            }
        }
    };
}

impl<'a> MapPair<'a> {
    map_pair!(vertices_edges, vertices, vertex_boxes, edges, edge_boxes);
    map_pair!(edges_vertices, edges, edge_boxes, vertices, vertex_boxes);
    map_pair!(vertices_faces, vertices, vertex_boxes, faces, face_boxes);
    map_pair!(faces_vertices, faces, face_boxes, vertices, vertex_boxes);
    map_pair!(edges_edges, edges, edge_boxes, edges, edge_boxes);
    map_pair!(edges_faces, edges, edge_boxes, faces, face_boxes);
    map_pair!(faces_edges, faces, face_boxes, edges, edge_boxes);
    map_pair!(faces_faces, faces, face_boxes, faces, face_boxes);
}

/// `BRepExtrema_CheckPair`: a pair of sub-shape indices and the distance of their boxes.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct CheckPair {
    pub(super) index1: i32,
    pub(super) index2: i32,
    pub(super) distance: f64,
}

/// One slot of `ThreadSolution`: the solutions of a task and its distance (DBL_MAX
/// until the task starts).
pub(super) struct TaskSolution {
    pub(super) shape1: Vec<BRepExtremaSolutionElem>,
    pub(super) shape2: Vec<BRepExtremaSolutionElem>,
    pub(super) dist: f64,
}

impl Default for TaskSolution {
    fn default() -> Self {
        Self {
            shape1: Vec::new(),
            shape2: Vec::new(),
            dist: f64::MAX,
        }
    }
}

impl TaskSolution {
    /// The solutions `pair` at `dist` offered to a task: a smaller distance (by more
    /// than `eps`) replaces the task solutions, an equal one (within it) is appended
    /// and lowers the task distance.
    pub(super) fn offer(
        &mut self,
        dist: f64,
        eps: f64,
        pair: impl FnOnce() -> (Vec<BRepExtremaSolutionElem>, Vec<BRepExtremaSolutionElem>),
    ) {
        if dist < self.dist - eps {
            (self.shape1, self.shape2) = pair();
            self.dist = dist;
        } else if (dist - self.dist).abs() < eps {
            let (shape1, shape2) = pair();
            self.shape1.extend(shape1);
            self.shape2.extend(shape2);
            if self.dist > dist {
                self.dist = dist;
            }
        }
    }
}

/// The lock of a slot; a poisoned slot is recovered (a task never panics while it holds
/// its slot, the failure transport panics after the slot is released).
pub(super) fn lock<T>(slot: &Mutex<T>) -> MutexGuard<'_, T> {
    slot.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// The number of threads of the default pool (`OSD_ThreadPool::DefaultPool()->NbThreads()`),
/// which sizes the tasks whether the run is parallel or not.
pub(super) fn nb_threads() -> i32 {
    ThreadPool::default_pool(-1).nb_threads()
}

/// `(int)std::ceil((double)a / b)`.
fn ceil_div(a: i32, b: i32) -> i32 {
    (f64::from(a) / f64::from(b)).ceil() as i32
}

/// The bands `[First, Last]` (from 1) of `count` indices: one per thread, or bands of
/// `min(count, min_size)` indices when the thread bands would be smaller; the last band
/// is cut at `count` (the `IndexBand` arrays of DistanceVertVert and DistanceMapMap).
pub(super) fn index_bands(count: i32, min_size: i32) -> Vec<(i32, i32)> {
    let min_task_size: i32 = count.min(min_size);
    let mut nb_tasks: i32 = nb_threads();
    let mut task_size: i32 = ceil_div(count, nb_tasks);
    if task_size < min_task_size {
        task_size = min_task_size;
        nb_tasks = ceil_div(count, task_size);
    }
    let mut first: i32 = 1;
    let mut bands: Vec<(i32, i32)> = Vec::with_capacity(nb_tasks.max(0) as usize);
    for _ in 0..nb_tasks {
        if count < first + task_size - 1 {
            task_size = count - first + 1;
        }
        bands.push((first, first + task_size - 1));
        first += task_size;
    }
    bands
}

/// `OSD_Parallel::For(0, nbTasks, functor, !isMultiThread)`: in the caller, in order,
/// when forced or for a single task (a failure stops the loop and is returned as is);
/// on the default pool otherwise, where a failure is caught by the pool and returned
/// as Standard_ProgramError, as the native launcher does.
pub(super) fn run_tasks(
    nb_tasks: i32,
    is_multi_thread: bool,
    task: impl Fn(i32) -> Result<(), Failure> + Sync,
) -> Result<(), Failure> {
    if !is_multi_thread || nb_tasks <= 1 {
        for index in 0..nb_tasks {
            task(index)?;
        }
        return Ok(());
    }
    let pooled = |index: i32| {
        if let Err(failure) = task(index) {
            // The pool transports a task exception as a panic payload (OSD port).
            std::panic::panic_any(failure);
        }
    };
    Parallel::for_range(0, nb_tasks, pooled, false)
}

/// The scope of a functor (`Scope(theRange, name, n)`) is created by the caller; this
/// is its `Ranges` array: one step of the scope per task, each behind its own lock so
/// the tasks can open their scopes in parallel.
pub(super) fn task_ranges<'s>(
    scope: &'s ProgressScope<'s>,
    nb_tasks: usize,
) -> Vec<Mutex<ProgressRange<'s>>> {
    (0..nb_tasks).map(|_| Mutex::new(scope.next(1.0))).collect()
}
