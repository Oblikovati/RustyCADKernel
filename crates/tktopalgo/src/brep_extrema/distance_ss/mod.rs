// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_DistanceSS.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepExtrema_DistanceSS`: the minimum distance between two sub-shapes (vertex,
//! edge or face), used by `BRepExtrema_DistShapeShape` (docs/topalgo-design.md
//! section 13). The private `Perform` overloads live in `vertex` (vertex-vertex,
//! vertex-edge, vertex-face) and `edge_face` (edge-edge, edge-face, face-face); the
//! file-static helpers in `helpers`, `trim` and `c0`.
mod c0;
mod edge_face;
mod helpers;
mod trim;
mod vertex;

use super::BRepExtremaSolutionElem;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag};
use rustycad_tkmath::bnd::BoundingBox3;

/// The search state the `Perform` overloads share: `myDstRef`, `myModif`, `myEps`,
/// `myFlag` and `myAlgo`.
struct SearchState {
    dst_ref: f64,
    modif: bool,
    eps: f64,
    flag: ExtFlag,
    algo: ExtAlgo,
}

impl SearchState {
    /// The native acceptance of a candidate distance: smaller than the reference by
    /// more than the deflection, or equal to it within the deflection.
    fn accepts(&self, dst: f64) -> bool {
        helpers::accepts(dst, self.dst_ref, self.eps)
    }

    /// Lowers the reference to `dst` and marks the solution improved.
    fn improve(&mut self, dst: f64) {
        if self.dst_ref > dst {
            self.dst_ref = dst;
        }
        self.modif = true;
    }
}

/// `BRepExtrema_DistanceSS` (docs/topalgo-design.md section 13): the solutions of the
/// minimum distance between two sub-shapes that are not farther than the start
/// distance `theDstRef` (within the deflection), and the smallest distance found.
///
/// ```
/// use rustycad_tkmath::bnd::BoundingBox3;
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeVertex;
/// use rustycad_tktopalgo::brep_extrema::BRepExtremaDistanceSS;
/// use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag};
/// let v1 = BRepBuilderAPIMakeVertex::new(Point3::new(0.0, 0.0, 0.0))?.vertex()?;
/// let v2 = BRepBuilderAPIMakeVertex::new(Point3::new(3.0, 4.0, 0.0))?.vertex()?;
/// let void = BoundingBox3::default();
/// let dist = BRepExtremaDistanceSS::new(&v1, &v2, (&void, &void), 10.0, 1e-7, (ExtFlag::MinMax, ExtAlgo::Grad))?;
/// assert!(dist.is_done());
/// assert_eq!(dist.dist_value(), 5.0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BRepExtremaDistanceSS {
    my_seq_sol_shape1: Vec<BRepExtremaSolutionElem>,
    my_seq_sol_shape2: Vec<BRepExtremaSolutionElem>,
    search: SearchState,
}

impl BRepExtremaDistanceSS {
    /// `BRepExtrema_DistanceSS(theS1, theS2, theBox1, theBox2, theDstRef, theDeflection,
    /// theExtFlag, theExtAlgo)` (the native defaults are Precision::Confusion(), MINMAX
    /// and Grad; the flag and the algorithm only apply to the vertex-face extrema);
    /// `BRepExtremaDistanceSS::new(&s1, &s2, (&b1, &b2), dst_ref, 1e-7, (flag, algo))?`.
    pub fn new(
        s1: &Shape,
        s2: &Shape,
        (box1, box2): (&BoundingBox3, &BoundingBox3),
        dst_ref: f64,
        deflection: f64,
        (ext_flag, ext_algo): (ExtFlag, ExtAlgo),
    ) -> Result<Self, Failure> {
        let mut distance = Self {
            my_seq_sol_shape1: Vec::new(),
            my_seq_sol_shape2: Vec::new(),
            search: SearchState {
                dst_ref,
                modif: false,
                eps: deflection,
                flag: ext_flag,
                algo: ext_algo,
            },
        };
        distance.perform(s1, s2, box1, box2)?;
        Ok(distance)
    }

    /// `IsDone()`: true when the distance was improved (a solution was found);
    /// `dist.is_done()`.
    pub fn is_done(&self) -> bool {
        self.search.modif
    }

    /// `DistValue()`: the smallest distance found (the start distance when none);
    /// `dist.dist_value()`.
    pub fn dist_value(&self) -> f64 {
        self.search.dst_ref
    }

