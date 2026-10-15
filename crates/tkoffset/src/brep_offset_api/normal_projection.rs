// Rust translation of ModelingAlgorithms/TKOffset/BRepOffsetAPI/BRepOffsetAPI_NormalProjection.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepOffsetAPI_NormalProjection`: the builder-API face of
//! `BRepAlgo_NormalProjection`, edges and wires projected along the normals of a shape.
use rustycad_tkbool::brep_algo::BRepAlgoNormalProjection;
use rustycad_tkbrep::topods::{Edge, Shape};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase};

/// `BRepOffsetAPI_NormalProjection`: projects the added shapes on the base shape at
/// `Build`; `IsDone` is the projector's flag.
///
/// ```
/// use rustycad_tkoffset::brep_offset_api::BRepOffsetAPINormalProjection;
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
/// let projection = BRepOffsetAPINormalProjection::new();
/// assert!(!projection.is_done());
/// ```
pub struct BRepOffsetAPINormalProjection {
    base: BRepBuilderAPIMakeShapeBase,
    my_normal_projector: BRepAlgoNormalProjection,
}

impl Default for BRepOffsetAPINormalProjection {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepBuilderAPIMakeShape for BRepOffsetAPINormalProjection {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        &self.base
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        &mut self.base
    }

    /// `IsDone()`: whether the projection is built.
    fn is_done(&self) -> bool {
        self.my_normal_projector.is_done()
    }

    /// `Build()`: the projection, then done.
    fn build(&mut self, _range: &ProgressRange<'_>) -> Result<(), Failure> {
        self.my_normal_projector.build()?;
        self.base.my_shape = self.my_normal_projector.projection().clone();
        self.base.set_done();
        Ok(())
    }

    /// `Generated(S)`: the projections of an added shape.
    fn generated(&mut self, shape: &Shape) -> Result<&List<Shape>, Failure> {
        self.my_normal_projector.generated(shape)
    }
}

impl BRepOffsetAPINormalProjection {
    /// `BRepOffsetAPI_NormalProjection()`: no base shape and the default parameters;
    /// `BRepOffsetAPINormalProjection::new()`.
    pub fn new() -> Self {
        Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            my_normal_projector: BRepAlgoNormalProjection::new(),
        }
    }

    /// `BRepOffsetAPI_NormalProjection(S)`: the base shape;
    /// `BRepOffsetAPINormalProjection::with_shape(&shape)`.
    pub fn with_shape(s: &Shape) -> Self {
        let mut projection = Self::new();
        projection.init(s);
        projection
    }

    /// `Init(S)`: the base shape; `projection.init(&shape)`.
    pub fn init(&mut self, s: &Shape) {
        self.my_normal_projector.init(s);
    }

    /// `Add(ToProj)`: an edge or wire to project; `projection.add(&wire)?`.
    pub fn add(&mut self, to_proj: &Shape) -> Result<(), Failure> {
        self.my_normal_projector.add(to_proj)
    }

    /// `SetParams(Tol3D, Tol2D, InternalContinuity, MaxDegree, MaxSeg)`;
    /// `projection.set_params(1e-4, 1e-3, Continuity::C1, 14, 16)`.
    pub fn set_params(
        &mut self,
        tol3d: f64,
        tol2d: f64,
        internal_continuity: Continuity,
        max_degree: i32,
        max_seg: i32,
    ) {
        self.my_normal_projector
            .set_params(tol3d, tol2d, internal_continuity, max_degree, max_seg);
    }

    /// `SetMaxDistance(MaxDist)`: no projection farther than the distance (a negative
    /// one for none); `projection.set_max_distance(1.0)`.
    pub fn set_max_distance(&mut self, max_dist: f64) {
        self.my_normal_projector.set_max_distance(max_dist);
    }

    /// `SetLimit(FaceBoundaries)` (natively defaulted to true): the projections cut by
    /// the face boundaries; `projection.set_limit(true)`.
    pub fn set_limit(&mut self, face_boundaries: bool) {
        self.my_normal_projector.set_limit(face_boundaries);
    }

    /// `Compute3d(With3d)` (natively defaulted to true); `projection.compute3d(true)`.
    pub fn compute3d(&mut self, with3d: bool) {
        self.my_normal_projector.compute3d(with3d);
    }

    /// `Projection()`: the compound of the projected edges; `projection.projection()`.
    pub fn projection(&self) -> &Shape {
        self.my_normal_projector.projection()
    }

    /// `Couple(E)`: the face a projected edge lies on; `projection.couple(&edge)?`.
    pub fn couple(&self, e: &Edge) -> Result<&Shape, Failure> {
        self.my_normal_projector.couple(e)
    }

    /// `Ancestor(E)`: the added edge a projected edge comes from;
    /// `projection.ancestor(&edge)?`.
    pub fn ancestor(&self, e: &Edge) -> Result<&Shape, Failure> {
        self.my_normal_projector.ancestor(e)
    }

    /// `BuildWire(Liste)`: the projected edges assembled into wires; false when they
    /// do not make wires; `projection.build_wire(&mut wires)?`.
    pub fn build_wire(&self, list_of_wire: &mut List<Shape>) -> Result<bool, Failure> {
        self.my_normal_projector.build_wire(list_of_wire)
    }
}
