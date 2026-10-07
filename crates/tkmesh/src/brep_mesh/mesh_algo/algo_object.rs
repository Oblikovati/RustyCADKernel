// Rust translation of the RTTI classes of the BRepMesh face algorithms
// (BRepMesh_BaseMeshAlgo, BRepMesh_ConstrainedBaseMeshAlgo, BRepMesh_CustomBaseMeshAlgo,
// BRepMesh_DelaunayBaseMeshAlgo) at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::base::{BaseMeshAlgoHooks, base_perform};
use super::deflection_control::DelaunayDeflectionControlMeshAlgo;
use super::delaunay_insertion::DelaunayNodeInsertionMeshAlgo;
use super::generator::{DelaunayBaseMeshAlgoCore, MeshGenerator};
use super::node_insertion::{NodeInsertionHooks, NodeInsertionMeshAlgo, node_insertion_perform};
use crate::brep_mesh::RangeSplitter;
use crate::imesh_data::FaceHandle;
use crate::imesh_tools::{MeshAlgo, MeshParameters};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, StandardTransient};
use rustycad_tkernel::{impl_transient, impl_transient_interface};
use std::sync::{Mutex, PoisonError};

/// `BRepMesh_BaseMeshAlgo` in the runtime type system (abstract).
pub trait BaseMeshAlgo: MeshAlgo {}
impl_transient_interface!(dyn BaseMeshAlgo, "BRepMesh_BaseMeshAlgo", dyn MeshAlgo);

/// `BRepMesh_ConstrainedBaseMeshAlgo` in the runtime type system (abstract).
pub trait ConstrainedBaseMeshAlgo: BaseMeshAlgo {}
impl_transient_interface!(
    dyn ConstrainedBaseMeshAlgo,
    "BRepMesh_ConstrainedBaseMeshAlgo",
    dyn BaseMeshAlgo
);

/// `BRepMesh_CustomBaseMeshAlgo` in the runtime type system (abstract; its only
/// native concrete descendant is `BRepMeshDelabellaBaseMeshAlgo`).
pub trait CustomBaseMeshAlgo: ConstrainedBaseMeshAlgo {}
impl_transient_interface!(
    dyn CustomBaseMeshAlgo,
    "BRepMesh_CustomBaseMeshAlgo",
    dyn ConstrainedBaseMeshAlgo
);

/// The `Perform` of one face algorithm object (the template instance behind a
/// handle); `core.perform_face(&face, &parameters, &range)?`.
pub trait FaceMeshingCore: Send {
    /// The algorithm's Perform; an error is a failure that leaves Perform natively.
    fn perform_face(
        &mut self,
        face: &FaceHandle,
        parameters: &MeshParameters,
        range: &ProgressRange<'_>,
    ) -> Result<(), Failure>;
}

impl FaceMeshingCore for DelaunayBaseMeshAlgoCore {
    fn perform_face(
        &mut self,
        face: &FaceHandle,
        parameters: &MeshParameters,
        range: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        base_perform(self, face, parameters, range);
        Ok(())
    }
}

macro_rules! node_insertion_core {
    ($($algo:ident),*) => {$(
        impl<R: RangeSplitter + Send, G: MeshGenerator + Send> FaceMeshingCore for $algo<R, G>
        where
            $algo<R, G>: NodeInsertionHooks + BaseMeshAlgoHooks,
        {
            fn perform_face(
                &mut self,
                face: &FaceHandle,
                parameters: &MeshParameters,
                range: &ProgressRange<'_>,
            ) -> Result<(), Failure> {
                node_insertion_perform(self, face, parameters, range)
            }
        }
    )*};
}
node_insertion_core!(
    NodeInsertionMeshAlgo,
    DelaunayNodeInsertionMeshAlgo,
    DelaunayDeflectionControlMeshAlgo
);

/// The algorithm object behind a face-algorithm handle, locked because `Perform` takes
/// the C++ object mutably through a shared handle; `LockedFaceCore::new(core)`.
pub(super) struct LockedFaceCore(Mutex<Box<dyn FaceMeshingCore>>);

impl LockedFaceCore {
    /// Boxes and locks one algorithm object.
    pub(super) fn new(core: impl FaceMeshingCore + 'static) -> Self {
        Self(Mutex::new(Box::new(core)))
    }
    /// The object's Perform under the lock.
    pub(super) fn perform(
        &self,
        face: &FaceHandle,
        parameters: &MeshParameters,
        range: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        // A panic cannot happen in the cores; a poisoned lock still holds a usable core.
        let mut core = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        core.perform_face(face, parameters, range)
    }
}

/// `BRepMesh_DelaunayBaseMeshAlgo` as a handle: the plain algorithm, or one of the
/// node-insertion template instances over it (whose dynamic type is this class, as
/// the C++ templates declare no RTTI of their own).
///
/// ```
/// use rustycad_tkmesh::brep_mesh::BRepMeshDelaunayBaseMeshAlgo;
/// use rustycad_tkernel::standard::RuntimeTyped;
/// assert_eq!(BRepMeshDelaunayBaseMeshAlgo::TYPE_NAME, "BRepMesh_DelaunayBaseMeshAlgo");
/// let _algo = BRepMeshDelaunayBaseMeshAlgo::new();
/// ```
pub struct BRepMeshDelaunayBaseMeshAlgo {
    transient: StandardTransient,
    core: LockedFaceCore,
}
impl_transient!(
    BRepMeshDelaunayBaseMeshAlgo,
    "BRepMesh_DelaunayBaseMeshAlgo",
    dyn ConstrainedBaseMeshAlgo,
    transient,
    interfaces: [dyn MeshAlgo, dyn BaseMeshAlgo, dyn ConstrainedBaseMeshAlgo]
);

impl Default for BRepMeshDelaunayBaseMeshAlgo {
    fn default() -> Self {
        Self::from_core(DelaunayBaseMeshAlgoCore::default())
    }
}

impl BRepMeshDelaunayBaseMeshAlgo {
    /// `BRepMesh_DelaunayBaseMeshAlgo()`; `BRepMeshDelaunayBaseMeshAlgo::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// A template instance over this class (`new NodeInsertionMeshAlgo<...>::Type`);
    /// `BRepMeshDelaunayBaseMeshAlgo::from_core(algo)`.
    pub fn from_core(core: impl FaceMeshingCore + 'static) -> Self {
        Self {
            transient: StandardTransient::default(),
            core: LockedFaceCore::new(core),
        }
    }
}

impl MeshAlgo for BRepMeshDelaunayBaseMeshAlgo {
    fn perform(
        &self,
        face: &FaceHandle,
        parameters: &MeshParameters,
        range: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        self.core.perform(face, parameters, range)
    }
}
impl BaseMeshAlgo for BRepMeshDelaunayBaseMeshAlgo {}
impl ConstrainedBaseMeshAlgo for BRepMeshDelaunayBaseMeshAlgo {}
