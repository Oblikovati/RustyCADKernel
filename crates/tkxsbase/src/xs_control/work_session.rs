// Rust port of DataExchange/TKXSBase/XSControl/XSControl_WorkSession (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XSControl_WorkSession`: the work session of a norm: an IFSelect work session with
//! the controller of the norm, the transfer reader and writer, a context and variables.
//! The transfers are in `work_session_transfer`.
use super::{
    XSControlContext, XSControlController, XSControlControllerBase, XSControlTransferReader,
    XSControlTransferWriter, XSControlVars, XSControlVarsOps,
};
use crate::if_select::{IFSelectWorkSession, IFSelectWorkSessionOps};
use crate::interface::InterfaceModelEntities;
use crate::transfer::{TransferFinderProcess, TransferTransientProcess};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, SharedState, WeakHandle, exceptions},
    tcollection::AsciiString,
};
use std::ops::Deref;
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};

/// The native global mutex of the norm selections, the new models and the shape writings.
pub(super) fn global_lock() -> MutexGuard<'static, ()> {
    static WS_GLOBAL_MUTEX: Mutex<()> = Mutex::new(());
    WS_GLOBAL_MUTEX
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

/// The fields of `XSControl_WorkSession`.
pub(super) struct XsState {
    pub(super) controller: Handle<dyn XSControlController>,
    pub(super) transfer_reader: Handle<XSControlTransferReader>,
    pub(super) transfer_writer: Handle<XSControlTransferWriter>,
    pub(super) context: XSControlContext,
    pub(super) vars: Handle<dyn XSControlVarsOps>,
}

/// `XSControl_WorkSession`: embeds its IFSelect work session (dereferenced to it), whose
/// `ClearData` it overrides.
///
/// ```
/// use rustycad_tkxsbase::xs_control::XSControlWorkSession;
/// let ws = XSControlWorkSession::new();
/// assert!(ws.transfer_reader().is_some() && ws.selected_norm(false).is_empty());
/// ```
pub struct XSControlWorkSession {
    base: IFSelectWorkSession,
    pub(super) state: SharedState<XsState>,
    this: OnceLock<WeakHandle<XSControlWorkSession>>,
}
impl_transient!(XSControlWorkSession, "XSControl_WorkSession", IFSelectWorkSession, base, interfaces: [dyn IFSelectWorkSessionOps]);

impl Deref for XSControlWorkSession {
    type Target = IFSelectWorkSession;
    fn deref(&self) -> &IFSelectWorkSession {
        &self.base
    }
}

impl IFSelectWorkSessionOps for XSControlWorkSession {
    fn work_session(&self) -> &IFSelectWorkSession {
        &self.base
    }
    /// `ClearData(theMode)`: the modes 1 to 4 of the IFSelect session, 5 and 7 the
    /// transfer reader and writer, 6 the results of the reader; then the reader takes
    /// the graph (computed).
    fn clear_data(&self, mode: i32) -> Result<(), Failure> {
        if (1..=4).contains(&mode) {
            self.base.clear_data_base(mode)?;
        }
        let (reader, writer) = (self.transfer_reader(), self.transfer_writer());
        if mode == 5 || mode == 7 {
            live_reader(&reader)?.clear(-1);
            live_writer(&writer)?.clear(-1);
        }
        if mode == 6
            && let Some(r) = reader.get()
        {
            r.clear(1);
        }
        live_reader(&reader)?.set_graph(&self.h_graph()?);
        Ok(())
    }
}

/// The reader, `Standard_NullObject` when null (a native null dereference).
pub(super) fn live_reader(
    reader: &Handle<XSControlTransferReader>,
) -> Result<&XSControlTransferReader, Failure> {
    reader
        .get()
        .ok_or_else(|| exceptions::null_object("XSControl_WorkSession: no transfer reader"))
}

/// The writer, `Standard_NullObject` when null (a native null dereference).
pub(super) fn live_writer(
    writer: &Handle<XSControlTransferWriter>,
) -> Result<&XSControlTransferWriter, Failure> {
    writer
        .get()
        .ok_or_else(|| exceptions::null_object("XSControl_WorkSession: no transfer writer"))
}

