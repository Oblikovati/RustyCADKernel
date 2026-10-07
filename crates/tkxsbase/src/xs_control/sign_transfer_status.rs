// Rust port of DataExchange/TKXSBase/XSControl/XSControl_SignTransferStatus (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XSControl_SignTransferStatus`: the signature of the transfer status of an entity
//! (its result types, warnings and fails) in a transient process.
use super::XSControlTransferReader;
use crate::if_select::{IFSelectSignature, IFSelectSignatureBase, impl_signature_text};
use crate::interface::{InterfaceCheckStatus, InterfaceInterfaceModel, InterfaceSignType};
use crate::moni_tool::MoniToolSignText;
use crate::transfer::{
    TransferBinder, TransferStatusExec, TransferStatusResult, TransferTransientProcess,
};
use rustycad_tkernel::{
    impl_transient,
    standard::{Handle, SharedState, StandardTransient},
};

/// `XSControl_SignTransferStatus`.
///
/// ```
/// use rustycad_tkxsbase::moni_tool::MoniToolSignText;
/// use rustycad_tkxsbase::xs_control::XSControlSignTransferStatus;
/// assert_eq!(XSControlSignTransferStatus::new().name(), "Transfer Status");
/// ```
pub struct XSControlSignTransferStatus {
    transient: StandardTransient,
    base: IFSelectSignatureBase,
    the_tr: SharedState<Handle<XSControlTransferReader>>,
    the_tp: SharedState<Handle<TransferTransientProcess>>,
}
impl_transient!(XSControlSignTransferStatus, "XSControl_SignTransferStatus", dyn IFSelectSignature, transient, interfaces: [dyn MoniToolSignText, dyn InterfaceSignType, dyn IFSelectSignature]);
impl_signature_text!(XSControlSignTransferStatus);

impl Default for XSControlSignTransferStatus {
    fn default() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: IFSelectSignatureBase::new("Transfer Status"),
            the_tr: SharedState::default(),
            the_tp: SharedState::default(),
        }
    }
}

/// `BinderStatus(binder)`: 20 for a binder running or looping, else 1, 2, 3 for no
/// result (ok, warning, fail) and 11, 12, 13 with a result; 0 for null.
fn binder_status(binder: &Handle<dyn TransferBinder>) -> i32 {
    let Some(live) = binder.get() else {
        return 0;
    };
    let (est, res) = (live.status_exec(), live.has_result());
    if matches!(
        est,
        TransferStatusExec::StatusRun | TransferStatusExec::StatusLoop
    ) {
        return 20;
    }
    let base = match live.check().status() {
        InterfaceCheckStatus::CheckOK => 1,
        InterfaceCheckStatus::CheckWarning => 2,
        InterfaceCheckStatus::CheckFail => 3,
        _ => return 0,
    };
    if res { base + 10 } else { base }
}

/// The result types of a chain of binders, "Result:" then separated by ",".
fn result_types(binder: &Handle<dyn TransferBinder>) -> String {
    let mut text = String::new();
    let mut bnd = binder.clone();
    while let Some(live) = bnd.get() {
        if live.status() != TransferStatusResult::StatusVoid {
            text.push_str(if text.is_empty() { "Result:" } else { "," });
            text.push_str(&live.result_type_name());
        }
        let next = live.next_result();
        bnd = next;
    }
    text
}

impl XSControlSignTransferStatus {
    /// `XSControl_SignTransferStatus()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `XSControl_SignTransferStatus(TR)`.
    pub fn with_reader(tr: &Handle<XSControlTransferReader>) -> Self {
        let sign = Self::default();
        sign.set_reader(tr);
        sign
    }
    /// `SetReader(TR)`.
    pub fn set_reader(&self, tr: &Handle<XSControlTransferReader>) {
        *self.the_tr.write() = tr.clone();
    }
    /// `SetMap(TP)`.
    pub fn set_map(&self, tp: &Handle<TransferTransientProcess>) {
        *self.the_tp.write() = tp.clone();
    }
    /// `Map`.
    pub fn map(&self) -> Handle<TransferTransientProcess> {
        self.the_tp.read().clone()
    }
    /// `Reader`.
    pub fn reader(&self) -> Handle<XSControlTransferReader> {
        self.the_tr.read().clone()
    }
}

impl InterfaceSignType for XSControlSignTransferStatus {
    /// `Value(ent, model)`: empty for no result or no data, "Warning", "Fail",
    /// "Fail on run", else the result types ("/Warning" or "/Fail" added).
    fn value(
        &self,
        ent: &Handle<StandardTransient>,
        _model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> String {
        if ent.is_null() {
            return String::new();
        }
        let mut tp = self.map();
        if tp.is_null()
            && let Some(reader) = self.reader().get()
        {
            tp = reader.transient_process();
        }
        let Some(process) = tp.get() else {
            return String::new();
        };
        let binder = process.find(ent);
        match binder_status(&binder) {
            s if s <= 1 => String::new(),
            2 => String::from("Warning"),
            3 => String::from("Fail"),
            20 => String::from("Fail on run"),
            s => {
                let mut text = result_types(&binder);
                if s == 12 {
                    text.push_str("/Warning");
                }
                if s == 13 {
                    text.push_str("/Fail");
                }
                text
            }
        }
    }
}

impl IFSelectSignature for XSControlSignTransferStatus {
    fn signature_base(&self) -> &IFSelectSignatureBase {
        &self.base
    }
}
