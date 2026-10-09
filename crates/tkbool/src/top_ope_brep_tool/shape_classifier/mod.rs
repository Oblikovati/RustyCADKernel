// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_ShapeClassifier.hxx/.cxx
// and TopOpeBRepTool_SC.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepTool_ShapeClassifier`: the state of a shape relative to a reference face
//! or solid, from a point of one of its edges (avoiding given shapes), and the
//! per-thread classifier of the `FSC_*` functions.
mod perform;

pub use perform::{fsc_get_psc, fsc_get_psc_with_reference, fsc_state_eonface, fsc_state_ponface};

use super::solid_classifier::TopOpeBRepToolSolidClassifier;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{IndexedMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkmath::gp::{Point2, Point3};

/// The state classifier of shapes (`TopOpeBRepTool_ShapeClassifier`); natively the
/// same domain flag and the reference map state of the default classifier are
/// uninitialized (0 and not done here);
/// `let mut sc = TopOpeBRepToolShapeClassifier::new(); sc.state_shape_shape(&e, &f, 0)?;`.
pub struct TopOpeBRepToolShapeClassifier {
    my_s: Shape,
    my_ref: Shape,
    my_av_s: Shape,
    /// `myPAvLS`: a copy of the list of shapes to avoid (a pointer natively).
    my_p_av_ls: Option<List<Shape>>,
    my_map_av_s: IndexedMap<Shape, ShapeMapHasher>,
    mymre: IndexedMap<Shape, ShapeMapHasher>,
    mymren: i32,
    mymredone: bool,
    my_state: State,
    my_edge: Edge,
    my_face: Face,
    my_p3d_def: bool,
    my_p3d: Point3,
    my_p2d_def: bool,
    my_p2d: Point2,
    my_solid_classifier: TopOpeBRepToolSolidClassifier,
    my_same_domain: i32,
}

impl Default for TopOpeBRepToolShapeClassifier {
    /// `TopOpeBRepTool_ShapeClassifier()`.
    fn default() -> Self {
        Self {
            my_s: Shape::default(),
            my_ref: Shape::default(),
            my_av_s: Shape::default(),
            my_p_av_ls: None,
            my_map_av_s: IndexedMap::default(),
            mymre: IndexedMap::default(),
            mymren: 0,
            mymredone: false,
            my_state: State::Unknown,
            my_edge: Edge::default(),
            my_face: Face::default(),
            my_p3d_def: false,
            my_p3d: Point3::default(),
            my_p2d_def: false,
            my_p2d: Point2::default(),
            my_solid_classifier: TopOpeBRepToolSolidClassifier::new(),
            my_same_domain: 0,
        }
    }
}

/// Standard_ProgramError of the shape classifier.
pub(crate) fn program_error(message: &str) -> Failure {
    Failure::with_type(
        "Standard_ProgramError",
        message,
        "TopOpeBRepTool_ShapeClassifier",
    )
}

impl TopOpeBRepToolShapeClassifier {
    /// `TopOpeBRepTool_ShapeClassifier()`; `TopOpeBRepToolShapeClassifier::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `TopOpeBRepTool_ShapeClassifier(SRef)`; `TopOpeBRepToolShapeClassifier::with_reference(&face)`.
    pub fn with_reference(s_ref: &Shape) -> Self {
        Self {
            my_ref: s_ref.clone(),
            ..Self::default()
        }
    }

    /// `ClearAll()`: the current data and the solid classifiers; `sc.clear_all()`.
    pub fn clear_all(&mut self) {
        self.clear_current();
        self.my_solid_classifier.clear();
    }

    /// `ClearCurrent()`: same domain -1, no shape, reference, avoided shape, edge or
    /// face, UNKNOWN; `sc.clear_current()`.
    pub fn clear_current(&mut self) {
        self.my_same_domain = -1;
        (self.my_s, self.my_ref, self.my_av_s) =
            (Shape::default(), Shape::default(), Shape::default());
        self.my_map_av_s.clear(false);
        self.mymre.clear(false);
        (self.mymren, self.mymredone, self.my_state) = (0, false, State::Unknown);
        (self.my_edge, self.my_face) = (Edge::default(), Face::default());
        (self.my_p3d_def, self.my_p2d_def) = (false, false);
    }

    /// `SameDomain()`; `sc.same_domain()`.
    pub fn same_domain(&self) -> i32 {
        self.my_same_domain
    }

    /// `SameDomain(samedomain)`; `sc.set_same_domain(1)`.
    pub fn set_same_domain(&mut self, sam: i32) {
        self.my_same_domain = sam;
    }

    /// `SetReference(SRef)`: the reference and its edge map; `sc.set_reference(&face)?`.
    pub fn set_reference(&mut self, s_ref: &Shape) -> Result<(), Failure> {
        self.my_ref = s_ref.clone();
        self.map_ref()
    }

    /// `MapRef()` (private): for a same domain face reference, its edges (0 when it has
    /// only a closed one).
    fn map_ref(&mut self) -> Result<(), Failure> {
        self.mymre.clear(false);
        self.mymren = 0;
        if self.my_ref.shape_type()? == ShapeEnum::Face && self.my_same_domain == 1 {
            topexp::map_shapes_of_type(&self.my_ref, ShapeEnum::Edge, &mut self.mymre)?;
            self.mymren = self.mymre.len() as i32;
            if self.mymren == 1 {
                let edges: Vec<Shape> =
                    Explorer::new(&self.my_ref, ShapeEnum::Edge, ShapeEnum::Shape)?.collect();
                let (v1, v2) = topexp::vertices(&topods::edge(&edges[0])?, false);
                if v1.as_ref().is_same(v2.as_ref()) {
                    self.mymren = 0;
                }
            }
        }
        self.mymredone = true;
        Ok(())
    }

    /// `StateShapeShape(S, SRef, samedomain)` (samedomain natively defaulted to 0): the
    /// state of `s` relative to `s_ref`; `sc.state_shape_shape(&e, &f, 0)?`.
    pub fn state_shape_shape(
        &mut self,
        s: &Shape,
        s_ref: &Shape,
        samedomain: i32,
    ) -> Result<State, Failure> {
        self.clear_current();
        self.my_same_domain = samedomain;
        (self.my_s, self.my_av_s, self.my_p_av_ls, self.my_ref) =
            (s.clone(), Shape::default(), None, s_ref.clone());
        self.perform()?;
        Ok(self.my_state)
    }

    /// `StateShapeShape(S, AvoidS, SRef)`: avoiding a shape; `sc.state_shape_shape_avoiding(&e, &avoid, &f)?`.
    pub fn state_shape_shape_avoiding(
        &mut self,
        s: &Shape,
        av_s: &Shape,
        s_ref: &Shape,
    ) -> Result<State, Failure> {
        self.clear_current();
        (self.my_s, self.my_av_s, self.my_p_av_ls, self.my_ref) =
            (s.clone(), av_s.clone(), None, s_ref.clone());
        self.perform()?;
        Ok(self.my_state)
    }

    /// `StateShapeShape(S, LAvoidS, SRef)`: avoiding a list of shapes;
    /// `sc.state_shape_shape_avoiding_list(&e, &avoid, &f)?`.
    pub fn state_shape_shape_avoiding_list(
        &mut self,
        s: &Shape,
        av_ls: &List<Shape>,
        s_ref: &Shape,
    ) -> Result<State, Failure> {
        self.clear_current();
        (self.my_s, self.my_av_s, self.my_p_av_ls, self.my_ref) = (
            s.clone(),
            Shape::default(),
            Some(av_ls.clone()),
            s_ref.clone(),
        );
        self.perform()?;
        Ok(self.my_state)
    }

    /// `StateShapeReference(S, AvoidS)`: relative to the current reference;
    /// `sc.state_shape_reference(&e, &avoid)?`.
    pub fn state_shape_reference(&mut self, s: &Shape, av_s: &Shape) -> Result<State, Failure> {
        (self.my_s, self.my_av_s, self.my_p_av_ls) = (s.clone(), av_s.clone(), None);
        self.perform()?;
        Ok(self.my_state)
    }

    /// `StateShapeReference(S, LAvoidS)`; `sc.state_shape_reference_avoiding_list(&e, &avoid)?`.
    pub fn state_shape_reference_avoiding_list(
        &mut self,
        s: &Shape,
        av_ls: &List<Shape>,
    ) -> Result<State, Failure> {
        (self.my_s, self.my_av_s, self.my_p_av_ls) =
            (s.clone(), Shape::default(), Some(av_ls.clone()));
        self.perform()?;
        Ok(self.my_state)
    }

    /// `ChangeSolidClassifier()`; `sc.change_solid_classifier()`.
    pub fn change_solid_classifier(&mut self) -> &mut TopOpeBRepToolSolidClassifier {
        &mut self.my_solid_classifier
    }

    /// `State()`; `sc.state()`.
    pub fn state(&self) -> State {
        self.my_state
    }

    /// `P3D()`: the classified 3d point; Standard_ProgramError when undefined; `sc.p3d()?`.
    pub fn p3d(&self) -> Result<Point3, Failure> {
        if self.my_p3d_def {
            return Ok(self.my_p3d);
        }
        Err(program_error("ShapeClassifier::P3D undefined"))
    }

    /// `P2D()`: the classified 2d point; Standard_ProgramError when undefined; `sc.p2d()?`.
    pub fn p2d(&self) -> Result<Point2, Failure> {
        if self.my_p2d_def {
            return Ok(self.my_p2d);
        }
        Err(program_error("ShapeClassifier::P2D undefined"))
    }

    /// `HasAvLS()` (private): a non-empty list of shapes to avoid.
    fn has_av_ls(&self) -> bool {
        self.my_p_av_ls.as_ref().is_some_and(|l| !l.is_empty())
    }
}
