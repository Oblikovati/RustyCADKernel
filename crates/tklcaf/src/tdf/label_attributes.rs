// Rust port of ApplicationFramework/TKLCAF/TDF/TDF_Label.cxx (attribute members)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The attribute members of `TDF_Label`: find, add, forget and resume.
use super::attribute_state::resume;
use super::{
    TdfAttribute, TdfAttributeIterator, TdfAttributeState, TdfChildIterator, TdfLabel,
    label_node::TdfLabelNode,
};
use crate::live;
use rustycad_tkernel::standard::{Failure, Guid, Handle, RuntimeTyped, exceptions};
use std::sync::Arc;

impl TdfLabel {
    /// `IsAttribute(anID)`; `Standard_NullObject` for a null label.
    pub fn is_attribute(&self, id: &Guid) -> Result<bool, Failure> {
        let mut found = Handle::default();
        self.find_attribute(id, &mut found)
    }
    /// `FindAttribute(anID, anAttribute)`: the valid (not forgotten) attribute of that ID;
    /// `Standard_NullObject` for a null label.
    pub fn find_attribute(
        &self,
        id: &Guid,
        attribute: &mut Handle<dyn TdfAttribute>,
    ) -> Result<bool, Failure> {
        let node = self.live_node("A null Label has no attribute.")?;
        let mut iterator = TdfAttributeIterator::from_node(node, true);
        while iterator.more() {
            let value = iterator.value();
            if value.get().is_some_and(|candidate| candidate.id() == *id) {
                *attribute = value;
                return Ok(true);
            }
            iterator.next();
        }
        Ok(false)
    }
    /// The template `FindAttribute<T>`: found and of type `T` (the output is the down cast,
    /// null when the type differs).
    pub fn find_attribute_as<T: RuntimeTyped + ?Sized>(
        &self,
        id: &Guid,
        attribute: &mut Handle<T>,
    ) -> Result<bool, Failure> {
        let mut found = Handle::default();
        if !self.find_attribute(id, &mut found)? {
            return Ok(false);
        }
        *attribute = found.cast::<T>();
        Ok(attribute.is_some())
    }
    /// `FindAttribute(anID, aTransaction, anAttribute)`: the version valid in `transaction`,
    /// following the backups.
    pub fn find_attribute_in_transaction(
        &self,
        id: &Guid,
        transaction: i32,
        attribute: &mut Handle<dyn TdfAttribute>,
    ) -> Result<bool, Failure> {
        let mut located = Handle::default();
        if !self.find_attribute(id, &mut located)? {
            return Ok(false);
        }
        while let Some(current) = located
            .get()
            .filter(|current| current.transaction() > transaction)
        {
            located = current.attribute_base().backup_attribute();
        }
        if located.is_null() {
            return Ok(false);
        }
        *attribute = located;
        Ok(true)
    }
    /// `HasAttribute`: at least one valid attribute; `Standard_NullObject` for a null label.
    pub fn has_attribute(&self) -> Result<bool, Failure> {
        let node = self.live_node("A null Label has no attribute.")?;
        Ok(TdfAttributeIterator::from_node(node, true).more())
    }
    /// `NbAttributes`: the valid attributes; `Standard_NullObject` for a null label.
    pub fn nb_attributes(&self) -> Result<i32, Failure> {
        let node = self.live_node("A null Label has no attribute.")?;
        let mut iterator = TdfAttributeIterator::from_node(node, true);
        let mut count = 0;
        while iterator.more() {
            count += 1;
            iterator.next();
        }
        Ok(count)
    }
    /// `AddAttribute(anAttribute, append)`: attaches the attribute (at the end when
    /// `append`, else first) in the current transaction and calls `AfterAddition`.
    /// `Standard_ImmutableObject` outside a transaction that allows it,
    /// `Standard_DomainError` for an attached attribute or a duplicate ID.
    pub fn add_attribute(
        &self,
        attribute: &Handle<dyn TdfAttribute>,
        append: bool,
    ) -> Result<(), Failure> {
        let node =
            self.live_node("TDF_Label::AddAttribute: a null Label cannot hold an attribute.")?;
        add_to_node(node, attribute, append)
    }
    /// `ForgetAttribute(anAttribute)`: forgets the attribute (removed at once when no
    /// transaction is open or when it was created in the current one).
    pub fn forget_attribute(&self, attribute: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let node = self.live_node("TDF_Label::ForgetAttribute: a null Label has no attribute.")?;
        forget_from_node(node, attribute)
    }
    /// `ForgetAttribute(aguid)`: forgets the attribute of that ID; false when there is none.
    pub fn forget_attribute_id(&self, id: &Guid) -> Result<bool, Failure> {
        let mut attribute = Handle::default();
        if !self.find_attribute(id, &mut attribute)? {
            return Ok(false);
        }
        self.forget_attribute(&attribute)?;
        Ok(true)
    }
    /// `ForgetAllAttributes(clearChildren)`: forgets every attribute (the iterator moves
    /// before each removal, AGV-OCC5031), then the children's when asked.
    pub fn forget_all_attributes(&self, clear_children: bool) -> Result<(), Failure> {
        let node =
            self.live_node("TDF_Label::ForgetAllAttributes: a null Label has no attribute.")?;
        let mut iterator = TdfAttributeIterator::from_node(node, true);
        while iterator.more() {
            let attribute = iterator.value();
            iterator.next();
            forget_from_node(node, &attribute)?;
        }
        if !clear_children {
            return Ok(());
        }
        let mut children = TdfChildIterator::new(self, false)?;
        while children.more() {
            children.value().forget_all_attributes(clear_children)?;
            children.next();
        }
        Ok(())
    }
    /// `ResumeAttribute(anAttribute)`: puts a forgotten, detached attribute back;
    /// `Standard_NullObject` for a null or attached attribute, `Standard_DomainError` for
    /// one that is not forgotten.
    pub fn resume_attribute(&self, attribute: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let node =
            self.live_node("TDF_Label::ResumeAttribute: a null Label cannot hold an attribute.")?;
        let resumed = attribute
            .get()
            .ok_or_else(|| exceptions::null_object("The attribute is a null handle."))?;
        if !resumed.label().is_null() {
            return Err(exceptions::null_object(
                "Cannot resume an attribute already attached to a label.",
            ));
        }
        if !resumed.is_forgotten() {
            return Err(exceptions::domain_error(
                "Cannot resume an unforgotten attribute.",
            ));
        }
        add_to_node(node, attribute, false)?;
        resume(resumed);
        if node_not_undo_mode(node) {
            resumed.after_resume()?;
        }
        Ok(())
    }
}

