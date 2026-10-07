// Rust port of DataExchange/TKXSBase/XSControl/XSControl_TransferWriter (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XSControl_TransferWriter`: the writing transfer of a work session: a finder process
//! in which the controller transfers transients and shapes to a model.
use super::{XSControlController, XSControlUtils};
use crate::if_select::IFSelectReturnStatus;
use crate::interface::{InterfaceCheckIterator, InterfaceInterfaceModel, InterfaceModelEntities};
use crate::transfer::{
    TransferBinder, TransferFinderProcess, TransferSimpleBinderOfTransient, TransferTransientMapper,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{
    impl_transient,
    message::ProgressRange,
    standard::{Failure, Handle, SharedState, StandardTransient, exceptions},
};
use std::fmt::Write as _;

/// The default size of a new finder process.
const FINDER_PROCESS_SIZE: i32 = 10000;

/// The fields of `XSControl_TransferWriter`.
struct WriterState {
    controller: Handle<dyn XSControlController>,
    fp: Handle<TransferFinderProcess>,
    mode: i32,
}

/// `XSControl_TransferWriter`.
///
/// ```
/// use rustycad_tkxsbase::xs_control::XSControlTransferWriter;
/// let tw = XSControlTransferWriter::new();
/// assert!(tw.finder_process().is_some() && tw.transfer_mode() == 0);
/// ```
pub struct XSControlTransferWriter {
    transient: StandardTransient,
    state: SharedState<WriterState>,
}
impl_transient!(
    XSControlTransferWriter,
    "XSControl_TransferWriter",
    StandardTransient,
    transient
);

impl Default for XSControlTransferWriter {
    fn default() -> Self {
        let state = WriterState {
            controller: Handle::default(),
            fp: TransferFinderProcess::new(FINDER_PROCESS_SIZE),
            mode: 0,
        };
        Self {
            transient: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
}

impl XSControlTransferWriter {
    /// `XSControl_TransferWriter()`: a new finder process, mode 0.
    pub fn new() -> Self {
        Self::default()
    }
    /// `FinderProcess`.
    pub fn finder_process(&self) -> Handle<TransferFinderProcess> {
        self.state.read().fp.clone()
    }
    /// `SetFinderProcess(theFP)`.
    pub fn set_finder_process(&self, fp: &Handle<TransferFinderProcess>) {
        self.state.write().fp = fp.clone();
    }
    /// `Controller`.
    pub fn controller(&self) -> Handle<dyn XSControlController> {
        self.state.read().controller.clone()
    }
    /// `SetController(theCtl)`: then a new finder process.
    pub fn set_controller(&self, ctl: &Handle<dyn XSControlController>) {
        self.state.write().controller = ctl.clone();
        self.clear(-1);
    }
    /// `Clear(theMode)`: a new finder process (negative mode or none), else the process
    /// cleared.
    pub fn clear(&self, mode: i32) {
        let mut st = self.state.write();
        match st.fp.get() {
            Some(fp) if mode >= 0 => fp.clear(),
            _ => st.fp = TransferFinderProcess::new(FINDER_PROCESS_SIZE),
        }
    }
    /// `TransferMode`.
    pub fn transfer_mode(&self) -> i32 {
        self.state.read().mode
    }
    /// `SetTransferMode(theMode)`.
    pub fn set_transfer_mode(&self, mode: i32) {
        self.state.write().mode = mode;
    }
    /// The live controller, `Standard_NullObject` when none.
    fn live_controller(&self) -> Result<Handle<dyn XSControlController>, Failure> {
        let ctl = self.controller();
        if ctl.is_null() {
            return Err(exceptions::null_object(
                "XSControl_TransferWriter: no controller",
            ));
        }
        Ok(ctl)
    }
    /// The text of `PrintStats`.
    fn stats_text(&self) -> Result<String, Failure> {
        let mode = self.transfer_mode();
        let ctl = self.live_controller()?;
        let mut out = format!(
            "\n*******************************************************************\n******        Statistics on Transfer (Write)                 ******\n\n*******************************************************************\n******        Transfer Mode = {mode}"
        );
        let modehelp = ctl.mode_write_help(mode, true);
        if !modehelp.is_empty() {
            let _ = write!(
                out,
                "  I.E.  {}",
                String::from_utf8_lossy(modehelp.as_bytes())
            );
        }
        out.push_str("       ******\n");
        Ok(out)
    }
    /// `PrintStats(theWhat, theMode = 0)`: the transfer mode and its help, on the process
    /// messenger; `Standard_NullObject` without controller.
    pub fn print_stats(&self, _what: i32, _mode: i32) -> Result<(), Failure> {
        let fp = self.finder_process();
        let process = fp.get().ok_or_else(|| {
            exceptions::null_object("XSControl_TransferWriter::PrintStats: no finder process")
        })?;
        let text = self.stats_text()?;
        let _ = process.messenger().send_info().write_str(&text);
        Ok(())
    }
    /// `RecognizeTransient(theObj)`: a shape (of a binder or a handle shape) by
    /// `RecognizeShape`, else the controller recognizes the transient.
    pub fn recognize_transient(&self, obj: &Handle<StandardTransient>) -> bool {
        let ctl = self.controller();
        let Some(live) = ctl.get() else {
            return false;
        };
        let sh = XSControlUtils::new().binder_shape(obj);
        if !sh.is_null() {
            return self.recognize_shape(&sh);
        }
        live.recognize_write_transient(obj, self.transfer_mode())
    }
    /// The process of a writing, created when none, its actor reset.
    fn writing_process(&self) -> Handle<TransferFinderProcess> {
        let mut st = self.state.write();
        if st.fp.is_null() {
            st.fp = TransferFinderProcess::new(FINDER_PROCESS_SIZE);
        }
        if let Some(fp) = st.fp.get() {
            fp.set_actor(&Handle::default());
        }
        st.fp.clone()
    }
    /// A guarded writing (natively a `try`): the statistics, a trace line, then the
    /// transfer; an error is reported and gives a failure.
    fn guarded_write(
        &self,
        fp: &Handle<TransferFinderProcess>,
        trace: impl FnOnce() -> Result<String, Failure>,
        write: impl FnOnce() -> Result<IFSelectReturnStatus, Failure>,
    ) -> Result<IFSelectReturnStatus, Failure> {
        let process = fp.get().ok_or_else(|| {
            exceptions::null_object("XSControl_TransferWriter: no finder process")
        })?;
        let messenger = process.messenger();
        let mut sout = messenger.send_info();
        let result = self
            .print_stats(self.transfer_mode(), 0)
            .and_then(|_| trace())
            .and_then(|text| {
                let _ = sout.write_str(&text);
                write()
            });
        match result {
            Ok(status) => Ok(status),
            Err(failure) => {
                let _ = writeln!(
                    sout,
                    "****  ****  TransferWriteShape, EXCEPTION : {}",
                    failure.message()
                );
                Ok(IFSelectReturnStatus::RetFail)
            }
        }
    }
    /// `TransferWriteTransient(theModel, theObj, theProgress)`: the transient transferred
    /// to the model by the controller; error without controller, void without model.
    pub fn transfer_write_transient(
        &self,
        model: &Handle<dyn InterfaceInterfaceModel>,
        obj: &Handle<StandardTransient>,
        progress: &ProgressRange<'_>,
    ) -> Result<IFSelectReturnStatus, Failure> {
        let ctl = self.controller();
        let Some(live) = ctl.get() else {
            return Ok(IFSelectReturnStatus::RetError);
        };
        if model.is_null() {
            return Ok(IFSelectReturnStatus::RetVoid);
        }
        let fp = self.writing_process();
        let trace = || {
            let kind = obj.dynamic_type().ok_or_else(|| {
                exceptions::null_object(
                    "XSControl_TransferWriter::TransferWriteTransient: null object",
                )
            })?;
            Ok(format!(
                "******        Transferring Transient, CDL Type = {}   ******\n",
                kind.name()
            ))
        };
        self.guarded_write(&fp, trace, || {
            live.transfer_write_transient(obj, &fp, model, self.transfer_mode(), progress)
        })
    }
    /// `RecognizeShape(theShape)`: the controller recognizes the shape (false for null).
    pub fn recognize_shape(&self, shape: &Shape) -> bool {
        let ctl = self.controller();
        match ctl.get() {
            Some(live) if !shape.is_null() => {
                live.recognize_write_shape(shape, self.transfer_mode())
            }
            _ => false,
        }
    }
    /// `TransferWriteShape(theModel, theShape, theProgress)`: the shape transferred to the
    /// model by the controller; error without controller, void without model.
    pub fn transfer_write_shape(
        &self,
        model: &Handle<dyn InterfaceInterfaceModel>,
        shape: &Shape,
        progress: &ProgressRange<'_>,
    ) -> Result<IFSelectReturnStatus, Failure> {
        let ctl = self.controller();
        let Some(live) = ctl.get() else {
            return Ok(IFSelectReturnStatus::RetError);
        };
        if model.is_null() {
            return Ok(IFSelectReturnStatus::RetVoid);
        }
        let fp = self.writing_process();
        let trace = || {
            Ok(format!(
                "******        Transferring Shape, ShapeType = {}                      ******\n",
                shape.shape_type()? as i32
            ))
        };
        self.guarded_write(&fp, trace, || {
            live.transfer_write_shape(shape, &fp, model, self.transfer_mode(), progress)
        })
    }
    /// The checks of the simple binders of the process that have both fails and
    /// warnings (native quirk kept: `||` for `&&` in the skip test), each with the entity
    /// of its key.
    fn binder_checks(
        &self,
    ) -> Result<Vec<(Handle<TransferSimpleBinderOfTransient>, i32)>, Failure> {
        let fp = self.finder_process();
        let Some(process) = fp.get() else {
            return Ok(Vec::new());
        };
        let mut found = Vec::new();
        for i in 1..=process.nb_mapped() {
            let tb = process
                .map_item(i)?
                .cast::<TransferSimpleBinderOfTransient>();
            let Some(live) = tb.get() else {
                continue;
            };
            let ach = live.check();
            if ach.nb_fails() == 0 || ach.nb_warnings() == 0 {
                continue;
            }
            found.push((tb.clone(), i));
        }
        Ok(found)
    }
    /// `CheckList`: the checks of the simple binders with fails and warnings, on their
    /// starting entities (or mapped finders).
    pub fn check_list(&self) -> Result<InterfaceCheckIterator, Failure> {
        let mut chl = InterfaceCheckIterator::new();
        let fp = self.finder_process();
        let Some(process) = fp.get() else {
            return Ok(chl);
        };
        for (tb, i) in self.binder_checks()? {
            let ach = tb.check();
            let finder = process.mapped(i)?;
            match finder.cast::<TransferTransientMapper>().get() {
                Some(tm) => ach.get_entity(tm.value()),
                None => ach.get_entity(&finder.cast()),
            }
            chl.add(&ach, 0);
        }
        Ok(chl)
    }
    /// `ResultCheckList(theModel)`: the same checks, numbered by their results in the
    /// model (0 otherwise).
    pub fn result_check_list(
        &self,
        model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> Result<InterfaceCheckIterator, Failure> {
        let mut chl = InterfaceCheckIterator::new();
        for (tb, _) in self.binder_checks()? {
            let ent = tb.result();
            let num = match model.get() {
                Some(m) if ent.is_some() => m.number(&ent),
                _ => 0,
            };
            chl.add(&tb.check(), num);
        }
        Ok(chl)
    }
}
