// IntTools_Root.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! A function root and the states and interval used to classify it.
use rustycad_tkg3d::top_abs::State;
/// `IntTools_Root`; `Root::new(0.5, 2).root() == 0.5`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Root {
    root: f64,
    root_type: i32,
    layer_height: f64,
    state_before: State,
    state_after: State,
    interval: (f64, f64, f64, f64),
}
impl Default for Root {
    fn default() -> Self {
        Self::new(0.0, -1)
    }
}
impl Root {
    /// Stores a root and kind, with unknown states; `Root::new(0.5, 2)`.
    pub fn new(root: f64, root_type: i32) -> Self {
        Self {
            root,
            root_type,
            layer_height: 0.0,
            state_before: State::Unknown,
            state_after: State::Unknown,
            interval: (0.0, 0.0, 0.0, 0.0),
        }
    }
    /// Replaces `root`; `root.set_root(0.25)`.
    pub fn set_root(&mut self, value: f64) {
        self.root = value;
    }
    /// Returns `root`; `root.root()`.
    pub fn root(&self) -> f64 {
        self.root
    }
    /// Replaces `root_type`; `root.set_type(2)`.
    pub fn set_type(&mut self, value: i32) {
        self.root_type = value;
    }
    /// Returns `root_type`; `root.root_type()`.
    pub fn root_type(&self) -> i32 {
        self.root_type
    }
    /// Replaces `layer_height`; `root.set_layer_height(0.1)`.
    pub fn set_layer_height(&mut self, value: f64) {
        self.layer_height = value;
    }
    /// Returns `layer_height`; `root.layer_height()`.
    pub fn layer_height(&self) -> f64 {
        self.layer_height
    }
    /// Replaces `state_before`; `root.set_state_before(State::In)`.
    pub fn set_state_before(&mut self, value: State) {
        self.state_before = value;
    }
    /// Returns `state_before`; `root.state_before()`.
    pub fn state_before(&self) -> State {
        self.state_before
    }
    /// Replaces `state_after`; `root.set_state_after(State::Out)`.
    pub fn set_state_after(&mut self, value: State) {
        self.state_after = value;
    }
    /// Returns `state_after`; `root.state_after()`.
    pub fn state_after(&self) -> State {
        self.state_after
    }
    /// Records the search interval and function values; `root.set_interval(0.0, 1.0, -1.0, 1.0)`.
    pub fn set_interval(&mut self, t1: f64, t2: f64, f1: f64, f2: f64) {
        self.interval = (t1, t2, f1, f2);
    }
    /// Returns `(t1, t2, f1, f2)`; `Root::default().interval() == (0.0,0.0,0.0,0.0)`.
    pub fn interval(&self) -> (f64, f64, f64, f64) {
        self.interval
    }
    /// Recognizes the four native crossing transitions; `Root::default().is_valid() == false`.
    pub fn is_valid(&self) -> bool {
        matches!(
            (self.state_before, self.state_after),
            (State::Out, State::In | State::On) | (State::In | State::On, State::Out)
        )
    }
}
