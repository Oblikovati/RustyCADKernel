// Rust port of ApplicationFramework/TKLCAF/TDF/TDF_Data.cxx (transactions, commit, abort and
// undo) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The transaction machinery of `TDF_Data` (ocaf-design.md section 5). The native build
//! defines `TDF_DATA_COMMIT_OPTIMIZED`, so the commit visits only the labels flagged
//! `MayBeModified` and the attribute lists flagged `AttributesModified`.
use super::attribute_state::{remove_backup, resume};
use super::{
    TdfAttribute, TdfAttributeDelta, TdfAttributeIterator, TdfAttributeState, TdfChildIterator,
    TdfDelta, TdfDeltaOnRemoval, TdfLabel, data::DataCore, label_node::TdfLabelNode,
};
use crate::live;
use rustycad_tkernel::standard::{Failure, Handle};
use std::sync::Arc;

/// `OpenTransaction`: pushes the current time and returns the new transaction number.
pub(crate) fn open_transaction(core: &DataCore) -> i32 {
    let mut state = core.state.write();
    let time = state.time;
    state.times.push(time);
    state.transaction += 1;
    state.transaction
}

/// `CommitTransaction(withDelta)`: commits the current transaction into the enclosing one;
/// with `with_delta`, returns what undoes it. Does nothing at transaction 0.
pub(crate) fn commit_transaction(
    core: &DataCore,
    with_delta: bool,
) -> Result<Handle<TdfDelta>, Failure> {
    if core.transaction() <= 0 {
        return Ok(Handle::default());
    }
    let delta = if with_delta {
        Handle::new(TdfDelta::new())
    } else {
        Handle::default()
    };
    core.state.write().nb_touched_att = 0;
    if core.root_node().may_be_modified() {
        let touched = commit_label(core, &core.root(), &delta, with_delta)?;
        core.state.write().nb_touched_att = touched;
    }
    let mut state = core.state.write();
    let delta_is_empty = delta.get().is_none_or(TdfDelta::is_empty);
    if state.nb_touched_att != 0 && !(with_delta && delta_is_empty) {
        state.time += 1;
    }
    state.transaction -= 1;
    let begin = state.times.pop().unwrap_or_default();
    if let Some(delta) = delta.get().filter(|delta| !delta.is_empty()) {
        delta.validity(begin, state.time);
    }
    Ok(delta)
}

/// `CommitUntilTransaction`: commits down to and including `until_transaction`.
pub(crate) fn commit_until_transaction(
    core: &DataCore,
    until_transaction: i32,
    with_delta: bool,
) -> Result<Handle<TdfDelta>, Failure> {
    if until_transaction <= 0 || core.transaction() < until_transaction {
        return Ok(Handle::default());
    }
    while core.transaction() > until_transaction {
        commit_transaction(core, false)?;
    }
    commit_transaction(core, with_delta)
}

/// `AbortTransaction`: undoes the current transaction. A private member that the native
/// framework declares but never calls (TDF_Transaction aborts until a transaction).
#[allow(dead_code)]
pub(crate) fn abort_transaction(core: &DataCore) -> Result<(), Failure> {
    if core.transaction() > 0 {
        let delta = commit_transaction(core, true)?;
        undo(core, &delta, false)?;
    }
    Ok(())
}

/// `AbortUntilTransaction`: undoes down to and including `until_transaction`.
pub(crate) fn abort_until_transaction(
    core: &DataCore,
    until_transaction: i32,
) -> Result<(), Failure> {
    if until_transaction > 0 {
        let delta = commit_until_transaction(core, until_transaction, true)?;
        undo(core, &delta, false)?;
    }
    Ok(())
}

/// `Undo(aDelta, withDelta)`: applies an applicable delta in undo mode, removal deltas last.
pub(crate) fn undo(
    core: &DataCore,
    delta: &Handle<TdfDelta>,
    with_delta: bool,
) -> Result<Handle<TdfDelta>, Failure> {
    let Some(applied) = delta
        .get()
        .filter(|applied| applied.is_applicable(core.time()))
    else {
        return Ok(Handle::default());
    };
    if with_delta {
        open_transaction(core);
    }
    applied.before_or_after_apply(true)?;
    core.state.write().not_undo_mode = false;
    fix_order(applied);
    // A failing delta leaves the undo mode on, as the native exception does.
    applied.apply()?;
    core.state.write().not_undo_mode = true;
    let mut new_delta = Handle::default();
    if with_delta {
        new_delta = commit_transaction(core, true)?;
        if let Some(created) = new_delta.get() {
            created.validity(applied.end_time(), applied.begin_time());
        }
    }
    core.state.write().time = applied.begin_time();
    applied.before_or_after_apply(false)?;
    Ok(new_delta)
}

/// Private `FixOrder`: the removal deltas (which add attributes back) go last, so that undo
/// never puts two attributes of one ID on a label.
fn fix_order(delta: &TdfDelta) {
    let deltas = delta.attribute_deltas();
    let is_removal =
        |d: &Handle<dyn TdfAttributeDelta>| d.cast::<dyn TdfDeltaOnRemoval>().is_some();
    let mut ordered: Vec<_> = deltas.iter().filter(|d| !is_removal(d)).cloned().collect();
    ordered.extend(deltas.iter().filter(|d| is_removal(d)).cloned());
    delta.replace_delta_list(ordered);
}

