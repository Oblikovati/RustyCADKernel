// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_WorkSession (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the sending and writing.

//! The sending of the split, of the whole model and of a selection, and the writing of
//! files and streams. Native defect kept (see `guarded_sticky`): a successful guarded
//! sending leaves the errors unhandled for the next calls.
use super::item_lists::h_text;
use super::params::info;
use super::{ERRHAND, IFSelectWorkSession, interrupted, report_interruption};
use crate::if_graph::{IFGraphSubPartsEvaluate, IFGraphSubPartsIterator};
use crate::if_select::{
    IFSelectDispatch, IFSelectReturnStatus, IFSelectSelectPointed, IFSelectSelection,
};
use crate::interface::{InterfaceCheckIterator, InterfaceGraph};
use rustycad_tkernel::{
    standard::{Failure, Handle, exceptions},
    tcollection::AsciiString,
};
use std::fmt::Write as _;
use std::io::Write;
use std::sync::atomic::Ordering;

/// The checks of a fail `text`.
fn fail_checks(text: &str) -> InterfaceCheckIterator {
    let mut checks = InterfaceCheckIterator::new();
    checks.c_check(0).add_fail_text(text, "");
    checks
}

/// The status of a sending by its checks.
fn sent_status(checks: &InterfaceCheckIterator) -> IFSelectReturnStatus {
    if checks.is_empty(true) {
        IFSelectReturnStatus::RetDone
    } else {
        IFSelectReturnStatus::RetError
    }
}

/// The output of a whole sending.
enum Output<'s> {
    File,
    Stream(&'s mut dyn Write),
}

