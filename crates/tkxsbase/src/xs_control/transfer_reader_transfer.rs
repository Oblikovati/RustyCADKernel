// Rust port of DataExchange/TKXSBase/XSControl/XSControl_TransferReader (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the transfers.

//! The transfers of [`XSControlTransferReader`]: one entity, a list, the roots of a
//! graph, with the final results recorded, and the clearing of a transfer.
use super::XSControlTransferReader;
use crate::if_select::{IFSelectSignatureList, IFSelectSignatureListOps};
use crate::interface::{
    InterfaceGraph, InterfaceInterfaceModel, InterfaceModelEntities, InterfaceMsg,
};
use crate::transfer::{TransferOutput, TransferTransientProcess};
use crate::transfer_brep::TransferBRep;
use rustycad_tkernel::{
    message::{ProgressRange, ProgressScope},
    standard::{Failure, Handle, StandardTransient},
};
use std::fmt::Write as _;

/// The frame line of the transfer traces.
const FRAME: &str = "\n*******************************************************************\n";

/// The trace of `TransferOne` (trace level above 1): the number, label and type of the
/// entity.
fn one_entity_trace(
    model: &dyn InterfaceInterfaceModel,
    ent: &Handle<StandardTransient>,
) -> String {
    let num = model.number(ent);
    let lab = model.string_label(ent);
    let mut out =
        format!("{FRAME}******           Transferring one Entity                     ******\n");
    if let Some(l) = lab.get() {
        let text = String::from_utf8_lossy(l.string().as_bytes()).into_owned();
        let _ = writeln!(
            out,
            "******    N0 in file : {}{num}      Ident : {text}{}******",
            InterfaceMsg::blanks_for_integer(num, 5),
            InterfaceMsg::blanks(14 - l.length())
        );
    }
    let tname = model.type_name(ent, false);
    let _ = write!(
        out,
        "******    Type : {tname}{}******{FRAME}",
        InterfaceMsg::blanks(44 - tname.len() as i32)
    );
    out
}

/// The trace of a list (or the roots) to transfer: its count per type.
fn list_trace(
    model: &dyn InterfaceInterfaceModel,
    ents: &[Handle<StandardTransient>],
    title: &str,
) -> String {
    let mut out = format!("{FRAME}{title}\n{FRAME}");
    let sl = IFSelectSignatureList::new(false);
    for ent in ents {
        sl.add(ent, &model.type_name(ent, false));
    }
    sl.set_name("Entities to Transfer");
    let mut counts = Vec::new();
    let _ = sl.print_count(&mut counts);
    out.push_str(&String::from_utf8_lossy(&counts));
    out.push_str(FRAME);
    out
}

