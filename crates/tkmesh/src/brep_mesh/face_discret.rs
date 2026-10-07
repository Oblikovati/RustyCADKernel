// Rust translation of BRepMesh_FaceDiscret.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepMesh_FaceDiscret`: the face stage of the mesher, one algorithm per face from
//! the algorithm factory.
use crate::imesh_data::{FaceHandle, MeshModel, Status};
use crate::imesh_tools::{MeshAlgoFactory, MeshParameters, ModelAlgo};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;

/// `BRepMesh_FaceDiscret(theAlgoFactory)`: each face that is neither failed nor
/// reused is meshed by the algorithm the factory gives for its surface type; a
/// failure marks the face (IMeshData_Failure) and the other faces go on.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmesh::brep_mesh::{BRepMeshFaceDiscret, BRepMeshMeshAlgoFactory};
/// let discret = BRepMeshFaceDiscret::new(&Handle::new(BRepMeshMeshAlgoFactory::new()).cast());
/// assert!(!discret.algo_factory().is_null());
/// ```
pub struct BRepMeshFaceDiscret {
    transient: StandardTransient,
    algo_factory: Handle<dyn MeshAlgoFactory>,
}
impl_transient!(BRepMeshFaceDiscret, "BRepMesh_FaceDiscret", dyn ModelAlgo, transient, interfaces: [dyn ModelAlgo]);

impl BRepMeshFaceDiscret {
    /// `BRepMesh_FaceDiscret(theAlgoFactory)`; `BRepMeshFaceDiscret::new(&factory)`.
    pub fn new(algo_factory: &Handle<dyn MeshAlgoFactory>) -> Self {
        Self {
            transient: StandardTransient::default(),
            algo_factory: algo_factory.clone(),
        }
    }

    /// `myAlgoFactory`; `discret.algo_factory()`.
    pub fn algo_factory(&self) -> &Handle<dyn MeshAlgoFactory> {
        &self.algo_factory
    }

    /// `process(theFaceIndex, theRange)`: the native exception boundary of one face.
    fn process(
        &self,
        face: &FaceHandle,
        parameters: &MeshParameters,
        range: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        if face.status().is_set(Status::Failure) || face.status().is_set(Status::Reused) {
            return Ok(());
        }
        if self.process_face(face, parameters, range).is_err() {
            face.status().set_status(Status::Failure);
        }
        Ok(())
    }

    /// The try block of `process`: the algorithm of the surface type, Failure when the
    /// factory has none, UserBreak when cancelled.
    fn process_face(
        &self,
        face: &FaceHandle,
        parameters: &MeshParameters,
        range: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let factory = self.algo_factory.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "BRepMesh_FaceDiscret: algorithm factory=null; expected a factory",
            )
        })?;
        let algo = factory.get_algo(face.get_surface().get_type()?, parameters)?;
        let Some(algo) = algo.get() else {
            face.status().set_status(Status::Failure);
            return Ok(());
        };
        if !range.more() {
            face.status().set_status(Status::UserBreak);
            return Ok(());
        }
        algo.perform(face, parameters, range)
    }
}

impl ModelAlgo for BRepMeshFaceDiscret {
    /// `performInternal`: one progress range per face; false for a null model or a
    /// user break.
    fn perform_internal(
        &self,
        model: &Handle<dyn MeshModel>,
        parameters: &MeshParameters,
        range: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        if model.is_null() {
            return Ok(false);
        }
        let faces_nb = model.faces_nb();
        let scope = ProgressScope::with_literal(range, "Face Discret", f64::from(faces_nb), false);
        let ranges: Vec<ProgressRange<'_>> = (0..faces_nb).map(|_| scope.next(1.0)).collect();
        // OSD_Parallel::For over the faces when InParallel: each face algorithm reads
        // and writes only its own face, so the sequential order gives the same meshes;
        // the Rust progress scopes are not shareable between threads.
        for (index, face_range) in ranges.iter().enumerate() {
            if !scope.more() {
                break;
            }
            let face_scope = ProgressScope::unnamed(face_range, 1.0, false);
            self.process(
                &model.get_face(index as i32)?,
                parameters,
                &face_scope.next(1.0),
            )?;
        }
        Ok(range.more())
    }
}
