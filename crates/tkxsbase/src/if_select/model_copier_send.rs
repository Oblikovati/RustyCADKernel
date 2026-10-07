// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_ModelCopier (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the copy and send operations.

//! The copy and send operations of [`IFSelectModelCopier`]: the split copy of a
//! share-out result, its writing, the sending of the whole or a selected part of a
//! model, and the copy of the remaining data.
use super::{
    IFSelectAppliedModifiers, IFSelectContextWrite, IFSelectModelCopier, IFSelectShareOut,
    IFSelectShareOutResult, IFSelectWorkLibrary,
};
use crate::interface::{
    InterfaceCheckIterator, InterfaceCopyTool, InterfaceEntityIterator, InterfaceGraph,
    InterfaceInterfaceModel, InterfaceProtocol,
};
use rustycad_tkernel::{
    message::Message,
    standard::{Failure, Handle, RuntimeTyped, exceptions},
    tcollection::AsciiString,
};
use std::fmt::Write as _;

/// The inputs of `CopiedModel(G, WL, protocol, topcopy, filename, dispnum, numod, ...)`.
pub struct IFSelectCopiedModelArgs<'a> {
    /// `G`: the graph of the original model.
    pub graph: &'a InterfaceGraph,
    /// `WL`: the work library which copies.
    pub wl: &'a Handle<dyn IFSelectWorkLibrary>,
    /// `protocol`.
    pub protocol: &'a Handle<dyn InterfaceProtocol>,
    /// `topcopy`: the roots of the packet to copy (used for `dispnum > 0`).
    pub tocopy: &'a InterfaceEntityIterator,
    /// `filename`: the file of the packet.
    pub filename: &'a AsciiString,
    /// `dispnum`: the rank of the dispatch (0: no copy, the model is taken as it is).
    pub dispnum: i32,
    /// `numod`: the rank of the packet in its dispatch (unused, as natively).
    pub numod: i32,
}

/// A copied packet: its file name, its model and the file modifiers applied.
type CopiedPacket = (
    AsciiString,
    Handle<dyn InterfaceInterfaceModel>,
    Handle<IFSelectAppliedModifiers>,
);

/// The text of an `AsciiString` (lossy for non UTF-8 bytes).
pub(super) fn text_of(s: &AsciiString) -> String {
    String::from_utf8_lossy(s.as_bytes()).into_owned()
}

/// The live handle of a null-dereferenced native argument.
pub(super) fn live<'h, T: ?Sized + RuntimeTyped>(
    h: &'h Handle<T>,
    what: &str,
) -> Result<&'h T, Failure> {
    h.get().ok_or_else(|| {
        exceptions::null_object(&format!(
            "IFSelect_ModelCopier: null {what}; expected a value"
        ))
    })
}

/// One line on the information messenger.
pub(super) fn info(line: &str) {
    let _ = writeln!(Message::send_info(), "{line}");
}

/// The report of `CopiedRemaining` on the copy counts `remain` of `nb` entities: the
/// size of the remaining model, then the entities copied nowhere.
pub(super) fn remaining_report(newcount: i32, remain: &[i32], nb: i32) -> String {
    let mut text = format!(" Remaining Model : {newcount} Entities\n");
    let refractory: Vec<i32> = (1..=nb).filter(|i| remain[*i as usize] == 0).collect();
    if refractory.is_empty() {
        text.push_str("  -- Remaining data complete\n");
        return text;
    }
    text.push_str(" Refractory : ");
    for i in &refractory {
        let _ = write!(text, " {i}");
    }
    let _ = writeln!(text, "  -- {} Entities", refractory.len());
    text
}

