// Rust port of DataExchange/TKXSBase/Transfer/Transfer_Binder (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Transfer_Binder`: the result of the transfer of one starting object, with its
//! statuses, its check and a chain of further results.
use super::{TransferStatusExec, TransferStatusResult, transfer_failure};
use crate::interface::InterfaceCheck;
use rustycad_tkernel::{
    impl_transient_interface,
    standard::{Failure, Handle, RuntimeType, SharedState, StandardTransient, Transient},
};

/// The private fields of `Transfer_Binder`.
struct BinderState {
    thestatus: TransferStatusResult,
    theexecst: TransferStatusExec,
    thecheck: Handle<InterfaceCheck>,
    thenextr: Handle<dyn TransferBinder>,
    theendr: Handle<dyn TransferBinder>,
}

/// The embedded base of a binder; `TransferBinderBase::default()` is the native protected
/// constructor (void, initial, an empty check).
pub struct TransferBinderBase {
    state: SharedState<BinderState>,
}

impl Default for TransferBinderBase {
    fn default() -> Self {
        Self {
            state: SharedState::new(BinderState {
                thestatus: TransferStatusResult::StatusVoid,
                theexecst: TransferStatusExec::StatusInitial,
                thecheck: Handle::new(InterfaceCheck::new()),
                thenextr: Handle::default(),
                theendr: Handle::default(),
            }),
        }
    }
}

impl Drop for TransferBinderBase {
    /// `~Transfer_Binder`: unlinks the chain of results iteratively while each next binder
    /// is held only by the chain, so a long chain does not recurse (as natively).
    fn drop(&mut self) {
        let mut current = {
            let mut state = self.state.write();
            state.theendr.nullify();
            std::mem::take(&mut state.thenextr)
        };
        while let Some(live) = current.get() {
            let next = live.binder_base().state.read().thenextr.clone();
            if next.is_null() || next.ref_count() > 2 {
                break;
            }
            live.binder_base().state.write().thenextr.nullify();
            current = next;
        }
    }
}

/// Whether two transient bases are those of one object.
fn same_object(first: &StandardTransient, second: &StandardTransient) -> bool {
    std::ptr::eq(first, second)
}

