// Rust translation of ModelingAlgorithms/TKBool/BRepAlgo/BRepAlgo_NormalProjection.hxx/.cxx
// (the constructors, the parameters and the accessors) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepAlgo_NormalProjection`: the normal projection of edges on the faces of a
//! shape, the projections approximated and trimmed by the faces.
mod build;
mod edge;

use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Compound, Edge, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::{CurveType, Shape as Continuity};
use rustycad_tktopalgo::brep_lib::BRepLibMakeWire;

/// The normal projection of edges on a shape (`BRepAlgo_NormalProjection`);
/// `let mut p = BRepAlgoNormalProjection::with_shape(&face); p.add(&edge)?; p.build()?;`.
pub struct BRepAlgoNormalProjection {
    my_shape: Shape,
    my_is_done: bool,
    my_tol3d: f64,
    my_tol2d: f64,
    my_max_dist: f64,
    my_with3d: bool,
    my_continuity: Continuity,
    my_max_degree: i32,
    my_max_seg: i32,
    my_face_bounds: bool,
    my_to_proj: Shape,
    my_ancestor_map: DataMap<Shape, Shape, ShapeMapHasher>,
    my_corresp: DataMap<Shape, Shape, ShapeMapHasher>,
    my_descendants: DataMap<Shape, List<Shape>, ShapeMapHasher>,
    my_res: Shape,
}

impl Default for BRepAlgoNormalProjection {
    /// `BRepAlgo_NormalProjection()`: an empty compound of edges to project, the
    /// default parameters, no maximal distance, with 3d curves, limited by the faces.
    fn default() -> Self {
        let mut to_proj = Compound::default();
        BRepBuilder::default().make_compound(&mut to_proj);
        let mut projection = Self {
            my_shape: Shape::default(),
            my_is_done: false,
            my_tol3d: 0.,
            my_tol2d: 0.,
            my_max_dist: -1.,
            my_with3d: true,
            my_continuity: Continuity::C1,
            my_max_degree: 0,
            my_max_seg: 0,
            my_face_bounds: true,
            my_to_proj: to_proj.into_shape(),
            my_ancestor_map: DataMap::default(),
            my_corresp: DataMap::default(),
            my_descendants: DataMap::default(),
            my_res: Shape::default(),
        };
        projection.set_default_params();
        projection
    }
}

impl BRepAlgoNormalProjection {
    /// `BRepAlgo_NormalProjection()`; `BRepAlgoNormalProjection::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `BRepAlgo_NormalProjection(S)`: the projection on the faces of `S`.
    pub fn with_shape(s: &Shape) -> Self {
        let mut projection = Self::default();
        projection.init(s);
        projection
    }

    /// `Init(S)`: the shape to project on; `p.init(&face)`.
    pub fn init(&mut self, s: &Shape) {
        self.my_shape = s.clone();
    }

    /// `Add(ToProj)`: an edge or wire to project; `p.add(&edge)?`.
    pub fn add(&mut self, to_proj: &Shape) -> Result<(), Failure> {
        BRepBuilder::default().add(&mut self.my_to_proj, to_proj)
    }

    /// `SetParams(Tol3D, Tol2D, InternalContinuity, MaxDegree, MaxSeg)`;
    /// `p.set_params(1e-4, 1e-3, Continuity::C1, 14, 16)`.
    pub fn set_params(
        &mut self,
        tol3d: f64,
        tol2d: f64,
        internal_continuity: Continuity,
        max_degree: i32,
        max_seg: i32,
    ) {
        (
            self.my_tol3d,
            self.my_tol2d,
            self.my_continuity,
            self.my_max_degree,
            self.my_max_seg,
        ) = (tol3d, tol2d, internal_continuity, max_degree, max_seg);
    }

    /// `SetDefaultParams()`: 1e-4 in 3d, its power 2/3 in 2d, C1, degree 14, 16
    /// segments; `p.set_default_params()`.
    pub fn set_default_params(&mut self) {
        self.my_tol3d = 1.0e-4;
        self.my_tol2d = self.my_tol3d.powf(2. / 3.);
        self.my_continuity = Continuity::C1;
        self.my_max_degree = 14;
        self.my_max_seg = 16;
    }

    /// `SetLimit(FaceBoundaries)` (natively true): whether the projections are trimmed
    /// by the faces; `p.set_limit(true)`.
    pub fn set_limit(&mut self, face_bounds: bool) {
        self.my_face_bounds = face_bounds;
    }

    /// `SetMaxDistance(MaxDist)`: the greatest projection distance (none when
    /// negative); `p.set_max_distance(1.0)`.
    pub fn set_max_distance(&mut self, max_dist: f64) {
        self.my_max_dist = max_dist;
    }

    /// `Compute3d(With3d)` (natively true): whether the 3d curves are computed;
    /// `p.compute3d(true)`.
    pub fn compute3d(&mut self, with3d: bool) {
        self.my_with3d = with3d;
    }

    /// `IsDone()`; `p.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_is_done
    }

    /// `Projection()`: the compound of the projected edges and vertices;
    /// `p.projection()`.
    pub fn projection(&self) -> &Shape {
        &self.my_res
    }

    /// `Ancestor(E)`: the projected edge of a projection; Standard_NoSuchObject for
    /// another edge; `p.ancestor(&edge)?`.
    pub fn ancestor(&self, e: &Edge) -> Result<&Shape, Failure> {
        self.my_ancestor_map.find(e.as_ref())
    }

    /// `Couple(E)`: the face of a projection; `p.couple(&edge)?`.
    pub fn couple(&self, e: &Edge) -> Result<&Shape, Failure> {
        self.my_corresp.find(e.as_ref())
    }

    /// `Generated(S)`: the projections of an edge (only those kept without the
    /// section, as natively); `p.generated(&edge)?`.
    pub fn generated(&self, s: &Shape) -> Result<&List<Shape>, Failure> {
        self.my_descendants.find(s)
    }

    /// `IsElementary(C)`: whether the curve is a line or a conic;
    /// `p.is_elementary(&curve)?`.
    pub fn is_elementary(&self, c: &dyn Adaptor3dCurve) -> Result<bool, Failure> {
        Ok(matches!(
            c.get_type()?,
            CurveType::Line
                | CurveType::Circle
                | CurveType::Ellipse
                | CurveType::Hyperbola
                | CurveType::Parabola
        ))
    }

    /// `BuildWire(Liste)`: the wire of all the projected edges, appended when it holds
    /// them all; `let ok = p.build_wire(&mut wires)?;`.
    pub fn build_wire(&self, list_of_wire: &mut List<Shape>) -> Result<bool, Failure> {
        let mut list: List<Shape> = List::default();
        let mut exp_of_shape = Explorer::new(&self.my_res, ShapeEnum::Edge, ShapeEnum::Shape)?;
        while exp_of_shape.more() {
            list.append(exp_of_shape.current().clone());
            exp_of_shape.next()?;
        }
        if list.is_empty() {
            return Ok(false);
        }
        let mut mw = BRepLibMakeWire::new();
        mw.add_list(&list)?;
        if !mw.is_done() {
            return Ok(false);
        }
        let wire: Shape = mw.wire()?.into_shape();
        // The result is several wires when the wire lacks edges.
        if Explorer::new(&wire, ShapeEnum::Edge, ShapeEnum::Shape)?.count() != list.len() {
            return Ok(false);
        }
        list_of_wire.append(wire);
        Ok(true)
    }
}
