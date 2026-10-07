// Rust port of DataExchange/TKXSBase/XSControl/XSControl_TransferReader (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XSControl_TransferReader`: the reading transfer of a work session: the transient
//! process, its actor (from the controller), and the final results recorded by entity
//! number. The results queries are in `transfer_reader_results`, the transfers in
//! `transfer_reader_transfer`, the statistics in `transfer_reader_stats`.
use super::XSControlController;
use crate::interface::{InterfaceHGraph, InterfaceInterfaceModel, InterfaceModelEntities};
use crate::transfer::{
    TransferActorOfTransientProcessOps, TransferBinder, TransferResultFromModel,
    TransferSimpleBinderOfTransient, TransferTransientProcess,
};
use crate::transfer_brep::TransferBRepShapeBinder;
use rustycad_tkbrep::topods::{HShape, Shape};
use rustycad_tkernel::{
    impl_transient,
    ncollection::DataMap,
    standard::{
        Failure, Handle, RuntimeType, RuntimeTyped, SharedState, StandardTransient, exceptions,
    },
    tcollection::AsciiString,
};
use std::collections::HashMap;
use std::sync::RwLockWriteGuard;

/// The context of the transfers (`NCollection_DataMap<TCollection_AsciiString,
/// occ::handle<Standard_Transient>>`).
pub type XSControlContext = DataMap<AsciiString, Handle<StandardTransient>>;

/// The fields of `XSControl_TransferReader` (the context aside).
#[derive(Default)]
pub(super) struct ReaderState {
    pub(super) controller: Handle<dyn XSControlController>,
    pub(super) filename: AsciiString,
    pub(super) model: Handle<dyn InterfaceInterfaceModel>,
    pub(super) graph: Handle<InterfaceHGraph>,
    pub(super) actor: Handle<dyn TransferActorOfTransientProcessOps>,
    pub(super) tp: Handle<TransferTransientProcess>,
    /// The final result (or the skipped entity) by entity number.
    pub(super) results: HashMap<i32, Handle<StandardTransient>>,
    /// The shape results, `None` for the native null sequence.
    pub(super) shape_result: Option<Vec<Shape>>,
}

/// `XSControl_TransferReader`.
///
/// ```
/// use rustycad_tkxsbase::xs_control::XSControlTransferReader;
/// let tr = XSControlTransferReader::new();
/// assert!(tr.model().is_null() && tr.recorded_list().is_empty());
/// ```
#[derive(Default)]
pub struct XSControlTransferReader {
    transient: StandardTransient,
    pub(super) state: SharedState<ReaderState>,
    context: SharedState<XSControlContext>,
}
impl_transient!(
    XSControlTransferReader,
    "XSControl_TransferReader",
    StandardTransient,
    transient
);

