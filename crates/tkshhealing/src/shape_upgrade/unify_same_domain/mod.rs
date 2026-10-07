// Rust translation of ModelingAlgorithms/TKShHealing/ShapeUpgrade/ShapeUpgrade_UnifySameDomain.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeUpgrade_UnifySameDomain`: unifies the faces lying on the same surface and the
//! edges lying on the same curve, rebuilding the shape through a ShapeBuild_ReShape
//! context and recording the history of the modifications.
mod edge_chains;
mod face_bounds;
mod face_groups;
mod face_wires;
mod geometry;
mod glue_edges;
mod history;
mod internal_wires;
mod merge_sub_seq;
mod new_faces;
mod pcurve_transform;
mod periodic_faces;
mod relocate;
mod same_domain;
mod seams;
mod split_wire;
mod unify_edges;
mod unify_faces;
mod union_pcurves;

use crate::shape_build::ShapeBuildReShape;
use rustycad_tkbrep::brep_tools::{History, ReShape};
use rustycad_tkbrep::topods::{Edge, Shape};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape, ShapeMapHasher};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{DataMap, HashPolicy};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `ShapeUpgrade_UnifySameDomain::DataMapOfFacePlane`: the plane shared by the faces
/// found coplanar.
pub type DataMapOfFacePlane = DataMap<Shape, Handle<GeomPlane>, ShapeMapHasher>;

/// `ShapeUpgrade_UnifySameDomain::DataMapOfShapeMapOfShape`: the shells of each face.
pub type DataMapOfShapeMapOfShape = DataMap<Shape, MapOfShape, ShapeMapHasher>;

/// `NCollection_DefaultHasher<TopoDS_Shape>`: the shape hash and `operator==` (`IsEqual`,
/// the orientation included), the hasher of the maps declared without one.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct ShapeDefaultHasher;

impl HashPolicy<Shape> for ShapeDefaultHasher {
    fn hash(&self, key: &Shape) -> usize {
        key.hash_code()
    }
    fn equal(&self, first: &Shape, second: &Shape) -> bool {
        first == second
    }
}

/// `NCollection_DataMap<TopoDS_Shape, occ::handle<Geom2d_Curve>>`: the new p-curve of each
/// edge.
pub(super) type EdgePCurveMap = DataMap<Shape, Handle<dyn Geom2dCurve>, ShapeDefaultHasher>;

/// `ShapeUpgrade_UnifySameDomain::SubSequenceOfEdges`: a chain of edges and the edge
/// that replaces it (null when the chain cannot be merged).
#[derive(Clone, Debug, Default)]
pub struct ShapeUpgradeSubSequenceOfEdges {
    /// `SeqsEdges`.
    pub seqs_edges: Vec<Shape>,
    /// `UnionEdges`.
    pub union_edges: Edge,
}

/// The members of the native class.
pub(super) struct UnifyState {
    init_shape: Shape,
    lin_tol: f64,
    ang_tol: f64,
    unify_faces: bool,
    unify_edges: bool,
    concat_bsplines: bool,
    allow_internal: bool,
    safe_input_mode: bool,
    shape: Shape,
    context: Handle<ShapeBuildReShape>,
    keep_shapes: MapOfShape,
    face_plane_map: DataMapOfFacePlane,
    ef_map: IndexedDataMapOfShapeListOfShape,
    face_new_face: DataMap<Shape, Shape, ShapeMapHasher>,
    history: Handle<History>,
}

/// `ShapeUpgrade_UnifySameDomain`: merges the faces of a shape lying on the same surface
/// (planes compared with GeomLib_IsPlanarSurface, elementary surfaces with
/// IntPatch_ImpImpIntersection, cylinders also as swept surfaces) and the edges lying on
/// the same line or circle (or any B-spline chain with `ConcatBSplines`).
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeUnifySameDomain;
/// let unifier = ShapeUpgradeUnifySameDomain::new(&Shape::default(), true, true, false);
/// unifier.build().unwrap();
/// assert!(unifier.shape().is_null());
/// ```
pub struct ShapeUpgradeUnifySameDomain {
    parent: StandardTransient,
    state: SharedState<UnifyState>,
}
impl_transient!(
    ShapeUpgradeUnifySameDomain,
    "ShapeUpgrade_UnifySameDomain",
    StandardTransient,
    parent
);

impl Default for ShapeUpgradeUnifySameDomain {
    fn default() -> Self {
        Self::empty()
    }
}

impl UnifyState {
    fn new(shape: &Shape, unify_edges: bool, unify_faces: bool, concat_bsplines: bool) -> Self {
        Self {
            init_shape: shape.clone(),
            lin_tol: precision::confusion(),
            ang_tol: precision::angular(),
            unify_faces,
            unify_edges,
            concat_bsplines,
            allow_internal: false,
            safe_input_mode: true,
            shape: shape.clone(),
            context: Handle::new(ShapeBuildReShape::new()),
            keep_shapes: MapOfShape::default(),
            face_plane_map: DataMapOfFacePlane::default(),
            ef_map: IndexedDataMapOfShapeListOfShape::default(),
            face_new_face: DataMap::default(),
            history: Handle::new(History::new()),
        }
    }

