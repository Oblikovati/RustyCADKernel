// Rust port of DataExchange/TKXSBase/Transfer/Transfer_ProcessForTransient_0.cxx and
// Transfer_ProcessForFinder_0.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871: the transfer
// itself (Recognize, Transferring, TransferProduct, Transfer) and the trace.

//! The transfer and the trace of [`TransferProcess`].
use super::{
    TransferActorOps, TransferBinder, TransferProcess, TransferProcessKind, TransferProcessOps,
    TransferStatusExec, transfer_failure,
};
use rustycad_tkernel::{
    message::{Gravity, ProgressRange, ProgressScope},
    standard::{Failure, Handle},
};
use std::io;

/// What `Transferring` does with a former binder without result.
enum FormerStep {
    /// Transfer (the binder is set "running").
    Run,
    /// Return the former binder (done, or loop detected).
    Return,
    /// The head of a dead loop: fails recorded on the former binder.
    DeadLoop,
}

impl<K: TransferProcessKind> TransferProcess<K> {
    /// Sends the text a native `SendInfo()` stream collected, at the end of its scope.
    fn send_info_text(&self, text: &str) {
        if !text.is_empty() {
            self.messenger()
                .send_string_stream(text.as_bytes(), Gravity::Info);
        }
    }
    /// `Recognize(start)`: an actor of the chain recognizes `start`.
    pub fn recognize(&self, start: &K::Start) -> bool {
        let mut actor = self.actor();
        while let Some(live) = actor.get() {
            if live.recognize(start) {
                return true;
            }
            let next = live.next();
            actor = next;
        }
        false
    }
    /// The status switch of `Transferring` for a former binder without result:
    /// `Transfer_TransferFailure` for a binder in error.
    fn former_step(
        &self,
        former: &Handle<dyn TransferBinder>,
        start: &K::Start,
    ) -> Result<FormerStep, Failure> {
        let (trace, level) = (self.trace_level(), self.nesting_level());
        let mode = if trace != 0 { 0 } else { 4 };
        let step = match former.status_exec() {
            TransferStatusExec::StatusInitial => FormerStep::Run,
            TransferStatusExec::StatusDone => {
                self.send_info_text(" .. and Transfer done\n");
                FormerStep::Return
            }
            TransferStatusExec::StatusRun => {
                former.set_status_exec(TransferStatusExec::StatusLoop);
                FormerStep::Return
            }
            TransferStatusExec::StatusError => {
                self.start_trace(former, start, level, mode);
                self.send_info_text(if trace != 0 {
                    "                  *** Transfer in Error Status  :\n"
                } else {
                    ""
                });
                return Err(transfer_failure(
                    "TransferProcess : Transfer in Error Status",
                ));
            }
            TransferStatusExec::StatusLoop => {
                self.start_trace(former, start, level, mode);
                self.send_info_text(if trace != 0 {
                    "                  *** Transfer  Head of Dead Loop  :\n"
                } else {
                    ""
                });
                FormerStep::DeadLoop
            }
        };
        Ok(step)
    }
    /// The dead loop branch of `Transferring`: the former binder gets two fails.
    fn dead_loop(&self, former: &Handle<dyn TransferBinder>, start: &K::Start) {
        let (trace, level) = (self.trace_level(), self.nesting_level());
        self.start_trace(former, start, level - 1, if trace != 0 { 0 } else { 4 });
        self.send_info_text(if trace != 0 {
            "                  *** Dead Loop : Finding head of Loop :\n"
        } else {
            ""
        });
        former.add_fail("Transfer: Head of Dead Loop", "");
        former.add_fail("Transfer in dead Loop", "");
    }
    /// The protected branch of `Transferring`: a failure of the actors binds a void binder
    /// with a fail and restores the level. Returns the binder and whether it was bound.
    fn protected_product(
        &self,
        start: &K::Start,
        progress: &ProgressRange<'_>,
    ) -> Result<(Handle<dyn TransferBinder>, bool), Failure> {
        let oldlev = self.nesting_level();
        let failure = match self.transfer_product(start, progress) {
            Ok(binder) => return Ok((binder, false)),
            Err(failure) => failure,
        };
        let mut text = String::from("                  *** Exception Raised with no Result\n");
        let binder: Handle<dyn TransferBinder> =
            Handle::new(super::TransferVoidBinder::new()).cast();
        self.bind(start, &binder)?;
        binder.add_fail("Transfer stopped by exception raising", "");
        if self.trace_level() != 0 {
            text.push_str(&format!("    *** Raised : {}\n", failure.message()));
            self.start_trace(&binder, start, self.nesting_level() - 1, 4);
        }
        self.write().thelevel = oldlev;
        self.send_info_text(&text);
        Ok((binder, true))
    }
    /// `Transferring(start, theProgress)`: the binder of `start`, transferred by the
    /// actors when needed (null when no actor gives a result). With the error handle a
    /// failure of an actor is caught and recorded (then null); without it, it is returned.
    /// `Transfer_TransferFailure` for a former binder in error.
    pub fn transferring(
        &self,
        start: &K::Start,
        progress: &ProgressRange<'_>,
    ) -> Result<Handle<dyn TransferBinder>, Failure> {
        let former = self.find_and_mask(start);
        let mut has_dead_loop = false;
        if let Some(live) = former.get() {
            if live.has_result() {
                live.set_already_used();
                return Ok(former);
            }
            match self.former_step(&former, start)? {
                FormerStep::Return => return Ok(former),
                FormerStep::DeadLoop => has_dead_loop = true,
                FormerStep::Run => live.set_status_exec(TransferStatusExec::StatusRun),
            }
        }
        let (binder, newbind) = if has_dead_loop {
            self.dead_loop(&former, start);
            (former.clone(), false)
        } else if self.error_handle() {
            self.protected_product(start, progress)?
        } else {
            (self.transfer_product(start, progress)?, false)
        };
        if progress.user_break() {
            return Ok(Handle::default());
        }
        if newbind || binder.is_null() {
            if let Some(live) = former.get() {
                live.set_status_exec(TransferStatusExec::StatusDone);
            }
            return Ok(Handle::default());
        }
        self.conclude(start, &binder)
    }
    /// The end of `Transferring`: binds the result and records the root.
    fn conclude(
        &self,
        start: &K::Start,
        binder: &Handle<dyn TransferBinder>,
    ) -> Result<Handle<dyn TransferBinder>, Failure> {
        self.bind(start, binder)?;
        let (rootl, level, rootm) = {
            let st = self.read();
            (st.therootl, st.thelevel, st.therootm)
        };
        if rootl >= level {
            self.write().therootl = 0;
            if rootm && binder.status() != super::TransferStatusResult::StatusVoid {
                self.set_root(start);
            }
        }
        Ok(self.read().thelastbnd.clone())
    }
    /// `TransferProduct(start, theProgress)` (private): the first actor result, one
    /// nesting level down.
    fn transfer_product(
        &self,
        start: &K::Start,
        progress: &ProgressRange<'_>,
    ) -> Result<Handle<dyn TransferBinder>, Failure> {
        self.write().thelevel += 1;
        let this = self.this();
        let scope = ProgressScope::unnamed(progress, 1.0, true);
        let mut binder: Handle<dyn TransferBinder> = Handle::default();
        let mut actor = self.actor();
        while let Some(live) = actor.get() {
            binder = if live.recognize(start) {
                live.transferring(start, &this, &scope.next(1.0))?
            } else {
                Handle::default()
            };
            if binder.is_some() {
                break;
            }
            let next = live.next();
            actor = next;
        }
        if scope.user_break() {
            return Ok(Handle::default());
        }
        let mut st = self.write();
        if binder.is_some()
            && st.therootl == 0
            && binder.status_exec() == TransferStatusExec::StatusDone
        {
            st.therootl = st.thelevel - 1;
        }
        if st.thelevel > 0 {
            st.thelevel -= 1;
        }
        Ok(binder)
    }
    /// `Transfer(start, theProgress)`: `Transferring` gave a binder.
    pub fn transfer(
        &self,
        start: &K::Start,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        Ok(self.transferring(start, progress)?.is_some())
    }
    /// The prefix of a trace line at trace level 4 and more.
    fn trace_prefix(&self, level: i32, mode: i32) -> String {
        let nroots = self.nb_roots();
        let mut text = String::from(match mode {
            1 => "  ###  Fail",
            2 => "  ###  Warning",
            4 => "  ###  Exception",
            5 => "  ###  Substitution",
            6 => "  ###  Information",
            _ => "",
        });
        if mode == 3 {
            text.push_str(&format!("  ###  New Root n0 {nroots}"));
        }
        if level > 1 {
            text.push_str(" (nested)");
        }
        if mode >= 0 && mode != 3 {
            text.push_str(&format!(" at {nroots} Roots"));
        }
        text
    }
    /// The result types of a chain of binders, or "No Result recorded" (mode > 2).
    fn trace_results(binder: &Handle<dyn TransferBinder>, mode: i32) -> String {
        let mut text = String::new();
        let mut bnd = binder.clone();
        while let Some(live) = bnd.get() {
            if live.status() != super::TransferStatusResult::StatusVoid {
                text.push_str(if text.is_empty() {
                    "\n  ---  Result Type : "
                } else {
                    " , "
                });
                text.push_str(&live.result_type_name());
            }
            let next = live.next_result();
            bnd = next;
        }
        if text.is_empty() && mode > 2 {
            text.push_str("\n  ---  No Result recorded");
        }
        text
    }
    /// `StartTrace(binder, start, level, mode)`: one trace line to the messenger (info):
    /// the mode prefix (trace level 4), `PrintTrace(start)` and the result types.
    pub fn start_trace(
        &self,
        binder: &Handle<dyn TransferBinder>,
        start: &K::Start,
        level: i32,
        mode: i32,
    ) {
        let mut text = if self.trace_level() > 3 {
            self.trace_prefix(level, mode)
        } else {
            String::new()
        };
        if !K::is_null(start) {
            let mut printed = Vec::new();
            let _ = self.print_trace(start, &mut printed);
            text.push_str(&String::from_utf8_lossy(&printed));
        }
        if binder.is_some() {
            text.push_str(&Self::trace_results(binder, mode));
        }
        text.push('\n');
        self.send_info_text(&text);
    }
    /// `PrintTrace(start, S)` (virtual): dispatched to the owning process.
    pub fn print_trace(&self, start: &K::Start, s: &mut dyn io::Write) -> io::Result<()> {
        match self.this().get() {
            Some(process) => process.print_trace(start, s),
            None => K::print_start(start, s),
        }
    }
    /// `CheckNum(start)` (virtual): dispatched to the owning process (0 by default).
    pub fn check_num(&self, start: &K::Start) -> i32 {
        self.this()
            .get()
            .map_or(0, |process| process.check_num(start))
    }
    /// `IsLooping(alevel)`: `alevel` exceeds the count of mapped objects.
    pub fn is_looping(&self, alevel: i32) -> bool {
        alevel > self.nb_mapped()
    }
}
