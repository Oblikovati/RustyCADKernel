// Rust translation of ChFiDS_Spine.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The guideline of a fillet or chamfer: a chain of edges with one curvilinear abscissa.
mod abscissa;
mod evaluation;
mod object;
mod topology;

pub use object::{SpineHandle, SpineObject};

use super::{ChamfMode, ErrorStatus, State, TypeOfConcavity};
use crate::chfi_ds::ElSpine;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Handle, StandardTransient};
use rustycad_tkmath::gp::{Point3, Vector3};

/// The guideline of a fillet (`ChFiDS_Spine`): the edges of the solid that are tangent
/// to each other, the curvilinear abscissa over them, the extremity states and extensions,
/// and the elementary spines (`ChFiDS_ElSpine`) cut from it for the computation.
///
/// Natively the guideline is not C2 although the computation path claims it; that note
/// of the upstream header stands. [`Spine::load`] must run before any evaluation: until
/// then the abscissa is null and the members that read it raise Standard_NullObject.
/// The native private `myOffsetCurve` and `offset_abscissa` are never read or set by
/// any member, so they have no Rust field.
/// The virtual members (`Reset`, `AppendElSpine`, `AppendOffsetElSpine`) are the
/// [`SpineObject`] trait, which `ChFiDS_FilSpine` and `ChFiDS_ChamfSpine` override.
///
/// ```
/// use rustycad_tkfillet::chfi_ds::{Spine, State};
/// let mut spine = Spine::with_tolerance(1e-4);
/// spine.set_first_status(State::Closed);
/// assert!(spine.is_periodic());
/// assert_eq!(spine.get_tolesp(), 1e-4);
/// ```
pub struct Spine {
    parent: StandardTransient,
    split_done: bool,
    el_spines: Vec<Handle<ElSpine>>,
    offset_el_spines: Vec<Handle<ElSpine>>,
    mode: ChamfMode,
    curve: BRepAdaptorCurve,
    index_of_curve: i32,
    type_of_concavity: TypeOfConcavity,
    first_state: State,
    last_state: State,
    edges: Vec<Edge>,
    offset_edges: Vec<Edge>,
    abscissa: Option<Vec<f64>>,
    tolesp: f64,
    ends: SpineEnds,
    error_state: ErrorStatus,
}
impl_transient!(Spine, "ChFiDS_Spine", StandardTransient, parent);

impl Default for Spine {
    fn default() -> Self {
        Self::with_tolerance(precision::confusion())
    }
}

impl Spine {
    /// `ChFiDS_Spine()`: an empty guideline with the tolerance Precision::Confusion();
    /// `Spine::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `ChFiDS_Spine(Tol)`: an empty guideline with the tolerance `tolerance`;
    /// `Spine::with_tolerance(1e-4)`.
    pub fn with_tolerance(tolerance: f64) -> Self {
        Self {
            parent: StandardTransient::default(),
            split_done: false,
            el_spines: Vec::new(),
            offset_el_spines: Vec::new(),
            mode: ChamfMode::ClassicChamfer,
            curve: BRepAdaptorCurve::default(),
            index_of_curve: 0,
            type_of_concavity: TypeOfConcavity::Other,
            first_state: State::OnSame,
            last_state: State::OnSame,
            edges: Vec::new(),
            offset_edges: Vec::new(),
            abscissa: None,
            tolesp: tolerance,
            ends: SpineEnds::default(),
            error_state: ErrorStatus::Ok,
        }
    }

    /// `SetTypeOfConcavity(theType)`; `spine.set_type_of_concavity(TypeOfConcavity::Convex)`.
    pub fn set_type_of_concavity(&mut self, kind: TypeOfConcavity) {
        self.type_of_concavity = kind;
    }

    /// `GetTypeOfConcavity()`; `spine.get_type_of_concavity()`.
    pub fn get_type_of_concavity(&self) -> TypeOfConcavity {
        self.type_of_concavity
    }

    /// `SetFirstStatus(S)`; `spine.set_first_status(State::FreeBoundary)`.
    pub fn set_first_status(&mut self, state: State) {
        self.first_state = state;
    }

    /// `SetLastStatus(S)`; `spine.set_last_status(State::FreeBoundary)`.
    pub fn set_last_status(&mut self, state: State) {
        self.last_state = state;
    }

    /// `FirstStatus()`; `spine.first_status()`.
    pub fn first_status(&self) -> State {
        self.first_state
    }

    /// `LastStatus()`; `spine.last_status()`.
    pub fn last_status(&self) -> State {
        self.last_state
    }

