// Rust port of ApplicationFramework/TKLCAF/TDF/TDF_Data.hxx, .lxx and .cxx (fields,
// accessors, entries and destruction) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDF_Data`: a data framework, its label tree and its transaction state
//! (ocaf-design.md sections 3 and 5).
use super::{TdfDelta, TdfLabel, TdfTool, label_node::TdfLabelNode};
use crate::json::dump_label_entry;
use rustycad_tkcdf::json_dump::class_begin;
use rustycad_tkcdf::json_dump::field_integer;
use rustycad_tkernel::{
    impl_transient,
    ncollection::DataMap,
    standard::{Failure, Handle, SharedState, StandardTransient, dump::DumpStream},
    tcollection::AsciiString,
};
use std::{io, sync::Arc};

/// The fields of `TDF_Data` that change.
pub(crate) struct DataState {
    pub(crate) transaction: i32,
    pub(crate) nb_touched_att: i32,
    pub(crate) not_undo_mode: bool,
    pub(crate) time: i32,
    /// Native `myTimes`, a list with the most recent time first; here the last element.
    pub(crate) times: Vec<i32>,
    pub(crate) allow_modification: bool,
    pub(crate) access_by_entries: bool,
    pub(crate) access_by_entries_table: DataMap<AsciiString, TdfLabel>,
}

/// The framework internals that the label nodes reach; they stay alive while the owning
/// `TdfData` runs its native destruction, when its handle can no longer be upgraded.
pub(crate) struct DataCore {
    root: Arc<TdfLabelNode>,
    pub(crate) state: SharedState<DataState>,
}

impl DataCore {
    pub(crate) fn root_node(&self) -> &Arc<TdfLabelNode> {
        &self.root
    }
    pub(crate) fn root(&self) -> TdfLabel {
        TdfLabel::from_node(Some(self.root.clone()))
    }
    pub(crate) fn transaction(&self) -> i32 {
        self.state.read().transaction
    }
    pub(crate) fn time(&self) -> i32 {
        self.state.read().time
    }
    pub(crate) fn not_undo_mode(&self) -> bool {
        self.state.read().not_undo_mode
    }
    pub(crate) fn is_modification_allowed(&self) -> bool {
        self.state.read().allow_modification
    }
    pub(crate) fn is_access_by_entries(&self) -> bool {
        self.state.read().access_by_entries
    }
    /// `RegisterLabel`: binds the label's entry for the fast access by entry.
    pub(crate) fn register_label(&self, label: &TdfLabel) {
        let mut entry = AsciiString::default();
        TdfTool::entry(label, &mut entry);
        self.state
            .write()
            .access_by_entries_table
            .bind(entry, label.clone());
    }
}

/// `TDF_Data`; `TdfData::new()` makes a framework with an empty root label.
///
/// ```
/// use rustycad_tklcaf::tdf::TdfData;
/// let data = TdfData::new();
/// assert_eq!((data.transaction(), data.time()), (0, 0));
/// assert!(data.root().is_root() && data.not_undo_mode());
/// ```
pub struct TdfData {
    transient: StandardTransient,
    pub(crate) core: Arc<DataCore>,
}
impl_transient!(TdfData, "TDF_Data", StandardTransient, transient);

