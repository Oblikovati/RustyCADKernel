// Rust port of the private fields of ApplicationFramework/TKLCAF/TDF/TDF_Attribute.hxx and
// TDF_Attribute.lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TdfAttributeBase`: the private fields of `TDF_Attribute` that every attribute embeds
//! (ocaf-design.md section 4).
use super::{TdfAttribute, TdfLabel, label_node::TdfLabelNode};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, WeakHandle, exceptions};
use std::sync::{Arc, OnceLock, Weak};

/// `TDF_AttributeValidMsk`.
pub(crate) const VALID_MASK: i32 = 1;
/// `TDF_AttributeBackupMsk`.
pub(crate) const BACKUP_MASK: i32 = 2;
/// `TDF_AttributeForgottenMsk`.
pub(crate) const FORGOTTEN_MASK: i32 = 4;

#[derive(Clone)]
pub(crate) struct AttributeState {
    /// Native `TDF_LabelNode* myLabelNode`.
    pub(crate) label_node: Weak<TdfLabelNode>,
    pub(crate) transaction: i32,
    pub(crate) saved_transaction: i32,
    pub(crate) flags: i32,
    pub(crate) next: Handle<dyn TdfAttribute>,
    pub(crate) backup: Handle<dyn TdfAttribute>,
}

/// The private fields of `TDF_Attribute` and the self handle (ocaf-design.md section 2);
/// `TdfAttributeBase::default()` is the protected constructor (valid, transaction 0).
pub struct TdfAttributeBase {
    this: OnceLock<WeakHandle<dyn TdfAttribute>>,
    pub(crate) state: SharedState<AttributeState>,
}
impl Default for TdfAttributeBase {
    fn default() -> Self {
        Self {
            this: OnceLock::new(),
            state: SharedState::new(AttributeState {
                label_node: Weak::new(),
                transaction: 0,
                saved_transaction: 0,
                flags: VALID_MASK,
                next: Handle::default(),
                backup: Handle::default(),
            }),
        }
    }
}

impl TdfAttributeBase {
    /// Records the owning handle once; attaching an attribute to a label binds it.
    /// `attribute.attribute_base().bind_self(&attribute)`.
    pub fn bind_self(&self, attribute: &Handle<dyn TdfAttribute>) {
        let _ = self.this.set(attribute.downgrade());
    }
    /// The handle `this` converts to natively; `Standard_NullObject` before `bind_self`.
    pub fn this(&self) -> Result<Handle<dyn TdfAttribute>, Failure> {
        let this = self.this.get().map(WeakHandle::upgrade).unwrap_or_default();
        if this.is_null() {
            return Err(exceptions::null_object(
                "TDF_Attribute: the attribute has no owning handle; expected bind_self or attachment to a label",
            ));
        }
        Ok(this)
    }
    /// The label of the attribute (`Label()`): null when it is not attached.
    pub fn label(&self) -> TdfLabel {
        TdfLabel::from_node(self.state.read().label_node.upgrade())
    }
    pub(crate) fn node(&self) -> Option<Arc<TdfLabelNode>> {
        self.state.read().label_node.upgrade()
    }
    pub(crate) fn has_node(&self) -> bool {
        self.state.read().label_node.strong_count() > 0
    }
    /// What `TDF_LabelNode::AddAttribute` writes: valid flags only, and the node.
    pub(crate) fn attach(&self, node: Weak<TdfLabelNode>) {
        let mut state = self.state.write();
        state.flags = VALID_MASK;
        state.label_node = node;
    }
    /// What `TDF_LabelNode::RemoveAttribute` writes: no flag, no node.
    pub(crate) fn detach(&self) {
        let mut state = self.state.write();
        state.flags = 0;
        state.label_node = Weak::new();
    }
    pub(crate) fn next(&self) -> Handle<dyn TdfAttribute> {
        self.state.read().next.clone()
    }
    pub(crate) fn set_next(&self, next: &Handle<dyn TdfAttribute>) {
        self.state.write().next = next.clone();
    }
    pub(crate) fn backup_attribute(&self) -> Handle<dyn TdfAttribute> {
        self.state.read().backup.clone()
    }
    pub(crate) fn flags(&self) -> i32 {
        self.state.read().flags
    }
    /// Private `Validate`.
    pub(crate) fn validate(&self, status: bool) {
        let mut state = self.state.write();
        state.flags = if status { state.flags | VALID_MASK } else { state.flags & !VALID_MASK };
    }
    /// Private `Backup(bool)`: the backup flag, and the valid flag as its opposite.
    pub(crate) fn set_backup_flag(&self, status: bool) {
        {
            let mut state = self.state.write();
            state.flags = if status { state.flags | BACKUP_MASK } else { state.flags & !BACKUP_MASK };
        }
        self.validate(!status);
    }
    pub(crate) fn transaction(&self) -> i32 {
        self.state.read().transaction
    }
    pub(crate) fn set_transaction(&self, transaction: i32) {
        self.state.write().transaction = transaction;
    }
    pub(crate) fn saved_transaction(&self) -> i32 {
        self.state.read().saved_transaction
    }
    pub(crate) fn set_saved_transaction(&self, transaction: i32) {
        self.state.write().saved_transaction = transaction;
    }
}