impl IFSelectModelCopier {
    /// The share-out of the current split.
    fn share_out_now(&self) -> Handle<IFSelectShareOut> {
        self.state.read().theshareout.clone()
    }
    /// Starts a split: the share-out of `eval`, the copy counts of its graph to zero.
    fn begin_split(&self, eval: &IFSelectShareOutResult) {
        let mut st = self.state.write();
        st.theshareout = eval.share_out();
        st.theremain = Some(vec![0; eval.graph().size() as usize + 1]);
    }
    /// Ends a split: the share-out records its dispatches as run.
    fn end_split(&self) -> Result<(), Failure> {
        let sho = self.share_out_now();
        let live = live(&sho, "share-out")?;
        live.set_last_run(live.nb_dispatches());
        Ok(())
    }
    /// The copy of the current packet of `eval`: its file name, its model and the
    /// file modifiers applied.
    fn copy_packet(
        &self,
        eval: &mut IFSelectShareOutResult,
        wl: &Handle<dyn IFSelectWorkLibrary>,
        protocol: &Handle<dyn InterfaceProtocol>,
        tc: &mut InterfaceCopyTool,
        checks: &mut InterfaceCheckIterator,
    ) -> Result<CopiedPacket, Failure> {
        let filename = eval.file_name()?;
        let (dispnum, (numod, _)) = (eval.dispatch_rank(), eval.packets_in_dispatch());
        let tocopy = eval.packet_root()?;
        let args = IFSelectCopiedModelArgs {
            graph: eval.graph(),
            wl,
            protocol,
            tocopy: &tocopy,
            filename: &filename,
            dispnum,
            numod,
        };
        let mut model = Handle::default();
        let applied = self.copied_model(&args, tc, &mut model, checks)?;
        Ok((filename, model, applied))
    }
    /// `Copy(eval, WL, protocol)`: `Copying` with a new copy tool.
    pub fn copy(
        &self,
        eval: &mut IFSelectShareOutResult,
        wl: &Handle<dyn IFSelectWorkLibrary>,
        protocol: &Handle<dyn InterfaceProtocol>,
    ) -> Result<InterfaceCheckIterator, Failure> {
        let mut tc = InterfaceCopyTool::with_protocol(eval.graph().model(), protocol)?;
        self.copying(eval, wl, protocol, &mut tc)
    }
    /// `Copying(eval, WL, protocol, TC)`: each packet copied and kept as a file. Native
    /// quirk kept: a packet whose file name is already used is not added, and its applied
    /// modifiers replace those of the last file.
    pub fn copying(
        &self,
        eval: &mut IFSelectShareOutResult,
        wl: &Handle<dyn IFSelectWorkLibrary>,
        protocol: &Handle<dyn InterfaceProtocol>,
        tc: &mut InterfaceCopyTool,
    ) -> Result<InterfaceCheckIterator, Failure> {
        info("** WorkSession : Copying split data before sending");
        let mut checks = InterfaceCheckIterator::new();
        self.begin_split(eval);
        eval.evaluate()?;
        while eval.more()? {
            let (filename, model, curapp) =
                self.copy_packet(eval, wl, protocol, tc, &mut checks)?;
            self.add_file(&filename, &model);
            if let Some(last) = self.state.write().theapplieds.last_mut() {
                *last = curapp;
            }
            eval.next_packet()?;
        }
        self.end_split()?;
        checks.set_name("X-STEP WorkSession : Split Copy (no Write)");
        Ok(checks)
    }
    /// Writes `model` as `filename`: false when the work library fails; the checks of
    /// the writing are merged in `checks`.
    pub(super) fn write_model(
        wl: &Handle<dyn IFSelectWorkLibrary>,
        model: &Handle<dyn InterfaceInterfaceModel>,
        protocol: &Handle<dyn InterfaceProtocol>,
        applied: &Handle<IFSelectAppliedModifiers>,
        filename: &str,
        checks: &mut InterfaceCheckIterator,
    ) -> Result<bool, Failure> {
        let mut ctx = IFSelectContextWrite::new(model, protocol, applied, filename);
        let res = live(wl, "work library")?.write_file(&mut ctx)?;
        checks.merge(&ctx.check_list());
        Ok(res)
    }
    /// `SendCopied(WL, protocol)`: the files kept are written (those with a name), then
    /// cleared. Native quirk kept: a failure returns the checks without their name.
    pub fn send_copied(
        &self,
        wl: &Handle<dyn IFSelectWorkLibrary>,
        protocol: &Handle<dyn InterfaceProtocol>,
    ) -> Result<InterfaceCheckIterator, Failure> {
        info("** WorkSession : Sending split data already copied");
        let nb = self.nb_files();
        let mut checks = InterfaceCheckIterator::new();
        if nb > 0 {
            for i in 1..=nb {
                let name = self.file_name(i)?;
                if name.is_empty() {
                    continue;
                }
                if !Self::write_model(
                    wl,
                    &self.file_model(i)?,
                    protocol,
                    &self.applied_modifiers(i)?,
                    &text_of(&name),
                    &mut checks,
                )? {
                    checks.c_check(0).add_fail_text(
                        &format!("Split Send (WriteFile) abandon on file n0.{i}"),
                        "",
                    );
                    info(&format!(
                        "  **  Sending File n0.{i} has failed, abandon  **"
                    ));
                    return Ok(checks);
                }
                self.add_sent_file(&text_of(&name));
            }
            self.clear_result();
        }
        checks.set_name("X-STEP WorkSession : Split Send (Copy+Write)");
        Ok(checks)
    }
    /// `Send(eval, WL, protocol)`: `Sending` with a new copy tool.
    pub fn send(
        &self,
        eval: &mut IFSelectShareOutResult,
        wl: &Handle<dyn IFSelectWorkLibrary>,
        protocol: &Handle<dyn InterfaceProtocol>,
    ) -> Result<InterfaceCheckIterator, Failure> {
        let mut tc = InterfaceCopyTool::with_protocol(eval.graph().model(), protocol)?;
        self.sending(eval, wl, protocol, &mut tc)
    }
    /// `Sending(eval, WL, protocol, TC)`: each packet copied then written, up to the first
    /// failure.
    pub fn sending(
        &self,
        eval: &mut IFSelectShareOutResult,
        wl: &Handle<dyn IFSelectWorkLibrary>,
        protocol: &Handle<dyn InterfaceProtocol>,
        tc: &mut InterfaceCopyTool,
    ) -> Result<InterfaceCheckIterator, Failure> {
        let mut checks = InterfaceCheckIterator::new();
        info("** WorkSession : Copying then sending split data");
        self.begin_split(eval);
        eval.evaluate()?;
        let mut i = 0;
        while eval.more()? {
            i += 1;
            let (filename, model, curapp) =
                self.copy_packet(eval, wl, protocol, tc, &mut checks)?;
            let name = text_of(&filename);
            if !Self::write_model(wl, &model, protocol, &curapp, &name, &mut checks)? {
                checks.c_check(0).add_fail_text(
                    &format!("Split Send (WriteFile) abandon on file n0.{i}"),
                    "",
                );
                info(&format!(
                    "  **  Sending File {name} has failed, abandon  **"
                ));
                checks.set_name("X-STEP WorkSession : Split Send (only Write)");
                return Ok(checks);
            }
            self.add_sent_file(&name);
            eval.next_packet()?;
        }
        self.end_split()?;
        checks.set_name("X-STEP WorkSession : Split Send (only Write)");
        Ok(checks)
    }
}
