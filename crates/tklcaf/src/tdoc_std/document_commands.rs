// Rust port of ApplicationFramework/TKLCAF/TDocStd/TDocStd_Document.cxx (commands,
// transactions, undo and redo) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The command and undo/redo members of `TDocStd_Document`. The native `myUndoTransaction`
//! (a `TDF_Transaction` named "UNDO") is kept as the number it opened, so that no lock is
//! held while the framework commits and calls back into attributes.
use super::{TDocStdApplication, TDocStdCompoundDelta, TDocStdDocument};
use crate::live;
use crate::tdf::{
    TdfData, TdfDelta, abort_until_transaction, commit_until_transaction, open_transaction,
};
use rustycad_tkcdf::cdm::CdmDocumentLifecycle;
use rustycad_tkernel::standard::{Failure, Handle, exceptions};

/// The application callbacks of the transaction members.
#[derive(Clone, Copy)]
enum TransactionEvent {
    Open,
    Commit,
    Abort,
}

impl TDocStdDocument {
    fn data_handle(&self) -> Handle<TdfData> {
        self.read().data.clone()
    }
    /// `myUndoTransaction.IsOpen()`.
    fn undo_is_open(&self) -> bool {
        self.read().undo_transaction > 0
    }
    /// `myUndoTransaction.Open()`.
    fn undo_open(&self) -> Result<(), Failure> {
        if self.undo_is_open() {
            return Err(exceptions::domain_error(
                "This transaction is already open.",
            ));
        }
        let data = self.data_handle();
        let data = data
            .get()
            .ok_or_else(|| exceptions::null_object("Null TDF_Data."))?;
        self.write().undo_transaction = open_transaction(&data.core);
        Ok(())
    }
    /// `myUndoTransaction.Commit(withDelta)`.
    fn undo_commit(&self, with_delta: bool) -> Result<Handle<TdfDelta>, Failure> {
        if !self.undo_is_open() {
            return Ok(Handle::default());
        }
        let until = std::mem::take(&mut self.write().undo_transaction);
        let data = self.data_handle();
        commit_until_transaction(&live(&data, "TDocStd_Document")?.core, until, with_delta)
    }
    /// `myUndoTransaction.Abort()`.
    fn undo_abort(&self) -> Result<(), Failure> {
        if !self.undo_is_open() {
            return Ok(());
        }
        let until = self.read().undo_transaction;
        let data = self.data_handle();
        abort_until_transaction(&live(&data, "TDocStd_Document")?.core, until)?;
        self.write().undo_transaction = 0;
        Ok(())
    }
    /// `myData->AllowModification(isAllowed)`.
    fn allow_modification(&self, allowed: bool) -> Result<(), Failure> {
        live(&self.data_handle(), "TDocStd_Document")?.allow_modification(allowed);
        Ok(())
    }
    /// The native closing step of every member: under transaction-only modification, the
    /// framework allows changes only inside an open command with an undo limit.
    fn update_modification_permission(&self) -> Result<(), Failure> {
        let (only, limit) = {
            let s = self.read();
            (s.only_transaction_modification, s.undo_limit)
        };
        if only {
            self.allow_modification(self.undo_is_open() && limit != 0)?;
        }
        Ok(())
    }
    /// The `OnOpenTransaction`/`OnCommitTransaction`/`OnAbortTransaction` notification of a
    /// `TDocStd_Application` that opened the document.
    fn notify(&self, event: TransactionEvent) -> Result<(), Failure> {
        if !self.is_opened() {
            return Ok(());
        }
        let application: Handle<dyn TDocStdApplication> = self.application()?.cast();
        let Some(application) = application.get() else {
            return Ok(());
        };
        let this = self.this_document()?;
        match event {
            TransactionEvent::Open => application.on_open_transaction(&this),
            TransactionEvent::Commit => application.on_commit_transaction(&this),
            TransactionEvent::Abort => application.on_abort_transaction(&this),
        }
    }
    /// `NewCommand`: commits, then opens a transaction.
    pub fn new_command(&self) -> Result<(), Failure> {
        self.commit_transaction()?;
        self.open_transaction()
    }
    /// `HasOpenCommand`.
    pub fn has_open_command(&self) -> bool {
        self.undo_is_open()
    }
    /// `OpenCommand`: `Standard_DomainError` when a command is open outside nested mode.
    pub fn open_command(&self) -> Result<(), Failure> {
        if !self.read().nested_transaction_mode && self.undo_is_open() {
            return Err(exceptions::domain_error(
                "TDocStd_Document::OpenCommand : already open",
            ));
        }
        self.open_transaction()
    }
    /// `CommitCommand`: true when an undo was recorded.
    pub fn commit_command(&self) -> Result<bool, Failure> {
        self.commit_transaction()
    }
    /// `AbortCommand`.
    pub fn abort_command(&self) -> Result<(), Failure> {
        self.abort_transaction()
    }
    /// Protected virtual `CommitTransaction`.
    pub fn commit_transaction(&self) -> Result<bool, Failure> {
        self.allow_modification(true)?;
        let is_done = if self.read().nested_transaction_mode && self.undo_is_open() {
            self.commit_nested()?
        } else {
            self.commit_flat()?
        };
        self.update_modification_permission()?;
        if is_done {
            self.notify(TransactionEvent::Commit)?;
        }
        Ok(is_done)
    }
    /// The nested branch of `CommitTransaction`: the delta joins the enclosing compound one.
    fn commit_nested(&self) -> Result<bool, Failure> {
        let delta = self.undo_commit(true)?;
        let first = self.read().undo_filo.first().cloned().unwrap_or_default();
        append_delta_to_the_first(&first, &delta)?;
        self.write().undo_filo.remove(0);
        let next = self.read().undo_filo.first().cloned();
        if let Some(next) = next {
            append_delta_to_the_first(&next, &first)?;
            self.undo_open()?;
            return Ok(false);
        }
        if live(&first, "TDocStd_Document::CommitTransaction")?.is_empty() {
            return Ok(false);
        }
        let mut state = self.write();
        state.undos.push(first);
        state.redos.clear();
        Ok(true)
    }
    /// The flat branch of `CommitTransaction`: a non-empty delta becomes the last undo, the
    /// oldest one dropping beyond the limit.
    fn commit_flat(&self) -> Result<bool, Failure> {
        if self.read().undo_limit == 0 || !self.undo_is_open() {
            return Ok(false);
        }
        let delta = self.undo_commit(true)?;
        if delta.get().is_none_or(TdfDelta::is_empty) {
            return Ok(false);
        }
        let mut state = self.write();
        state.redos.clear();
        state.undos.push(delta);
        if state.undos.len() as i32 > state.undo_limit {
            let oldest = state.undos.remove(0);
            if state.from_undo == oldest {
                // The oldest undo was the start of a compaction (SRN_DELTA_COMPACT).
                if state.undos.len() == 1 {
                    state.from_undo = Handle::default();
                    state.from_redo = Handle::default();
                } else {
                    state.from_undo = state.undos[0].clone();
                }
            }
        }
        Ok(true)
    }
    /// Protected virtual `AbortTransaction`.
    pub fn abort_transaction(&self) -> Result<(), Failure> {
        self.allow_modification(true)?;
        if self.undo_is_open() && self.read().undo_limit != 0 {
            self.undo_abort()?;
        }
        let first = self.read().undo_filo.first().cloned();
        if let Some(first) = first.filter(|_| self.read().nested_transaction_mode) {
            if !live(&first, "TDocStd_Document::AbortTransaction")?.is_empty() {
                live(&self.data_handle(), "TDocStd_Document::AbortTransaction")?
                    .undo(&first, true)?;
            }
            self.write().undo_filo.remove(0);
            if !self.read().undo_filo.is_empty() {
                self.undo_open()?;
            }
        }
        self.update_modification_permission()?;
        self.notify(TransactionEvent::Abort)
    }
    /// Protected virtual `OpenTransaction`: in nested mode, a new compound delta level.
    pub fn open_transaction(&self) -> Result<(), Failure> {
        self.allow_modification(true)?;
        if self.read().nested_transaction_mode {
            self.open_nested_level()?;
        }
        if self.read().undo_limit != 0 {
            self.undo_open()?;
        }
        self.update_modification_permission()?;
        self.notify(TransactionEvent::Open)
    }
    /// The nested part of `OpenTransaction`.
    fn open_nested_level(&self) -> Result<(), Failure> {
        if self.undo_is_open() {
            let delta = self.undo_commit(true)?;
            let first = self.read().undo_filo.first().cloned().unwrap_or_default();
            append_delta_to_the_first(&first, &delta)?;
        }
        let first = self.read().undo_filo.first().cloned();
        let last_time = match first {
            Some(first) => live(&first, "TDocStd_Document::OpenTransaction")?.end_time(),
            None => live(&self.data_handle(), "TDocStd_Document::OpenTransaction")?.time(),
        };
        let compound = Handle::new(TDocStdCompoundDelta::new());
        live(&compound, "TDocStd_Document::OpenTransaction")?
            .delta()
            .validity(last_time, last_time);
        self.write().undo_filo.insert(0, compound.cast());
        Ok(())
    }
    /// `SetUndoLimit(L)`: commits, keeps at most `L` (>= 0) undos (aborts a compaction).
    pub fn set_undo_limit(&self, limit: i32) -> Result<(), Failure> {
        {
            let mut state = self.write();
            state.from_undo = Handle::default();
            state.from_redo = Handle::default();
        }
        self.commit_transaction()?;
        {
            let mut state = self.write();
            state.undo_limit = limit.max(0);
            let excess = state.undos.len() as i32 - state.undo_limit;
            state.undos.drain(..excess.max(0) as usize);
        }
        self.update_modification_permission()
    }
    /// `GetUndoLimit`.
    pub fn get_undo_limit(&self) -> i32 {
        self.read().undo_limit
    }
    /// `GetAvailableUndos`.
    pub fn get_available_undos(&self) -> i32 {
        self.read().undos.len() as i32
    }
    /// `GetAvailableRedos`.
    pub fn get_available_redos(&self) -> i32 {
        self.read().redos.len() as i32
    }
    /// `ClearUndos`: forgets the undos, the redos and the compaction start.
    pub fn clear_undos(&self) {
        let mut state = self.write();
        state.undos.clear();
        state.redos.clear();
        state.from_redo = Handle::default();
        state.from_undo = Handle::default();
    }
    /// `ClearRedos`.
    pub fn clear_redos(&self) {
        let mut state = self.write();
        state.redos.clear();
        state.from_redo = Handle::default();
    }
    /// `Undo`: aborts the open command, applies the last undo and pushes its redo; reopens
    /// a command that was open.
    pub fn undo(&self) -> Result<bool, Failure> {
        let was_open = self.undo_is_open();
        let last = self.read().undos.last().cloned();
        let done = match last {
            Some(last) => {
                let redo = self.apply_stack_delta(&last)?;
                let mut state = self.write();
                state.redos.insert(0, redo);
                state.undos.pop();
                true
            }
            None => false,
        };
        self.finish_undo_or_redo(was_open && done)?;
        Ok(done)
    }
    /// `Redo`: applies the first redo and pushes the result back as an undo.
    pub fn redo(&self) -> Result<bool, Failure> {
        let was_open = self.undo_is_open();
        let first = self.read().redos.first().cloned();
        let done = match first {
            Some(first) => {
                let undo = self.apply_stack_delta(&first)?;
                let mut state = self.write();
                state.undos.push(undo);
                state.redos.remove(0);
                true
            }
            None => false,
        };
        self.finish_undo_or_redo(was_open && done)?;
        Ok(done)
    }
    /// The common part of `Undo` and `Redo`: resets every transaction, applies the delta
    /// with modifications allowed and names its reverse after it.
    fn apply_stack_delta(&self, delta: &Handle<TdfDelta>) -> Result<Handle<TdfDelta>, Failure> {
        self.abort_transaction()?;
        while self.read().nested_transaction_mode && !self.read().undo_filo.is_empty() {
            self.abort_transaction()?;
        }
        self.allow_modification(true)?;
        let reverse = live(&self.data_handle(), "TDocStd_Document::Undo")?.undo(delta, true)?;
        let name = live(delta, "TDocStd_Document::Undo")?.name();
        live(&reverse, "TDocStd_Document::Undo")?.set_name(&name);
        Ok(reverse)
    }
    fn finish_undo_or_redo(&self, reopen: bool) -> Result<(), Failure> {
        if reopen {
            self.open_transaction()?;
        }
        self.update_modification_permission()
    }
    /// `GetUndos`: a copy of the undo list, oldest first.
    pub fn get_undos(&self) -> Vec<Handle<TdfDelta>> {
        self.read().undos.clone()
    }
    /// `GetRedos`: a copy of the redo list, next first.
    pub fn get_redos(&self) -> Vec<Handle<TdfDelta>> {
        self.read().redos.clone()
    }
    /// `RemoveFirstUndo`.
    pub fn remove_first_undo(&self) {
        let mut state = self.write();
        if !state.undos.is_empty() {
            state.undos.remove(0);
        }
    }
    /// `SetNestedTransactionMode`.
    pub fn set_nested_transaction_mode(&self, allowed: bool) {
        self.write().nested_transaction_mode = allowed;
    }
    /// `IsNestedTransactionMode`.
    pub fn is_nested_transaction_mode(&self) -> bool {
        self.read().nested_transaction_mode
    }
    /// `SetModificationMode`: with true, the framework allows changes only inside an open
    /// command.
    pub fn set_modification_mode(&self, transaction_only: bool) -> Result<(), Failure> {
        self.write().only_transaction_modification = transaction_only;
        let allowed = !transaction_only || self.undo_is_open();
        self.allow_modification(allowed)
    }
    /// `ModificationMode`.
    pub fn modification_mode(&self) -> bool {
        self.read().only_transaction_modification
    }
    /// `BeforeClose` (virtual natively, never overridden): allows modifications, aborts and
    /// forgets every undo.
    pub fn before_close(&self) -> Result<(), Failure> {
        self.set_modification_mode(false)?;
        self.abort_transaction()?;
        if self.read().nested_transaction_mode {
            self.write().undo_filo.clear();
        }
        self.clear_undos();
        Ok(())
    }
}

