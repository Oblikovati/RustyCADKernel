// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_WorkSession (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFSelect_WorkSession`: the work session of a norm: a model with its protocol, graph
//! and checks, the items (selections, dispatches, modifiers, parameters ...) defined
//! on it, by number and by name, the share-out of the model into files, and the
//! evaluations and dumps of all of them.
//!
//! The members are split by subject: `model` (model, protocol, reading), `entities`
//! (entities, graph, checks), `items`, `item_lists`, `params`, `share_out`, `files`,
//! `sending`, `selections`, `give_list`, `queries`, `statics`, `dumps`, `evaluations`.
use super::{IFSelectModelCopier, IFSelectShareOut, IFSelectWorkLibrary};
use crate::interface::{
    InterfaceCheckIterator, InterfaceGTool, InterfaceHGraph, InterfaceInterfaceModel,
    InterfaceProtocol,
};
use rustycad_tkernel::{
    impl_transient, impl_transient_interface,
    message::Message,
    ncollection::DataMap,
    standard::{Failure, Handle, SharedState, StandardTransient, Transient, WeakHandle},
    tcollection::AsciiString,
};
use std::cell::Cell;
use std::fmt::Write as _;
use std::sync::OnceLock;
use std::sync::atomic::Ordering;

#[cfg(test)]
mod dump_tests;
mod dumps;
mod entities;
mod evaluations;
mod files;
mod give_list;
mod item_lists;
mod items;
mod model;
mod params;
mod queries;
mod selections;
mod sending;
#[cfg(test)]
mod sending_tests;
#[cfg(test)]
mod session_tests;
mod share_out;
mod statics;

/// A list of transients: `None` for a null sequence.
pub type IFSelectTransientList = Option<Vec<Handle<StandardTransient>>>;

/// The flag of the incorrect entities in the bit map of the graph (natively a define).
pub(crate) const FLAG_INCORRECT: i32 = 2;

thread_local! {
    /// The flag behind [`ERRHAND`].
    static ERRHAND_FLAG: Cell<bool> = const { Cell::new(true) };
}

/// The native static `errhand`: true while the next guarded call catches the errors
/// ("only one at a time" natively). It is per thread here: the native sessions run on
/// one thread, and a process-wide flag would let a guarded call on one thread change
/// the error handling of the calls of the others.
pub(crate) struct ErrHand;

/// The `errhand` flag of the current thread.
pub(crate) static ERRHAND: ErrHand = ErrHand;

impl ErrHand {
    /// The flag.
    pub(crate) fn load(&self, _order: Ordering) -> bool {
        ERRHAND_FLAG.with(Cell::get)
    }
    /// Sets the flag.
    pub(crate) fn store(&self, value: bool, _order: Ordering) {
        ERRHAND_FLAG.with(|f| f.set(value));
    }
    /// Sets the flag, returning the previous value.
    pub(crate) fn swap(&self, value: bool, _order: Ordering) -> bool {
        ERRHAND_FLAG.with(|f| f.replace(value))
    }
}

/// The fields of `IFSelect_WorkSession` (the protected ones first).
pub(crate) struct WorkSessionState {
    pub(crate) thegraph: Handle<InterfaceHGraph>,
    pub(crate) thecheckrun: InterfaceCheckIterator,
    /// The items with their attributes (the item itself, its name, or null once removed),
    /// by ident (natively an indexed data map).
    pub(crate) theitems: Vec<(Handle<StandardTransient>, Handle<StandardTransient>)>,
    pub(crate) thenames: DataMap<AsciiString, Handle<StandardTransient>>,
    pub(crate) theerrhand: bool,
    pub(crate) theshareout: Handle<IFSelectShareOut>,
    pub(crate) thelibrary: Handle<dyn IFSelectWorkLibrary>,
    pub(crate) theprotocol: Handle<dyn InterfaceProtocol>,
    pub(crate) my_model: Handle<dyn InterfaceInterfaceModel>,
    pub(crate) theloaded: AsciiString,
    pub(crate) thegtool: Handle<InterfaceGTool>,
    pub(crate) thecheckdone: bool,
    pub(crate) thecheckana: Vec<u8>,
    pub(crate) thecopier: Handle<IFSelectModelCopier>,
    pub(crate) theoldel: Handle<dyn InterfaceInterfaceModel>,
    pub(crate) themodelstat: bool,
}

/// `IFSelect_WorkSession`. Its only virtual member, `ClearData`, is in
/// [`IFSelectWorkSessionOps`]; a derived session embeds this struct and binds its
/// handle with `bind_self`, so that the session calls its override.
///
/// ```
/// use rustycad_tkxsbase::if_select::IFSelectWorkSession;
/// let session = IFSelectWorkSession::new();
/// assert!(!session.has_model() && session.error_handle());
/// ```
pub struct IFSelectWorkSession {
    transient: StandardTransient,
    pub(crate) state: SharedState<WorkSessionState>,
    this: OnceLock<WeakHandle<dyn IFSelectWorkSessionOps>>,
}
impl_transient!(IFSelectWorkSession, "IFSelect_WorkSession", StandardTransient, transient, interfaces: [dyn IFSelectWorkSessionOps]);

