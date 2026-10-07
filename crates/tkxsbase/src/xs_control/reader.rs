// Rust port of DataExchange/TKXSBase/XSControl/XSControl_Reader (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XSControl_Reader`: the reading front end of a norm: a file read in a work session,
//! its roots for transfer, their transfer to shapes and the reports. The virtual members
//! and the members that call them are in `reader_transfer` ([`XSControlReaderOps`]).
use super::XSControlWorkSession;
use crate::if_select::{IFSelectPrintCount, IFSelectTransientList};
use crate::interface::{InterfaceCheckStatus, InterfaceInterfaceModel};
use crate::transfer::{
    TransferActorOfTransientProcessOps, TransferIteratorOfProcess, TransferTransientKind,
};
use crate::xs_algo::{DEShapeFixParameters, XSAlgoParameterMap, XSAlgoProcessingFlags};
use rustycad_tkbrep::topods::{Builder, Compound, Shape};
use rustycad_tkernel::{
    message::Message,
    standard::{Failure, Handle, StandardTransient, exceptions},
};
use rustycad_tkshhealing::shape_process::ShapeProcessOperationsFlags;
use std::fmt::Write as _;
use std::io::Write;

/// The counts of `GetStatsTransfer`: the mapped entities, those with a result, those
/// failed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct XSControlTransferStats {
    /// The mapped entities (`nbMapped`).
    pub nb_mapped: i32,
    /// The entities with a result without fail (`nbWithResult`).
    pub nb_with_result: i32,
    /// The entities without result or with a fail (`nbWithFail`).
    pub nb_with_fail: i32,
}

/// `XSControl_Reader`: a value class (natively with virtual members, see
/// [`XSControlReaderOps`](super::XSControlReaderOps)).
///
/// ```
/// use rustycad_tkxsbase::xs_control::XSControlReader;
/// let reader = XSControlReader::new();
/// assert!(reader.ws().is_some() && reader.nb_shapes() == 0 && reader.one_shape().is_null());
/// ```
#[derive(Default)]
pub struct XSControlReader {
    /// `therootsta` (protected): true when the roots for transfer are computed.
    pub therootsta: bool,
    /// `theroots` (protected): the roots for transfer.
    pub theroots: Vec<Handle<StandardTransient>>,
    thesession: Handle<XSControlWorkSession>,
    theshapes: Vec<Shape>,
}

/// The session of a reader, `Standard_NullObject` when null (a native null dereference).
pub(super) fn live_session(
    ws: &Handle<XSControlWorkSession>,
) -> Result<&XSControlWorkSession, Failure> {
    ws.get()
        .ok_or_else(|| exceptions::null_object("XSControl_Reader: no work session"))
}

