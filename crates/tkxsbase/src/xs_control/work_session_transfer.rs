// Rust port of DataExchange/TKXSBase/XSControl/XSControl_WorkSession (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the transfers.

//! The transfers of [`XSControlWorkSession`]: the reader set up and its process, the
//! results, the reading of entities and roots, a new model and the writing of shapes.
use super::work_session::{global_lock, live_reader, live_writer};
use super::{XSControlTransferReader, XSControlWorkSession};
use crate::if_select::IFSelectReturnStatus;
use crate::interface::{InterfaceCheckIterator, InterfaceInterfaceModel};
use crate::transfer::{TransferBinder, TransferSimpleBinderOfTransient, TransferTransientProcess};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{
    message::ProgressRange,
    standard::{Failure, Handle, StandardTransient, exceptions},
};

/// The default size of a transient process created for the mode 2 of `InitTransferReader`.
const DEFAULT_PROCESS_SIZE: i32 = 10000;

impl XSControlWorkSession {
    /// `InitTransferReader(theMode)`: 0 and 5 clear the reader, every mode sets it up
    /// again; 1 clears it, 2 sets the recorded entities as roots, 3 records the roots,
    /// 4 and 5 begin a transfer.
    pub fn init_transfer_reader(&self, mode: i32) -> Result<(), Failure> {
        let reader = self.transfer_reader();
        if mode == 0 || mode == 5 {
            live_reader(&reader)?.clear(-1);
        }
        match reader.get() {
            Some(_) => self.set_transfer_reader(&reader)?,
            None => self.set_transfer_reader(&Handle::new(XSControlTransferReader::new()))?,
        }
        let reader = self.transfer_reader();
        let live = live_reader(&reader)?;
        match mode {
            1 => live.clear(-1),
            2 => self.roots_from_recorded(live)?,
            3 => record_roots(live)?,
            4 | 5 => {
                live.begin_transfer();
            }
            _ => {}
        }
        Ok(())
    }
    /// The mode 2 of `InitTransferReader`: the recorded entities set as roots of the
    /// process (created on the graph when none).
    fn roots_from_recorded(&self, reader: &XSControlTransferReader) -> Result<(), Failure> {
        let mut tp = reader.transient_process();
        if tp.is_null() {
            tp = TransferTransientProcess::new(DEFAULT_PROCESS_SIZE);
            reader.set_transient_process(&tp);
            if let Some(process) = tp.get() {
                process.set_graph(&self.h_graph()?);
            }
        }
        let process = tp.get().ok_or_else(|| {
            exceptions::null_object("XSControl_WorkSession::InitTransferReader: no process")
        })?;
        reader
            .recorded_list()
            .iter()
            .for_each(|ent| process.set_root(ent));
        Ok(())
    }
    /// `SetTransferReader(theTR)`: the reader of the session, set on the controller and
    /// the graph, with a new process (sized on the model, with error handling) when
    /// it has none.
    pub fn set_transfer_reader(&self, tr: &Handle<XSControlTransferReader>) -> Result<(), Failure> {
        self.state.write().transfer_reader = tr.clone();
        let Some(live) = tr.get() else {
            return Ok(());
        };
        live.set_controller(&self.norm_adaptor());
        let hgraph = self.h_graph()?;
        live.set_graph(&hgraph);
        if live.transient_process().is_some() {
            return Ok(());
        }
        let tp = TransferTransientProcess::new(self.process_size());
        if let Some(process) = tp.get() {
            process.set_graph(&hgraph);
            process.set_error_handle(true);
        }
        live.set_transient_process(&tp);
        Ok(())
    }
    /// `SetMapReader(theTP)`: the process (given the model of the session when it has
    /// none, and the graph) becomes the one of the reader, cleared and set up again;
    /// false for null or for the process of another model.
    pub fn set_map_reader(&self, tp: &Handle<TransferTransientProcess>) -> Result<bool, Failure> {
        let Some(process) = tp.get() else {
            return Ok(false);
        };
        if process.model().is_null() {
            process.set_model(&self.model());
        }
        process.set_graph(&self.h_graph()?);
        if process.model() != self.model() {
            return Ok(false);
        }
        let reader = self.transfer_reader();
        live_reader(&reader)?.clear(-1);
        self.set_transfer_reader(&reader)?;
        live_reader(&reader)?.set_transient_process(tp);
        Ok(true)
    }
    /// `Result(theEnt, theMode)`: the result of `ent`: units of `mode` 0 the final result,
    /// 1 the last one of the process, 2 the final one else the last one; tens 0 the
    /// result, 1 its binder, 2 the final result itself (mode 20).
    pub fn result(
        &self,
        ent: &Handle<StandardTransient>,
        mode: i32,
    ) -> Result<Handle<StandardTransient>, Failure> {
        let (ouca, kica) = (mode % 10, mode / 10);
        let reader = self.transfer_reader();
        let live = live_reader(&reader)?;
        let resu = if ouca == 1 {
            Handle::default()
        } else {
            live.final_result(ent)
        };
        if mode == 20 {
            return Ok(resu.cast());
        }
        let mut binder: Handle<dyn TransferBinder> = match resu.get() {
            Some(r) => r
                .main_result()
                .get()
                .ok_or_else(|| {
                    exceptions::null_object(
                        "XSControl_WorkSession::Result: final result without main result",
                    )
                })?
                .binder(),
            None => Handle::default(),
        };
        if binder.is_null() && ouca > 0 {
            let tp = live.transient_process();
            binder = tp
                .get()
                .ok_or_else(|| {
                    exceptions::null_object("XSControl_WorkSession::Result: no transient process")
                })?
                .find(ent);
        }
        if kica == 1 {
            return Ok(binder.cast());
        }
        Ok(
            match binder.cast::<TransferSimpleBinderOfTransient>().get() {
                Some(trb) => trb.result(),
                None => binder.cast(),
            },
        )
    }
    /// `TransferReadOne(theEnt, theProgress)`: the roots for the model itself, else the
    /// list `ent` gives (one entity or a list); `Standard_NullObject` for a null `ent`
    /// (natively a null list dereference).
    pub fn transfer_read_one(
        &self,
        ent: &Handle<StandardTransient>,
        progress: &ProgressRange<'_>,
    ) -> Result<i32, Failure> {
        if ent.object_address() == self.model().object_address() {
            return self.transfer_read_roots(progress);
        }
        let list = self.give_list(ent)?.ok_or_else(|| {
            exceptions::null_object(
                "XSControl_WorkSession::TransferReadOne: no list for a null entity",
            )
        })?;
        let reader = self.transfer_reader();
        let live = live_reader(&reader)?;
        if list.len() == 1 {
            return live.transfer_one(&list[0], true, progress);
        }
        live.transfer_list(&list, true, progress)
    }
    /// `TransferReadRoots(theProgress)`: the roots of the graph transferred by the reader.
    pub fn transfer_read_roots(&self, progress: &ProgressRange<'_>) -> Result<i32, Failure> {
        let hgraph = self.graph()?;
        let graph = hgraph.get().ok_or_else(|| {
            exceptions::null_object("XSControl_WorkSession::TransferReadRoots: no graph")
        })?;
        live_reader(&self.transfer_reader())?.transfer_roots(&graph.graph(), progress)
    }
    /// `NewModel`: a new model of the controller set in the session, the reader process
    /// and results and the writer cleared; null without controller.
    pub fn new_model(&self) -> Result<Handle<dyn InterfaceInterfaceModel>, Failure> {
        let _lock = global_lock();
        let ctl = self.norm_adaptor();
        let Some(live) = ctl.get() else {
            return Ok(Handle::default());
        };
        let newmod = live.new_model();
        self.set_model(&newmod, true)?;
        let reader = self.transfer_reader();
        let reader_live = live_reader(&reader)?;
        if let Some(tp) = reader_live.transient_process().get() {
            tp.clear();
        }
        reader_live.clear(3);
        live_writer(&self.transfer_writer())?.clear(-1);
        Ok(newmod)
    }
    /// `TransferWriteShape(theShape, theCompGraph = true, theProgress)`: the shape
    /// written to the model of the session by the writer (the graph then computed with
    /// `compgraph`); error without controller, void without model or for a null shape,
    /// stop on a user break.
    pub fn transfer_write_shape(
        &self,
        shape: &Shape,
        compgraph: bool,
        progress: &ProgressRange<'_>,
    ) -> Result<IFSelectReturnStatus, Failure> {
        let _lock = global_lock();
        if self.norm_adaptor().is_null() {
            return Ok(IFSelectReturnStatus::RetError);
        }
        let model = self.model();
        if model.is_null() || shape.is_null() {
            return Ok(IFSelectReturnStatus::RetVoid);
        }
        let status =
            live_writer(&self.transfer_writer())?.transfer_write_shape(&model, shape, progress)?;
        if progress.user_break() {
            return Ok(IFSelectReturnStatus::RetStop);
        }
        if compgraph {
            self.compute_graph(true)?;
        }
        Ok(status)
    }
    /// `TransferWriteCheckList`: the checks of the writing, on the model of the session.
    pub fn transfer_write_check_list(&self) -> Result<InterfaceCheckIterator, Failure> {
        live_writer(&self.transfer_writer())?.result_check_list(&self.model())
    }
}

/// The mode 3 of `InitTransferReader`: the roots of the process recorded as results
/// (nothing without process).
fn record_roots(reader: &XSControlTransferReader) -> Result<(), Failure> {
    let tp = reader.transient_process();
    let Some(process) = tp.get() else {
        return Ok(());
    };
    for i in 1..=process.nb_roots() {
        reader.record_result(&process.root(i)?)?;
    }
    Ok(())
}