impl TdfData {
    /// `TDF_Data()`: a new and empty framework, already in a handle (the root node keeps a
    /// pointer to it).
    pub fn new() -> Handle<TdfData> {
        let core = Arc::new(DataCore {
            root: Arc::new(TdfLabelNode::default()),
            state: SharedState::new(DataState {
                transaction: 0,
                nb_touched_att: 0,
                not_undo_mode: true,
                time: 0,
                times: Vec::new(),
                allow_modification: true,
                access_by_entries: false,
                access_by_entries_table: DataMap::default(),
            }),
        });
        let data = Handle::new(TdfData {
            transient: StandardTransient::default(),
            core: core.clone(),
        });
        let mut root = core.root.state.write();
        root.data = data.downgrade();
        root.core = Arc::downgrade(&core);
        drop(root);
        data
    }
    /// `Root`: the root label.
    pub fn root(&self) -> TdfLabel {
        self.core.root()
    }
    /// `Transaction`: the current transaction number.
    pub fn transaction(&self) -> i32 {
        self.core.transaction()
    }
    /// `Time`: the current tick, incremented by each commit that touched attributes.
    pub fn time(&self) -> i32 {
        self.core.time()
    }
    /// `IsApplicable`: the delta ends at the current time.
    pub fn is_applicable(&self, delta: &Handle<TdfDelta>) -> bool {
        delta
            .get()
            .is_some_and(|delta| delta.is_applicable(self.time()))
    }
    /// `Undo(aDelta, withDelta)`: applies an applicable delta; with `with_delta`, returns
    /// the delta that redoes it.
    pub fn undo(
        &self,
        delta: &Handle<TdfDelta>,
        with_delta: bool,
    ) -> Result<Handle<TdfDelta>, Failure> {
        super::data_commit::undo(&self.core, delta, with_delta)
    }
    /// `NotUndoMode`: false while a delta is being applied.
    pub fn not_undo_mode(&self) -> bool {
        self.core.not_undo_mode()
    }
    /// `Dump` (and `operator<<`): the transaction and the tick.
    pub fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        let state = self.core.state.read();
        writeln!(stream, "Dump of a TDF_Data.")?;
        write!(stream, "Current transaction: {}", state.transaction)?;
        writeln!(stream, "; Current tick: {};", state.time)
    }
    /// `AllowModification`; `data.allow_modification(false)`.
    pub fn allow_modification(&self, allowed: bool) {
        self.core.state.write().allow_modification = allowed;
    }
    /// `IsModificationAllowed`; `data.is_modification_allowed()`.
    pub fn is_modification_allowed(&self) -> bool {
        self.core.is_modification_allowed()
    }
    /// `SetAccessByEntries`: (re)builds or clears the entry table of every label.
    pub fn set_access_by_entries(&self, set: bool) -> Result<(), Failure> {
        {
            let mut state = self.core.state.write();
            state.access_by_entries = set;
            state.access_by_entries_table.clear(true);
        }
        if !set {
            return Ok(());
        }
        let root = self.root();
        self.core.register_label(&root);
        let mut children = super::TdfChildIterator::new(&root, true)?;
        while children.more() {
            self.core.register_label(&children.value());
            children.next();
        }
        Ok(())
    }
    /// `IsAccessByEntries`; `data.is_access_by_entries()`.
    pub fn is_access_by_entries(&self) -> bool {
        self.core.is_access_by_entries()
    }
    /// `GetLabel(anEntry, aLabel)`: the label bound to the entry by the fast access table.
    pub fn get_label(&self, entry: &AsciiString, label: &mut TdfLabel) -> bool {
        let state = self.core.state.read();
        match state.access_by_entries_table.find(entry) {
            Ok(found) => {
                *label = found.clone();
                true
            }
            Err(_) => false,
        }
    }
    /// `RegisterLabel`: binds a new label's entry (an internal method of the framework).
    pub fn register_label(&self, label: &TdfLabel) {
        self.core.register_label(label);
    }
    /// `DumpJson`: the root entry and the counters.
    pub fn dump_json(&self, stream: &mut DumpStream, _depth: i32) -> io::Result<()> {
        class_begin(stream, "TDF_Data")?;
        dump_label_entry(stream, "aStrForTDF_Label", &self.root())?;
        let state = self.core.state.read();
        field_integer(stream, "myTransaction", state.transaction)?;
        field_integer(stream, "myNbTouchedAtt", state.nb_touched_att)?;
        field_integer(stream, "myNotUndoMode", i32::from(state.not_undo_mode))?;
        field_integer(stream, "myTime", state.time)?;
        for time in state.times.iter().rev() {
            field_integer(stream, "aTime", *time)?;
        }
        field_integer(
            stream,
            "myAllowModification",
            i32::from(state.allow_modification),
        )
    }
    /// `Destroy`: aborts every open transaction, forgets the root attributes without
    /// backup, clears the entry table and frees the label tree.
    pub fn destroy(&self) {
        // A failure while aborting cannot be reported from a destructor; the tree is freed
        // anyway, as natively.
        let _ = super::data_commit::abort_until_transaction(&self.core, 1);
        let root = self.core.root.clone();
        loop {
            let first = root.first_attribute();
            if first.is_null() {
                break;
            }
            root.remove_attribute(&Handle::default(), &first);
        }
        self.core.state.write().access_by_entries_table.clear(true);
        root.destroy();
    }
}

impl Drop for TdfData {
    /// `~TDF_Data`: `Destroy()`.
    fn drop(&mut self) {
        self.destroy();
    }
}
