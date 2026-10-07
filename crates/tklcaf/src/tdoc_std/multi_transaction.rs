// Rust port of ApplicationFramework/TKLCAF/TDocStd/TDocStd_ApplicationDelta and
// TDocStd_MultiTransactionManager (.hxx, .lxx, .cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Transactions over several documents at once.
use super::TDocStdDocument;
use crate::live;
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, SharedState, StandardTransient},
    tcollection::ExtendedString,
};
use std::{io, sync::RwLockWriteGuard};

/// `TDocStd_ApplicationDelta`: the documents one application command changed.
#[derive(Default)]
pub struct TDocStdApplicationDelta {
    transient: StandardTransient,
    documents: SharedState<ApplicationDeltaDocuments>,
    name: SharedState<ExtendedString>,
}
impl_transient!(
    TDocStdApplicationDelta,
    "TDocStd_ApplicationDelta",
    StandardTransient,
    transient
);

impl TDocStdApplicationDelta {
    /// `TDocStd_ApplicationDelta()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GetDocuments`: the documents, lent through a write guard.
    pub fn get_documents(&self) -> RwLockWriteGuard<'_, ApplicationDeltaDocuments> {
        self.documents.write()
    }
    /// `GetName`.
    pub fn get_name(&self) -> ExtendedString {
        self.name.read().clone()
    }
    /// `SetName`.
    pub fn set_name(&self, name: &ExtendedString) {
        *self.name.write() = name.clone();
    }
    /// `Dump`: `\t<name> - <n> documents  ( "<address>" ... ) `.
    pub fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        let documents = self.documents.read();
        write!(
            stream,
            "\t{} - {} documents  ( ",
            self.get_name().to_utf8(),
            documents.len()
        )?;
        for document in documents.iter() {
            write!(stream, "\"{:p}\" ", document.object_address())?;
        }
        write!(stream, ") ")
    }
}

/// The document list of an application delta.
pub type ApplicationDeltaDocuments = Vec<Handle<TDocStdDocument>>;

#[derive(Default)]
struct ManagerState {
    documents: Vec<Handle<TDocStdDocument>>,
    undos: Vec<Handle<TDocStdApplicationDelta>>,
    redos: Vec<Handle<TDocStdApplicationDelta>>,
    undo_limit: i32,
    open_transaction: bool,
    nested_transaction_mode: bool,
    only_transaction_modification: bool,
}

/// `TDocStd_MultiTransactionManager`: one undo stack over several documents; the most
/// recent application delta comes first.
#[derive(Default)]
pub struct TDocStdMultiTransactionManager {
    transient: StandardTransient,
    state: SharedState<ManagerState>,
}
impl_transient!(
    TDocStdMultiTransactionManager,
    "TDocStd_MultiTransactionManager",
    StandardTransient,
    transient
);