    /// `Seq1Value()`: the solutions on the first shape; `dist.seq1_value().len()`.
    pub fn seq1_value(&self) -> &[BRepExtremaSolutionElem] {
        &self.my_seq_sol_shape1
    }

    /// `Seq2Value()`: the solutions on the second shape; `dist.seq2_value().len()`.
    pub fn seq2_value(&self) -> &[BRepExtremaSolutionElem] {
        &self.my_seq_sol_shape2
    }

    /// `Perform(theS1, theS2, theBox1, theBox2)`: nothing when the box distance exceeds
    /// the start distance by more than the deflection; otherwise the overload of the
    /// two shape types (compound and wire types give nothing).
    fn perform(
        &mut self,
        s1: &Shape,
        s2: &Shape,
        box1: &BoundingBox3,
        box2: &BoundingBox3,
    ) -> Result<(), Failure> {
        self.my_seq_sol_shape1.clear();
        self.my_seq_sol_shape2.clear();
        self.search.modif = false;
        let bb_dist: f64 = box1.distance(box2);
        if bb_dist - self.search.dst_ref > self.search.eps {
            // The box distance exceeds the start distance: no better solution.
            return Ok(());
        }
        match s1.shape_type()? {
            ShapeEnum::Vertex => self.perform_vertex(s1, s2),
            ShapeEnum::Edge => self.perform_edge(s1, s2),
            ShapeEnum::Face => self.perform_face(s1, s2),
            _ => Ok(()),
        }
    }

    /// The branch of a first vertex.
    fn perform_vertex(&mut self, s1: &Shape, s2: &Shape) -> Result<(), Failure> {
        let v1 = topods::vertex(s1)?;
        let (seq1, seq2) = (&mut self.my_seq_sol_shape1, &mut self.my_seq_sol_shape2);
        match s2.shape_type()? {
            ShapeEnum::Vertex => self
                .search
                .vertex_vertex(&v1, &topods::vertex(s2)?, seq1, seq2),
            ShapeEnum::Edge => self.search.vertex_edge(&v1, &topods::edge(s2)?, seq1, seq2),
            ShapeEnum::Face => self.search.vertex_face(&v1, &topods::face(s2)?, seq1, seq2),
            _ => Ok(()),
        }
    }

    /// The branch of a first edge: an infinite edge or face is trimmed first.
    fn perform_edge(&mut self, s1: &Shape, s2: &Shape) -> Result<(), Failure> {
        let mut e1 = topods::edge(s1)?;
        let (seq1, seq2) = (&mut self.my_seq_sol_shape1, &mut self.my_seq_sol_shape2);
        match s2.shape_type()? {
            ShapeEnum::Vertex => self
                .search
                .vertex_edge(&topods::vertex(s2)?, &e1, seq2, seq1),
            ShapeEnum::Edge => {
                let mut e2 = topods::edge(s2)?;
                let trimmed = trim::trim_infinite_edge(&e1, &e2)?;
                if trimmed.trim1 {
                    e1 = trimmed.edge.clone();
                }
                if trimmed.trim2 {
                    e2 = trimmed.edge;
                }
                self.search.edge_edge(&e1, &e2, seq1, seq2)
            }
            ShapeEnum::Face => {
                let f2 = topods::face(s2)?;
                let f2 = trim::trim_infinite_face(&e1, &f2)?.unwrap_or(f2);
                self.search.edge_face(&e1, &f2, seq1, seq2)
            }
            _ => Ok(()),
        }
    }

    /// The branch of a first face: an infinite face is trimmed first against an edge.
    fn perform_face(&mut self, s1: &Shape, s2: &Shape) -> Result<(), Failure> {
        let f1 = topods::face(s1)?;
        let (seq1, seq2) = (&mut self.my_seq_sol_shape1, &mut self.my_seq_sol_shape2);
        match s2.shape_type()? {
            ShapeEnum::Vertex => self
                .search
                .vertex_face(&topods::vertex(s2)?, &f1, seq2, seq1),
            ShapeEnum::Edge => {
                let e2 = topods::edge(s2)?;
                let f1 = trim::trim_infinite_face(&e2, &f1)?.unwrap_or(f1);
                self.search.edge_face(&e2, &f1, seq2, seq1)
            }
            ShapeEnum::Face => self.search.face_face(&f1, &topods::face(s2)?, seq1, seq2),
            _ => Ok(()),
        }
    }
}
