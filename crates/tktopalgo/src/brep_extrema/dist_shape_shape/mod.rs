// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_DistShapeShape.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepExtrema_DistShapeShape`: the minimum distance between two shapes, its
//! solutions and their supports (docs/topalgo-design.md section 13). The fields, the
//! loading members and the accessors are here; `Perform` is in `perform`, the
//! parallel functors of the native file in `tasks`, the solid treatment in `solid`
//! and `Dump` in `dump`.
mod dump;
mod perform;
mod solid;
mod tasks;
mod vert_vert;
mod map_map;

use super::{BRepExtremaSolutionElem, BRepExtremaSupportType, uncompatible_shape};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::std_fail::not_done;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::Point3;
use super::extrema_capture::sequence_value;

/// The vertex, edge and face maps of one shape and their boxes (`myMapV1`, `myBV1`, ...).
#[derive(Clone, Default)]
struct ShapeMaps {
    vertices: IndexedMapOfShape,
    edges: IndexedMapOfShape,
    faces: IndexedMapOfShape,
    vertex_boxes: Vec<BoundingBox3>,
    edge_boxes: Vec<BoundingBox3>,
    face_boxes: Vec<BoundingBox3>,
}

impl ShapeMaps {
    /// `Decomposition(S, MapV, MapE, MapF)`: the maps are cleared and refilled with the
    /// vertices, edges and faces of `shape` (the boxes are kept until rebuilt).
    fn decompose(&mut self, shape: &Shape) -> Result<(), Failure> {
        self.vertices.clear(true);
        self.edges.clear(true);
        self.faces.clear(true);
        if shape.is_null() {
            return Ok(());
        }
        topexp::map_shapes_of_type(shape, ShapeEnum::Vertex, &mut self.vertices)?;
        topexp::map_shapes_of_type(shape, ShapeEnum::Edge, &mut self.edges)?;
        topexp::map_shapes_of_type(shape, ShapeEnum::Face, &mut self.faces)
    }
}

/// `BRepExtrema_DistShapeShape` (docs/topalgo-design.md section 13): the minimum
/// distance between two shapes, found between their vertices, edges and faces (the
/// sub-shape pairs sorted by the distance of their boxes), or 0 with an inner solution
/// when a vertex of one shape is inside the other (a solid). Every solution within the
/// deflection of the minimum is kept, with its point, support and parameters.
///
/// ```
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeVertex;
/// use rustycad_tktopalgo::brep_extrema::BRepExtremaDistShapeShape;
/// let v1 = BRepBuilderAPIMakeVertex::new(Point3::new(0.0, 0.0, 0.0))?.vertex()?;
/// let v2 = BRepBuilderAPIMakeVertex::new(Point3::new(0.0, 3.0, 4.0))?.vertex()?;
/// let dist = BRepExtremaDistShapeShape::with_shapes(&v1, &v2, Default::default())?;
/// assert!(dist.is_done());
/// assert_eq!(dist.value()?, 5.0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct BRepExtremaDistShapeShape {
    my_dist_ref: f64,
    my_is_done: bool,
    my_solutions_shape1: Vec<BRepExtremaSolutionElem>,
    my_solutions_shape2: Vec<BRepExtremaSolutionElem>,
    my_inner_sol: bool,
    my_eps: f64,
    my_shape1: Shape,
    my_shape2: Shape,
    maps1: ShapeMaps,
    maps2: ShapeMaps,
    my_is_init_s1: bool,
    my_is_init_s2: bool,
    my_flag: ExtFlag,
    my_algo: ExtAlgo,
    my_is_multi_thread: bool,
}

/// The obsolete flag and algorithm of the constructors (MINMAX and Grad natively).
pub type BRepExtremaObsoleteOptions = (ExtFlag, ExtAlgo);

impl Default for BRepExtremaDistShapeShape {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepExtremaDistShapeShape {
    /// `BRepExtrema_DistShapeShape()`: no shapes, deflection Precision::Confusion(),
    /// single thread; `BRepExtremaDistShapeShape::new()`.
    pub fn new() -> Self {
        Self::with_options(precision::confusion(), (ExtFlag::MinMax, ExtAlgo::Grad))
    }

    fn with_options(deflection: f64, (flag, algo): BRepExtremaObsoleteOptions) -> Self {
        Self {
            my_dist_ref: 0.0,
            my_is_done: false,
            my_solutions_shape1: Vec::new(),
            my_solutions_shape2: Vec::new(),
            my_inner_sol: false,
            my_eps: deflection,
            my_shape1: Shape::default(),
            my_shape2: Shape::default(),
            maps1: ShapeMaps::default(),
            maps2: ShapeMaps::default(),
            my_is_init_s1: false,
            my_is_init_s2: false,
            my_flag: flag,
            my_algo: algo,
            my_is_multi_thread: false,
        }
    }

