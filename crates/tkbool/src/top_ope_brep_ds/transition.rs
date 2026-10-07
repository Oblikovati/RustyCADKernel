// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_Transition.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepDS_Transition`: the states before and after an interference along the
//! interfered geometry, with the shape types and indices of the shapes they are on.
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};

/// A transition (`TopOpeBRepDS_Transition`): UNKNOWN to UNKNOWN on faces of index 0 by
/// default; `let t = TopOpeBRepDSTransition::from_orientation(Orientation::Forward);`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TopOpeBRepDSTransition {
    my_state_before: State,
    my_state_after: State,
    my_shape_before: ShapeEnum,
    my_shape_after: ShapeEnum,
    my_index_before: i32,
    my_index_after: i32,
}

impl Default for TopOpeBRepDSTransition {
    /// `TopOpeBRepDS_Transition()`.
    fn default() -> Self {
        Self::new(
            State::Unknown,
            State::Unknown,
            ShapeEnum::Face,
            ShapeEnum::Face,
        )
    }
}

impl TopOpeBRepDSTransition {
    /// `TopOpeBRepDS_Transition(StateBefore, StateAfter, ShapeBefore, ShapeAfter)`
    /// (the shapes natively defaulted to FACE), indices 0;
    /// `TopOpeBRepDSTransition::new(State::Out, State::In, ShapeEnum::Face, ShapeEnum::Face)`.
    pub fn new(sb: State, sa: State, onb: ShapeEnum, ona: ShapeEnum) -> Self {
        Self {
            my_state_before: sb,
            my_state_after: sa,
            my_shape_before: onb,
            my_shape_after: ona,
            my_index_before: 0,
            my_index_after: 0,
        }
    }

    /// `TopOpeBRepDS_Transition(O)`: the states of the orientation on faces;
    /// `TopOpeBRepDSTransition::from_orientation(Orientation::Reversed)`.
    pub fn from_orientation(o: Orientation) -> Self {
        let mut t = Self::default();
        t.set_orientation(o);
        t
    }

    /// `Set(StateBefore, StateAfter, ShapeBefore, ShapeAfter)` (indices kept);
    /// `t.set(State::In, State::Out, ShapeEnum::Face, ShapeEnum::Face)`.
    pub fn set(&mut self, sb: State, sa: State, onb: ShapeEnum, ona: ShapeEnum) {
        (
            self.my_state_before,
            self.my_state_after,
            self.my_shape_before,
            self.my_shape_after,
        ) = (sb, sa, onb, ona);
    }

    /// `StateBefore(S)`; `t.set_state_before(State::In)`.
    pub fn set_state_before(&mut self, s: State) {
        self.my_state_before = s;
    }

    /// `StateAfter(S)`; `t.set_state_after(State::Out)`.
    pub fn set_state_after(&mut self, s: State) {
        self.my_state_after = s;
    }

    /// `ShapeBefore(SE)`; `t.set_shape_before(ShapeEnum::Edge)`.
    pub fn set_shape_before(&mut self, se: ShapeEnum) {
        self.my_shape_before = se;
    }

    /// `ShapeAfter(SE)`; `t.set_shape_after(ShapeEnum::Edge)`.
    pub fn set_shape_after(&mut self, se: ShapeEnum) {
        self.my_shape_after = se;
    }

    /// `Before(S, ShapeBefore)` (natively defaulted to FACE); `t.set_before(State::On, ShapeEnum::Face)`.
    pub fn set_before(&mut self, s: State, onb: ShapeEnum) {
        (self.my_state_before, self.my_shape_before) = (s, onb);
    }

    /// `After(S, ShapeAfter)` (natively defaulted to FACE); `t.set_after(State::In, ShapeEnum::Face)`.
    pub fn set_after(&mut self, s: State, ona: ShapeEnum) {
        (self.my_state_after, self.my_shape_after) = (s, ona);
    }

    /// `Index(I)`: both indices; `t.set_index(3)`.
    pub fn set_index(&mut self, i: i32) {
        (self.my_index_before, self.my_index_after) = (i, i);
    }

    /// `IndexBefore(I)`; `t.set_index_before(3)`.
    pub fn set_index_before(&mut self, i: i32) {
        self.my_index_before = i;
    }

    /// `IndexAfter(I)`; `t.set_index_after(4)`.
    pub fn set_index_after(&mut self, i: i32) {
        self.my_index_after = i;
    }

    /// `Before()`; `t.before()`.
    pub fn before(&self) -> State {
        self.my_state_before
    }

