// Rust port of ApplicationFramework/TKLCAF/TDF/TDF_LabelNode.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDF_LabelNode`: one node of the label tree (ocaf-design.md section 3).
use super::{TdfAttribute, data::DataCore, TdfData};
use rustycad_tkernel::standard::{Handle, SharedState, WeakHandle};
use std::sync::{Arc, Weak};

/// `TDF_LabelNodeImportMsk` (the sign bit).
pub(crate) const IMPORT_MASK: i32 = i32::MIN;
/// `TDF_LabelNodeAttModMsk`.
pub(crate) const ATTRIBUTES_MODIFIED_MASK: i32 = 0x4000_0000;
/// `TDF_LabelNodeMayModMsk`.
pub(crate) const MAY_BE_MODIFIED_MASK: i32 = 0x2000_0000;
/// `TDF_LabelNodeFlagsMsk`: the three flags; the other bits hold the depth.
pub(crate) const FLAGS_MASK: i32 = IMPORT_MASK | ATTRIBUTES_MODIFIED_MASK | MAY_BE_MODIFIED_MASK;

/// The fields of `TDF_LabelNode`. The father and the data are raw pointers natively; the
/// children are owned through the first-child/brother chain.
#[derive(Clone, Default)]
pub(crate) struct NodeState {
    /// `None` for a root (and for a node detached by `TDF_Data::Destroy`).
    pub(crate) father: Option<Weak<TdfLabelNode>>,
    pub(crate) brother: Option<Arc<TdfLabelNode>>,
    pub(crate) first_child: Option<Arc<TdfLabelNode>>,
    pub(crate) last_found_child: Weak<TdfLabelNode>,
    pub(crate) tag: i32,
    pub(crate) flags: i32,
    pub(crate) first_attribute: Handle<dyn TdfAttribute>,
    /// The owning handle of the data framework (`Data()`).
    pub(crate) data: WeakHandle<TdfData>,
    /// The framework internals, alive while the data is being destroyed.
    pub(crate) core: Weak<DataCore>,
}

/// `TDF_LabelNode`, shared by the labels that designate it.
#[derive(Default)]
pub(crate) struct TdfLabelNode {
    pub(crate) state: SharedState<NodeState>,
}

impl TdfLabelNode {
    /// The child constructor `TDF_LabelNode(aTag, aFather)`: depth and data of the father.
    pub(crate) fn new_child(tag: i32, father: &Arc<Self>) -> Arc<Self> {
        let parent = father.state.read();
        let depth = (parent.flags & !FLAGS_MASK) + 1;
        Arc::new(Self {
            state: SharedState::new(NodeState {
                father: Some(Arc::downgrade(father)),
                tag,
                flags: depth,
                data: parent.data.clone(),
                core: parent.core.clone(),
                ..NodeState::default()
            }),
        })
    }
    pub(crate) fn father(&self) -> Option<Arc<Self>> {
        self.state.read().father.as_ref().and_then(Weak::upgrade)
    }
    pub(crate) fn is_root(&self) -> bool {
        self.state.read().father.is_none()
    }
    pub(crate) fn brother(&self) -> Option<Arc<Self>> {
        self.state.read().brother.clone()
    }
    pub(crate) fn first_child(&self) -> Option<Arc<Self>> {
        self.state.read().first_child.clone()
    }
    pub(crate) fn first_attribute(&self) -> Handle<dyn TdfAttribute> {
        self.state.read().first_attribute.clone()
    }
    pub(crate) fn tag(&self) -> i32 {
        self.state.read().tag
    }
    pub(crate) fn depth(&self) -> i32 {
        self.state.read().flags & !FLAGS_MASK
    }
    pub(crate) fn core(&self) -> Option<Arc<DataCore>> {
        self.state.read().core.upgrade()
    }
    fn flag(&self, mask: i32) -> bool {
        self.state.read().flags & mask != 0
    }
    fn set_flag(&self, mask: i32, status: bool) {
        let mut state = self.state.write();
        state.flags = if status { state.flags | mask } else { state.flags & !mask };
    }
    pub(crate) fn is_imported(&self) -> bool {
        self.flag(IMPORT_MASK)
    }
    pub(crate) fn set_imported(&self, status: bool) {
        self.set_flag(IMPORT_MASK, status);
    }
    pub(crate) fn attributes_modified(&self) -> bool {
        self.flag(ATTRIBUTES_MODIFIED_MASK)
    }
    /// `AttributesModified(bool)`: setting it also marks every ancestor `MayBeModified`.
    pub(crate) fn set_attributes_modified(&self, status: bool) {
        self.set_flag(ATTRIBUTES_MODIFIED_MASK, status);
        if status {
            self.all_may_be_modified();
        }
    }
    pub(crate) fn may_be_modified(&self) -> bool {
        self.flag(MAY_BE_MODIFIED_MASK)
    }
    pub(crate) fn set_may_be_modified(&self, status: bool) {
        self.set_flag(MAY_BE_MODIFIED_MASK, status);
    }
    /// `AllMayBeModified`: this node and its ancestors up to the first one already marked.
    pub(crate) fn all_may_be_modified(&self) {
        self.set_may_be_modified(true);
        let mut father = self.father();
        while let Some(node) = father.filter(|node| !node.may_be_modified()) {
            node.set_may_be_modified(true);
            father = node.father();
        }
    }
    /// `AddAttribute(afterAtt, newAtt)`: links `new` after `after` (at the head for null)
    /// and makes it valid on this node.
    pub(crate) fn add_attribute(
        self: &Arc<Self>,
        after: &Handle<dyn TdfAttribute>,
        new: &Handle<dyn TdfAttribute>,
    ) {
        let Some(new_attribute) = new.get() else {
            return;
        };
        let base = new_attribute.attribute_base();
        base.attach(Arc::downgrade(self));
        match after.get() {
            None => {
                let mut state = self.state.write();
                base.set_next(&state.first_attribute);
                state.first_attribute = new.clone();
            }
            Some(after_attribute) => {
                let after_base = after_attribute.attribute_base();
                base.set_next(&after_base.next());
                after_base.set_next(new);
            }
        }
    }
    /// `RemoveAttribute(afterAtt, oldAtt)`: unlinks `old`, which keeps its `myNext` so that
    /// an iterator standing on it continues (native comment: nullifying it breaks the
    /// attribute iterator).
    pub(crate) fn remove_attribute(
        &self,
        after: &Handle<dyn TdfAttribute>,
        old: &Handle<dyn TdfAttribute>,
    ) {
        let Some(old_attribute) = old.get() else {
            return;
        };
        let base = old_attribute.attribute_base();
        base.detach();
        let next = base.next();
        match after.get() {
            None => self.state.write().first_attribute = next,
            Some(after_attribute) => after_attribute.attribute_base().set_next(&next),
        }
    }
    /// `Destroy`: frees the sub-tree in a loop over the children (MSV 21.03.2003: no
    /// recursion through brothers, to avoid a stack overflow) and resets every field.
    pub(crate) fn destroy(&self) {
        let mut child = self.state.write().first_child.take();
        while let Some(node) = child {
            child = node.state.write().brother.take();
            node.destroy();
        }
        *self.state.write() = NodeState::default();
    }
}