/// The recursive `CommitTransaction(aLabel, aDelta, withDelta)`: commits the attributes
/// of the label and of its children flagged `MayBeModified`; returns how many attributes
/// the current transaction touched.
fn commit_label(
    core: &DataCore,
    label: &TdfLabel,
    delta: &Handle<TdfDelta>,
    with_delta: bool,
) -> Result<i32, Failure> {
    let Some(node) = label.node().cloned() else {
        return Ok(0);
    };
    node.set_may_be_modified(false);
    let mut touched = 0;
    if node.attributes_modified() {
        let mut commit = LabelCommit {
            core,
            node: &node,
            delta,
            with_delta,
            touched: 0,
            modified: false,
        };
        commit.run()?;
        touched = commit.touched;
        node.set_attributes_modified(commit.modified);
    }
    let mut children = TdfChildIterator::new(label, false)?;
    while children.more() {
        let child = children.value();
        if child.may_be_modified() {
            touched += commit_label(core, &child, delta, with_delta)?;
        }
        children.next();
    }
    Ok(touched)
}

/// The attribute loop of one label in the recursive commit.
struct LabelCommit<'a> {
    core: &'a DataCore,
    node: &'a Arc<TdfLabelNode>,
    delta: &'a Handle<TdfDelta>,
    with_delta: bool,
    touched: i32,
    /// The native `attMod`: some attribute still belongs to an open transaction.
    modified: bool,
}

impl LabelCommit<'_> {
    fn run(&mut self) -> Result<(), Failure> {
        let mut last: Handle<dyn TdfAttribute> = Handle::default();
        let mut iterator = TdfAttributeIterator::from_node(self.node, false);
        while iterator.more() {
            let current = iterator.value();
            iterator.next();
            let removed = self.commit_attribute(&current, &last)?;
            if !removed {
                last = current;
            }
        }
        Ok(())
    }
    /// One attribute; returns whether it was removed from the label.
    fn commit_attribute(
        &mut self,
        current: &Handle<dyn TdfAttribute>,
        last: &Handle<dyn TdfAttribute>,
    ) -> Result<bool, Failure> {
        let attribute = live(current, "TDF_Data::CommitTransaction")?;
        attribute.before_commit_transaction()?;
        let backup = attribute.attribute_base().backup_attribute();
        let transaction = self.core.transaction();
        if attribute.transaction() != transaction {
            self.modified = self.modified || attribute.transaction() > 0;
            return Ok(false);
        }
        self.touched += 1;
        attribute.attribute_base().set_transaction(transaction - 1);
        if attribute.is_forgotten() {
            return self.commit_forgotten(current, attribute, last, backup);
        }
        if attribute.attribute_base().saved_transaction() < 0 {
            self.add_delta(|| Ok(attribute.delta_on_resume()?.cast()))?;
            attribute.attribute_base().set_saved_transaction(0);
        } else if backup.is_null() {
            self.add_delta(|| Ok(attribute.delta_on_addition()?.cast()))?;
        } else {
            self.add_delta(|| Ok(attribute.delta_on_modification(&backup)?.cast()))?;
            if backup
                .get()
                .is_some_and(|b| b.transaction() == attribute.transaction())
            {
                remove_backup(attribute)?;
            }
        }
        self.modified = self.modified || attribute.transaction() > 0;
        Ok(false)
    }
    /// The forgotten branch of the commit.
    fn commit_forgotten(
        &mut self,
        current: &Handle<dyn TdfAttribute>,
        attribute: &dyn TdfAttribute,
        last: &Handle<dyn TdfAttribute>,
        backup: Handle<dyn TdfAttribute>,
    ) -> Result<bool, Failure> {
        let base = attribute.attribute_base();
        if base.saved_transaction() < attribute.transaction() {
            // Forgotten in a lower transaction than the current one.
            self.add_delta(|| Ok(attribute.delta_on_forget()?.cast()))?;
            return Ok(false);
        }
        if backup.is_null() {
            self.remove(current, attribute, last)?;
            self.modified = true;
            return Ok(true);
        }
        // Modified then forgotten: the previous version comes back.
        resume(attribute);
        attribute.restore(&backup)?;
        base.set_transaction(live(&backup, "TDF_Data::CommitTransaction")?.transaction());
        remove_backup(attribute)?;
        if self.core.transaction() == 1 {
            self.remove(current, attribute, last)?;
            return Ok(true);
        }
        // BeforeForget has already been called once.
        attribute.forget(self.core.transaction() - 1);
        self.add_delta(|| Ok(attribute.delta_on_forget()?.cast()))?;
        self.modified = true;
        Ok(false)
    }
    /// Removes a committed forgotten attribute with its removal delta.
    fn remove(
        &mut self,
        current: &Handle<dyn TdfAttribute>,
        attribute: &dyn TdfAttribute,
        last: &Handle<dyn TdfAttribute>,
    ) -> Result<(), Failure> {
        self.add_delta(|| Ok(attribute.delta_on_removal()?.cast()))?;
        if self.core.not_undo_mode() {
            attribute.before_removal()?;
        }
        self.node.remove_attribute(last, current);
        Ok(())
    }
    /// The native `TDF_Data_DeltaCreation` macro: the delta is made (a virtual call) only
    /// when deltas are requested.
    fn add_delta(
        &self,
        make: impl FnOnce() -> Result<Handle<dyn TdfAttributeDelta>, Failure>,
    ) -> Result<(), Failure> {
        if !self.with_delta {
            return Ok(());
        }
        let attribute_delta = make()?;
        if let Some(delta) = self.delta.get() {
            delta.add_attribute_delta(&attribute_delta);
        }
        Ok(())
    }
}