    /// `SetStatus(S, IsFirst)`; `spine.set_status(State::Tangent, true)`.
    pub fn set_status(&mut self, state: State, is_first: bool) {
        if is_first {
            self.first_state = state;
        } else {
            self.last_state = state;
        }
    }

    /// `Status(IsFirst)`; `spine.status(false)`.
    pub fn status(&self, is_first: bool) -> State {
        if is_first {
            self.first_state
        } else {
            self.last_state
        }
    }

    /// `SetTangencyExtremity(IsTangency, IsFirst)`; `spine.set_tangency_extremity(true, true)`.
    pub fn set_tangency_extremity(&mut self, is_tangency: bool, is_first: bool) {
        if is_first {
            self.ends.first_is_tgt = is_tangency;
        } else {
            self.ends.last_is_tgt = is_tangency;
        }
    }

    /// `IsTangencyExtremity(IsFirst)`; `spine.is_tangency_extremity(true)`.
    pub fn is_tangency_extremity(&self, is_first: bool) -> bool {
        if is_first {
            self.ends.first_is_tgt
        } else {
            self.ends.last_is_tgt
        }
    }

    /// `SplitDone()`; `spine.split_done()`.
    pub fn split_done(&self) -> bool {
        self.split_done
    }

    /// `SplitDone(B)`; `spine.set_split_done(true)`.
    pub fn set_split_done(&mut self, done: bool) {
        self.split_done = done;
    }

    /// `ChangeElSpines()`: the elementary spines, editable; `spine.change_el_spines().clear()`.
    pub fn change_el_spines(&mut self) -> &mut Vec<Handle<ElSpine>> {
        &mut self.el_spines
    }

    /// `ChangeOffsetElSpines()`; `spine.change_offset_el_spines().clear()`.
    pub fn change_offset_el_spines(&mut self) -> &mut Vec<Handle<ElSpine>> {
        &mut self.offset_el_spines
    }

    /// The elementary spines in order (the native protected list, read only);
    /// `spine.el_spines().len()`.
    pub fn el_spines(&self) -> &[Handle<ElSpine>] {
        &self.el_spines
    }

    /// `SetErrorStatus(state)`; `spine.set_error_status(ErrorStatus::WalkingFailure)`.
    pub fn set_error_status(&mut self, state: ErrorStatus) {
        self.error_state = state;
    }

    /// `ErrorStatus()`; `spine.error_status()`.
    pub fn error_status(&self) -> ErrorStatus {
        self.error_state
    }

    /// `Mode()`: the chamfer mode; `spine.mode()`.
    pub fn mode(&self) -> ChamfMode {
        self.mode
    }

    /// Sets the protected `myMode`, as `ChFiDS_ChamfSpine::SetMode` does.
    pub(crate) fn set_mode(&mut self, mode: ChamfMode) {
        self.mode = mode;
    }

    /// `GetTolesp()`: the tolerance of the guideline; `spine.get_tolesp()`.
    pub fn get_tolesp(&self) -> f64 {
        self.tolesp
    }

    /// `CurrentIndexOfElementarySpine()`: the index of the current edge;
    /// `spine.current_index_of_elementary_spine()`.
    pub fn current_index_of_elementary_spine(&self) -> i32 {
        self.index_of_curve
    }

    /// `HasFirstTgt()`; `spine.has_first_tgt()`.
    pub fn has_first_tgt(&self) -> bool {
        self.ends.has_first_tgt
    }

    /// `HasLastTgt()`; `spine.has_last_tgt()`.
    pub fn has_last_tgt(&self) -> bool {
        self.ends.has_last_tgt
    }

    /// `UnsetReference()`; `spine.unset_reference()`.
    pub fn unset_reference(&mut self) {
        self.ends.has_ref = false;
    }

    /// `Resolution(R3d)`: the abscissa is a length, so the resolution is `r3d` itself;
    /// `spine.resolution(1e-7)`.
    pub fn resolution(&self, r3d: f64) -> f64 {
        r3d
    }
}

/// The extensions, end tangents and approximation reference of a guideline; both native
/// constructors set them to zero or false.
#[derive(Clone, Copy, Default)]
struct SpineEnds {
    first_param: f64,
    last_param: f64,
    first_prolon: bool,
    last_prolon: bool,
    first_is_tgt: bool,
    last_is_tgt: bool,
    first_tgt_par: f64,
    last_tgt_par: f64,
    has_first_tgt: bool,
    has_last_tgt: bool,
    first_ori: Point3,
    last_ori: Point3,
    first_tgt: Vector3,
    last_tgt: Vector3,
    val_ref: f64,
    has_ref: bool,
}

#[cfg(test)]
#[path = "spine_tests.rs"]
mod tests;
