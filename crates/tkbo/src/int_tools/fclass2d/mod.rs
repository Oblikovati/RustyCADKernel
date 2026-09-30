// IntTools_FClass2d.hxx/.cxx at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Wire polygon classification with the Boolean toolkit's distinct sampling and tolerances.
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_adaptor::{BRepAdaptorCurve, BRepAdaptorCurve2d, BRepAdaptorSurface},
    brep_tools::WireExplorer,
    topexp::{self, Explorer},
    topods::{Edge, Face, Wire},
};
use rustycad_tkernel::{ncollection::Array1, precision, standard::Failure};
use rustycad_tkg2d::{
    adaptor2d::Adaptor2dCurve2d,
    geom2d::native::{std_max, std_min},
};
use rustycad_tkg3d::{
    adaptor3d::{Adaptor3dCurve, Adaptor3dSurface},
    top_abs::{Orientation, ShapeEnum, State},
};
use rustycad_tkmath::{
    cslib::{Class2d, Class2dResult},
    gp::{Point2, Point3},
};
use rustycad_tktopalgo::brep_class::{BRepClassFaceClassifier, BRepClassFaceExplorer};
use std::sync::Mutex;
mod build;
mod classify;
mod edge;
mod polygon;
mod refine;
/// `IntTools_FClass2d`; `FClass2d::new(&face, 1e-7)?`.
#[derive(Default)]
pub struct FClass2d {
    classifiers: Vec<Class2d>,
    orientations: Vec<i32>,
    tolerance: f64,
    face: Face,
    bounds: [f64; 4],
    is_hole: bool,
    explorer: Mutex<Option<BRepClassFaceExplorer>>,
}
impl FClass2d {
    /// Samples the face's wires; `FClass2d::new(&face, 1e-7)?`.
    pub fn new(face: &Face, tolerance: f64) -> Result<Self, Failure> {
        let mut classifier = Self::default();
        classifier.init(face, tolerance)?;
        Ok(classifier)
    }
    /// Appends native wire classifiers on reinitialization; `classifier.init(&face, 1e-7)?`.
    pub fn init(&mut self, face: &Face, tolerance: f64) -> Result<(), Failure> {
        self.is_hole = true;
        self.tolerance = tolerance;
        self.face = Face::unchecked(face.oriented(Orientation::Forward));
        BRepAdaptorSurface::with_face(face, false)?;
        self.bounds = [f64::MAX, -f64::MAX, f64::MAX, -f64::MAX];
        // Pinned Init does not clear TabClass, TabOrien, or the cached exact explorer.
        self.sample_wires()
    }
    /// Reads the last nondegenerate wire's orientation; `classifier.is_hole()`.
    pub fn is_hole(&self) -> bool {
        self.is_hole
    }
    /// Classifies beyond the lower corner, or IN for an unbounded empty face; `classifier.perform_infinite_point()?`.
    pub fn perform_infinite_point(&self) -> Result<State, Failure> {
        let [u0, u1, v0, v1] = self.bounds;
        if u0 == f64::MAX || v0 == f64::MAX || u1 == -f64::MAX || v1 == -f64::MAX {
            return Ok(State::In);
        }
        self.perform(Point2::new(u0 - (u1 - u0), v0 - (v1 - v0)), false)
    }
    /// Classifies a UV point, optionally checking periodic copies; `classifier.perform(point, true)?`.
    pub fn perform(&self, point: Point2, recadre: bool) -> Result<State, Failure> {
        self.classify(point, recadre, None)
    }
    /// Tests tolerance offsets around the restriction; `classifier.test_on_restriction(point, 1e-7, true)?`.
    pub fn test_on_restriction(
        &self,
        point: Point2,
        tolerance: f64,
        recadre: bool,
    ) -> Result<State, Failure> {
        self.classify(point, recadre, Some(tolerance))
    }
}
fn boundary_edge(edge: &Edge) -> bool {
    matches!(
        edge.orientation(),
        Orientation::Forward | Orientation::Reversed
    )
}