impl XSControlTransferReader {
    /// `BeginTransfer`: the process (created for the model when none) gets the actor,
    /// the error handling and the context; false without model or actor.
    pub fn begin_transfer(&self) -> bool {
        let model = self.model();
        let Some(m) = model.get() else {
            return false;
        };
        if self.actor().is_null() {
            return false;
        }
        let tp = {
            let mut st = self.state.write();
            st.shape_result = None;
            if st.tp.is_null() {
                st.tp = TransferTransientProcess::new(m.nb_entities());
            }
            st.tp.clone()
        };
        let Some(process) = tp.get() else {
            return false;
        };
        process.set_actor(&Handle::default()); // -> Reset
        process.set_actor(&self.actor().cast()); // Set properly
        process.set_error_handle(true);
        let context = self.context().clone();
        *process.context() = context;
        true
    }
    /// `Recognize(theEnt)`: the actor recognizes `ent` (false without actor).
    pub fn recognize(&self, ent: &Handle<StandardTransient>) -> bool {
        let actor = self.state.read().actor.clone();
        actor.get().is_some_and(|a| a.recognize(ent))
    }
    /// The output of a transfer on the process (begun when needed), set on the graph or
    /// the model; `None` without actor or model, or when the transfer does not begin.
    fn prepare_output(&self) -> Option<(Handle<TransferTransientProcess>, TransferOutput)> {
        let (actor, model) = {
            let st = self.state.read();
            (st.actor.clone(), st.model.clone())
        };
        if actor.is_null() || model.is_null() {
            return None;
        }
        if self.transient_process().is_null() && !self.begin_transfer() {
            return None;
        }
        let (tp, graph) = {
            let st = self.state.read();
            (st.tp.clone(), st.graph.clone())
        };
        let process = tp.get()?;
        let output = TransferOutput::with_process(&tp, &model);
        if graph.is_null() {
            process.set_model(&model);
        } else {
            process.set_graph(&graph);
        }
        Some((tp, output))
    }
    /// The root of a transfer recorded: 1 when it gives a result, else 0.
    fn record_root(
        &self,
        process: &TransferTransientProcess,
        obj: &Handle<StandardTransient>,
        rec: bool,
    ) -> Result<i32, Failure> {
        process.set_root(obj);
        let binder = process.find(obj);
        let Some(live) = binder.get() else {
            return Ok(0);
        };
        if rec {
            self.record_result(obj)?;
        }
        Ok(i32::from(live.has_result()))
    }
    /// `TransferOne(theEnt, theRec = true, theProgress)`: `ent` transferred (traced from
    /// the trace level 2), recorded as a root and, with `rec`, its final result; 1 when it
    /// gives a result. 0 without actor or model.
    pub fn transfer_one(
        &self,
        ent: &Handle<StandardTransient>,
        rec: bool,
        progress: &ProgressRange<'_>,
    ) -> Result<i32, Failure> {
        let Some((tp, output)) = self.prepare_output() else {
            return Ok(0);
        };
        let process = tp.get().ok_or_else(|| {
            rustycad_tkernel::standard::exceptions::null_object(
                "XSControl_TransferReader::TransferOne",
            )
        })?;
        let messenger = process.messenger();
        let mut sout = messenger.send_info();
        if process.trace_level() > 1 {
            let model = self.model();
            if let Some(m) = model.get() {
                let _ = sout.write_str(&one_entity_trace(m, ent));
            }
        }
        output.transfer(ent, progress)?;
        if progress.user_break() {
            return Ok(0);
        }
        self.record_root(process, ent, rec)
    }
    /// `TransferList(theList, theRec = true, theProgress)`: each entity transferred (the
    /// list traced from the trace level 1), recorded as a root and, with `rec`, its
    /// final result; the count of entities with a result.
    pub fn transfer_list(
        &self,
        list: &[Handle<StandardTransient>],
        rec: bool,
        progress: &ProgressRange<'_>,
    ) -> Result<i32, Failure> {
        let Some((tp, output)) = self.prepare_output() else {
            return Ok(0);
        };
        let process = tp.get().ok_or_else(|| {
            rustycad_tkernel::standard::exceptions::null_object(
                "XSControl_TransferReader::TransferList",
            )
        })?;
        let nb = list.len() as i32;
        if process.trace_level() > 0 {
            let model = self.model();
            if let Some(m) = model.get() {
                let title = format!(
                    "******           Transferring a list of {} Entities       ******",
                    InterfaceMsg::blanks_for_integer(nb, 5)
                );
                let _ = process
                    .messenger()
                    .send_info()
                    .write_str(&list_trace(m, list, &title));
            }
        }
        let scope = ProgressScope::unnamed(progress, f64::from(nb), false);
        let mut res = 0;
        for obj in list {
            if !scope.more() {
                break;
            }
            output.transfer(obj, &scope.next(1.0))?;
            res += self.record_root(process, obj, rec)?;
        }
        Ok(res)
    }
    /// `TransferRoots(theGraph, theProgress)`: the roots of the graph transferred (traced
    /// from the trace level 1), the results recorded; the count of shapes of the roots
    /// (-1 for another model, no transfer, or a user break).
    pub fn transfer_roots(
        &self,
        g: &InterfaceGraph,
        progress: &ProgressRange<'_>,
    ) -> Result<i32, Failure> {
        let model = self.model();
        if model.object_address() != g.model().object_address() || !self.begin_transfer() {
            return Ok(-1);
        }
        let (tp, graph) = {
            let st = self.state.read();
            (st.tp.clone(), st.graph.clone())
        };
        let process = tp.get().ok_or_else(|| {
            rustycad_tkernel::standard::exceptions::null_object(
                "XSControl_TransferReader::TransferRoots",
            )
        })?;
        let output = TransferOutput::with_process(&tp, &model);
        if graph.is_null() {
            process.set_model(&model);
        } else {
            process.set_graph(&graph);
        }
        if process.trace_level() > 0 {
            let roots = g.root_entities()?.content();
            let title = format!(
                "******           Transferring the {} Root Entities        ******",
                InterfaceMsg::blanks_for_integer(roots.len() as i32, 5)
            );
            if let Some(m) = model.get() {
                let _ = process
                    .messenger()
                    .send_info()
                    .write_str(&list_trace(m, &roots, &title));
            }
        }
        output.transfer_roots_graph(g, progress)?;
        if progress.user_break() {
            return Ok(-1);
        }
        for i in 1..=process.nb_mapped() {
            let ent = process.mapped(i)?;
            let bnd = process.map_item(i)?;
            if bnd.get().is_some_and(|b| b.has_result()) {
                self.record_result(&ent)?;
            }
        }
        let shapes = TransferBRep::shapes(&tp, true)?.unwrap_or_default();
        let count = shapes.len() as i32;
        self.state.write().shape_result = Some(shapes);
        Ok(count)
    }
    /// `TransferClear(theEnt, theLevel = 0)`: the process cleared for the model, else the
    /// result of `ent` removed from the process and its mark cleared.
    pub fn transfer_clear(&self, ent: &Handle<StandardTransient>, level: i32) {
        let tp = self.transient_process();
        let Some(process) = tp.get() else {
            return;
        };
        if ent.object_address() == self.model().object_address() {
            process.clear();
            return;
        }
        process.remove_result(ent, level, true);
        self.clear_result(ent, -1);
    }
}
