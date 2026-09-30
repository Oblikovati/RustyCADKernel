//! Native model healing, endpoint repair and five-pass edge amplification.
use super::{BRepMeshFaceChecker, FaceCheckerParameters, FaceEdgeSet};
use crate::{
    imesh_data::{FaceHandle, FacePtr, MeshModel, Status},
    imesh_tools::{MeshParameters, ModelAlgo},
};
use rustycad_tkernel::{
    impl_transient,
    message::ProgressRange,
    ncollection::{DataMap, Shared},
    standard::{Failure, Handle, SharedState, StandardTransient},
};
mod amplify;
mod boundaries;
mod connect;
mod vertex;
type FaceIntersections = Handle<Shared<SharedState<DataMap<FacePtr, Handle<FaceEdgeSet>>>>>;
/// Native model healing stage; `BRepMeshModelHealer::new()`.
pub struct BRepMeshModelHealer {
    transient: StandardTransient,
    model: SharedState<Handle<dyn MeshModel>>,
    parameters: FaceCheckerParameters,
    intersections: SharedState<FaceIntersections>,
}
impl_transient!(BRepMeshModelHealer,"BRepMesh_ModelHealer",dyn ModelAlgo,transient,interfaces:[dyn ModelAlgo]);
impl BRepMeshModelHealer {
    /// Construct the native stage with owned parameter storage; `BRepMeshModelHealer::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Process an active model face by index; `healer.process_index(0)?`.
    pub fn process_index(&self, index: i32) -> Result<(), Failure> {
        let model = self.model.snapshot();
        let model = model.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "null model; expected active model healing",
            )
        })?;
        self.process_face(&model.get_face(index)?);
        Ok(())
    }
    /// Catch native face-level failures and retain its failure flag; `healer.process_face(&face)`.
    pub fn process_face(&self, face: &FaceHandle) {
        if self.check_face(face).is_err() {
            face.status().set_status(Status::Failure);
        }
    }
    fn check_face(&self, face: &FaceHandle) -> Result<(), Failure> {
        self.assign_intersections(face, Handle::default())?;
        boundaries::fix(face, &self.parameters.read())?;
        if face.status().is_set(Status::Failure) {
            return Ok(());
        }
        let checker = BRepMeshFaceChecker::new(face, &self.parameters);
        if !checker.perform()? {
            return self.assign_intersections(face, checker.get_intersecting_edges());
        }
        self.assign_intersections(face, boundaries::degenerate_edges(face)?)
    }
    fn assign_intersections(
        &self,
        face: &FaceHandle,
        edges: Handle<FaceEdgeSet>,
    ) -> Result<(), Failure> {
        let intersections = self.intersections.snapshot();
        let intersections = intersections.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "null intersection map; expected active model healing",
            )
        })?;
        *intersections.write().change_find(&face.downgrade())? = edges;
        Ok(())
    }
    fn initialize_faces(&self, model: &Handle<dyn MeshModel>) -> Result<(), Failure> {
        let intersections = Handle::new(Shared::new(SharedState::new(DataMap::default())));
        *self.intersections.write() = intersections.clone();
        for index in 0..model.faces_nb() {
            intersections
                .write()
                .bind(model.get_face(index)?.downgrade(), Handle::default());
        }
        Ok(())
    }
    fn process_model_faces(
        &self,
        model: &Handle<dyn MeshModel>,
        parallel: bool,
    ) -> Result<(), Failure> {
        crate::model_parallel::model_indices(
            model.faces_nb(),
            parallel && model.faces_nb() > 1,
            |index| self.process_index(index),
        )
    }
    fn release_model(&self) {
        *self.intersections.write() = Handle::default();
        *self.model.write() = Handle::default();
    }
    fn mark_unresolved(&self) {
        for (face, edges) in self.intersections.read().read().items() {
            if edges.is_null() {
                continue;
            }
            let face = face.upgrade();
            face.status().set_status(Status::SelfIntersectingWire);
            face.status().set_status(Status::Failure);
        }
    }
}
impl ModelAlgo for BRepMeshModelHealer {
    fn perform_internal(
        &self,
        model: &Handle<dyn MeshModel>,
        parameters: &MeshParameters,
        _range: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        *self.model.write() = model.clone();
        *self.parameters.write() = *parameters;
        if model.is_null() {
            return Ok(false);
        }
        // Native testmesh standard_mesh C7 face 17 needs fine enough boundary repair.
        self.parameters.write().min_size = rustycad_tkernel::precision::confusion();
        self.initialize_faces(model)?;
        self.process_model_faces(model, parameters.in_parallel)?;
        self.amplify_edges()?;
        self.mark_unresolved();
        self.release_model();
        Ok(true)
    }
}

impl Default for BRepMeshModelHealer {
    fn default() -> Self {
        Self {
            transient: StandardTransient::default(),
            model: SharedState::default(),
            parameters: Handle::new(Shared::new(SharedState::new(MeshParameters::default()))),
            intersections: SharedState::default(),
        }
    }
}
