// Rust translation of ModelingAlgorithms/TKHLR/HLRAlgo/HLRAlgo_Coincidence.hxx, HLRAlgo_Intersection.hxx/.lxx/.cxx and HLRAlgo_Interference.hxx/HLRAlgo_Interference_0.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The records of an edge-face interference: the intersection point on the edge, its
//! coincidence with the face boundary, and the interference that joins them.
use rustycad_tkg3d::top_abs::{Orientation, State};

/// HLRAlgo_Coincidence: where an interference lies on the face boundary (the index of
/// the boundary edge and the parameter on it) and the 3D states of the edge before
/// and after it.
///
/// ```
/// use rustycad_tkg3d::top_abs::State;
/// use rustycad_tkhlr::hlr_algo::Coincidence;
/// let mut boundary = Coincidence::new();
/// boundary.set_2d(4, 0.5);
/// boundary.set_state_3d(State::Out, State::On);
/// assert_eq!(boundary.value_2d(), (4, 0.5));
/// assert_eq!(boundary.state_3d(), (State::Out, State::On));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coincidence {
    fe: i32,
    param: f64,
    st_bef: State,
    st_aft: State,
}

impl Default for Coincidence {
    fn default() -> Self {
        Self::new()
    }
}

impl Coincidence {
    /// HLRAlgo_Coincidence(): edge 0, parameter 0, both states IN; `Coincidence::new()`.
    pub fn new() -> Self {
        Self {
            fe: 0,
            param: 0.0,
            st_bef: State::In,
            st_aft: State::In,
        }
    }

    /// Set2D: the boundary edge index and the parameter on it; `c.set_2d(1, 0.5)`.
    pub fn set_2d(&mut self, fe: i32, param: f64) {
        self.fe = fe;
        self.param = param;
    }

    /// SetState3D: the states before and after; `c.set_state_3d(State::In, State::Out)`.
    pub fn set_state_3d(&mut self, st_bef: State, st_aft: State) {
        self.st_bef = st_bef;
        self.st_aft = st_aft;
    }

    /// Value2D: `(boundary edge index, parameter)`; `c.value_2d()`.
    pub fn value_2d(&self) -> (i32, f64) {
        (self.fe, self.param)
    }

    /// State3D: `(state before, state after)`; `c.state_3d()`.
    pub fn state_3d(&self) -> (State, State) {
        (self.st_bef, self.st_aft)
    }
}

/// HLRAlgo_Intersection: an intersection point on an edge: its orientation, level,
/// segment index, index, parameter, float tolerance and state.
///
/// The native default constructor leaves the orientation and the state uninitialized;
/// the Rust default is FORWARD and IN (docs/hlr-design.md section 5).
///
/// ```
/// use rustycad_tkg3d::top_abs::{Orientation, State};
/// use rustycad_tkhlr::hlr_algo::Intersection;
/// let mut point = Intersection::with_values(Orientation::Reversed, 1, 2, 3, 0.5, 1e-3, State::On);
/// point.set_level(4);
/// assert_eq!((point.level(), point.seg_index(), point.index()), (4, 2, 3));
/// assert_eq!(point.tolerance(), 1e-3);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Intersection {
    orien: Orientation,
    seg_index: i32,
    index: i32,
    level: i32,
    param: f64,
    toler: f32,
    state: State,
}

impl Default for Intersection {
    fn default() -> Self {
        Self::new()
    }
}

impl Intersection {
    /// HLRAlgo_Intersection(): every number 0 (see the type documentation for the
    /// orientation and the state); `Intersection::new()`.
    pub fn new() -> Self {
        Self::with_values(Orientation::Forward, 0, 0, 0, 0.0, 0.0, State::In)
    }

    /// HLRAlgo_Intersection(Ori, Lev, SegInd, Ind, P, Tol, S);
    /// `Intersection::with_values(o, 0, 1, 2, 0.5, 1e-3, s)`.
    pub fn with_values(
        orien: Orientation,
        level: i32,
        seg_index: i32,
        index: i32,
        param: f64,
        toler: f32,
        state: State,
    ) -> Self {
        Self {
            orien,
            seg_index,
            index,
            level,
            param,
            toler,
            state,
        }
    }

    /// Orientation(Ori); `point.set_orientation(Orientation::Forward)`.
    pub fn set_orientation(&mut self, orien: Orientation) {
        self.orien = orien;
    }

    /// Orientation(); `point.orientation()`.
    pub fn orientation(&self) -> Orientation {
        self.orien
    }

    /// Level(Lev); `point.set_level(1)`.
    pub fn set_level(&mut self, level: i32) {
        self.level = level;
    }

    /// Level(); `point.level()`.
    pub fn level(&self) -> i32 {
        self.level
    }

