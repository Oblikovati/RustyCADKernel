// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_SolidClassifier.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepTool_SolidClassifier`: the state of points in solids and shells, with a
//! classifier kept per solid or shell.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Shape, Shell, Solid};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::IndexedDataMap;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_class3d::BRepClass3dSolidClassifier;

/// The point classifier of solids and shells (`TopOpeBRepTool_SolidClassifier`);
/// `let mut sc = TopOpeBRepToolSolidClassifier::new(); sc.classify_solid(&solid, p, 1e-7)?;`.
pub struct TopOpeBRepToolSolidClassifier {
    /// The index of the current classifier in the map (`myPClassifier`).
    my_p_classifier: Option<usize>,
    my_shape_classifier_map: IndexedDataMap<Shape, BRepClass3dSolidClassifier, ShapeMapHasher>,
    my_state: State,
    my_solid: Solid,
    my_builder: BRepBuilder,
}

impl Default for TopOpeBRepToolSolidClassifier {
    /// `TopOpeBRepTool_SolidClassifier()`.
    fn default() -> Self {
        Self::new()
    }
}

impl TopOpeBRepToolSolidClassifier {
    /// `TopOpeBRepTool_SolidClassifier()`: `Clear()`; `TopOpeBRepToolSolidClassifier::new()`.
    pub fn new() -> Self {
        Self {
            my_p_classifier: None,
            my_shape_classifier_map: IndexedDataMap::default(),
            my_state: State::Unknown,
            my_solid: Solid::default(),
            my_builder: BRepBuilder::default(),
        }
    }

    /// `Clear()`: no classifier, UNKNOWN; `sc.clear()`.
    pub fn clear(&mut self) {
        self.my_p_classifier = None;
        self.my_shape_classifier_map.clear(false);
        self.my_state = State::Unknown;
        self.my_solid = Solid::default();
    }

    /// `LoadSolid(S)`: the classifier of the solid, made once; `sc.load_solid(&solid)?`.
    pub fn load_solid(&mut self, sol: &Solid) -> Result<(), Failure> {
        let found = self.my_shape_classifier_map.find_index(sol.as_ref());
        if found != 0 {
            self.my_p_classifier = Some(found);
            return Ok(());
        }
        let classifier = BRepClass3dSolidClassifier::with_shape(sol.as_ref())?;
        self.my_p_classifier = Some(
            self.my_shape_classifier_map
                .add(sol.as_ref().clone(), classifier),
        );
        Ok(())
    }

    /// `LoadShell(S)`: the classifier of a solid made of the shell, made once;
    /// `sc.load_shell(&shell)?`.
    pub fn load_shell(&mut self, she: &Shell) -> Result<(), Failure> {
        let found = self.my_shape_classifier_map.find_index(she.as_ref());
        if found != 0 {
            self.my_p_classifier = Some(found);
            return Ok(());
        }
        self.my_solid = Solid::default();
        self.my_builder.make_solid(&mut self.my_solid);
        let mut solid = self.my_solid.clone().into_shape();
        self.my_builder.add(&mut solid, she.as_ref())?;
        let classifier = BRepClass3dSolidClassifier::with_shape(&solid)?;
        self.my_p_classifier = Some(
            self.my_shape_classifier_map
                .add(she.as_ref().clone(), classifier),
        );
        Ok(())
    }

    /// The point classified by the current classifier; its state and face.
    fn perform(&mut self, p: Point3, tol: f64) -> Result<(State, Shape), Failure> {
        let index = self
            .my_p_classifier
            .ok_or_else(|| Failure::new("TopOpeBRepTool_SolidClassifier: no classifier"))?;
        let classifier = self.my_shape_classifier_map.change_from_index(index)?;
        classifier.perform(&p, tol)?;
        Ok((classifier.state(), classifier.face().into_shape()))
    }

    /// `Classify(S, P, Tol)` (solid): the state of the point, an IN/OUT/UNKNOWN on an
    /// EXTERNAL face read as OUT and on an INTERNAL face as IN;
    /// `sc.classify_solid(&solid, p, 1e-7)?`.
    pub fn classify_solid(&mut self, sol: &Solid, p: Point3, tol: f64) -> Result<State, Failure> {
        self.my_p_classifier = None;
        self.my_state = State::Unknown;
        self.load_solid(sol)?;
        let (state, fres) = self.perform(p, tol)?;
        let mut state = state;
        if !fres.is_null() {
            state = match (fres.orientation(), state) {
                (Orientation::External, State::On) | (Orientation::Internal, State::On) => {
                    State::On
                }
                (Orientation::External, _) => State::Out,
                (Orientation::Internal, _) => State::In,
                (_, s) => s,
            };
        }
        self.my_state = state;
        Ok(state)
    }

    /// `Classify(S, P, Tol)` (shell): the state of the point in the solid of the shell;
    /// `sc.classify_shell(&shell, p, 1e-7)?`.
    pub fn classify_shell(&mut self, she: &Shell, p: Point3, tol: f64) -> Result<State, Failure> {
        self.my_p_classifier = None;
        self.my_state = State::Unknown;
        self.load_shell(she)?;
        let (state, _) = self.perform(p, tol)?;
        self.my_state = state;
        Ok(state)
    }

    /// `State()`; `sc.state()`.
    pub fn state(&self) -> State {
        self.my_state
    }
}
