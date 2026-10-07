// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_WorkSession (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the files of the split.

//! The file names of the split (prefix, roots, extension), the evaluation of the files,
//! the sent files and the remaining data.
use super::item_lists::h_text;
use super::params::info;
use super::{ERRHAND, IFSelectWorkSession, interrupted};
use crate::if_select::{
    IFSelectDispatch, IFSelectPacketList, IFSelectRemainMode, IFSelectSelectPointed,
    IFSelectShareOut, IFSelectShareOutResult,
};
use crate::interface::{
    InterfaceCheckIterator, InterfaceCopyTool, InterfaceEntityIterator, InterfaceInterfaceModel,
};
use rustycad_tkernel::{
    osd::{CadPath, SysType},
    standard::{Failure, Handle, exceptions},
    tcollection::{AsciiString, HAsciiString},
};
use std::sync::atomic::Ordering;

/// The share-out of a session, `Standard_NullObject` when null.
fn live(sho: &Handle<IFSelectShareOut>) -> Result<&IFSelectShareOut, Failure> {
    sho.get()
        .ok_or_else(|| exceptions::null_object("IFSelect_WorkSession: no share-out"))
}

/// A text handle, null for an empty text.
fn text_or_null(name: &str) -> Handle<HAsciiString> {
    if name.is_empty() {
        Handle::default()
    } else {
        Handle::new(HAsciiString::new(name))
    }
}

/// True when `byte` is among the first `nb` bytes of `text`.
fn has_byte(text: &[u8], nb: usize, byte: u8) -> bool {
    text.iter().take(nb).any(|b| *b == byte)
}

