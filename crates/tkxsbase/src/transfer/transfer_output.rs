// Rust port of DataExchange/TKXSBase/Transfer/Transfer_TransferOutput (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Transfer_TransferOutput`: transfers the entities (or the roots) of a model with a
//! transient process, then lists the entities by transfer status.
use super::{
    TransferActorOfTransientProcessOps, TransferProcessKind, TransferTransientKind,
    TransferTransientProcess, transfer_failure,
};
use crate::interface::{
    InterfaceEntityIterator, InterfaceGraph, InterfaceInterfaceModel, InterfaceModelEntities,
    InterfaceProtocol, InterfaceProtocolBase, InterfaceShareFlags,
};
use rustycad_tkernel::{
    message::{ProgressRange, ProgressScope},
    standard::{Failure, Handle, RuntimeTyped, StandardTransient, exceptions},
};

/// `Transfer_TransferOutput`.
///
/// ```
/// use rustycad_tkxsbase::transfer::{TransferOutput, TransferTransientProcess};
/// use rustycad_tkernel::standard::Handle;
/// let output = TransferOutput::with_process(&TransferTransientProcess::new(10), &Handle::default());
/// assert!(output.model().is_null() && output.list_for_status(true, true).unwrap().nb_entities() == 0);
/// ```
#[derive(Clone)]
pub struct TransferOutput {
    theproc: Handle<TransferTransientProcess>,
    themodel: Handle<dyn InterfaceInterfaceModel>,
}

impl TransferOutput {
    /// `Transfer_TransferOutput(actor, amodel)`: a new transient process sized by the
    /// model, with `actor`; `Standard_NullObject` for a null model (a native null
    /// dereference).
    pub fn with_actor<A>(
        actor: &Handle<A>,
        amodel: &Handle<dyn InterfaceInterfaceModel>,
    ) -> Result<Self, Failure>
    where
        A: TransferActorOfTransientProcessOps + RuntimeTyped + ?Sized,
    {
        let model = amodel.get().ok_or_else(|| {
            exceptions::null_object("Transfer_TransferOutput: null model; expected a model")
        })?;
        let theproc = TransferTransientProcess::new(model.nb_entities());
        theproc.set_actor(&actor.cast::<<TransferTransientKind as TransferProcessKind>::Actor>());
        Ok(Self {
            theproc,
            themodel: amodel.clone(),
        })
    }
    /// `Transfer_TransferOutput(proc, amodel)`.
    pub fn with_process(
        proc: &Handle<TransferTransientProcess>,
        amodel: &Handle<dyn InterfaceInterfaceModel>,
    ) -> Self {
        Self {
            theproc: proc.clone(),
            themodel: amodel.clone(),
        }
    }
    /// `Model`.
    pub fn model(&self) -> Handle<dyn InterfaceInterfaceModel> {
        self.themodel.clone()
    }
    /// `TransientProcess`.
    pub fn transient_process(&self) -> Handle<TransferTransientProcess> {
        self.theproc.clone()
    }
    /// The process, live (a null one is natively dereferenced).
    fn process(&self) -> Result<&TransferTransientProcess, Failure> {
        self.theproc.get().ok_or_else(|| {
            exceptions::null_object("Transfer_TransferOutput: null transient process; expected one")
        })
    }
    /// `Transfer(obj, theProgress)`: `Transfer_TransferFailure` when `obj` is not in the
    /// model.
    pub fn transfer(
        &self,
        obj: &Handle<StandardTransient>,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let model = self.themodel.get().ok_or_else(|| {
            exceptions::null_object(
                "Transfer_TransferOutput::Transfer: null model; expected a model",
            )
        })?;
        if model.number(obj) == 0 {
            return Err(transfer_failure(
                "TransferOutput : Transfer, entities do not come from same initial model",
            ));
        }
        self.process()?.transfer(obj, progress)?;
        Ok(())
    }
    /// `TransferRoots(theProgress)`: with the active protocol.
    pub fn transfer_roots(&self, progress: &ProgressRange<'_>) -> Result<(), Failure> {
        self.transfer_roots_protocol(&InterfaceProtocolBase::active(), progress)
    }
    /// `TransferRoots(protocol, theProgress)`: transfers the root entities of the model,
    /// recording each transferred one as a root.
    pub fn transfer_roots_protocol(
        &self,
        protocol: &Handle<dyn InterfaceProtocol>,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let process = self.process()?;
        process.set_root_management(false);
        let tool = InterfaceShareFlags::with_protocol(&self.themodel, protocol)?;
        self.transfer_list(&tool.root_entities(), progress)
    }
    /// `TransferRoots(G, theProgress)`: the roots of the graph, whose model the process
    /// takes.
    pub fn transfer_roots_graph(
        &self,
        graph: &InterfaceGraph,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let process = self.process()?;
        process.set_root_management(false);
        let tool = InterfaceShareFlags::from_graph(graph)?;
        process.set_model(graph.model());
        self.transfer_list(&tool.root_entities(), progress)
    }
    /// Transfers each entity of `list`, recording it as a root when transferred.
    fn transfer_list(
        &self,
        list: &InterfaceEntityIterator,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let process = self.process()?;
        let scope = ProgressScope::unnamed(progress, f64::from(list.nb_entities()), false);
        for ent in list.content() {
            if !scope.more() {
                break;
            }
            if process.transfer(&ent, &scope.next(1.0))? {
                process.set_root(&ent);
            }
        }
        Ok(())
    }
    /// `ListForStatus(normal, roots = true)`: the roots (or all the mapped entities) whose
    /// status is normal (initial or done), or abnormal.
    pub fn list_for_status(
        &self,
        normal: bool,
        roots: bool,
    ) -> Result<InterfaceEntityIterator, Failure> {
        let process = self.process()?;
        let mut list = InterfaceEntityIterator::new();
        let max = if roots {
            process.nb_roots()
        } else {
            process.nb_mapped()
        };
        for i in 1..=max {
            let binder = if roots {
                process.root_item(i)?
            } else {
                process.map_item(i)?
            };
            let Some(live) = binder.get() else { continue };
            let ok = matches!(
                live.status_exec(),
                super::TransferStatusExec::StatusInitial | super::TransferStatusExec::StatusDone
            );
            if ok == normal {
                list.add_item(&if roots {
                    process.root(i)?
                } else {
                    process.mapped(i)?
                });
            }
        }
        Ok(list)
    }
    /// `ModelForStatus(protocol, normal, roots = true)`: a new empty model of the same kind
    /// with the entities of `ListForStatus` and their references (null without model).
    pub fn model_for_status(
        &self,
        protocol: &Handle<dyn InterfaceProtocol>,
        normal: bool,
        roots: bool,
    ) -> Result<Handle<dyn InterfaceInterfaceModel>, Failure> {
        let Some(model) = self.themodel.get() else {
            return Ok(Handle::default());
        };
        let newmod = model.new_empty_model();
        for ent in self.list_for_status(normal, roots)?.content() {
            newmod.add_with_refs_protocol(&ent, protocol, 0, false)?;
        }
        Ok(newmod)
    }
}