    /// `BRepExtrema_DistShapeShape(Shape1, Shape2, F, A, theRange)`: loads both shapes
    /// and performs with the deflection Precision::Confusion() in a single thread (`F`
    /// and `A` are obsolete and unused);
    /// `BRepExtremaDistShapeShape::with_shapes(&s1, &s2, Default::default())?`.
    pub fn with_shapes(
        shape1: &Shape,
        shape2: &Shape,
        options: BRepExtremaObsoleteOptions,
    ) -> Result<Self, Failure> {
        Self::with_deflection(shape1, shape2, precision::confusion(), options, &ProgressRange::default())
    }

    /// `BRepExtrema_DistShapeShape(Shape1, Shape2, theDeflection, F, A, theRange)`: loads
    /// both shapes and performs with the deflection in a single thread;
    /// `BRepExtremaDistShapeShape::with_deflection(&s1, &s2, 1e-7, options, &range)?`.
    pub fn with_deflection(
        shape1: &Shape,
        shape2: &Shape,
        deflection: f64,
        options: BRepExtremaObsoleteOptions,
        range: &ProgressRange<'_>,
    ) -> Result<Self, Failure> {
        let mut dist: Self = Self::with_options(deflection, options);
        dist.load_s1(shape1)?;
        dist.load_s2(shape2)?;
        dist.perform(range)?;
        Ok(dist)
    }

    /// `SetDeflection(theDeflection)`: the deflection of the next `Perform`;
    /// `dist.set_deflection(1e-3)`.
    pub fn set_deflection(&mut self, deflection: f64) {
        self.my_eps = deflection;
    }

    /// `LoadS1(Shape1)`: the first shape and its sub-shape maps (its boxes are rebuilt
    /// by the next `Perform`); `dist.load_s1(&shape)?`.
    pub fn load_s1(&mut self, shape1: &Shape) -> Result<(), Failure> {
        self.my_shape1 = shape1.clone();
        self.my_is_init_s1 = false;
        self.maps1.decompose(shape1)
    }

    /// `LoadS2(Shape2)`: the second shape and its sub-shape maps; `dist.load_s2(&shape)?`.
    pub fn load_s2(&mut self, shape2: &Shape) -> Result<(), Failure> {
        self.my_shape2 = shape2.clone();
        self.my_is_init_s2 = false;
        self.maps2.decompose(shape2)
    }

    /// `IsDone()`: true when the minimum distance is found; `dist.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_is_done
    }

    /// `NbSolution()`: the number of solutions at the minimum distance;
    /// `dist.nb_solution()`.
    pub fn nb_solution(&self) -> i32 {
        self.my_solutions_shape1.len() as i32
    }

    /// `Value()`: the minimum distance; StdFail_NotDone when it is not found;
    /// `dist.value()?`.
    pub fn value(&self) -> Result<f64, Failure> {
        if !self.my_is_done {
            return Err(not_done("BRepExtrema_DistShapeShape::Value: There's no solution "));
        }
        Ok(self.my_dist_ref)
    }

    /// `InnerSolution()`: true when one shape is a solid with (part of) the other shape
    /// inside it; `dist.inner_solution()`.
    pub fn inner_solution(&self) -> bool {
        self.my_inner_sol
    }

    /// The solution `n` on the first or the second shape (Standard_OutOfRange outside
    /// `[1, NbSolution]`, the check of the native sequence).
    fn solution(&self, first: bool, n: i32) -> Result<BRepExtremaSolutionElem, Failure> {
        match first {
            true => sequence_value("mySolutionsShape1", &self.my_solutions_shape1, n),
            false => sequence_value("mySolutionsShape2", &self.my_solutions_shape2, n),
        }
    }

    /// `PointOnShape1(N)`: the point of the `N`-th solution on the first shape;
    /// `dist.point_on_shape1(1)?`.
    pub fn point_on_shape1(&self, n: i32) -> Result<Point3, Failure> {
        Ok(*self.solution(true, n)?.point())
    }

    /// `PointOnShape2(N)`: the point of the `N`-th solution on the second shape;
    /// `dist.point_on_shape2(1)?`.
    pub fn point_on_shape2(&self, n: i32) -> Result<Point3, Failure> {
        Ok(*self.solution(false, n)?.point())
    }

    /// `SupportTypeShape1(N)`: the support kind of the `N`-th solution on the first
    /// shape; `dist.support_type_shape1(1)?`.
    pub fn support_type_shape1(&self, n: i32) -> Result<BRepExtremaSupportType, Failure> {
        Ok(self.solution(true, n)?.support_kind())
    }

    /// `SupportTypeShape2(N)`: the support kind of the `N`-th solution on the second
    /// shape; `dist.support_type_shape2(1)?`.
    pub fn support_type_shape2(&self, n: i32) -> Result<BRepExtremaSupportType, Failure> {
        Ok(self.solution(false, n)?.support_kind())
    }