impl TDocStdMultiTransactionManager {
    /// `TDocStd_MultiTransactionManager()`: no document, no undo limit.
    pub fn new() -> Self {
        Self::default()
    }
    fn documents_copy(&self) -> Vec<Handle<TDocStdDocument>> {
        self.state.read().documents.clone()
    }
    /// `SetUndoLimit`: commits, trims the undos and gives every document the limit.
    pub fn set_undo_limit(&self, limit: i32) -> Result<(), Failure> {
        self.state.write().undo_limit = limit;
        self.commit_command()?;
        let excess = self.state.read().undos.len() as i32 - limit;
        for _ in 0..excess.max(0) {
            self.remove_last_undo();
        }
        for document in self.documents_copy().iter().rev() {
            live(document, "TDocStd_MultiTransactionManager::SetUndoLimit")?
                .set_undo_limit(limit)?;
        }
        Ok(())
    }
    /// `GetUndoLimit`.
    pub fn get_undo_limit(&self) -> i32 {
        self.state.read().undo_limit
    }
    /// `Undo`: undoes the most recent application delta in each of its documents (last
    /// first) and moves it to the redos.
    pub fn undo(&self) -> Result<(), Failure> {
        let Some(delta) = self.state.read().undos.first().cloned() else {
            return Ok(());
        };
        let documents = live(&delta, "TDocStd_MultiTransactionManager::Undo")?
            .get_documents()
            .clone();
        for document in documents.iter().rev().filter_map(Handle::get) {
            if document.get_available_undos() != 0 {
                document.undo()?;
            }
        }
        let mut state = self.state.write();
        state.redos.insert(0, delta);
        state.undos.remove(0);
        state.open_transaction = false;
        Ok(())
    }
    /// `Redo`: redoes the first redo in each of its documents and moves it to the undos.
    pub fn redo(&self) -> Result<(), Failure> {
        let Some(delta) = self.state.read().redos.first().cloned() else {
            return Ok(());
        };
        let documents = live(&delta, "TDocStd_MultiTransactionManager::Redo")?
            .get_documents()
            .clone();
        for document in documents.iter().rev().filter_map(Handle::get) {
            if document.get_available_redos() != 0 {
                document.redo()?;
            }
        }
        let mut state = self.state.write();
        state.undos.insert(0, delta);
        state.redos.remove(0);
        state.open_transaction = false;
        Ok(())
    }
    /// `GetAvailableUndos`: a copy, most recent first.
    pub fn get_available_undos(&self) -> Vec<Handle<TDocStdApplicationDelta>> {
        self.state.read().undos.clone()
    }
    /// `GetAvailableRedos`: a copy.
    pub fn get_available_redos(&self) -> Vec<Handle<TDocStdApplicationDelta>> {
        self.state.read().redos.clone()
    }
    /// `OpenCommand`: `Standard_Failure` while a command is open; else every document
    /// aborts its open commands and opens one.
    pub fn open_command(&self) -> Result<(), Failure> {
        if self.state.read().open_transaction {
            return Err(Failure::new(
                "Can't start new application transactionwhile a previous one is not committed or aborted",
            ));
        }
        self.state.write().open_transaction = true;
        for document in self.documents_copy().iter().rev() {
            let document = live(document, "TDocStd_MultiTransactionManager::OpenCommand")?;
            while document.has_open_command() {
                document.abort_command()?;
            }
            document.open_command()?;
        }
        Ok(())
    }
    /// `AbortCommand`: every document aborts its open commands.
    pub fn abort_command(&self) -> Result<(), Failure> {
        self.state.write().open_transaction = false;
        for document in self.documents_copy().iter().rev() {
            let document = live(document, "TDocStd_MultiTransactionManager::AbortCommand")?;
            while document.has_open_command() {
                document.abort_command()?;
            }
        }
        Ok(())
    }
    /// `CommitCommand()`: commits every document; the documents that recorded an undo form
    /// a new application delta.
    pub fn commit_command(&self) -> Result<bool, Failure> {
        let delta = Handle::new(TDocStdApplicationDelta::new());
        let delta_ref = live(&delta, "TDocStd_MultiTransactionManager::CommitCommand")?;
        let mut committed = false;
        for document in self.documents_copy().iter().rev() {
            committed = commit_document(document)?;
            if committed {
                delta_ref.get_documents().push(document.clone());
            }
        }
        if !delta_ref.get_documents().is_empty() {
            self.state.write().undos.insert(0, delta.clone());
            let over = {
                let s = self.state.read();
                s.undos.len() as i32 > s.undo_limit
            };
            if over {
                self.remove_last_undo();
            }
            self.state.write().redos.clear();
            committed = true;
        }
        self.state.write().open_transaction = false;
        Ok(committed)
    }
    /// `CommitCommand(theName)`: names the new application delta.
    pub fn commit_command_named(&self, name: &ExtendedString) -> Result<bool, Failure> {
        let committed = self.commit_command()?;
        if committed && let Some(first) = self.state.read().undos.first() {
            live(first, "TDocStd_MultiTransactionManager::CommitCommand")?.set_name(name);
        }
        Ok(committed)
    }
    /// `HasOpenCommand`.
    pub fn has_open_command(&self) -> bool {
        self.state.read().open_transaction
    }
    /// `RemoveLastUndo`: drops the oldest application delta and the first undo of each of
    /// its documents.
    pub fn remove_last_undo(&self) {
        let Some(last) = self.state.read().undos.last().cloned() else {
            return;
        };
        if let Some(last) = last.get() {
            for document in last.get_documents().iter().filter_map(Handle::get) {
                document.remove_first_undo();
            }
        }
        self.state.write().undos.pop();
    }
    /// `DumpTransaction`: the documents, the mode and the undo/redo lists.
    pub fn dump_transaction(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        super::multi_transaction_dump::dump_transaction(self, stream)
    }
    /// `AddDocument`: takes the manager's modes and limit, matches its open command, and
    /// forgets the document's own undos and redos.
    pub fn add_document(&self, document: &Handle<TDocStdDocument>) -> Result<(), Failure> {
        if self
            .state
            .read()
            .documents
            .iter()
            .any(|kept| kept == document)
        {
            return Ok(());
        }
        let target = live(document, "TDocStd_MultiTransactionManager::AddDocument")?;
        let (nested, only, limit) = {
            let s = self.state.read();
            (
                s.nested_transaction_mode,
                s.only_transaction_modification,
                s.undo_limit,
            )
        };
        if target.is_nested_transaction_mode() != nested {
            target.set_nested_transaction_mode(nested);
        }
        target.set_modification_mode(only)?;
        self.state.write().documents.push(document.clone());
        target.set_undo_limit(limit)?;
        if self.has_open_command() {
            if !target.has_open_command() {
                target.open_command()?;
            }
        } else if target.has_open_command() {
            target.commit_command()?;
        }
        target.clear_undos();
        target.clear_redos();
        Ok(())
    }
    /// `RemoveDocument`: removes the document from the manager and from its deltas (a delta
    /// left without document is dropped).
    pub fn remove_document(&self, document: &Handle<TDocStdDocument>) {
        let mut state = self.state.write();
        state.documents.retain(|kept| kept != document);
        remove_from_deltas(&mut state.undos, document);
        remove_from_deltas(&mut state.redos, document);
    }
    /// `Documents`: a copy of the managed documents.
    pub fn documents(&self) -> Vec<Handle<TDocStdDocument>> {
        self.documents_copy()
    }
    /// `SetNestedTransactionMode`: also for every document.
    pub fn set_nested_transaction_mode(&self, allowed: bool) {
        self.state.write().nested_transaction_mode = allowed;
        for document in self.documents_copy().iter().rev().filter_map(Handle::get) {
            if document.is_nested_transaction_mode() != allowed {
                document.set_nested_transaction_mode(allowed);
            }
        }
    }
    /// `IsNestedTransactionMode`.
    pub fn is_nested_transaction_mode(&self) -> bool {
        self.state.read().nested_transaction_mode
    }
    /// `SetModificationMode`: also for every document.
    pub fn set_modification_mode(&self, transaction_only: bool) -> Result<(), Failure> {
        self.state.write().only_transaction_modification = transaction_only;
        for document in self.documents_copy().iter().rev() {
            live(
                document,
                "TDocStd_MultiTransactionManager::SetModificationMode",
            )?
            .set_modification_mode(transaction_only)?;
        }
        Ok(())
    }
    /// `ModificationMode`.
    pub fn modification_mode(&self) -> bool {
        self.state.read().only_transaction_modification
    }
    /// `ClearUndos`: aborts, then forgets every undo (of the documents too).
    pub fn clear_undos(&self) -> Result<(), Failure> {
        self.abort_command()?;
        self.state.write().undos.clear();
        for document in self.documents_copy().iter().rev().filter_map(Handle::get) {
            document.clear_undos();
        }
        Ok(())
    }
    /// `ClearRedos`: aborts, then forgets every redo (of the documents too).
    pub fn clear_redos(&self) -> Result<(), Failure> {
        self.abort_command()?;
        self.state.write().redos.clear();
        for document in self.documents_copy().iter().rev().filter_map(Handle::get) {
            document.clear_redos();
        }
        Ok(())
    }
    pub(crate) fn dump_state(&self) -> ManagerDumpState {
        let s = self.state.read();
        (
            s.documents.clone(),
            s.nested_transaction_mode,
            s.undos.clone(),
            s.redos.clone(),
        )
    }
}