/// `myUndoTransaction.Initialize(myData)` of `SetData`: aborts an open transaction.
pub(crate) fn reinitialize_undo_transaction(document: &TDocStdDocument) -> Result<(), Failure> {
    document.undo_abort()?;
    document.write().undo_transaction = 0;
    Ok(())
}

/// Private static `AppendDeltaToTheFirst`: adds to the compound delta the attribute deltas
/// of `delta` whose (label, ID) it does not hold yet, and extends its validity.
pub(crate) fn append_delta_to_the_first(
    compound: &Handle<TdfDelta>,
    delta: &Handle<TdfDelta>,
) -> Result<(), Failure> {
    let added = live(delta, "TDocStd_Document::AppendDeltaToTheFirst")?;
    if added.is_empty() {
        return Ok(());
    }
    let target = live(compound, "TDocStd_Document::AppendDeltaToTheFirst")?;
    let mut seen = super::document_compaction::LabelIdMap::default();
    for existing in target.attribute_deltas().iter().filter_map(Handle::get) {
        seen.add(&existing.label(), existing.id()?);
    }
    target.validity(target.begin_time(), added.end_time());
    for attribute_delta in added.attribute_deltas() {
        let delta_ref = live(&attribute_delta, "TDocStd_Document::AppendDeltaToTheFirst")?;
        if !seen.contains(&delta_ref.label(), &delta_ref.id()?) {
            target.add_attribute_delta(&attribute_delta);
        }
    }
    Ok(())
}