impl XSControlReader {
    /// `XSControl_Reader()`: a new work session (without norm).
    pub fn new() -> Self {
        Self {
            thesession: XSControlWorkSession::new(),
            ..Self::default()
        }
    }
    /// `XSControl_Reader(norm)`: a new session of the norm recorded as `norm`.
    pub fn with_norm(norm: &str) -> Result<Self, Failure> {
        let mut reader = Self::default();
        reader.set_norm(norm)?;
        Ok(reader)
    }
    /// `XSControl_Reader(WS, scratch = true)`: the session `ws` (see `set_ws`).
    pub fn with_session(ws: &Handle<XSControlWorkSession>, scratch: bool) -> Result<Self, Failure> {
        let mut reader = Self::default();
        reader.set_ws(ws, scratch)?;
        Ok(reader)
    }
    /// `SetNorm(norm)`: the norm selected in the session (created when none), its
    /// transfer reader prepared; false for an unknown norm.
    pub fn set_norm(&mut self, norm: &str) -> Result<bool, Failure> {
        if self.thesession.is_null() {
            self.set_ws(&XSControlWorkSession::new(), true)?;
        }
        let ws = live_session(&self.thesession)?;
        let stat = ws.select_norm(norm)?;
        if stat {
            ws.init_transfer_reader(0)?;
            ws.init_transfer_reader(4)?;
        }
        Ok(stat)
    }
    /// `SetWS(WS, scratch = true)`: the session, the roots forgotten; with a norm, a new
    /// model (with `scratch` or without model) and the transfer reader prepared.
    /// `Standard_NullObject` for a null session.
    pub fn set_ws(
        &mut self,
        ws: &Handle<XSControlWorkSession>,
        scratch: bool,
    ) -> Result<(), Failure> {
        self.therootsta = false;
        self.theroots.clear();
        self.thesession = ws.clone();
        let live = live_session(ws)?;
        if live.norm_adaptor().is_null() {
            return Ok(());
        }
        if scratch || live.model().is_null() {
            live.new_model()?;
        }
        live.init_transfer_reader(0)?;
        live.init_transfer_reader(4)
    }
    /// `WS`.
    pub fn ws(&self) -> Handle<XSControlWorkSession> {
        self.thesession.clone()
    }
    /// The live session, `Standard_NullObject` when null.
    pub(super) fn session(&self) -> Result<&XSControlWorkSession, Failure> {
        live_session(&self.thesession)
    }
    /// `Model`.
    pub fn model(&self) -> Result<Handle<dyn InterfaceInterfaceModel>, Failure> {
        Ok(self.session()?.model())
    }
    /// `GiveList(first, ent)`: the list `first` gives applied to the list of `ent`.
    pub fn give_list_from(
        &self,
        first: &str,
        ent: &Handle<StandardTransient>,
    ) -> Result<IFSelectTransientList, Failure> {
        self.session()?.give_list_from_list(first, ent)
    }
    /// `ClearShapes`.
    pub fn clear_shapes(&mut self) {
        self.theshapes.clear();
    }
    /// `NbShapes`.
    pub fn nb_shapes(&self) -> i32 {
        self.theshapes.len() as i32
    }
    /// `Shapes` (protected): the shapes, modifiable.
    pub fn shapes(&mut self) -> &mut Vec<Shape> {
        &mut self.theshapes
    }
    /// `Shape(num = 1)`: `Standard_OutOfRange` out of the shapes.
    pub fn shape(&self, num: i32) -> Result<Shape, Failure> {
        let found = usize::try_from(num - 1)
            .ok()
            .and_then(|i| self.theshapes.get(i));
        found.cloned().ok_or_else(|| {
            exceptions::out_of_range(&format!(
                "XSControl_Reader::Shape: {num}; expected [1, {}]",
                self.theshapes.len()
            ))
        })
    }
    /// `OneShape`: null without shapes, the shape when one, else a compound of the
    /// non-null shapes (null when all are null).
    pub fn one_shape(&self) -> Shape {
        match self.theshapes.as_slice() {
            [] => Shape::default(),
            [one] => one.clone(),
            many => compound_of(many),
        }
    }
    /// `PrintCheckLoad(theStream, failsonly, mode)`: the checks of the model.
    pub fn print_check_load_to(
        &self,
        s: &mut dyn Write,
        failsonly: bool,
        mode: IFSelectPrintCount,
    ) -> Result<(), Failure> {
        let ws = self.session()?;
        ws.print_check_list(s, &ws.model_check_list(true)?, failsonly, mode)
    }
    /// `PrintCheckLoad(failsonly, mode)`: on the default messenger (info).
    pub fn print_check_load(
        &self,
        failsonly: bool,
        mode: IFSelectPrintCount,
    ) -> Result<(), Failure> {
        send_info_with(|out| self.print_check_load_to(out, failsonly, mode))
    }
    /// `PrintCheckTransfer(theStream, failsonly, mode)`: the checks of the last transfer.
    pub fn print_check_transfer_to(
        &self,
        s: &mut dyn Write,
        failsonly: bool,
        mode: IFSelectPrintCount,
    ) -> Result<(), Failure> {
        let ws = self.session()?;
        let checks = live_reader(ws)?.last_check_list();
        ws.print_check_list(s, &checks, failsonly, mode)
    }
    /// `PrintCheckTransfer(failsonly, mode)`: on the default messenger (info).
    pub fn print_check_transfer(
        &self,
        failsonly: bool,
        mode: IFSelectPrintCount,
    ) -> Result<(), Failure> {
        send_info_with(|out| self.print_check_transfer_to(out, failsonly, mode))
    }
    /// `PrintStatsTransfer(theStream, what, mode = 0)`: the statistics of the transfer.
    pub fn print_stats_transfer_to(
        &self,
        s: &mut dyn Write,
        what: i32,
        mode: i32,
    ) -> Result<(), Failure> {
        live_reader(self.session()?)?.print_stats(s, what, mode)
    }
    /// `PrintStatsTransfer(what, mode = 0)`: on the default messenger (info).
    pub fn print_stats_transfer(&self, what: i32, mode: i32) -> Result<(), Failure> {
        send_info_with(|out| self.print_stats_transfer_to(out, what, mode))
    }
    /// `GetStatsTransfer(list, nbMapped, nbWithResult, nbWithFail)`: the counts over the
    /// complete results of the process (filtered on `list` when given);
    /// `Standard_NullObject` without process.
    pub fn get_stats_transfer(
        &self,
        list: Option<&[Handle<StandardTransient>]>,
    ) -> Result<XSControlTransferStats, Failure> {
        let tp = live_reader(self.session()?)?.transient_process();
        let process = tp.get().ok_or_else(|| {
            exceptions::null_object("XSControl_Reader::GetStatsTransfer: no transient process")
        })?;
        let mut itrp = process.complete_result(true);
        if list.is_some() {
            itrp.filter(list, true);
        }
        count_results(&mut itrp)
    }
}