    /// The context as the ReShape interface.
    fn context(&self) -> Result<&ShapeBuildReShape, Failure> {
        self.context.get().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "ShapeUpgrade_UnifySameDomain: null context; expected a ShapeBuild_ReShape",
                "",
            )
        })
    }

    /// `myContext->Apply(shape)`.
    fn apply(&self, shape: &Shape) -> Result<Shape, Failure> {
        self.context()?.apply(shape, ShapeEnum::Shape)
    }

    /// `Build()`.
    fn build(&mut self) -> Result<(), Failure> {
        rustycad_tkbrep::topexp::map_shapes_and_ancestors(
            &self.init_shape,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut self.ef_map,
        )?;
        if self.unify_faces {
            self.unify_faces()?;
        }
        if self.unify_edges {
            self.unify_edges()?;
        }
        self.fill_history()
    }
}

impl ShapeUpgradeUnifySameDomain {
    /// `ShapeUpgrade_UnifySameDomain()`: no shape, both unifications on, B-spline
    /// concatenation off, Precision::Confusion and Precision::Angular tolerances;
    /// `ShapeUpgradeUnifySameDomain::empty()`.
    pub fn empty() -> Self {
        Self::new(&Shape::default(), true, true, false)
    }

    /// `ShapeUpgrade_UnifySameDomain(aShape, UnifyEdges, UnifyFaces, ConcatBSplines)`;
    /// `ShapeUpgradeUnifySameDomain::new(&shape, true, true, false)`.
    pub fn new(shape: &Shape, unify_edges: bool, unify_faces: bool, concat_bsplines: bool) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(UnifyState::new(
                shape,
                unify_edges,
                unify_faces,
                concat_bsplines,
            )),
        }
    }

    /// `Initialize(aShape, UnifyEdges, UnifyFaces, ConcatBSplines)`: a new shape and
    /// flags; the context, the kept shapes, the internal maps and the history are cleared;
    /// `unifier.initialize(&shape, true, true, false)`.
    pub fn initialize(
        &self,
        shape: &Shape,
        unify_edges: bool,
        unify_faces: bool,
        concat_bsplines: bool,
    ) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.init_shape = shape.clone();
        state.shape = shape.clone();
        state.unify_edges = unify_edges;
        state.unify_faces = unify_faces;
        state.concat_bsplines = concat_bsplines;
        state.context()?.clear();
        state.keep_shapes.clear(false);
        state.face_plane_map.clear(false);
        state.ef_map.clear(false);
        state.face_new_face.clear(false);
        state.history.get().ok_or_else(null_history)?.clear();
        Ok(())
    }

    /// `AllowInternalEdges(theValue)`; `unifier.allow_internal_edges(true)`.
    pub fn allow_internal_edges(&self, value: bool) {
        self.state.write().allow_internal = value;
    }

    /// `KeepShape(theShape)`: an edge or a vertex kept from the unification (other
    /// types are ignored); `unifier.keep_shape(&edge)`.
    pub fn keep_shape(&self, shape: &Shape) -> Result<(), Failure> {
        if matches!(shape.shape_type()?, ShapeEnum::Edge | ShapeEnum::Vertex) {
            self.state.write().keep_shapes.add(shape.clone());
        }
        Ok(())
    }

    /// `KeepShapes(theShapes)`: [`Self::keep_shape`] for each shape;
    /// `unifier.keep_shapes(&shapes)`.
    pub fn keep_shapes(&self, shapes: &MapOfShape) -> Result<(), Failure> {
        for shape in shapes.iter() {
            self.keep_shape(shape)?;
        }
        Ok(())
    }

    /// `SetSafeInputMode(theValue)`: when true (the default), the input shapes are not
    /// modified (merged vertices are copied); `unifier.set_safe_input_mode(false)`.
    pub fn set_safe_input_mode(&self, value: bool) {
        self.state.write().safe_input_mode = value;
    }

    /// `SetLinearTolerance(theValue)`; `unifier.set_linear_tolerance(1e-5)`.
    pub fn set_linear_tolerance(&self, value: f64) {
        self.state.write().lin_tol = value;
    }

    /// `SetAngularTolerance(theValue)`: at least Precision::Angular;
    /// `unifier.set_angular_tolerance(1e-3)`.
    pub fn set_angular_tolerance(&self, value: f64) {
        self.state.write().ang_tol = value.max(precision::angular());
    }

    /// `Build()`: the face unification, then the edge unification (as enabled), then the
    /// history; `unifier.build()?`.
    pub fn build(&self) -> Result<(), Failure> {
        self.state.write().build()
    }

    /// `Shape()`: the result; `unifier.shape()`.
    pub fn shape(&self) -> Shape {
        self.state.read().shape.clone()
    }

    /// `History()`: the history of the modifications; `unifier.history()`.
    pub fn history(&self) -> Handle<History> {
        self.state.read().history.clone()
    }

    /// Writes the non-const `History()` reference (a null handle disables the history);
    /// `unifier.set_history(&Handle::default())`.
    pub fn set_history(&self, history: &Handle<History>) {
        self.state.write().history = history.clone();
    }

    /// `UnifyFaces()` (protected): the faces of each shell, then the free faces, merged;
    /// `unifier.unify_faces()?`.
    pub fn unify_faces(&self) -> Result<(), Failure> {
        self.state.write().unify_faces()
    }

    /// `UnifyEdges()` (protected): the chains of edges merged and the changed faces and
    /// shells fixed; `unifier.unify_edges()?`.
    pub fn unify_edges(&self) -> Result<(), Failure> {
        self.state.write().unify_edges()
    }

    /// `FillHistory()` (protected): the modifications of the context relevant to the
    /// result merged into the history; `unifier.fill_history()?`.
    pub fn fill_history(&self) -> Result<(), Failure> {
        self.state.read().fill_history()
    }
}

fn null_history() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "ShapeUpgrade_UnifySameDomain: null history; expected a BRepTools_History",
        "",
    )
}
