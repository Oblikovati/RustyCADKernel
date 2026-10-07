// Rust port of DataExchange/TKXSBase/Transfer/Transfer_TransientProcess (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Transfer_TransientProcess`: the process that reads the entities of an interface
//! model, with its graph and a context of named objects.
use super::{
    TransferProcess, TransferProcessForTransient, TransferProcessOps, TransferTransientKind,
    bind_new,
};
use crate::interface::{
    InterfaceEntityIterator, InterfaceEntityListHandle, InterfaceGraph, InterfaceHGraph,
    InterfaceInterfaceModel, InterfaceModelEntities, InterfaceModelReports, InterfaceMsg,
    InterfaceSignTypeBase,
};
use rustycad_tkernel::{
    impl_transient,
    ncollection::DataMap,
    standard::{Failure, Handle, RuntimeType, SharedState, StandardTransient, exceptions},
    tcollection::AsciiString,
};
use std::io::{self, Write};
use std::ops::Deref;
use std::sync::{Arc, RwLockReadGuard, RwLockWriteGuard};

/// The private fields of `Transfer_TransientProcess`.
#[derive(Default)]
struct TransientProcessState {
    themodel: Handle<dyn InterfaceInterfaceModel>,
    thegraph: Handle<InterfaceHGraph>,
}

/// The map of named contexts.
type ContextMap = DataMap<AsciiString, Handle<StandardTransient>>;

/// `Transfer_TransientProcess`: embeds its `Transfer_ProcessForTransient` (registered as
/// its RTTI parent) and dereferences to it.
///
/// ```
/// use rustycad_tkxsbase::transfer::TransferTransientProcess;
/// let tp = TransferTransientProcess::new(100);
/// assert!(tp.model().is_null() && !tp.has_graph());
/// let mut stats = Vec::new();
/// tp.print_stats(1, &mut stats).unwrap();
/// assert!(String::from_utf8_lossy(&stats).contains("Model unknown"));
/// ```
pub struct TransferTransientProcess {
    base: TransferProcessForTransient,
    state: SharedState<TransientProcessState>,
    thectx: SharedState<ContextMap>,
    thetrroots: InterfaceEntityListHandle,
}
impl_transient!(TransferTransientProcess, "Transfer_TransientProcess", TransferProcessForTransient, base, interfaces: [dyn TransferProcessOps<TransferTransientKind>]);

impl Deref for TransferTransientProcess {
    type Target = TransferProcessForTransient;
    fn deref(&self) -> &TransferProcessForTransient {
        &self.base
    }
}

impl TransferTransientProcess {
    /// `Transfer_TransientProcess(nb = 10000)`.
    pub fn new(nb: i32) -> Handle<Self> {
        bind_new(Self {
            base: TransferProcessForTransient::unbound(TransferProcess::new(nb)),
            state: SharedState::default(),
            thectx: SharedState::default(),
            thetrroots: Arc::new(SharedState::new(Vec::new())),
        })
    }
    fn read(&self) -> RwLockReadGuard<'_, TransientProcessState> {
        self.state.read()
    }
    /// `SetModel(model)`.
    pub fn set_model(&self, model: &Handle<dyn InterfaceInterfaceModel>) {
        self.state.write().themodel = model.clone();
    }
    /// `Model`.
    pub fn model(&self) -> Handle<dyn InterfaceInterfaceModel> {
        self.read().themodel.clone()
    }
    /// `SetGraph(HG)`: also sets the model of the graph (null for a null graph).
    pub fn set_graph(&self, hg: &Handle<InterfaceHGraph>) {
        let model = hg
            .get()
            .map(|g| g.graph().model().clone())
            .unwrap_or_default();
        let mut st = self.state.write();
        st.thegraph = hg.clone();
        st.themodel = model;
    }
    /// `HasGraph`.
    pub fn has_graph(&self) -> bool {
        self.read().thegraph.is_some()
    }
    /// `HGraph`.
    pub fn h_graph(&self) -> Handle<InterfaceHGraph> {
        self.read().thegraph.clone()
    }
    /// `Graph`: the graph of the HGraph (a read guard); `Standard_NullObject` without
    /// one (a native null dereference).
    pub fn graph(&self) -> Result<GraphGuard, Failure> {
        let hgraph = self.h_graph();
        if hgraph.is_null() {
            return Err(exceptions::null_object(
                "Transfer_TransientProcess::Graph: no graph; expected SetGraph first",
            ));
        }
        Ok(GraphGuard { hgraph })
    }
    /// `SetContext(name, ctx)`.
    pub fn set_context(&self, name: &str, ctx: &Handle<StandardTransient>) {
        self.thectx
            .write()
            .bind(AsciiString::new(name), ctx.clone());
    }
    /// `GetContext(name, type, ctx)`: the context `name`, of kind `atype` when given.
    pub fn get_context(
        &self,
        name: &str,
        atype: Option<&RuntimeType>,
        ctx: &mut Handle<StandardTransient>,
    ) -> bool {
        let map = self.thectx.read();
        if map.is_empty() {
            return false;
        }
        *ctx = map
            .seek(&AsciiString::new(name))
            .cloned()
            .unwrap_or_default();
        if ctx.is_null() {
            return false;
        }
        let Some(atype) = atype else { return true };
        if !ctx.is_kind(atype) {
            ctx.nullify();
        }
        ctx.is_some()
    }
    /// `Context`: the map of contexts (the write guard; natively a modifiable reference).
    pub fn context(&self) -> RwLockWriteGuard<'_, ContextMap> {
        self.thectx.write()
    }
    /// `TypedSharings(start, type)`: the sharings of type `type` of `start` in the graph
    /// (empty without graph).
    pub fn typed_sharings(
        &self,
        start: &Handle<StandardTransient>,
        atype: &RuntimeType,
    ) -> Result<InterfaceEntityIterator, Failure> {
        let hgraph = self.h_graph();
        match hgraph.get() {
            Some(graph) => graph.graph().typed_sharings(start, atype),
            None => Ok(InterfaceEntityIterator::new()),
        }
    }
    /// `IsDataLoaded(ent)`: true out of the model; false for an unknown or redefined entity.
    pub fn is_data_loaded(&self, start: &Handle<StandardTransient>) -> bool {
        let model = self.model();
        let Some(live) = model.get() else { return true };
        let num = live.number(start);
        if num == 0 {
            return true;
        }
        !live.is_unknown_entity(num) && !live.is_redefined_content(num)
    }
    /// `IsDataFail(ent)`: an error entity, or a semantic check with fails.
    pub fn is_data_fail(&self, start: &Handle<StandardTransient>) -> bool {
        let model = self.model();
        let Some(live) = model.get() else {
            return false;
        };
        let num = live.number(start);
        if num == 0 {
            return false;
        }
        live.is_error_entity(num) || live.check(num, false).has_failed()
    }
    /// `PrintStats(mode, S)`: the count of entities, results, errors and warnings.
    pub fn print_stats(&self, _mode: i32, s: &mut dyn Write) -> io::Result<()> {
        s.write_all(b"\n*******************************************************************\n")?;
        s.write_all(b"********                 Basic Statistics                  ********\n")?;
        let model = self.model();
        match model.get() {
            None => s.write_all(b"****        Model unknown\n")?,
            Some(live) => writeln!(
                s,
                "****        Nb Entities         : {}",
                live.nb_entities()
            )?,
        }
        print_result_counts(self.process(), s, " ")?;
        s.write_all(b"*******************************************************************\n")
    }
    /// `RootsForTransfer`: the shared list of the roots to transfer.
    pub fn roots_for_transfer(&self) -> InterfaceEntityListHandle {
        self.thetrroots.clone()
    }
}