/// Whether the framework of the node is not undoing (`Data()->NotUndoMode()`).
fn node_not_undo_mode(node: &Arc<TdfLabelNode>) -> bool {
    node.core().is_none_or(|core| core.not_undo_mode())
}

/// `Standard_ImmutableObject` unless the framework allows modifications.
fn check_modification_allowed(
    node: &Arc<TdfLabelNode>,
    attribute: &dyn TdfAttribute,
    action: &str,
) -> Result<(), Failure> {
    if node
        .core()
        .is_none_or(|core| core.is_modification_allowed())
    {
        return Ok(());
    }
    Err(exceptions::immutable_object(&format!(
        "Attribute \"{}\" is {action} label outside transaction",
        attribute.dynamic_type().name()
    )))
}

/// Private `AddToNode`.
fn add_to_node(
    node: &Arc<TdfLabelNode>,
    attribute: &Handle<dyn TdfAttribute>,
    append: bool,
) -> Result<(), Failure> {
    let added = live(attribute, "TDF_Label::AddAttribute")?;
    check_modification_allowed(node, added, "added to")?;
    if !added.label().is_null() {
        return Err(exceptions::domain_error(
            "Attribute to add is already attached to a label.",
        ));
    }
    let label = TdfLabel::from_node(Some(node.clone()));
    if label.is_attribute(&added.id())? {
        return Err(exceptions::domain_error(
            "This label has already such an attribute.",
        ));
    }
    added.attribute_base().bind_self(attribute);
    let transaction = node.core().map_or(0, |core| core.transaction());
    added.attribute_base().set_transaction(transaction);
    added.attribute_base().set_saved_transaction(0);
    let after = if append {
        last_attribute(node)
    } else {
        Handle::default()
    };
    node.add_attribute(&after, attribute);
    node.set_attributes_modified(transaction != 0);
    if node_not_undo_mode(node) {
        added.after_addition()?;
    }
    Ok(())
}

/// The last valid attribute of the node (the native append loop).
fn last_attribute(node: &Arc<TdfLabelNode>) -> Handle<dyn TdfAttribute> {
    let mut iterator = TdfAttributeIterator::from_node(node, true);
    let mut last = Handle::default();
    while iterator.more() {
        last = iterator.value();
        iterator.next();
    }
    last
}

/// Private `ForgetFromNode`.
fn forget_from_node(
    node: &Arc<TdfLabelNode>,
    attribute: &Handle<dyn TdfAttribute>,
) -> Result<(), Failure> {
    let forgotten = live(attribute, "TDF_Label::ForgetAttribute")?;
    check_modification_allowed(node, forgotten, "removed from")?;
    if forgotten
        .attribute_base()
        .node()
        .is_none_or(|own| !Arc::ptr_eq(&own, node))
    {
        return Err(exceptions::domain_error(
            "Attribute to forget not attached to my label.",
        ));
    }
    if forgotten.is_forgotten() {
        return Ok(());
    }
    let current = node.core().map_or(0, |core| core.transaction());
    let created_now = forgotten.transaction() == current
        && forgotten.attribute_base().backup_attribute().is_null();
    if current == 0 || created_now {
        return remove_from_node(node, attribute, forgotten, current);
    }
    // Native comment: an attribute modified in the current transaction is forgotten
    // without restoring its previous version first.
    if node_not_undo_mode(node) {
        forgotten.before_forget()?;
    }
    forgotten.forget(current);
    Ok(())
}

/// The complete disappearance of an attribute: no transaction, or created in this one.
fn remove_from_node(
    node: &Arc<TdfLabelNode>,
    attribute: &Handle<dyn TdfAttribute>,
    forgotten: &dyn TdfAttribute,
    current: i32,
) -> Result<(), Failure> {
    let mut iterator = TdfAttributeIterator::from_node(node, false);
    let mut last = Handle::default();
    while iterator.more() {
        let value = iterator.value();
        if value == *attribute {
            if node_not_undo_mode(node) {
                forgotten.before_forget()?;
                forgotten.before_removal()?;
            }
            node.remove_attribute(&last, attribute);
            forgotten.forget(current);
            return Ok(());
        }
        last = value;
        iterator.next();
    }
    Ok(())
}