/// The virtual members of `IFSelect_WorkSession`.
pub trait IFSelectWorkSessionOps: Transient {
    /// The embedded work session.
    fn work_session(&self) -> &IFSelectWorkSession;
    /// `ClearData(mode)`: 1 all the data, 2 the graph and the checks, 3 the checks, 4 the
    /// pointed selections, counters and edit forms (see `clear_data_base`).
    fn clear_data(&self, mode: i32) -> Result<(), Failure> {
        self.work_session().clear_data_base(mode)
    }
}
impl_transient_interface!(
    dyn IFSelectWorkSessionOps,
    "IFSelect_WorkSession",
    StandardTransient
);

impl IFSelectWorkSessionOps for IFSelectWorkSession {
    fn work_session(&self) -> &IFSelectWorkSession {
        self
    }
}

impl Default for IFSelectWorkSession {
    fn default() -> Self {
        Self::new()
    }
}

impl IFSelectWorkSession {
    /// `IFSelect_WorkSession()`: a share-out with its model copier, a general tool, the
    /// errors handled.
    pub fn new() -> Self {
        let shareout = Handle::new(IFSelectShareOut::new());
        let copier = Handle::new(IFSelectModelCopier::new());
        copier.set_share_out(&shareout);
        ERRHAND.store(true, Ordering::Relaxed);
        let state = WorkSessionState {
            thegraph: Handle::default(),
            thecheckrun: InterfaceCheckIterator::new(),
            theitems: Vec::new(),
            thenames: DataMap::default(),
            theerrhand: true,
            theshareout: shareout,
            thelibrary: Handle::default(),
            theprotocol: Handle::default(),
            my_model: Handle::default(),
            theloaded: AsciiString::default(),
            thegtool: Handle::new(InterfaceGTool::new()),
            thecheckdone: false,
            thecheckana: Vec::new(),
            thecopier: copier,
            theoldel: Handle::default(),
            themodelstat: false,
        };
        Self {
            transient: StandardTransient::default(),
            state: SharedState::new(state),
            this: OnceLock::new(),
        }
    }
    /// Records the handle of the (derived) session, through which `ClearData` is called.
    pub fn bind_self(&self, session: &Handle<dyn IFSelectWorkSessionOps>) {
        let _ = self.this.set(session.downgrade());
    }
    /// `ClearData(mode)` called virtually: on the bound session, else on this one.
    pub(crate) fn clear_data_virtual(&self, mode: i32) -> Result<(), Failure> {
        match self
            .this
            .get()
            .map(WeakHandle::upgrade)
            .filter(|h| h.is_some())
        {
            Some(this) => this.clear_data(mode),
            None => self.clear_data_base(mode),
        }
    }
    /// `SetErrorHandle(toHandle)`: the errors of the guarded calls are caught (and
    /// reported) or raised.
    pub fn set_error_handle(&self, to_handle: bool) {
        self.state.write().theerrhand = to_handle;
        ERRHAND.store(to_handle, Ordering::Relaxed);
    }
    /// `ErrorHandle`.
    pub fn error_handle(&self) -> bool {
        self.state.read().theerrhand
    }
    /// Runs `call` as a native guarded call (`try` around the call with `errhand`
    /// cleared): when the errors are handled, an error is reported under `header` and
    /// gives `None`; else it is raised.
    pub(crate) fn guarded<T>(
        &self,
        header: &str,
        call: impl FnOnce() -> Result<T, Failure>,
    ) -> Result<Option<T>, Failure> {
        if !ERRHAND.swap(false, Ordering::Relaxed) {
            return call().map(Some);
        }
        let result = call();
        ERRHAND.store(self.error_handle(), Ordering::Relaxed);
        match result {
            Ok(value) => Ok(Some(value)),
            Err(failure) => {
                report_interruption(header, &failure);
                Ok(None)
            }
        }
    }
}

impl IFSelectWorkSession {
    /// As `guarded`, with the native defect of the sending calls kept: after a success
    /// `errhand` stays cleared (the next guarded calls raise their errors) until an error
    /// is caught or `SetErrorHandle` is called.
    pub(crate) fn guarded_sticky<T>(
        &self,
        header: &str,
        call: impl FnOnce() -> Result<T, Failure>,
    ) -> Result<Option<T>, Failure> {
        if !ERRHAND.swap(false, Ordering::Relaxed) {
            return call().map(Some);
        }
        match call() {
            Ok(value) => Ok(Some(value)),
            Err(failure) => {
                report_interruption(header, &failure);
                ERRHAND.store(self.error_handle(), Ordering::Relaxed);
                Ok(None)
            }
        }
    }
}

/// The report of a guarded call interrupted by an error: the header line, the message,
/// then "Abandon".
pub(crate) fn report_interruption(header: &str, failure: &Failure) {
    let mut sout = Message::send_info();
    let _ = write!(sout, "{header}\n{}\n    Abandon\n", failure.message());
}

/// Writes `text` to a native output stream; an output error is a `Standard_Failure`.
pub(crate) fn write_out(s: &mut dyn std::io::Write, text: &[u8]) -> Result<(), Failure> {
    s.write_all(text)
        .map_err(|e| Failure::new(&format!("IFSelect_WorkSession: output failed: {e}")))
}

/// The header of most guarded calls.
pub(crate) fn interrupted(what: &str) -> String {
    format!("    ****    {what} Interrupted by Exception :   ****")
}