    /// `ONBefore()`; `t.on_before()`.
    pub fn on_before(&self) -> ShapeEnum {
        self.my_shape_before
    }

    /// `After()`; `t.after()`.
    pub fn after(&self) -> State {
        self.my_state_after
    }

    /// `ONAfter()`; `t.on_after()`.
    pub fn on_after(&self) -> ShapeEnum {
        self.my_shape_after
    }

    /// `ShapeBefore()`; `t.shape_before()`.
    pub fn shape_before(&self) -> ShapeEnum {
        self.my_shape_before
    }

    /// `ShapeAfter()`; `t.shape_after()`.
    pub fn shape_after(&self) -> ShapeEnum {
        self.my_shape_after
    }

    /// `Index()`: the common index; Standard_Failure when the indices differ;
    /// `t.index()?`.
    pub fn index(&self) -> Result<i32, Failure> {
        if self.my_index_after != self.my_index_before {
            let message = format!(
                "Transition::Index() on different shapes: before {} after {}; expected equal indices",
                self.my_index_before, self.my_index_after
            );
            return Err(Failure::new(&message));
        }
        Ok(self.my_index_before)
    }

    /// `IndexBefore()`; `t.index_before()`.
    pub fn index_before(&self) -> i32 {
        self.my_index_before
    }

    /// `IndexAfter()`; `t.index_after()`.
    pub fn index_after(&self) -> i32 {
        self.my_index_after
    }

    /// `Set(O)`: OUT/IN for FORWARD, IN/OUT for REVERSED, IN/IN for INTERNAL and
    /// OUT/OUT for EXTERNAL (shapes and indices kept); `t.set_orientation(Orientation::Internal)`.
    pub fn set_orientation(&mut self, o: Orientation) {
        (self.my_state_before, self.my_state_after) = match o {
            Orientation::Forward => (State::Out, State::In),
            Orientation::Reversed => (State::In, State::Out),
            Orientation::Internal => (State::In, State::In),
            Orientation::External => (State::Out, State::Out),
        };
    }

    /// `Orientation(S, T)` (T natively defaulted to FACE and unused): the orientation of
    /// the transition seen from the state `s`; `t.orientation(State::In, ShapeEnum::Face)`.
    pub fn orientation(&self, s: State, t: ShapeEnum) -> Orientation {
        if self.my_state_before == State::On || self.my_state_after == State::On {
            return self.orientation_on(s, t);
        }
        match (self.my_state_before == s, self.my_state_after == s) {
            (true, true) => Orientation::Internal,
            (true, false) => Orientation::Reversed,
            (false, true) => Orientation::Forward,
            (false, false) => Orientation::External,
        }
    }

    /// `OrientationON(S, T)` (private): the orientation when a state is ON.
    fn orientation_on(&self, s: State, _t: ShapeEnum) -> Orientation {
        if self.my_state_before == State::On && self.my_state_after == State::On {
            return match s {
                State::Out => Orientation::External,
                State::In | State::On => Orientation::Internal,
                State::Unknown => Orientation::Forward,
            };
        }
        if self.my_state_before == State::On {
            return if self.my_state_after == s {
                Orientation::Forward
            } else {
                Orientation::Reversed
            };
        }
        if self.my_state_before == s {
            Orientation::Reversed
        } else {
            Orientation::Forward
        }
    }

    /// `Complement()`: the states swapped (with the shapes) when unknown or ON, else the
    /// opposite states of the orientation seen from IN (shapes kept); the indices kept;
    /// `t.complement()`.
    pub fn complement(&self) -> Self {
        let mut t = Self {
            my_index_before: self.my_index_before,
            my_index_after: self.my_index_after,
            ..Self::default()
        };
        let unknown =
            self.my_state_before == State::Unknown && self.my_state_after == State::Unknown;
        if unknown || self.my_state_before == State::On || self.my_state_after == State::On {
            t.set(
                self.my_state_after,
                self.my_state_before,
                self.my_shape_after,
                self.my_shape_before,
            );
            return t;
        }
        let (sb, sa) = match self.orientation(State::In, ShapeEnum::Face) {
            Orientation::Forward => (State::In, State::Out),
            Orientation::Reversed => (State::Out, State::In),
            Orientation::External => (State::In, State::In),
            Orientation::Internal => (State::Out, State::Out),
        };
        t.set(sb, sa, self.my_shape_before, self.my_shape_after);
        t
    }

    /// `IsUnknown()`: both states UNKNOWN; `t.is_unknown()`.
    pub fn is_unknown(&self) -> bool {
        self.my_state_before == State::Unknown && self.my_state_after == State::Unknown
    }
}