impl IFSelectWorkSession {
    /// Records the last run checks.
    fn set_check_run(&self, checks: InterfaceCheckIterator) {
        self.state.write().thecheckrun = checks;
    }
    /// The current graph, `Standard_NullObject` when none.
    fn live_graph(&self) -> Result<Handle<crate::interface::InterfaceHGraph>, Failure> {
        self.current_graph()
            .ok_or_else(|| exceptions::null_object("IFSelect_WorkSession: no graph"))
    }
    /// `SendSplit`: the files already copied are written, else each packet of each
    /// dispatch is sent as a selection; false when not available.
    pub fn send_split(&self) -> Result<bool, Failure> {
        match self.guarded_sticky(&interrupted("SendSplit"), || self.send_split_now())? {
            Some(sent) => Ok(sent),
            None => {
                self.set_check_run(fail_checks("Exception Raised -> Abandon"));
                Ok(false)
            }
        }
    }
    /// The body of `SendSplit`.
    fn send_split_now(&self) -> Result<bool, Failure> {
        let (lib, protocol, copier, sho) = {
            let st = self.state.read();
            (
                st.thelibrary.clone(),
                st.theprotocol.clone(),
                st.thecopier.clone(),
                st.theshareout.clone(),
            )
        };
        if lib.is_null() {
            self.set_check_run(fail_checks("WorkLibrary undefined"));
            return Ok(false);
        }
        if !self.is_loaded() {
            info(" ***  Data for SendSplit not available  ***\n");
            self.set_check_run(fail_checks("Data not available"));
            return Ok(false);
        }
        let mut checks = InterfaceCheckIterator::new();
        if self.nb_files() > 0 {
            let c = copier.get().ok_or_else(|| {
                exceptions::null_object("IFSelect_WorkSession::SendSplit: no copier")
            })?;
            checks = c.send_copied(&lib, &protocol)?;
        } else {
            let Some(live) = sho.get() else {
                return Ok(false);
            };
            let mut sout = String::from(" SendSplit .. ");
            let mut nf = 0;
            for i in 1..=live.nb_dispatches() {
                self.send_dispatch_packets(&live.dispatch(i)?, &mut nf)?;
            }
            let _ = writeln!(sout, " .. Files Written : {nf}");
            info(&sout);
        }
        self.set_check_run(checks);
        Ok(true)
    }
    /// Each packet of a dispatch sent as a pointed selection, the files counted in `nf`.
    fn send_dispatch_packets(
        &self,
        disp: &Handle<dyn IFSelectDispatch>,
        nf: &mut i32,
    ) -> Result<(), Failure> {
        let Some(live) = disp.get() else {
            return Ok(());
        };
        let hgraph = self.live_graph()?;
        let mut lists = Vec::new();
        {
            let graph = hgraph.graph();
            let mut packs = IFGraphSubPartsIterator::new(&graph, false)?;
            live.packets(&graph, &mut packs)?;
            packs.start()?;
            while packs.more()? {
                lists.push(packs.entities()?);
                packs.next();
            }
        }
        for iter in lists.into_iter().filter(|it| it.nb_entities() > 0) {
            let sp = IFSelectSelectPointed::new();
            sp.set_list(Some(&iter.content()));
            *nf += 1;
            let filnam = self.split_file_name(disp, *nf)?;
            let stat = self.send_selected(&filnam, &Handle::new(sp).cast(), false)?;
            if stat != IFSelectReturnStatus::RetDone {
                println!("File {filnam} failed");
            }
        }
        Ok(())
    }
    /// The file of the packet `nf` of `disp`: prefix, root of the dispatch, "_nf",
    /// extension.
    fn split_file_name(
        &self,
        disp: &Handle<dyn IFSelectDispatch>,
        nf: i32,
    ) -> Result<String, Failure> {
        let root = self.file_root(disp)?;
        Ok(format!(
            "{}{}_{nf}{}",
            h_text(&self.file_prefix()?),
            h_text(&root),
            h_text(&self.file_extension()?)
        ))
    }
    /// The checks of a whole sending (a guarded call computing the graph first).
    fn send_whole(
        &self,
        name: &str,
        output: Output<'_>,
        computegraph: bool,
    ) -> Result<Option<InterfaceCheckIterator>, Failure> {
        let (lib, protocol, copier) = {
            let st = self.state.read();
            (
                st.thelibrary.clone(),
                st.theprotocol.clone(),
                st.thecopier.clone(),
            )
        };
        let send = |output: Output<'_>| -> Result<InterfaceCheckIterator, Failure> {
            let hgraph = self.live_graph()?;
            let c = copier.get().ok_or_else(|| {
                exceptions::null_object("IFSelect_WorkSession::SendAll: no copier")
            })?;
            let graph: std::sync::RwLockReadGuard<'_, InterfaceGraph> = hgraph.graph();
            match output {
                Output::File => c.send_all(name, &graph, &lib, &protocol),
                Output::Stream(s) => c.send_all_stream(s, name, &graph, &lib, &protocol),
            }
        };
        if !ERRHAND.swap(false, Ordering::Relaxed) {
            return send(output).map(Some);
        }
        let result = self.compute_graph(computegraph).and_then(|_| send(output));
        match result {
            Ok(checks) => Ok(Some(checks)),
            Err(failure) => {
                report_interruption(&interrupted("SendAll"), &failure);
                ERRHAND.store(self.error_handle(), Ordering::Relaxed);
                self.set_check_run(fail_checks("Exception Raised -> Abandon"));
                Ok(None)
            }
        }
    }
    /// `SendAll(filename, computegraph = false)`: the whole model written as `filename`.
    /// Native quirks kept: a main fail stops without recording the checks, and a
    /// successful guarded sending leaves the errors unhandled.
    pub fn send_all(
        &self,
        filename: &str,
        computegraph: bool,
    ) -> Result<IFSelectReturnStatus, Failure> {
        if !self.is_loaded() {
            return Ok(IFSelectReturnStatus::RetVoid);
        }
        if self.work_library().is_null() {
            self.set_check_run(fail_checks("WorkLibrary undefined"));
            return Ok(IFSelectReturnStatus::RetError);
        }
        let Some(checks) = self.send_whole(filename, Output::File, computegraph)? else {
            return Ok(IFSelectReturnStatus::RetFail);
        };
        if checks.check(0).has_failed() {
            return Ok(IFSelectReturnStatus::RetStop);
        }
        self.keep_loaded_name(filename);
        let status = sent_status(&checks);
        self.set_check_run(checks);
        Ok(status)
    }
    /// The loaded file named `name` when it had no name.
    fn keep_loaded_name(&self, name: &str) {
        let mut st = self.state.write();
        if st.theloaded.is_empty() {
            st.theloaded = AsciiString::new(name);
        }
    }
    /// `SendAll(theOStream, theName = "", computegraph = false)`: the whole model written
    /// to a stream; a main fail is a failure.
    pub fn send_all_stream(
        &self,
        ostream: &mut dyn Write,
        name: &str,
        computegraph: bool,
    ) -> Result<IFSelectReturnStatus, Failure> {
        if !self.is_loaded() {
            return Ok(IFSelectReturnStatus::RetVoid);
        }
        if self.work_library().is_null() {
            self.set_check_run(fail_checks("WorkLibrary undefined"));
            return Ok(IFSelectReturnStatus::RetError);
        }
        let enabled = ERRHAND.load(Ordering::Relaxed);
        let Some(checks) = self.send_whole(name, Output::Stream(ostream), computegraph)? else {
            return Ok(IFSelectReturnStatus::RetFail);
        };
        if enabled {
            ERRHAND.store(self.error_handle(), Ordering::Relaxed);
        }
        let failed = checks.check(0).has_failed();
        let status = sent_status(&checks);
        self.set_check_run(checks);
        if failed {
            return Ok(IFSelectReturnStatus::RetFail);
        }
        if !name.is_empty() {
            self.keep_loaded_name(name);
        }
        Ok(status)
    }
    /// `SendSelected(filename, sel, computegraph = false)`: the result of `sel` (with
    /// what it shares) written as `filename`, counted in the remaining data.
    pub fn send_selected(
        &self,
        filename: &str,
        sel: &Handle<dyn IFSelectSelection>,
        computegraph: bool,
    ) -> Result<IFSelectReturnStatus, Failure> {
        if !self.is_loaded() {
            return Ok(IFSelectReturnStatus::RetVoid);
        }
        if self.work_library().is_null() {
            self.set_check_run(fail_checks("WorkLibrary undefined"));
            return Ok(IFSelectReturnStatus::RetVoid);
        }
        if !ERRHAND.load(Ordering::Relaxed) {
            return self.send_selected_now(filename, sel);
        }
        let sent = self.guarded_sticky(&interrupted("SendSelected"), || {
            self.compute_graph(computegraph)?;
            self.send_selected(filename, sel, false) // normal call
        })?;
        match sent {
            Some(status) => Ok(status),
            None => {
                self.set_check_run(fail_checks("Exception Raised -> Abandon"));
                Ok(IFSelectReturnStatus::RetFail)
            }
        }
    }
    /// The body of `SendSelected`.
    fn send_selected_now(
        &self,
        filename: &str,
        sel: &Handle<dyn IFSelectSelection>,
    ) -> Result<IFSelectReturnStatus, Failure> {
        let (lib, protocol, copier) = {
            let st = self.state.read();
            (
                st.thelibrary.clone(),
                st.theprotocol.clone(),
                st.thecopier.clone(),
            )
        };
        let hgraph = self.live_graph()?;
        let live = sel.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_WorkSession::SendSelected: null selection")
        })?;
        let iter = live.unique_result(&hgraph.graph())?;
        if iter.nb_entities() == 0 {
            return Ok(IFSelectReturnStatus::RetVoid);
        }
        let c = copier.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_WorkSession::SendSelected: no copier")
        })?;
        let checks = c.send_selected(filename, &hgraph.graph(), &lib, &protocol, &iter)?;
        c.set_remaining(&mut hgraph.c_graph())?;
        let status = sent_status(&checks);
        self.set_check_run(checks);
        Ok(status)
    }
    /// The graph recomputed for a writing; false when not loaded after it.
    fn ready_to_write(&self) -> Result<bool, Failure> {
        if self.work_library().is_null() {
            return Ok(false);
        }
        self.compute_graph(true)?;
        Ok(self.is_loaded())
    }
    /// `WriteFile(filename)`: the graph recomputed, then `SendAll`.
    pub fn write_file(&self, filename: &str) -> Result<IFSelectReturnStatus, Failure> {
        if !self.ready_to_write()? {
            return Ok(IFSelectReturnStatus::RetVoid);
        }
        self.send_all(filename, false)
    }
    /// `WriteStream(theOStream, theName = "")`: the graph recomputed, then `SendAll` to
    /// the stream.
    pub fn write_stream(
        &self,
        ostream: &mut dyn Write,
        name: &str,
    ) -> Result<IFSelectReturnStatus, Failure> {
        if !self.ready_to_write()? {
            return Ok(IFSelectReturnStatus::RetVoid);
        }
        self.send_all_stream(ostream, name, false)
    }
    /// `WriteFile(filename, sel)`: the graph recomputed, then `SendSelected`.
    pub fn write_file_selected(
        &self,
        filename: &str,
        sel: &Handle<dyn IFSelectSelection>,
    ) -> Result<IFSelectReturnStatus, Failure> {
        if sel.is_null() || !self.ready_to_write()? {
            return Ok(IFSelectReturnStatus::RetVoid);
        }
        self.send_selected(filename, sel, false)
    }
}