impl XSControlTransferReader {
    /// `XSControl_TransferReader()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `SetController(theControl)`: the actor is reset and the results cleared.
    pub fn set_controller(&self, control: &Handle<dyn XSControlController>) {
        {
            let mut st = self.state.write();
            st.controller = control.clone();
            st.actor = Handle::default();
        }
        self.clear(-1);
    }
    /// `SetActor(theActor)`.
    pub fn set_actor(&self, actor: &Handle<dyn TransferActorOfTransientProcessOps>) {
        self.state.write().actor = actor.clone();
    }
    /// `Actor`: the actor, taken from the controller (for the model) when not set.
    pub fn actor(&self) -> Handle<dyn TransferActorOfTransientProcessOps> {
        let (actor, controller, model) = {
            let st = self.state.read();
            (st.actor.clone(), st.controller.clone(), st.model.clone())
        };
        if actor.is_some() || model.is_null() {
            return actor;
        }
        let Some(ctl) = controller.get() else {
            return actor;
        };
        let actor = ctl.actor_read(&model);
        self.state.write().actor = actor.clone();
        actor
    }
    /// `SetModel(theModel)`: also for the transient process.
    pub fn set_model(&self, model: &Handle<dyn InterfaceInterfaceModel>) {
        let tp = {
            let mut st = self.state.write();
            st.model = model.clone();
            st.tp.clone()
        };
        if let Some(process) = tp.get() {
            process.set_model(model);
        }
    }
    /// `SetGraph(theGraph)`: the model of the graph (null for a null graph), also for the
    /// transient process.
    pub fn set_graph(&self, graph: &Handle<InterfaceHGraph>) {
        let tp = {
            let mut st = self.state.write();
            st.model = graph
                .get()
                .map(|g| g.graph().model().clone())
                .unwrap_or_default();
            st.graph = graph.clone();
            st.tp.clone()
        };
        if let Some(process) = tp.get() {
            process.set_graph(graph);
        }
    }
    /// `Model`.
    pub fn model(&self) -> Handle<dyn InterfaceInterfaceModel> {
        self.state.read().model.clone()
    }
    /// `SetContext(theName, theCtx)`.
    pub fn set_context(&self, name: &str, ctx: &Handle<StandardTransient>) {
        self.context
            .write()
            .bind(AsciiString::new(name), ctx.clone());
    }
    /// `GetContext(theName, theType, theCtx)`: the context `name` (of the kind `kind`
    /// when given), `None` when absent.
    pub fn get_context(
        &self,
        name: &str,
        kind: Option<&RuntimeType>,
    ) -> Option<Handle<StandardTransient>> {
        let ctx = self
            .context
            .read()
            .seek(&AsciiString::new(name))
            .cloned()
            .filter(|c| c.is_some())?;
        match kind {
            Some(k) if !ctx.is_kind(k) => None,
            _ => Some(ctx),
        }
    }
    /// `Context`: the context, modifiable (a guard to drop before other calls).
    pub fn context(&self) -> RwLockWriteGuard<'_, XSControlContext> {
        self.context.write()
    }
    /// `SetFileName(theName)`.
    pub fn set_file_name(&self, name: &str) {
        self.state.write().filename = AsciiString::new(name);
    }
    /// `FileName`.
    pub fn file_name(&self) -> AsciiString {
        self.state.read().filename.clone()
    }
    /// `Clear(theMode)`: the results (bit 1) and the model, graph, process, actor and file
    /// name (bit 2).
    pub fn clear(&self, mode: i32) {
        let mut st = self.state.write();
        if mode & 1 != 0 {
            st.results.clear();
            st.shape_result = None;
        }
        if mode & 2 != 0 {
            st.model = Handle::default();
            st.graph = Handle::default();
            st.tp = Handle::default();
            st.actor = Handle::default();
            st.filename = AsciiString::default();
        }
    }
    /// `TransientProcess`.
    pub fn transient_process(&self) -> Handle<TransferTransientProcess> {
        self.state.read().tp.clone()
    }
    /// `SetTransientProcess(theTP)`.
    pub fn set_transient_process(&self, tp: &Handle<TransferTransientProcess>) {
        self.state.write().tp = tp.clone();
    }
    /// The number of `ent` in the model, 0 without model.
    pub(super) fn number(&self, ent: &Handle<StandardTransient>) -> i32 {
        self.model().get().map_or(0, |m| m.number(ent))
    }
    /// The result recorded for the number `num` (`None` when not bound).
    pub(super) fn result_at(&self, num: i32) -> Option<Handle<StandardTransient>> {
        self.state.read().results.get(&num).cloned()
    }
    /// `RecordResult(theEnt)`: the final result of an entity of the model recorded (a
    /// shape result as a transient result of its shape); false without model or process.
    pub fn record_result(&self, ent: &Handle<StandardTransient>) -> Result<bool, Failure> {
        let (model, tp, filename) = {
            let st = self.state.read();
            (st.model.clone(), st.tp.clone(), st.filename.clone())
        };
        let Some(m) = model.get().filter(|_| tp.is_some()) else {
            return Ok(false);
        };
        let num = m.number(ent);
        if num == 0 {
            return Ok(false);
        }
        let _lab = m.string_label(ent);
        let res = Handle::new(TransferResultFromModel::new());
        res.fill(&tp, ent);
        let main = res.main_result();
        let main_live = main.get().ok_or_else(|| {
            exceptions::null_object(&format!(
                "XSControl_TransferReader::RecordResult: entity {num} has no binder in the process"
            ))
        })?;
        let binder = main_live.binder();
        if let Some(shb) = binder.cast::<TransferBRepShapeBinder>().get() {
            let trb = TransferSimpleBinderOfTransient::new();
            trb.set_result(&Handle::new(HShape::new(&shb.result())).cast())?;
            trb.merge(&binder);
            main_live.set_binder(&Handle::new(trb).cast());
        }
        res.set_file_name(&String::from_utf8_lossy(filename.as_bytes()));
        self.state.write().results.insert(num, res.cast());
        Ok(true)
    }
    /// The type test of `IsRecorded` and `IsSkipped` on the mark of `ent` (false when not
    /// marked); `Standard_NullObject` for a cleared mark (a native null dereference).
    fn mark_is(
        &self,
        ent: &Handle<StandardTransient>,
        what: &str,
        test: fn(&Handle<StandardTransient>) -> bool,
    ) -> Result<bool, Failure> {
        let num = self.number(ent);
        let Some(mark) = self.result_at(num).filter(|_| num != 0) else {
            return Ok(false);
        };
        if mark.is_null() {
            return Err(exceptions::null_object(&format!(
                "XSControl_TransferReader::{what}: the result of entity {num} was cleared"
            )));
        }
        Ok(test(&mark))
    }
    /// `IsRecorded(theEnt)`: a final result is recorded for `ent`.
    pub fn is_recorded(&self, ent: &Handle<StandardTransient>) -> Result<bool, Failure> {
        self.mark_is(ent, "IsRecorded", |r| {
            r.is_instance(TransferResultFromModel::static_type())
        })
    }
    /// `HasResult(theEnt)`: the final result of `ent` has a result.
    pub fn has_result(&self, ent: &Handle<StandardTransient>) -> bool {
        let num = self.number(ent);
        num != 0
            && self.result_at(num).is_some_and(|r| {
                r.cast::<TransferResultFromModel>()
                    .get()
                    .is_some_and(TransferResultFromModel::has_result)
            })
    }
    /// `RecordedList`: the entities with a recorded result (or skipped), in the model
    /// order.
    pub fn recorded_list(&self) -> Vec<Handle<StandardTransient>> {
        let model = self.model();
        let Some(m) = model.get() else {
            return Vec::new();
        };
        (1..=m.nb_entities())
            .filter(|i| self.result_at(*i).is_some_and(|r| r.is_some()))
            .filter_map(|i| m.value(i).ok())
            .collect()
    }
    /// `Skip(theEnt)`: `ent` marked as skipped; false without model or process.
    pub fn skip(&self, ent: &Handle<StandardTransient>) -> bool {
        if self.transient_process().is_null() {
            return false;
        }
        let num = self.number(ent);
        if num == 0 {
            return false;
        }
        self.state.write().results.insert(num, ent.clone());
        true
    }
    /// `IsSkipped(theEnt)`: marked with something else than a final result.
    pub fn is_skipped(&self, ent: &Handle<StandardTransient>) -> Result<bool, Failure> {
        self.mark_is(ent, "IsSkipped", |r| {
            !r.is_instance(TransferResultFromModel::static_type())
        })
    }
    /// `IsMarked(theEnt)`: recorded or skipped.
    pub fn is_marked(&self, ent: &Handle<StandardTransient>) -> bool {
        let num = self.number(ent);
        num != 0 && self.result_at(num).is_some_and(|r| r.is_some())
    }
    /// `FinalResult(theEnt)`: the final result recorded (null otherwise).
    pub fn final_result(&self, ent: &Handle<StandardTransient>) -> Handle<TransferResultFromModel> {
        let num = self.number(ent);
        if num == 0 {
            return Handle::default();
        }
        self.result_at(num).map(|r| r.cast()).unwrap_or_default()
    }
    /// `FinalEntityLabel(theEnt)`: the label of the main entity of the final result.
    pub fn final_entity_label(&self, ent: &Handle<StandardTransient>) -> AsciiString {
        self.final_result(ent)
            .get()
            .map(TransferResultFromModel::main_label)
            .unwrap_or_default()
    }
    /// `FinalEntityNumber(theEnt)`: the number of the main entity of the final result.
    pub fn final_entity_number(&self, ent: &Handle<StandardTransient>) -> i32 {
        self.final_result(ent)
            .get()
            .map_or(0, TransferResultFromModel::main_number)
    }
    /// `ResultFromNumber(theNum)`: the final result of the entity `num` (null otherwise);
    /// `Standard_NullObject` without model (a native null dereference).
    pub fn result_from_number(&self, num: i32) -> Result<Handle<TransferResultFromModel>, Failure> {
        let model = self.model();
        let m = model.get().ok_or_else(|| {
            exceptions::null_object("XSControl_TransferReader::ResultFromNumber: no model")
        })?;
        if num < 1 || num > m.nb_entities() {
            return Ok(Handle::default());
        }
        Ok(self.result_at(num).map(|r| r.cast()).unwrap_or_default())
    }
}