/// The graph of a transient process: a read view of its `Interface_HGraph`.
pub struct GraphGuard {
    hgraph: Handle<InterfaceHGraph>,
}

impl GraphGuard {
    /// The graph; `tp.graph()?.get().size()`.
    pub fn get(&self) -> RwLockReadGuard<'_, InterfaceGraph> {
        self.hgraph.graph()
    }
}

/// The result counts of `PrintStats`, common to the transient and finder processes
/// (`sep` is the native text between "Errors on :" and the blanks).
pub(super) fn print_result_counts<K: super::TransferProcessKind>(
    tp: &TransferProcess<K>,
    s: &mut dyn Write,
    sep: &str,
) -> io::Result<()> {
    let (mut nbr, mut nbe, mut nbw) = (0, 0, 0);
    let nbroots = tp.nb_roots();
    writeln!(s, "****        Nb Final Results    : {nbroots}")?;
    for i in 1..=tp.nb_mapped() {
        let binder = tp.map_item(i).unwrap_or_default();
        let Some(live) = binder.get() else { continue };
        let stat = live.status_exec();
        if !matches!(
            stat,
            super::TransferStatusExec::StatusInitial | super::TransferStatusExec::StatusDone
        ) {
            nbe += 1;
            continue;
        }
        nbw += i32::from(live.check().nb_warnings() > 0);
        nbr += i32::from(live.has_result());
    }
    if nbr > nbroots {
        writeln!(s, "****      ( Itermediate Results : {} )", nbr - nbroots)?;
    }
    if nbe > 0 {
        writeln!(
            s,
            "****                  Errors on :{sep}{}{nbe} Entities",
            InterfaceMsg::blanks_for_integer(nbe, 4)
        )?;
    }
    if nbw > 0 {
        writeln!(
            s,
            "****                Warnings on : {}{nbw} Entities",
            InterfaceMsg::blanks_for_integer(nbw, 4)
        )?;
    }
    Ok(())
}

impl TransferProcessOps<TransferTransientKind> for TransferTransientProcess {
    fn process(&self) -> &TransferProcess<TransferTransientKind> {
        &self.base
    }
    /// `PrintTrace(start, S)`: "Entity " and the model print of `start` (mode 1), else
    /// "Entity Type:" and its class name.
    fn print_trace(&self, start: &Handle<StandardTransient>, s: &mut dyn Write) -> io::Result<()> {
        let Some(kind) = start.dynamic_type() else {
            return Ok(());
        };
        let model = self.model();
        match model.get() {
            Some(live) => {
                s.write_all(b"Entity ")?;
                live.print(start, s, 1)
            }
            None => write!(
                s,
                "Entity Type:{}",
                InterfaceSignTypeBase::class_name(kind.name())
            ),
        }
    }
    /// `CheckNum(ent)`: the number of `ent` in the model (0 without model).
    fn check_num(&self, start: &Handle<StandardTransient>) -> i32 {
        self.model().get().map_or(0, |live| live.number(start))
    }
}