impl XSControlWorkSession {
    /// `XSControl_WorkSession()`: a transfer reader, a transfer writer and variables;
    /// built as its handle, bound for the overridden `ClearData`.
    pub fn new() -> Handle<Self> {
        let state = XsState {
            controller: Handle::default(),
            transfer_reader: Handle::new(XSControlTransferReader::new()),
            transfer_writer: Handle::new(XSControlTransferWriter::new()),
            context: XSControlContext::default(),
            vars: Handle::new(XSControlVars::new()).cast(),
        };
        let ws = Handle::new(Self {
            base: IFSelectWorkSession::new(),
            state: SharedState::new(state),
            this: OnceLock::new(),
        });
        ws.base.bind_self(&ws.clone().cast());
        let _ = ws.this.set(ws.downgrade());
        ws
    }
    /// The handle of this session (natively `this`).
    pub(super) fn this(&self) -> Handle<Self> {
        self.this.get().map(WeakHandle::upgrade).unwrap_or_default()
    }
    /// `SelectNorm(theNormName)`: the controller recorded under the name becomes the one
    /// of the session (the reader cleared); false when unknown.
    pub fn select_norm(&self, normname: &str) -> Result<bool, Failure> {
        let _lock = global_lock();
        live_reader(&self.transfer_reader())?.clear(-1);
        let newadapt = XSControlControllerBase::recorded(normname);
        if newadapt.is_null() {
            return Ok(false);
        }
        if newadapt == self.norm_adaptor() {
            return Ok(true);
        }
        self.set_controller(&newadapt)?;
        Ok(true)
    }
    /// `SetController(theCtl)`: the library, protocol and items of the norm (the session
    /// cleared then customised), for the reader and writer too; `Standard_NullObject`
    /// for a null controller.
    pub fn set_controller(&self, ctl: &Handle<dyn XSControlController>) -> Result<(), Failure> {
        self.state.write().controller = ctl.clone();
        let live = ctl.get().ok_or_else(|| {
            exceptions::null_object("XSControl_WorkSession::SetController: null controller")
        })?;
        self.set_library(&live.work_library());
        self.set_protocol(&live.protocol());
        self.clear_items();
        self.clear_final_modifiers()?;
        self.clear_share_out(false)?;
        self.clear_file()?;
        live.customise(&self.this())?;
        live_reader(&self.transfer_reader())?.set_controller(ctl);
        live_writer(&self.transfer_writer())?.set_controller(ctl);
        Ok(())
    }
    /// `SelectedNorm(theRsc = false)`: the short (`rsc`) or long name of the controller
    /// (empty without one).
    pub fn selected_norm(&self, rsc: bool) -> AsciiString {
        self.norm_adaptor()
            .get()
            .map(|c| c.name(rsc))
            .unwrap_or_default()
    }
    /// `NormAdaptor`: the controller.
    pub fn norm_adaptor(&self) -> Handle<dyn XSControlController> {
        self.state.read().controller.clone()
    }
    /// `Context`: the context of the session (a copy).
    pub fn context(&self) -> XSControlContext {
        self.state.read().context.clone()
    }
    /// `SetAllContext(theContext)`: the context of the session and of the reader.
    pub fn set_all_context(&self, context: &XSControlContext) -> Result<(), Failure> {
        self.state.write().context = context.clone();
        *live_reader(&self.transfer_reader())?.context() = context.clone();
        Ok(())
    }
    /// `ClearContext`: the context of the session and of the reader cleared.
    pub fn clear_context(&self) -> Result<(), Failure> {
        self.state.write().context.clear(false);
        live_reader(&self.transfer_reader())?.context().clear(false);
        Ok(())
    }
    /// `TransferReader`.
    pub fn transfer_reader(&self) -> Handle<XSControlTransferReader> {
        self.state.read().transfer_reader.clone()
    }
    /// `TransferWriter`.
    pub fn transfer_writer(&self) -> Handle<XSControlTransferWriter> {
        self.state.read().transfer_writer.clone()
    }
    /// `MapReader`: the transient process of the reader.
    pub fn map_reader(&self) -> Result<Handle<TransferTransientProcess>, Failure> {
        Ok(live_reader(&self.transfer_reader())?.transient_process())
    }
    /// `SetMapWriter(theFP)`: the finder process of the writer (false for null).
    pub fn set_map_writer(&self, fp: &Handle<TransferFinderProcess>) -> Result<bool, Failure> {
        if fp.is_null() {
            return Ok(false);
        }
        live_writer(&self.transfer_writer())?.set_finder_process(fp);
        Ok(true)
    }
    /// `Vars`.
    pub fn vars(&self) -> Handle<dyn XSControlVarsOps> {
        self.state.read().vars.clone()
    }
    /// `SetVars(theVars)`.
    pub fn set_vars(&self, vars: &Handle<dyn XSControlVarsOps>) {
        self.state.write().vars = vars.clone();
    }
    /// `ClearBinders` (private, from the destructor): the finder process cleared, then
    /// the data (1) and the transfers (5).
    fn clear_binders(&self) -> Result<(), Failure> {
        let writer = self.transfer_writer();
        if let Some(fp) = writer.get().map(XSControlTransferWriter::finder_process)
            && let Some(process) = fp.get()
        {
            process.clear();
        }
        self.clear_data(1)?;
        self.clear_data(5)
    }
    /// The count of entities of the model plus 100 (100 without model).
    pub(super) fn process_size(&self) -> i32 {
        self.model().get().map_or(100, |m| m.nb_entities() + 100)
    }
}

impl Drop for XSControlWorkSession {
    /// `~XSControl_WorkSession`: `ClearBinders` (its errors cannot be reported).
    fn drop(&mut self) {
        let _ = self.clear_binders();
    }
}
