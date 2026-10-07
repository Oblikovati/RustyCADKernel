// Rust port of ApplicationFramework/TKLCAF/TDataStd/TDataStd_TreeNode.cxx (the
// TDF_Attribute overrides) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `TDF_Attribute` members of `TDataStd_TreeNode`: the callbacks that reconnect a
//! node to its tree on addition, resume and undo, and copy, paste and dump.
use super::support::cast_attribute;
use super::tree_node::{TDataStdTreeNode, TreeNodeState};
use crate::json::dump_attribute_json_base;
use crate::live;
use crate::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeDelta, TdfAttributeState, TdfDataSet,
    TdfDeltaOnAddition, TdfDeltaOnRemoval, TdfLabel, TdfRelocationTable, TdfTool, dump_attribute,
};
use rustycad_tkcdf::json_dump::{class_begin, field_string};
use rustycad_tkernel::{
    standard::{Failure, Guid, Handle, RuntimeTyped, WeakHandle, dump::DumpStream},
    tcollection::AsciiString,
};
use std::io;

/// The links `Dump` and `DumpJson` print, in native order, with their names.
fn printed_links(
    state: &TreeNodeState,
) -> [(&'static str, &'static str, Handle<TDataStdTreeNode>); 5] {
    [
        ("Father", "aFather", state.father.upgrade()),
        ("Previous", "aPrevious", state.previous.upgrade()),
        ("Next", "aNext", state.next.upgrade()),
        ("First", "aFirst", state.first.upgrade()),
        ("Last", "aLast", state.last.upgrade()),
    ]
}

/// The label of a linked node (null for a null node).
fn label_of(node: &Handle<TDataStdTreeNode>) -> TdfLabel {
    node.get().map(|node| node.label()).unwrap_or_default()
}

/// One link of `Paste`: the relocated node when the table knows it; else null when the
/// table relocates after the copy, else the previous link's result (the native `func`
/// variable is reused from one link to the next).
fn relocated(
    table: &TdfRelocationTable,
    source: &Handle<TDataStdTreeNode>,
    carried: Handle<TDataStdTreeNode>,
) -> Handle<TDataStdTreeNode> {
    let mut target = carried;
    if !table.has_attribute_relocation_as(&source.cast(), &mut target) && table.after_relocate() {
        return Handle::default();
    }
    target
}

impl TdfAttribute for TDataStdTreeNode {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    /// `ID`: the tree ID.
    fn id(&self) -> Guid {
        self.state.read().tree_id
    }
    /// `AfterAddition`: unless backed up, the neighbours and the father point back to
    /// this node.
    fn after_addition(&self) -> Result<(), Failure> {
        if self.is_backuped() {
            return Ok(());
        }
        let this = self.this_node()?;
        let (previous, father, next) = (self.previous(), self.father(), self.next());
        if let Some(previous) = previous.get() {
            previous.set_next(&this)?;
        } else if let Some(father) = father.get() {
            father.set_first(&this)?;
        }
        if let Some(next) = next.get() {
            next.set_previous(&this)?;
        }
        Ok(())
    }
    /// `BeforeForget`: unless backed up, detaches the node and all its children.
    fn before_forget(&self) -> Result<(), Failure> {
        if self.is_backuped() {
            return Ok(());
        }
        self.remove()?;
        while self.has_first() {
            live(&self.first(), "TDataStd_TreeNode::BeforeForget")?.remove()?;
        }
        Ok(())
    }
    /// `AfterResume`: as `AfterAddition`.
    fn after_resume(&self) -> Result<(), Failure> {
        self.after_addition()
    }
    /// `BeforeUndo`: undoing an addition disconnects the node.
    fn before_undo(
        &self,
        delta: &Handle<dyn TdfAttributeDelta>,
        _force_it: bool,
    ) -> Result<bool, Failure> {
        if delta.is_kind(TdfDeltaOnAddition::static_type()) {
            self.before_forget()?;
        }
        Ok(true)
    }
    /// `AfterUndo`: undoing a removal reconnects the node.
    fn after_undo(
        &self,
        delta: &Handle<dyn TdfAttributeDelta>,
        _force_it: bool,
    ) -> Result<bool, Failure> {
        if delta.is_kind(<dyn TdfDeltaOnRemoval>::static_type()) {
            self.after_addition()?;
        }
        Ok(true)
    }
    /// `Restore(other)`: the other node's links and tree ID; the cached last child is dropped.
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = cast_attribute::<Self>(with, "TDataStd_TreeNode::Restore")?;
        let mut copy = live(&other, "TDataStd_TreeNode::Restore")?
            .state
            .read()
            .clone();
        copy.last = WeakHandle::default();
        *self.state.write() = copy;
        Ok(())
    }
    /// `Paste(into, RT)`: `into` takes the relocated father, next, previous and first
    /// links (in that order) and the tree ID.
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = into.cast::<Self>();
        let target = live(&target, "TDataStd_TreeNode::Paste")?;
        let table = live(table, "TDataStd_TreeNode::Paste")?;
        let mut link = relocated(table, &self.father(), Handle::default());
        target.set_father(&link)?;
        link = relocated(table, &self.next(), link);
        target.set_next(&link)?;
        link = relocated(table, &self.previous(), link);
        target.set_previous(&link)?;
        link = relocated(table, &self.first(), link);
        target.set_first(&link)?;
        target.set_tree_id(&self.id());
        Ok(())
    }
    /// `NewEmpty`: a node of the same tree.
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        let node = Self::new();
        node.set_tree_id(&self.id());
        Handle::new(node).cast()
    }
    /// `References`: every direct child node.
    fn references(&self, data_set: &Handle<TdfDataSet>) {
        let Some(data_set) = data_set.get() else {
            return;
        };
        let mut child = self.first();
        while let Some(current) = child.get() {
            data_set.add_attribute(&child.cast());
            child = current.next();
        }
    }
    /// `Dump`: the attribute dump, then `"  Father=<entry>"` and the other set links.
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        dump_attribute(self, stream)?;
        let state = self.state.read().clone();
        for (name, _, node) in printed_links(&state)
            .iter()
            .filter(|(_, _, node)| node.is_some())
        {
            write!(stream, "  {name}=")?;
            let label = label_of(node);
            if !label.is_null() {
                label.entry_dump(stream)?;
            }
        }
        writeln!(stream)
    }
    /// `DumpJson`: the entry of every set link.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "TDataStd_TreeNode")?;
        dump_attribute_json_base(self, stream, depth)?;
        let state = self.state.read().clone();
        for (_, field, node) in printed_links(&state)
            .iter()
            .filter(|(_, _, node)| node.is_some())
        {
            let mut entry = AsciiString::default();
            TdfTool::entry(&label_of(node), &mut entry);
            field_string(stream, field, &String::from_utf8_lossy(entry.as_bytes()))?;
        }
        Ok(())
    }
}