/// The counts of `GetStatsTransfer` over an iterator.
fn count_results(
    itrp: &mut TransferIteratorOfProcess<TransferTransientKind>,
) -> Result<XSControlTransferStats, Failure> {
    let mut stats = XSControlTransferStats::default();
    itrp.start();
    while itrp.more() {
        let binder = itrp.value()?;
        stats.nb_mapped += 1;
        let ok = binder.get().is_some_and(|b| {
            b.has_result()
                && matches!(
                    b.check().status(),
                    InterfaceCheckStatus::CheckOK | InterfaceCheckStatus::CheckWarning
                )
        });
        if ok {
            stats.nb_with_result += 1;
        } else {
            stats.nb_with_fail += 1;
        }
        itrp.next();
    }
    Ok(stats)
}

/// A compound of the non-null shapes (null when all are null).
fn compound_of(shapes: &[Shape]) -> Shape {
    let mut compound = Compound::default();
    Builder.make_compound(&mut compound);
    let mut result = compound.into_shape();
    let mut added = false;
    for shape in shapes.iter().filter(|s| !s.is_null()) {
        // A new compound is free and holds any shape: the addition cannot fail.
        added |= Builder.add(&mut result, shape).is_ok();
    }
    if added { result } else { Shape::default() }
}

/// The transfer reader of a session, `Standard_NullObject` when null.
pub(super) fn live_reader(
    ws: &XSControlWorkSession,
) -> Result<Handle<super::XSControlTransferReader>, Failure> {
    let reader = ws.transfer_reader();
    if reader.is_null() {
        return Err(exceptions::null_object(
            "XSControl_Reader: no transfer reader",
        ));
    }
    Ok(reader)
}

/// A report written to a buffer, then sent to the default messenger (info).
fn send_info_with(
    report: impl FnOnce(&mut dyn Write) -> Result<(), Failure>,
) -> Result<(), Failure> {
    let mut out = Vec::new();
    let result = report(&mut out);
    let _ = Message::send_info().write_str(&String::from_utf8_lossy(&out));
    result
}

impl XSControlReader {
    /// `GetActor` (private): the reading actor of the transfer reader (null without
    /// session or reader).
    fn actor(&self) -> Handle<dyn TransferActorOfTransientProcessOps> {
        let reader = self
            .thesession
            .get()
            .map(XSControlWorkSession::transfer_reader)
            .unwrap_or_default();
        reader.get().map(|r| r.actor()).unwrap_or_default()
    }
    /// `SetShapeFixParameters(theParameters)` (both the copy and the move overloads):
    /// for the actor (nothing without actor).
    pub fn set_shape_fix_parameters(&self, parameters: XSAlgoParameterMap) {
        if let Some(actor) = self.actor().get() {
            actor
                .shape_processing()
                .set_shape_fix_parameters(parameters);
        }
    }
    /// `SetShapeFixParameters(theParameters, theAdditionalParameters = {})`: for the
    /// actor (nothing without actor).
    pub fn set_shape_fix_parameters_from(
        &self,
        parameters: &DEShapeFixParameters,
        additional: &XSAlgoParameterMap,
    ) {
        if let Some(actor) = self.actor().get() {
            actor
                .shape_processing()
                .set_shape_fix_parameters_from(parameters, additional);
        }
    }
    /// `GetShapeFixParameters`: a copy of those of the actor (empty without actor).
    pub fn shape_fix_parameters(&self) -> XSAlgoParameterMap {
        self.actor()
            .get()
            .map(|a| a.shape_processing().shape_fix_parameters())
            .unwrap_or_default()
    }
    /// `SetShapeProcessFlags(theFlags)`: for the actor (nothing without actor).
    pub fn set_shape_process_flags(&self, flags: ShapeProcessOperationsFlags) {
        if let Some(actor) = self.actor().get() {
            actor.shape_processing().set_flags(flags);
        }
    }
    /// `GetShapeProcessFlags`: those of the actor (empty and unset without actor).
    pub fn shape_process_flags(&self) -> XSAlgoProcessingFlags {
        self.actor()
            .get()
            .map(|a| a.shape_processing().flags())
            .unwrap_or_default()
    }
}
