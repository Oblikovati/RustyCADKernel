// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntSurf/IntSurf_Transition.hxx/.lxx/.cxx,
// IntSurf_TypeTrans.hxx and IntSurf_Situation.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntSurf_Transition`: the transition of an intersection line at a point.
use rustycad_tkernel::standard::Failure;

/// The kind of a transition (`IntSurf_TypeTrans`); `IntSurfTypeTrans::Touch as i32 == 2`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum IntSurfTypeTrans {
    /// `IntSurf_In`: the line goes inside.
    In,
    /// `IntSurf_Out`: the line goes outside.
    Out,
    /// `IntSurf_Touch`: the line touches without crossing.
    Touch,
    /// `IntSurf_Undecided`: the transition cannot be computed.
    Undecided,
}

/// Where a touching line stays (`IntSurf_Situation`); `IntSurfSituation::Unknown as i32 == 2`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum IntSurfSituation {
    /// `IntSurf_Inside`.
    Inside,
    /// `IntSurf_Outside`.
    Outside,
    /// `IntSurf_Unknown`.
    Unknown,
}

/// The transition of an intersection line compared to an arc of restriction
/// (`IntSurf_Transition`): IN, OUT, TOUCH (with a situation) or UNDECIDED.
///
/// `IntSurfTransition::with_type(false, IntSurfTypeTrans::In).transition_type() == IntSurfTypeTrans::In`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IntSurfTransition {
    tangent: bool,
    typetra: IntSurfTypeTrans,
    situat: IntSurfSituation,
    oppos: bool,
}

impl Default for IntSurfTransition {
    fn default() -> Self {
        Self::new()
    }
}

/// The Standard_DomainError of the accessors that need a given transition type.
fn wrong_type(member: &str, expected: &str, found: IntSurfTypeTrans) -> Failure {
    Failure::with_message(
        "Standard_DomainError",
        &format!("IntSurf_Transition::{member}: transition type {found:?}, expected {expected}"),
    )
}

impl IntSurfTransition {
    /// An UNDECIDED transition (`IntSurf_Transition()`); `IntSurfTransition::new()`.
    pub const fn new() -> Self {
        Self {
            tangent: false,
            typetra: IntSurfTypeTrans::Undecided,
            situat: IntSurfSituation::Unknown,
            oppos: false,
        }
    }

    /// An IN or OUT transition (`IntSurf_Transition(Tangent, Type)`); the situation
    /// stays Unknown. `IntSurfTransition::with_type(true, IntSurfTypeTrans::Out)`.
    pub const fn with_type(tangent: bool, kind: IntSurfTypeTrans) -> Self {
        Self {
            tangent,
            typetra: kind,
            situat: IntSurfSituation::Unknown,
            oppos: false,
        }
    }

    /// A TOUCH transition (`IntSurf_Transition(Tangent, Situ, Oppos)`).
    /// `IntSurfTransition::touch(true, IntSurfSituation::Inside, false)`.
    pub const fn touch(tangent: bool, situation: IntSurfSituation, opposite: bool) -> Self {
        Self {
            tangent,
            typetra: IntSurfTypeTrans::Touch,
            situat: situation,
            oppos: opposite,
        }
    }

    /// Sets an IN or OUT transition (`SetValue(Tangent, Type)`); the situation and
    /// opposite flag keep their values, as natively. `t.set_type(false, IntSurfTypeTrans::In)`.
    pub fn set_type(&mut self, tangent: bool, kind: IntSurfTypeTrans) {
        self.tangent = tangent;
        self.typetra = kind;
    }

    /// Sets a TOUCH transition (`SetValue(Tangent, Situ, Oppos)`).
    /// `t.set_touch(true, IntSurfSituation::Outside, true)`.
    pub fn set_touch(&mut self, tangent: bool, situation: IntSurfSituation, opposite: bool) {
        self.tangent = tangent;
        self.typetra = IntSurfTypeTrans::Touch;
        self.situat = situation;
        self.oppos = opposite;
    }

    /// Makes the transition UNDECIDED (`SetValue()`); only the type changes.
    /// `t.set_undecided()`.
    pub fn set_undecided(&mut self) {
        self.typetra = IntSurfTypeTrans::Undecided;
    }

    /// The type of the transition (`TransitionType`); `t.transition_type()`.
    pub const fn transition_type(&self) -> IntSurfTypeTrans {
        self.typetra
    }

    /// Whether the line is tangent to the arc (`IsTangent`); Standard_DomainError
    /// for an UNDECIDED transition. `t.is_tangent()?`.
    pub fn is_tangent(&self) -> Result<bool, Failure> {
        if self.typetra == IntSurfTypeTrans::Undecided {
            return Err(wrong_type("IsTangent", "not Undecided", self.typetra));
        }
        Ok(self.tangent)
    }

    /// Where a touching line stays (`Situation`); Standard_DomainError unless the
    /// transition is TOUCH. `t.situation()?`.
    pub fn situation(&self) -> Result<IntSurfSituation, Failure> {
        if self.typetra != IntSurfTypeTrans::Touch {
            return Err(wrong_type("Situation", "Touch", self.typetra));
        }
        Ok(self.situat)
    }

    /// Whether the two curves bound different parts of space (`IsOpposite`);
    /// Standard_DomainError unless the transition is TOUCH. `t.is_opposite()?`.
    pub fn is_opposite(&self) -> Result<bool, Failure> {
        if self.typetra != IntSurfTypeTrans::Touch {
            return Err(wrong_type("IsOpposite", "Touch", self.typetra));
        }
        Ok(self.oppos)
    }
}