impl IFSelectWorkSession {
    /// `FilePrefix`.
    pub fn file_prefix(&self) -> Result<Handle<HAsciiString>, Failure> {
        Ok(live(&self.share_out())?.prefix())
    }
    /// `DefaultFileRoot`.
    pub fn default_file_root(&self) -> Result<Handle<HAsciiString>, Failure> {
        Ok(live(&self.share_out())?.default_root_name())
    }
    /// `FileExtension`.
    pub fn file_extension(&self) -> Result<Handle<HAsciiString>, Failure> {
        Ok(live(&self.share_out())?.extension())
    }
    /// `FileRoot(disp)`: the root name of a dispatch of the share-out.
    pub fn file_root(
        &self,
        disp: &Handle<dyn IFSelectDispatch>,
    ) -> Result<Handle<HAsciiString>, Failure> {
        let sho = self.share_out();
        let l = live(&sho)?;
        Ok(l.root_name(l.dispatch_rank(disp)))
    }
    /// `SetFilePrefix(name)`.
    pub fn set_file_prefix(&self, name: &str) -> Result<(), Failure> {
        live(&self.share_out())?.set_prefix(&Handle::new(HAsciiString::new(name)));
        Ok(())
    }
    /// `SetDefaultFileRoot(name)`: an empty name removes it; false when used by a dispatch.
    pub fn set_default_file_root(&self, name: &str) -> Result<bool, Failure> {
        live(&self.share_out())?.set_default_root_name(&text_or_null(name))
    }
    /// `SetFileExtension(name)`.
    pub fn set_file_extension(&self, name: &str) -> Result<(), Failure> {
        live(&self.share_out())?.set_extension(&Handle::new(HAsciiString::new(name)));
        Ok(())
    }
    /// `SetFileRoot(disp, namefile)`: the root name of a dispatch item of the share-out
    /// (an empty name removes it); false otherwise.
    pub fn set_file_root(
        &self,
        disp: &Handle<dyn IFSelectDispatch>,
        namefile: &str,
    ) -> Result<bool, Failure> {
        if self.item_ident(&disp.cast()) == 0 {
            return Ok(false);
        }
        let sho = self.share_out();
        let l = live(&sho)?;
        let nd = l.dispatch_rank(disp);
        if nd == 0 {
            return Ok(false);
        }
        l.set_root_name(nd, &text_or_null(namefile))
    }
    /// `GiveFileRoot(file)`: the name part of a file path (the file itself when not a
    /// valid path).
    pub fn give_file_root(&self, file: &str) -> AsciiString {
        let path = CadPath::new(file);
        if !CadPath::is_valid(file, SysType::Default) {
            return AsciiString::new(file); // tant pis ..
        }
        path.name()
    }
    /// `GiveFileComplete(file)`: the file with the extension (unless its first character
    /// is in the file) and the prefix (unless its last character is in the file).
    pub fn give_file_complete(&self, file: &str) -> Result<AsciiString, Failure> {
        let mut bufstr = file.as_bytes().to_vec();
        let nb = bufstr.len();
        let ext = h_text(&self.file_extension()?);
        let val0 = ext.bytes().next().unwrap_or(0);
        if !has_byte(&bufstr, nb, val0) {
            bufstr.extend_from_slice(ext.as_bytes());
        }
        let pre = h_text(&self.file_prefix()?);
        let val1 = pre.bytes().last().unwrap_or(0);
        if !has_byte(&bufstr, nb, val1) {
            bufstr.splice(0..0, pre.bytes());
        }
        Ok(AsciiString::new(&String::from_utf8_lossy(&bufstr)))
    }
    /// `ClearFile`: the files copied and the share-out results are cleared.
    pub fn clear_file(&self) -> Result<(), Failure> {
        let (copier, sho) = {
            let st = self.state.read();
            (st.thecopier.clone(), st.theshareout.clone())
        };
        if let Some(c) = copier.get() {
            c.clear_result();
        }
        live(&sho)?.clear_result(true);
        Ok(())
    }
    /// `EvaluateFile`: the share-out copied into file models (counted in the remaining
    /// data). Native quirk kept: when the errors are handled, the last run checks are
    /// then replaced by empty checks (a fail on error).
    pub fn evaluate_file(&self) -> Result<(), Failure> {
        if !self.is_loaded() {
            return Ok(());
        }
        let guarded_on = ERRHAND.load(Ordering::Relaxed);
        let result = self.guarded(&interrupted("EvaluateFile"), || self.evaluate_file_now())?;
        if guarded_on {
            let mut checks = InterfaceCheckIterator::new();
            if result.is_none() {
                checks
                    .c_check(0)
                    .add_fail_text("Exception Raised -> Abandon", "");
            }
            self.state.write().thecheckrun = checks;
        }
        Ok(())
    }
    /// The body of `EvaluateFile`.
    fn evaluate_file_now(&self) -> Result<(), Failure> {
        let (hgraph, copier, sho, lib, protocol, model) = {
            let st = self.state.read();
            (
                st.thegraph.clone(),
                st.thecopier.clone(),
                st.theshareout.clone(),
                st.thelibrary.clone(),
                st.theprotocol.clone(),
                st.my_model.clone(),
            )
        };
        let graph = hgraph.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_WorkSession::EvaluateFile: no graph")
        })?;
        let mut eval = IFSelectShareOutResult::with_share_out_graph(&sho, &graph.graph())?;
        let c = copier.get().ok_or_else(|| {
            exceptions::null_object("IFSelect_WorkSession::EvaluateFile: no copier")
        })?;
        let checks = c.copy(&mut eval, &lib, &protocol)?;
        print_checks(
            "  **    EvaluateFile has produced Check Messages :    **",
            &checks,
            &model,
        );
        c.set_remaining(&mut graph.c_graph())?;
        self.state.write().thecheckrun = checks;
        Ok(())
    }
    /// `NbFiles`.
    pub fn nb_files(&self) -> i32 {
        self.model_copier().get().map_or(0, |c| c.nb_files())
    }
    /// `FileModel(num)`: null out of range.
    pub fn file_model(&self, num: i32) -> Result<Handle<dyn InterfaceInterfaceModel>, Failure> {
        if num <= 0 || num > self.nb_files() {
            return Ok(Handle::default());
        }
        self.model_copier()
            .get()
            .map_or(Ok(Handle::default()), |c| c.file_model(num))
    }
    /// `FileName(num)`: empty out of range.
    pub fn file_name(&self, num: i32) -> Result<AsciiString, Failure> {
        if num <= 0 || num > self.nb_files() {
            return Ok(AsciiString::default());
        }
        self.model_copier()
            .get()
            .map_or(Ok(AsciiString::default()), |c| c.file_name(num))
    }
    /// `BeginSentFiles(record)`.
    pub fn begin_sent_files(&self, record: bool) {
        if let Some(c) = self.model_copier().get() {
            c.begin_sent_files(&self.share_out(), record);
        }
    }
    /// `SentFiles`: `None` when not recorded.
    pub fn sent_files(&self) -> Option<Vec<Handle<HAsciiString>>> {
        self.model_copier().get().and_then(|c| c.sent_files())
    }
    /// `EvalSplit`: the packets of the share-out (null when not loaded).
    pub fn eval_split(&self) -> Result<Option<IFSelectPacketList>, Failure> {
        let Some(hgraph) = self.current_graph().filter(|_| self.is_loaded()) else {
            return Ok(None);
        };
        let mut sho =
            IFSelectShareOutResult::with_share_out_graph(&self.share_out(), &hgraph.graph())?;
        sho.packets(true).map(Some)
    }
    /// `SentList(count = -1)`: the entities sent `count` times (any count for -1).
    pub fn sent_list(&self, count: i32) -> Result<InterfaceEntityIterator, Failure> {
        let mut iter = InterfaceEntityIterator::new();
        let Some(hgraph) = self.current_graph().filter(|_| self.is_loaded()) else {
            return Ok(iter);
        };
        let graph = hgraph.graph();
        for i in 1..=graph.size() {
            let stat = graph.status(i)?;
            if (stat > 0 && count < 0) || stat == count {
                iter.get_one_item(&graph.entity(i)?);
            }
        }
        Ok(iter)
    }
    /// `MaxSendingCount`: the highest count of sendings of an entity.
    pub fn max_sending_count(&self) -> Result<i32, Failure> {
        let Some(hgraph) = self.current_graph().filter(|_| self.is_loaded()) else {
            return Ok(0);
        };
        let graph = hgraph.graph();
        (1..=graph.size()).try_fold(0, |max, i| Ok(max.max(graph.status(i)?)))
    }
    /// `SetRemaining(mode)`: forgets the sendings, computes the remaining model, displays
    /// the remaining entities, or comes back to the model before the remaining one.
    pub fn set_remaining(&self, mode: IFSelectRemainMode) -> Result<bool, Failure> {
        if !self.is_loaded() {
            return Ok(false);
        }
        match mode {
            IFSelectRemainMode::RemainForget => self.forget_sendings(),
            IFSelectRemainMode::RemainCompute => self.compute_remaining(),
            IFSelectRemainMode::RemainDisplay => self.display_remaining(),
            IFSelectRemainMode::RemainUndo => {
                let old = std::mem::take(&mut self.state.write().theoldel);
                if old.is_null() {
                    return Ok(false);
                }
                self.set_model(&old, true)?;
                Ok(true)
            }
        }
    }
    /// `SetRemaining(IFSelect_RemainForget)`.
    fn forget_sendings(&self) -> Result<bool, Failure> {
        let hgraph = self.current_graph().ok_or_else(|| {
            exceptions::null_object("IFSelect_WorkSession::SetRemaining: no graph")
        })?;
        let mut graph = hgraph.c_graph();
        for i in 1..=graph.size() {
            graph.set_status(i, 0)?;
        }
        drop(graph);
        self.state.write().theoldel = Handle::default();
        Ok(true)
    }
    /// `SetRemaining(IFSelect_RemainCompute)`.
    fn compute_remaining(&self) -> Result<bool, Failure> {
        let (hgraph, copier, lib, protocol, model) = {
            let st = self.state.read();
            (
                st.thegraph.clone(),
                st.thecopier.clone(),
                st.thelibrary.clone(),
                st.theprotocol.clone(),
                st.my_model.clone(),
            )
        };
        let (graph, c) = match (hgraph.get(), copier.get()) {
            (Some(g), Some(c)) => (g, c),
            _ => {
                return Err(exceptions::null_object(
                    "IFSelect_WorkSession::SetRemaining: no graph or copier",
                ));
            }
        };
        let mut tc = InterfaceCopyTool::with_protocol(&model, &protocol)?;
        let newmod = c.copied_remaining(&graph.graph(), &lib, &mut tc)?;
        if newmod.is_null() {
            info(" No Remaining Data recorded\n");
            return Ok(false);
        }
        if newmod == model {
            info(" Remaining causes all original data to be kept\n");
            c.set_remaining(&mut graph.c_graph())?;
            return Ok(false);
        }
        self.state.write().theoldel = model;
        self.set_model(&newmod, false)?;
        for sp in self.items_of::<IFSelectSelectPointed>() {
            let live_sp = sp.get().ok_or_else(|| {
                exceptions::null_object(
                    "IFSelect_WorkSession::SetRemaining: pointed selection expected",
                )
            })?;
            live_sp.update_control(&tc.control())?;
        }
        Ok(true)
    }
    /// `SetRemaining(IFSelect_RemainDisplay)`.
    fn display_remaining(&self) -> Result<bool, Failure> {
        let hgraph = self.current_graph().ok_or_else(|| {
            exceptions::null_object("IFSelect_WorkSession::SetRemaining: no graph")
        })?;
        let ne = {
            let graph = hgraph.graph();
            (1..=graph.size()).try_fold(0, |n, i| {
                Ok::<i32, Failure>(n + i32::from(graph.status(i)? >= 0))
            })?
        };
        if ne == 0 {
            info(" - All entities are remaining, none yet sent\n");
            return Ok(true);
        }
        let iter = self.sent_list(0)?;
        if iter.nb_entities() == 0 {
            info(" - No recorded remaining entities\n");
            return Ok(true);
        }
        let mut out = Vec::new();
        out.extend_from_slice(b" --  Recorded Remaining (not yet sent) Entities  --\n");
        self.list_entities(&iter, 2, &mut out)?;
        let text = format!(
            "{} -- Maximum Sending Count (i.e. duplication in files) {}\n",
            String::from_utf8_lossy(&out),
            self.max_sending_count()?
        );
        info(&text);
        Ok(true)
    }
}

/// The checks of an operation printed under `title` when not empty.
pub(crate) fn print_checks(
    title: &str,
    checks: &InterfaceCheckIterator,
    model: &Handle<dyn InterfaceInterfaceModel>,
) {
    if checks.is_empty(false) {
        return;
    }
    let mut out = Vec::new();
    let _ = checks.print_with_model(&mut out, model, false, 0);
    info(&format!("{title}\n{}", String::from_utf8_lossy(&out)));
}
