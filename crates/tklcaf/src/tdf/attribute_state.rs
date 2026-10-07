// Rust port of ApplicationFramework/TKLCAF/TDF/TDF_Attribute.cxx and .lxx (non-virtual
// members) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The non-virtual members of `TDF_Attribute`, provided for every attribute.
use super::attribute_base::{BACKUP_MASK, FORGOTTEN_MASK, VALID_MASK};
use super::{TdfAttribute, TdfLabel};
use crate::live;
use rustycad_tkernel::standard::{exceptions, Failure, Guid, Handle, RuntimeTyped};

/// The non-virtual members of `TDF_Attribute` (ocaf-design.md section 4).
pub trait TdfAttributeState: TdfAttribute {
    /// `Label`: the label the attribute is attached to, null otherwise.
    fn label(&self) -> TdfLabel {
        self.attribute_base().label()
    }
    /// `Transaction`: the transaction in which it was created or last modified.
    fn transaction(&self) -> i32 {
        self.attribute_base().transaction()
    }
    /// `UntilTransaction`: the last transaction in which this version is valid;
    /// `Standard_DomainError` for an inconsistent structure (and for a valid attribute
    /// outside a framework, a crash natively).
    fn until_transaction(&self) -> Result<i32, Failure> {
        let base = self.attribute_base();
        if self.is_forgotten() {
            return Ok(base.transaction());
        }
        if self.is_backuped() {
            let next = base.next();
            return Ok(live(&next, "TDF_Attribute::UntilTransaction")?.transaction() - 1);
        }
        if self.is_valid() {
            if let Some(core) = base.node().and_then(|node| node.core()) {
                return Ok(core.transaction());
            }
        }
        Err(exceptions::domain_error(&format!(
            "The attribute structure is wrong. (flags {})",
            base.flags()
        )))
    }
    /// `IsValid`: neither a backup nor a removed attribute.
    fn is_valid(&self) -> bool {
        self.attribute_base().flags() & VALID_MASK != 0
    }
    /// `IsNew`: valid and without backup.
    fn is_new(&self) -> bool {
        self.is_valid() && self.attribute_base().backup_attribute().is_null()
    }
    /// `IsForgotten`.
    fn is_forgotten(&self) -> bool {
        self.attribute_base().flags() & FORGOTTEN_MASK != 0
    }
    /// `IsBackuped`.
    fn is_backuped(&self) -> bool {
        self.attribute_base().flags() & BACKUP_MASK != 0
    }
    /// `IsAttribute(anID)`: the label holds an attribute with that ID.
    fn is_attribute(&self, id: &Guid) -> Result<bool, Failure> {
        self.label().is_attribute(id)
    }
    /// `FindAttribute(anID, anAttribute)`: an attribute of the same label.
    fn find_attribute(&self, id: &Guid, attribute: &mut Handle<dyn TdfAttribute>) -> Result<bool, Failure> {
        self.label().find_attribute(id, attribute)
    }
    /// The template `FindAttribute<T>`: found and of type `T`.
    fn find_attribute_as<T: RuntimeTyped + ?Sized>(&self, id: &Guid, attribute: &mut Handle<T>) -> Result<bool, Failure>
    where
        Self: Sized,
    {
        self.label().find_attribute_as(id, attribute)
    }
    /// `AddAttribute(other)`: adds `other` to the label of this attribute.
    fn add_attribute(&self, other: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        self.label().add_attribute(other, true)
    }
    /// `ForgetAttribute(aguid)`: forgets the attribute of that ID on the same label.
    fn forget_attribute(&self, id: &Guid) -> Result<bool, Failure> {
        self.label().forget_attribute_id(id)
    }
    /// `ForgetAllAttributes(clearChildren)` on the label of this attribute.
    fn forget_all_attributes(&self, clear_children: bool) -> Result<(), Failure> {
        self.label().forget_all_attributes(clear_children)
    }
    /// `Backup`: keeps a copy of the current contents before the first modification in a
    /// transaction; nothing for an invalid or unattached attribute.
    /// `Standard_ImmutableObject` when the framework forbids modifications.
    fn backup(&self) -> Result<(), Failure> {
        let base = self.attribute_base();
        let Some(node) = base.node().filter(|_| self.is_valid()) else {
            return Ok(());
        };
        let Some(core) = node.core() else {
            return Ok(());
        };
        if !core.is_modification_allowed() {
            return Err(exceptions::immutable_object(&format!(
                "Attribute \"{}\" is changed outside transaction",
                self.dynamic_type().name()
            )));
        }
        let current = core.transaction();
        if base.transaction() >= current {
            return Ok(());
        }
        let backup = self.backup_copy()?;
        node.set_attributes_modified(true);
        let backup_base = live(&backup, "TDF_Attribute::Backup")?.attribute_base();
        {
            let own = base.state.read();
            let mut copy = backup_base.state.write();
            copy.label_node = own.label_node.clone();
            copy.backup = own.backup.clone();
            copy.transaction = own.transaction;
        }
        backup_base.set_next(&base.this()?);
        backup_base.set_backup_flag(true);
        let mut own = base.state.write();
        own.backup = backup;
        own.transaction = current;
        Ok(())
    }
    /// `Forget(aTransaction)`: the attribute becomes forgotten and invalid in `transaction`.
    fn forget(&self, transaction: i32) {
        let base = self.attribute_base();
        {
            let mut state = base.state.write();
            state.saved_transaction = state.transaction;
            state.transaction = transaction;
            state.flags |= FORGOTTEN_MASK;
        }
        if let Some(node) = base.node() {
            node.set_attributes_modified(true);
        }
        base.validate(false);
    }
}
impl<T: TdfAttribute + ?Sized> TdfAttributeState for T {}

/// Private `Resume`: undoes `Forget`; the saved transaction -1 marks "just resumed".
pub(crate) fn resume(attribute: &dyn TdfAttribute) {
    let base = attribute.attribute_base();
    {
        let mut state = base.state.write();
        state.transaction = state.saved_transaction;
        state.saved_transaction = -1;
        state.flags &= !FORGOTTEN_MASK;
    }
    base.validate(true);
}

/// Private `RemoveBackup`: drops the last backup, after its `BeforeRemoval`.
pub(crate) fn remove_backup(attribute: &dyn TdfAttribute) -> Result<(), Failure> {
    let base = attribute.attribute_base();
    let backup = base.backup_attribute();
    let old = live(&backup, "TDF_Attribute::RemoveBackup")?;
    old.before_removal()?;
    let older = {
        let mut state = old.attribute_base().state.write();
        state.label_node = std::sync::Weak::new();
        state.next = Handle::default();
        state.backup.clone()
    };
    base.state.write().backup = older.clone();
    if let Some(older) = older.get() {
        older.attribute_base().set_next(&base.this()?);
    }
    Ok(())
}