/// `Transfer_Binder`, abstract: `ResultType` and `ResultTypeName` are required,
/// `IsMultiple` is provided; the other members are non-virtual.
///
/// ```
/// use rustycad_tkxsbase::transfer::{TransferBinder, TransferVoidBinder};
/// let binder = TransferVoidBinder::new();
/// binder.add_warning("careful", "");
/// assert!(!binder.has_result() && binder.check().has_warnings());
/// ```
pub trait TransferBinder: Transient {
    /// The embedded private fields.
    fn binder_base(&self) -> &TransferBinderBase;
    /// `ResultType`.
    fn result_type(&self) -> &'static RuntimeType;
    /// `ResultTypeName`.
    fn result_type_name(&self) -> String;
    /// `IsMultiple`: several results in the chain.
    fn is_multiple(&self) -> bool {
        let next = self.next_result();
        let Some(first) = next.get() else {
            return false;
        };
        if !self.has_result() {
            return first.is_multiple();
        }
        let mut current = next.clone();
        while let Some(live) = current.get() {
            if live.has_result() {
                return true;
            }
            current = live.next_result();
        }
        false
    }
    /// `Merge(other)`: the higher execution status and the messages of `other`.
    fn merge(&self, other: &Handle<dyn TransferBinder>) {
        let Some(live) = other.get() else { return };
        let (exec, check) = (live.status_exec(), live.check());
        let mine = {
            let mut state = self.binder_base().state.write();
            if (state.theexecst as i32) < (exec as i32) {
                state.theexecst = exec;
            }
            state.thecheck.clone()
        };
        mine.get_messages(&check);
    }
    /// `AddResult(next)`: appends a binder to the chain of results (not itself, not null),
    /// after cutting it from the chain of `next`.
    fn add_result(&self, next: &Handle<dyn TransferBinder>) {
        let Some(added) = next.get() else { return };
        if same_object(added.transient_base(), self.transient_base()) {
            return;
        }
        added.cut_result_of(self.transient_base());
        let (first, end) = {
            let state = self.binder_base().state.read();
            (state.thenextr.clone(), state.theendr.clone())
        };
        if first.is_null() {
            self.binder_base().state.write().thenextr = next.clone();
            return;
        }
        let mut binder = if end.is_null() { first } else { end };
        while binder != *next {
            let following = binder.next_result();
            if following.is_null() {
                binder.add_result(next);
                self.binder_base().state.write().theendr = next.clone();
                return;
            }
            binder = following;
        }
    }
    /// `CutResult(next)` (private): removes `next` (given by its transient base, as the
    /// native caller passes `this`) from this chain.
    fn cut_result_of(&self, next: &StandardTransient) {
        let first = self.next_result();
        let Some(live) = first.get() else { return };
        if same_object(live.transient_base(), next) {
            let mut state = self.binder_base().state.write();
            state.thenextr.nullify();
            state.theendr.nullify();
            return;
        }
        let mut current = first.clone();
        loop {
            let following = current.next_result();
            let Some(candidate) = following.get() else {
                return;
            };
            if same_object(candidate.transient_base(), next) {
                current.cut_result_of(next);
                return;
            }
            current = following.clone();
        }
    }
    /// `NextResult`.
    fn next_result(&self) -> Handle<dyn TransferBinder> {
        self.binder_base().state.read().thenextr.clone()
    }
    /// `HasResult`: a result was set.
    fn has_result(&self) -> bool {
        self.binder_base().state.read().thestatus != TransferStatusResult::StatusVoid
    }
    /// `SetAlreadyUsed`: a set result becomes used.
    fn set_already_used(&self) {
        let mut state = self.binder_base().state.write();
        if state.thestatus != TransferStatusResult::StatusVoid {
            state.thestatus = TransferStatusResult::StatusUsed;
        }
    }
    /// `Status`.
    fn status(&self) -> TransferStatusResult {
        self.binder_base().state.read().thestatus
    }
    /// `StatusExec`.
    fn status_exec(&self) -> TransferStatusExec {
        self.binder_base().state.read().theexecst
    }
    /// `SetStatusExec(stat)`.
    fn set_status_exec(&self, stat: TransferStatusExec) {
        self.binder_base().state.write().theexecst = stat;
    }
    /// `AddFail(mess, orig = "")`: the execution status becomes "error".
    fn add_fail(&self, mess: &str, orig: &str) {
        let check = {
            let mut state = self.binder_base().state.write();
            state.theexecst = TransferStatusExec::StatusError;
            state.thecheck.clone()
        };
        check.add_fail_text(mess, orig);
    }
    /// `AddWarning(mess, orig = "")`.
    fn add_warning(&self, mess: &str, orig: &str) {
        self.check().add_warning_text(mess, orig);
    }
    /// `Check`.
    fn check(&self) -> Handle<InterfaceCheck> {
        self.binder_base().state.read().thecheck.clone()
    }
    /// `CCheck`: the check, modifiable (the same shared object).
    fn c_check(&self) -> Handle<InterfaceCheck> {
        self.check()
    }
    /// `SetResultPresent` (protected): "done" and "defined";
    /// `Transfer_TransferFailure` when the result was already used.
    fn set_result_present(&self) -> Result<(), Failure> {
        let mut state = self.binder_base().state.write();
        if state.thestatus == TransferStatusResult::StatusUsed {
            return Err(transfer_failure(
                "Binder : SetResult, Result is Already Set and Used",
            ));
        }
        state.theexecst = TransferStatusExec::StatusDone;
        state.thestatus = TransferStatusResult::StatusDefined;
        Ok(())
    }
}
impl_transient_interface!(dyn TransferBinder, "Transfer_Binder", StandardTransient);