/// The documents, nested mode, undos and redos read by `DumpJson`.
pub(crate) type ManagerDumpState = (
    Vec<Handle<TDocStdDocument>>,
    bool,
    Vec<Handle<TDocStdApplicationDelta>>,
    Vec<Handle<TDocStdApplicationDelta>>,
);

/// The commit loop of one document: commits while a command is open; true when one
/// of the commits recorded an undo.
fn commit_document(document: &Handle<TDocStdDocument>) -> Result<bool, Failure> {
    let target = live(document, "TDocStd_MultiTransactionManager::CommitCommand")?;
    let mut committed = false;
    while target.has_open_command() {
        if target.commit_command()? {
            committed = true;
        }
    }
    Ok(committed)
}

/// The native removal loops of `RemoveDocument` over one delta list, from the end; a
/// delta left without document is dropped.
fn remove_from_deltas(
    deltas: &mut Vec<Handle<TDocStdApplicationDelta>>,
    document: &Handle<TDocStdDocument>,
) {
    let mut index = deltas.len();
    while index > 0 {
        let delta = deltas[index - 1].clone();
        if let Some(delta) = delta.get() {
            let mut documents = delta.get_documents();
            let mut position = documents.len();
            while position > 0 {
                if documents[position - 1] == *document {
                    documents.remove(position - 1);
                    if documents.is_empty() && index - 1 < deltas.len() {
                        deltas.remove(index - 1);
                    }
                }
                position -= 1;
            }
        }
        index -= 1;
    }
}