    /// SegIndex(SegInd); `point.set_seg_index(1)`.
    pub fn set_seg_index(&mut self, seg_index: i32) {
        self.seg_index = seg_index;
    }

    /// SegIndex(); `point.seg_index()`.
    pub fn seg_index(&self) -> i32 {
        self.seg_index
    }

    /// Index(Ind); `point.set_index(1)`.
    pub fn set_index(&mut self, index: i32) {
        self.index = index;
    }

    /// Index(); `point.index()`.
    pub fn index(&self) -> i32 {
        self.index
    }

    /// Parameter(P); `point.set_parameter(0.5)`.
    pub fn set_parameter(&mut self, param: f64) {
        self.param = param;
    }

    /// Parameter(); `point.parameter()`.
    pub fn parameter(&self) -> f64 {
        self.param
    }

    /// Tolerance(T); `point.set_tolerance(1e-3)`.
    pub fn set_tolerance(&mut self, toler: f32) {
        self.toler = toler;
    }

    /// Tolerance(); `point.tolerance()`.
    pub fn tolerance(&self) -> f32 {
        self.toler
    }

    /// State(S); `point.set_state(State::On)`.
    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }

    /// State(); `point.state()`.
    pub fn state(&self) -> State {
        self.state
    }
}

/// HLRAlgo_Interference: an interference of an edge with a face: its intersection
/// point, its coincidence with the face boundary, and three orientations (of the point
/// on the boundary, of the transition, of the boundary transition).
///
/// The native default constructor leaves the three orientations uninitialized; the
/// Rust default is FORWARD (docs/hlr-design.md section 5).
///
/// ```
/// use rustycad_tkg3d::top_abs::Orientation;
/// use rustycad_tkhlr::hlr_algo::{Coincidence, Interference, Intersection};
/// let mut interference = Interference::new();
/// interference.change_intersection().set_parameter(0.25);
/// interference.set_transition(Orientation::Reversed);
/// assert_eq!(interference.intersection().parameter(), 0.25);
/// assert_eq!(interference.transition(), Orientation::Reversed);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interference {
    intersection: Intersection,
    boundary: Coincidence,
    orientation: Orientation,
    transition: Orientation,
    b_transition: Orientation,
}

impl Default for Interference {
    fn default() -> Self {
        Self::new()
    }
}

impl Interference {
    /// HLRAlgo_Interference(): default records (see the type documentation);
    /// `Interference::new()`.
    pub fn new() -> Self {
        let forward = Orientation::Forward;
        Self::with_values(
            Intersection::new(),
            Coincidence::new(),
            forward,
            forward,
            forward,
        )
    }

    /// HLRAlgo_Interference(Inters, Bound, Orient, Trans, BTrans);
    /// `Interference::with_values(i, b, o, t, bt)`.
    pub fn with_values(
        intersection: Intersection,
        boundary: Coincidence,
        orientation: Orientation,
        transition: Orientation,
        b_transition: Orientation,
    ) -> Self {
        Self {
            intersection,
            boundary,
            orientation,
            transition,
            b_transition,
        }
    }

    /// Intersection(I); `interference.set_intersection(point)`.
    pub fn set_intersection(&mut self, intersection: Intersection) {
        self.intersection = intersection;
    }

    /// Boundary(B); `interference.set_boundary(coincidence)`.
    pub fn set_boundary(&mut self, boundary: Coincidence) {
        self.boundary = boundary;
    }

    /// Orientation(O); `interference.set_orientation(Orientation::Forward)`.
    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }

    /// Transition(Tr); `interference.set_transition(Orientation::Forward)`.
    pub fn set_transition(&mut self, transition: Orientation) {
        self.transition = transition;
    }

    /// BoundaryTransition(BTr); `interference.set_boundary_transition(Orientation::Forward)`.
    pub fn set_boundary_transition(&mut self, b_transition: Orientation) {
        self.b_transition = b_transition;
    }

    /// Intersection(); `interference.intersection()`.
    pub fn intersection(&self) -> &Intersection {
        &self.intersection
    }

    /// ChangeIntersection; `interference.change_intersection().set_level(1)`.
    pub fn change_intersection(&mut self) -> &mut Intersection {
        &mut self.intersection
    }

    /// Boundary(); `interference.boundary()`.
    pub fn boundary(&self) -> &Coincidence {
        &self.boundary
    }

    /// ChangeBoundary; `interference.change_boundary().set_2d(1, 0.5)`.
    pub fn change_boundary(&mut self) -> &mut Coincidence {
        &mut self.boundary
    }

    /// Orientation(); `interference.orientation()`.
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    /// Transition(); `interference.transition()`.
    pub fn transition(&self) -> Orientation {
        self.transition
    }

    /// BoundaryTransition(); `interference.boundary_transition()`.
    pub fn boundary_transition(&self) -> Orientation {
        self.b_transition
    }
}