    /// The support of a solution: its vertex, edge or face.
    fn support(&self, first: bool, n: i32, member: &str) -> Result<Shape, Failure> {
        if !self.my_is_done {
            return Err(not_done(&format!("BRepExtrema_DistShapeShape::{member}: There's no solution ")));
        }
        let sol: BRepExtremaSolutionElem = self.solution(first, n)?;
        Ok(match sol.support_kind() {
            BRepExtremaSupportType::IsVertex => sol.vertex().clone().into_shape(),
            BRepExtremaSupportType::IsOnEdge => sol.edge().clone().into_shape(),
            BRepExtremaSupportType::IsInFace => sol.face().clone().into_shape(),
        })
    }

    /// `SupportOnShape1(N)`: the vertex, edge or face that supports the `N`-th solution
    /// on the first shape; StdFail_NotDone when not done; `dist.support_on_shape1(1)?`.
    pub fn support_on_shape1(&self, n: i32) -> Result<Shape, Failure> {
        self.support(true, n, "SupportOnShape1")
    }

    /// `SupportOnShape2(N)`: the support of the `N`-th solution on the second shape;
    /// `dist.support_on_shape2(1)?`.
    pub fn support_on_shape2(&self, n: i32) -> Result<Shape, Failure> {
        self.support(false, n, "SupportOnShape2")
    }

    /// A solution whose support must have the kind `expected` for a parameter read;
    /// StdFail_NotDone when not done, BRepExtrema_UnCompatibleShape for another kind.
    fn solution_on(&self, first: bool, n: i32, expected: BRepExtremaSupportType, member: &str) -> Result<BRepExtremaSolutionElem, Failure> {
        if !self.my_is_done {
            return Err(not_done(&format!("BRepExtrema_DistShapeShape::{member}: There's no solution")));
        }
        let sol: BRepExtremaSolutionElem = self.solution(first, n)?;
        if sol.support_kind() != expected {
            return Err(uncompatible_shape(&format!(
                "BRepExtrema_DistShapeShape::{member}: support {:?} of solution {n}; expected {expected:?}",
                sol.support_kind()
            )));
        }
        Ok(sol)
    }

    /// `ParOnEdgeS1(N, t)`: the edge parameter of the `N`-th solution on the first
    /// shape; `let t = dist.par_on_edge_s1(1)?;`.
    pub fn par_on_edge_s1(&self, n: i32) -> Result<f64, Failure> {
        let sol = self.solution_on(true, n, BRepExtremaSupportType::IsOnEdge, "ParOnEdgeS1")?;
        Ok(sol.edge_parameter())
    }

    /// `ParOnEdgeS2(N, t)`: the edge parameter of the `N`-th solution on the second
    /// shape; `let t = dist.par_on_edge_s2(1)?;`.
    pub fn par_on_edge_s2(&self, n: i32) -> Result<f64, Failure> {
        let sol = self.solution_on(false, n, BRepExtremaSupportType::IsOnEdge, "ParOnEdgeS2")?;
        Ok(sol.edge_parameter())
    }

    /// `ParOnFaceS1(N, u, v)`: the face parameters of the `N`-th solution on the first
    /// shape; `let (u, v) = dist.par_on_face_s1(1)?;`.
    pub fn par_on_face_s1(&self, n: i32) -> Result<(f64, f64), Failure> {
        let sol = self.solution_on(true, n, BRepExtremaSupportType::IsInFace, "ParOnFaceS1")?;
        Ok(sol.face_parameter())
    }

    /// `ParOnFaceS2(N, u, v)`: the face parameters of the `N`-th solution on the second
    /// shape; `let (u, v) = dist.par_on_face_s2(1)?;`.
    pub fn par_on_face_s2(&self, n: i32) -> Result<(f64, f64), Failure> {
        let sol = self.solution_on(false, n, BRepExtremaSupportType::IsInFace, "ParOnFaceS2")?;
        Ok(sol.face_parameter())
    }

    /// `SetFlag(F)`: stores the obsolete, unused flag; `dist.set_flag(ExtFlag::Min)`.
    pub fn set_flag(&mut self, flag: ExtFlag) {
        self.my_flag = flag;
    }

    /// `SetAlgo(A)`: stores the obsolete, unused algorithm; `dist.set_algo(ExtAlgo::Tree)`.
    pub fn set_algo(&mut self, algo: ExtAlgo) {
        self.my_algo = algo;
    }

    /// The obsolete flag and algorithm, as stored (`myFlag`, `myAlgo`);
    /// `let (flag, algo) = dist.obsolete_options();`.
    pub fn obsolete_options(&self) -> BRepExtremaObsoleteOptions {
        (self.my_flag, self.my_algo)
    }

    /// `SetMultiThread(theIsMultiThread)`: whether the next `Perform` runs its tasks on
    /// the default thread pool; `dist.set_multi_thread(true)`.
    pub fn set_multi_thread(&mut self, is_multi_thread: bool) {
        self.my_is_multi_thread = is_multi_thread;
    }

    /// `IsMultiThread()`: false by default; `dist.is_multi_thread()`.
    pub fn is_multi_thread(&self) -> bool {
        self.my_is_multi_thread
    }
}
