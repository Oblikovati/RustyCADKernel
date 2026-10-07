// Rust translation of BRepMesh_IncrementalMesh.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepMesh_IncrementalMesh`: the meshing of a shape through BRepMesh_Context.
use super::BRepMeshContext;
use super::discret_root::{BRepMeshDiscretRoot, DiscretRootBase};
use crate::imesh_data::{MeshModel, Status};
use crate::imesh_tools::{MeshBuilder, MeshContext, MeshParameters};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use std::sync::RwLockWriteGuard;
use std::sync::atomic::{AtomicBool, Ordering};

/// `IS_IN_PARALLEL`: the parallel flag given to the algorithms of the factories.
static IS_IN_PARALLEL: AtomicBool = AtomicBool::new(false);

/// `BRepMesh_IncrementalMesh`: builds the triangulation of every face and the
/// polygons of every edge of a shape.
///
/// ```
/// use rustycad_tkmesh::brep_mesh::BRepMeshIncrementalMesh;
/// use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
/// use rustycad_tkmesh::brep_mesh::BRepMeshDiscretRoot;
/// let shape = BRepPrimAPIMakeBox::from_sizes(1.0, 2.0, 3.0)?.shape()?.clone();
/// let mesher = BRepMeshIncrementalMesh::with_shape(&shape, 0.1, false, 0.5, false)?;
/// assert!(mesher.is_done());
/// assert_eq!(mesher.get_status_flags(), 0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BRepMeshIncrementalMesh {
    transient: StandardTransient,
    root: DiscretRootBase,
    parameters: SharedState<MeshParameters>,
    modified: SharedState<bool>,
    status: SharedState<i32>,
}
impl_transient!(BRepMeshIncrementalMesh, "BRepMesh_IncrementalMesh", dyn BRepMeshDiscretRoot, transient, interfaces: [dyn BRepMeshDiscretRoot]);

impl Default for BRepMeshIncrementalMesh {
    fn default() -> Self {
        Self::with_parameter_state(MeshParameters::default())
    }
}

