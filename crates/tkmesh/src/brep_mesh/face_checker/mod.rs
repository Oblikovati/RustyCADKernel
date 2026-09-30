//! Native segment-tree face intersection checks (OCCT 3d097a03).
use crate::imesh_data::FaceHandle;
use rustycad_tkernel::{
    impl_transient,
    ncollection::{Array1, Map, Shared},
    standard::{Failure, Handle, SharedState, StandardTransient},
};
mod collect;
mod intersect;
mod selector;
mod types;
pub use types::*;
/// Native self-intersection checker with live parameter ownership; `BRepMeshFaceChecker::new(&face,&parameters)`.
pub struct BRepMeshFaceChecker {
    transient: StandardTransient,
    face: FaceHandle,
    parameters: FaceCheckerParameters,
    wires: SharedState<WireCollections>,
    intersecting: SharedState<Handle<FaceEdgeSet>>,
}
#[derive(Clone, Default)]
struct WireCollections {
    segments: Handle<FaceSegmentArray>,
    trees: Handle<FaceBoundsTreeArray>,
    intersections: Handle<FaceEdgeSetArray>,
}
impl_transient!(
    BRepMeshFaceChecker,
    "BRepMesh_FaceChecker",
    StandardTransient,
    transient
);
impl BRepMeshFaceChecker {
    /// Preserve the native parameter reference through a shared owner; `BRepMeshFaceChecker::new(&face,&parameters)`.
    pub fn new(face: &FaceHandle, parameters: &FaceCheckerParameters) -> Self {
        Self {
            transient: StandardTransient::default(),
            face: face.clone(),
            parameters: parameters.clone(),
            wires: SharedState::default(),
            intersecting: SharedState::default(),
        }
    }
    /// Check all wires, returning false for intersections; `checker.perform()?`.
    pub fn perform(&self) -> Result<bool, Failure> {
        *self.intersecting.write() = new_edge_set();
        self.collect_segments()?;
        crate::model_parallel::model_indices(self.face.wires_nb(), self.is_parallel(), |index| {
            self.process_wire(index)
        })?;
        self.collect_result()?;
        *self.wires.write() = WireCollections::default();
        Ok(self.intersecting.read().read().is_empty())
    }
    /// Preserve the nullable result handle before Perform; `checker.get_intersecting_edges()`.
    pub fn get_intersecting_edges(&self) -> Handle<FaceEdgeSet> {
        self.intersecting.snapshot()
    }
    /// Native operator() entry point for prepared wire collections; `checker.process_wire(0)?`.
    pub fn process_wire(&self, index: i32) -> Result<(), Failure> {
        intersect::process(self, index)
    }
    fn is_parallel(&self) -> bool {
        self.parameters.read().in_parallel && self.face.wires_nb() > 1
    }
    fn collect_segments(&self) -> Result<(), Failure> {
        let segments = Handle::new(Shared::new(SharedState::new(
            Array1::with_bounds(0, self.face.wires_nb() - 1).map_err(array_failure)?,
        )));
        self.wires.write().segments = segments;
        let trees = Handle::new(Shared::new(SharedState::new(
            Array1::with_bounds(0, self.face.wires_nb() - 1).map_err(array_failure)?,
        )));
        self.wires.write().trees = trees;
        crate::model_parallel::model_indices(self.face.wires_nb(), self.is_parallel(), |index| {
            collect::wire(self, index)
        })?;
        self.wires.write().intersections = Handle::new(Shared::new(SharedState::new(
            Array1::with_bounds(0, self.face.wires_nb() - 1).map_err(array_failure)?,
        )));
        Ok(())
    }
    fn collect_result(&self) -> Result<(), Failure> {
        let collections = self.wires.snapshot();
        let destination = self.get_intersecting_edges();
        for index in 0..self.face.wires_nb() {
            let edges = collections
                .intersections
                .read()
                .value(index)
                .map_err(array_failure)?
                .clone();
            if edges.is_null() {
                continue;
            }
            rustycad_tkernel::ncollection::map_algo::unite(&mut destination.write(), &edges.read());
        }
        Ok(())
    }
}
fn new_edge_set() -> Handle<FaceEdgeSet> {
    Handle::new(Shared::new(SharedState::new(Map::default())))
}

fn array_failure(error: rustycad_tkernel::ncollection::ArrayError) -> Failure {
    error.into_failure("BRepMesh_FaceChecker")
}

fn shared_collection<T: Send + Sync + 'static>(value: T) -> Handle<Shared<SharedState<T>>> {
    Handle::new(Shared::new(SharedState::new(value)))
}