impl BRepMeshIncrementalMesh {
    /// `BRepMesh_IncrementalMesh()`: not modified, status IMeshData_NoError;
    /// `BRepMeshIncrementalMesh::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    fn with_parameter_state(parameters: MeshParameters) -> Self {
        Self {
            transient: StandardTransient::default(),
            root: DiscretRootBase::default(),
            parameters: SharedState::new(parameters),
            modified: SharedState::new(false),
            status: SharedState::new(Status::NoError.0),
        }
    }

    /// `BRepMesh_IncrementalMesh(theShape, theLinDeflection, isRelative,
    /// theAngDeflection, isInParallel)`: meshes at once; Standard_NumericError for a
    /// deflection below Precision::Confusion() or an angle below Precision::Angular();
    /// `BRepMeshIncrementalMesh::with_shape(&shape, 0.1, false, 0.5, false)?`.
    pub fn with_shape(
        shape: &Shape,
        lin_deflection: f64,
        is_relative: bool,
        ang_deflection: f64,
        is_in_parallel: bool,
    ) -> Result<Self, Failure> {
        let parameters = MeshParameters {
            deflection: lin_deflection,
            angle: ang_deflection,
            relative: is_relative,
            in_parallel: is_in_parallel,
            ..MeshParameters::default()
        };
        Self::with_parameters(shape, &parameters, &ProgressRange::default())
    }

    /// `BRepMesh_IncrementalMesh(theShape, theParameters, theRange)`: meshes at once.
    /// The native constructor leaves myModified and myStatus uninitialized; Rust starts
    /// them as the default constructor does;
    /// `BRepMeshIncrementalMesh::with_parameters(&shape, &parameters, &range)?`.
    pub fn with_parameters(
        shape: &Shape,
        parameters: &MeshParameters,
        range: &ProgressRange<'_>,
    ) -> Result<Self, Failure> {
        let mesher = Self::with_parameter_state(*parameters);
        mesher.set_shape(shape);
        mesher.perform(range)?;
        Ok(mesher)
    }

    /// `Parameters()`; `mesher.parameters().deflection`.
    pub fn parameters(&self) -> MeshParameters {
        self.parameters.snapshot()
    }
    /// `ChangeParameters()`; `mesher.change_parameters().deflection = 0.2`.
    pub fn change_parameters(&self) -> RwLockWriteGuard<'_, MeshParameters> {
        self.parameters.write()
    }
    /// `IsModified()`; `mesher.is_modified()`.
    pub fn is_modified(&self) -> bool {
        *self.modified.read()
    }
    /// `GetStatusFlags()`: the face and wire statuses of the last Perform, or
    /// IMeshData_UserBreak; `mesher.get_status_flags()`.
    pub fn get_status_flags(&self) -> i32 {
        *self.status.read()
    }

    /// `Perform(theContext, theRange)`: the parameters checked and completed, the
    /// mesh builder run on the context, the statuses collected, then done;
    /// `mesher.perform_with_context(&context, &range)?`.
    pub fn perform_with_context(
        &self,
        context: &Handle<dyn MeshContext>,
        range: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        self.init_parameters()?;
        let context_ref = context.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "BRepMesh_IncrementalMesh::Perform: context=null; expected a context",
            )
        })?;
        context_ref.set_shape(&self.shape());
        *context_ref.change_parameters() = self.parameters();
        context_ref.change_parameters().clean_model = false;
        let scope = ProgressScope::with_literal(range, "Perform incmesh", 10.0, false);
        MeshBuilder::with_context(context).perform(&scope.next(9.0));
        if !scope.more() {
            *self.status.write() = Status::UserBreak.0;
            return Ok(());
        }
        *self.status.write() = model_status(&context_ref.get_model())?;
        drop(scope.next(1.0));
        self.set_done();
        Ok(())
    }

    /// `initParameters`: Standard_NumericError for a deflection or angle below the
    /// precision; interior deflection, minimal size and interior angle defaulted.
    fn init_parameters(&self) -> Result<(), Failure> {
        let mut parameters = self.parameters.write();
        if parameters.deflection < precision::confusion() {
            return Err(invalid_parameter(parameters.deflection, "deflection"));
        }
        if parameters.deflection_interior < precision::confusion() {
            parameters.deflection_interior = parameters.deflection;
        }
        if parameters.min_size < precision::confusion() {
            parameters.min_size = (MeshParameters::rel_min_size()
                * parameters.deflection.min(parameters.deflection_interior))
            .max(precision::confusion());
        }
        if parameters.angle < precision::angular() {
            return Err(invalid_parameter(parameters.angle, "angle"));
        }
        if parameters.angle_interior < precision::angular() {
            parameters.angle_interior = 2.0 * parameters.angle;
        }
        Ok(())
    }

    /// `Discret(theShape, theLinDeflection, theAngDeflection, theAlgo)`: a new mesher
    /// with these parameters and the default parallel flag, not yet performed; the
    /// returned code is always 0; `BRepMeshIncrementalMesh::discret(&shape, 0.1, 0.5)`.
    pub fn discret(
        shape: &Shape,
        deflection: f64,
        angle: f64,
    ) -> (i32, Handle<dyn BRepMeshDiscretRoot>) {
        let algo = Self::new();
        {
            let mut parameters = algo.change_parameters();
            parameters.deflection = deflection;
            parameters.angle = angle;
            parameters.in_parallel = Self::is_parallel_default();
        }
        algo.set_shape(shape);
        (0, Handle::new(algo).cast())
    }

    /// `IsParallelDefault()`; `BRepMeshIncrementalMesh::is_parallel_default()`.
    pub fn is_parallel_default() -> bool {
        IS_IN_PARALLEL.load(Ordering::SeqCst)
    }
    /// `SetParallelDefault(isInParallel)`; `BRepMeshIncrementalMesh::set_parallel_default(true)`.
    pub fn set_parallel_default(in_parallel: bool) {
        IS_IN_PARALLEL.store(in_parallel, Ordering::SeqCst);
    }
}

impl BRepMeshDiscretRoot for BRepMeshIncrementalMesh {
    fn discret_root_base(&self) -> &DiscretRootBase {
        &self.root
    }
    /// `Perform(theRange)`: through a new BRepMesh_Context of the parameters' type.
    fn perform(&self, range: &ProgressRange<'_>) -> Result<(), Failure> {
        let context = BRepMeshContext::new(self.parameters().mesh_algo)?;
        self.perform_with_context(&Handle::new(context).cast(), range)
    }
}

/// The union of the face and wire status masks of the model (0 without model).
fn model_status(model: &Handle<dyn MeshModel>) -> Result<i32, Failure> {
    let mut status = Status::NoError.0;
    if model.is_null() {
        return Ok(status);
    }
    for face_index in 0..model.faces_nb() {
        let face = model.get_face(face_index)?;
        status |= face.status().get_status_mask();
        for wire_index in 0..face.wires_nb() {
            status |= face.get_wire(wire_index)?.status().get_status_mask();
        }
    }
    Ok(status)
}

fn invalid_parameter(value: f64, name: &str) -> Failure {
    Failure::with_message(
        "Standard_NumericError",
        &format!(
            "BRepMesh_IncrementalMesh::initParameters : invalid parameter value ({name} {value}; expected at least the precision)"
        ),
    )
}
