// Rust port of ApplicationFramework/TKLCAF/TDataStd/TDataStd_TreeNode.hxx, .lxx and .cxx
// (fields, constructors, structure and accessors) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDataStd_TreeNode`: a node of an explicit tree laid over labels. The native raw
//! pointers `myFather`, `myPrevious`, `myNext`, `myFirst` and `myLast` are weak handles:
//! they do not own the nodes (the labels do), and a dangling one reads as null.
use crate::guid_from_text;
use crate::live;
use crate::tdf::{TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfLabel};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Guid, Handle, SharedState, StandardTransient, WeakHandle, exceptions},
};

/// `TDataStd_TreeNode::GetDefaultTreeID()`.
const DEFAULT_TREE_ID: Guid = guid_from_text("2a96b621-ec8b-11d0-bee7-080009dc3333");

/// The native private fields.
#[derive(Clone, Default)]
pub(crate) struct TreeNodeState {
    pub(crate) father: WeakHandle<TDataStdTreeNode>,
    pub(crate) previous: WeakHandle<TDataStdTreeNode>,
    pub(crate) next: WeakHandle<TDataStdTreeNode>,
    pub(crate) first: WeakHandle<TDataStdTreeNode>,
    pub(crate) last: WeakHandle<TDataStdTreeNode>,
    pub(crate) tree_id: Guid,
}

/// `TDataStd_TreeNode`; `TDataStdTreeNode::set(&label)`. The structure members need the
/// node's own handle (`this`), bound when the node is added to a label.
///
/// ```
/// use rustycad_tklcaf::{tdata_std::TDataStdTreeNode, tdf::TdfData};
/// let data = TdfData::new();
/// let root = TDataStdTreeNode::set(&data.root()).unwrap();
/// let child = TDataStdTreeNode::set(&data.root().find_child(1, true).unwrap()).unwrap();
/// root.append(&child).unwrap();
/// assert_eq!((root.nb_children(false), child.depth()), (1, 1));
/// ```
pub struct TDataStdTreeNode {
    transient: StandardTransient,
    pub(crate) base: TdfAttributeBase,
    pub(crate) state: SharedState<TreeNodeState>,
}
impl_transient!(TDataStdTreeNode, "TDataStd_TreeNode", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl Default for TDataStdTreeNode {
    /// `TDataStd_TreeNode()`: no links. The native constructor leaves `myTreeID`
    /// default-constructed (the null GUID); `Set` and `NewEmpty` give the tree ID.
    fn default() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: TdfAttributeBase::default(),
            state: SharedState::new(TreeNodeState::default()),
        }
    }
}

/// The address a native `TDataStd_TreeNode*` compares by.
fn address_of(node: &TDataStdTreeNode) -> *const () {
    std::ptr::from_ref(node).cast()
}

impl TDataStdTreeNode {
    /// `Find(L, T)`: the node of the default tree on the label.
    pub fn find(label: &TdfLabel, node: &mut Handle<TDataStdTreeNode>) -> Result<bool, Failure> {
        label.find_attribute_as(&DEFAULT_TREE_ID, node)
    }
    /// `Set(L)`: the node of the default tree, added when absent.
    pub fn set(label: &TdfLabel) -> Result<Handle<Self>, Failure> {
        Self::set_with_id(label, &DEFAULT_TREE_ID)
    }
    /// `Set(L, explicitID)`: the node of that tree, added (with the tree ID) when absent.
    pub fn set_with_id(label: &TdfLabel, explicit_id: &Guid) -> Result<Handle<Self>, Failure> {
        let mut found: Handle<Self> = Handle::default();
        if label.find_attribute_as(explicit_id, &mut found)? {
            return Ok(found);
        }
        let created = Handle::new(Self::new());
        live(&created, "TDataStd_TreeNode::Set")?.set_tree_id(explicit_id);
        label.add_attribute(&created.cast(), false)?;
        Ok(created)
    }
    /// `GetDefaultTreeID`.
    pub fn get_default_tree_id() -> Guid {
        DEFAULT_TREE_ID
    }
    /// `TDataStd_TreeNode()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `this` as a node handle; `Standard_NullObject` for a node never added to a label.
    pub(crate) fn this_node(&self) -> Result<Handle<Self>, Failure> {
        Ok(self.base.this()?.cast())
    }
    /// The node a link names (null when unset or dangling).
    fn linked(&self, pick: fn(&TreeNodeState) -> &WeakHandle<Self>) -> Handle<Self> {
        pick(&self.state.read()).upgrade()
    }
    /// `Standard_DomainError` unless `node` belongs to this tree (the `Append`-family check).
    fn check_tree(&self, node: &TDataStdTreeNode, member: &str) -> Result<(), Failure> {
        if node.id() == self.id() {
            return Ok(());
        }
        let message = format!(
            "TDataStd_TreeNode::{member} : uncompatible GUID {}; expected {}",
            node.id(),
            self.id()
        );
        Err(exceptions::domain_error(&message))
    }
    /// `Append(Child)`: `Child` becomes the last child; `Standard_DomainError` for another tree.
    pub fn append(&self, child: &Handle<Self>) -> Result<bool, Failure> {
        let node = live(child, "TDataStd_TreeNode::Append")?;
        self.check_tree(node, "Append")?;
        node.set_next(&Handle::default())?;
        if !self.has_first() {
            self.set_first(child)?;
            node.set_previous(&Handle::default())?;
        } else {
            let last = self.last();
            live(&last, "TDataStd_TreeNode::Append")?.set_next(child)?;
            node.set_previous(&last)?;
        }
        node.set_father(&self.this_node()?)?;
        self.state.write().last = child.downgrade();
        Ok(true)
    }
    /// `Prepend(Child)`: `Child` becomes the first child; `Standard_DomainError` for another tree.
    pub fn prepend(&self, child: &Handle<Self>) -> Result<bool, Failure> {
        let node = live(child, "TDataStd_TreeNode::Prepend")?;
        self.check_tree(node, "Prepend")?;
        node.set_previous(&Handle::default())?;
        if self.has_first() {
            let first = self.first();
            node.set_next(&first)?;
            live(&first, "TDataStd_TreeNode::Prepend")?.set_previous(child)?;
        } else {
            node.set_next(&Handle::default())?;
            self.set_last(child)?;
        }
        node.set_father(&self.this_node()?)?;
        self.set_first(child)?;
        Ok(true)
    }
    /// `InsertBefore(Node)`: `Node` becomes the previous brother; `Standard_NullObject`
    /// for a root without father (a null dereference natively).
    pub fn insert_before(&self, brother: &Handle<Self>) -> Result<bool, Failure> {
        let node = live(brother, "TDataStd_TreeNode::InsertBefore")?;
        self.check_tree(node, "InsertBefore")?;
        node.set_father(&self.father())?;
        node.set_previous(&self.previous())?;
        node.set_next(&self.this_node()?)?;
        if !self.has_previous() {
            live(&self.father(), "TDataStd_TreeNode::InsertBefore")?.set_first(brother)?;
        } else {
            live(&self.previous(), "TDataStd_TreeNode::InsertBefore")?.set_next(brother)?;
        }
        self.set_previous(brother)?;
        Ok(true)
    }
    /// `InsertAfter(Node)`: `Node` becomes the next brother.
    pub fn insert_after(&self, brother: &Handle<Self>) -> Result<bool, Failure> {
        let node = live(brother, "TDataStd_TreeNode::InsertAfter")?;
        self.check_tree(node, "InsertAfter")?;
        if self.has_father() && !self.has_next() {
            live(&self.father(), "TDataStd_TreeNode::InsertAfter")?.set_last(brother)?;
        }
        node.set_father(&self.father())?;
        node.set_previous(&self.this_node()?)?;
        node.set_next(&self.next())?;
        if self.has_next() {
            live(&self.next(), "TDataStd_TreeNode::InsertAfter")?.set_previous(brother)?;
        }
        self.set_next(brother)?;
        Ok(true)
    }
    /// `Remove`: detaches the node from its father and brothers (nothing for a root).
    pub fn remove(&self) -> Result<bool, Failure> {
        if self.is_root() {
            return Ok(true);
        }
        let (father, previous, next) = (self.father(), self.previous(), self.next());
        let null = Handle::default();
        match previous.get() {
            None => live(&father, "TDataStd_TreeNode::Remove")?.set_first(&next)?,
            Some(previous) => previous.set_next(&next)?,
        }
        match (next.get(), previous.is_some()) {
            (Some(next), true) => next.set_previous(&previous)?,
            (Some(next), false) => next.set_previous(&null)?,
            (None, true) => live(&previous, "TDataStd_TreeNode::Remove")?.set_next(&null)?,
            (None, false) => {}
        }
        self.unlink_from_father(&father, &next)?;
        self.set_father(&null)?;
        self.set_next(&null)?;
        self.set_previous(&null)?;
        Ok(true)
    }
    /// The father part of `Remove`: its first child moves past this node and its last
    /// child is forgotten.
    fn unlink_from_father(
        &self,
        father: &Handle<Self>,
        next: &Handle<Self>,
    ) -> Result<(), Failure> {
        let father = live(father, "TDataStd_TreeNode::Remove")?;
        if father.has_first() && father.first().object_address() == address_of(self) {
            father.set_first(next)?;
        }
        if father.has_last() {
            father.set_last(&Handle::default())?;
        }
        Ok(())
    }
    /// `Depth`: the number of fathers above the node.
    pub fn depth(&self) -> i32 {
        let mut depth = 0;
        let mut node = self.father();
        while let Some(current) = node.get() {
            depth += 1;
            node = current.father();
        }
        depth
    }
    /// `NbChildren(allLevels)`: the direct children, or all the descendants.
    pub fn nb_children(&self, all_levels: bool) -> i32 {
        let mut count = 0;
        let mut child = self.first();
        while let Some(current) = child.get() {
            if all_levels && current.has_first() {
                count += current.nb_children(all_levels);
            }
            count += 1;
            child = current.next();
        }
        count
    }
    /// `IsAscendant(of)`: `of` descends from this node.
    pub fn is_ascendant(&self, of: &Handle<Self>) -> Result<bool, Failure> {
        let of = live(of, "TDataStd_TreeNode::IsAscendant")?;
        Ok(of.has_ancestor_at(address_of(self)))
    }
    /// `IsDescendant(of)`: `of` is a father of a father ... of this node.
    pub fn is_descendant(&self, of: &Handle<Self>) -> bool {
        self.has_ancestor_at(of.object_address())
    }
    /// The native father walk of `IsDescendant`, comparing addresses.
    fn has_ancestor_at(&self, address: *const ()) -> bool {
        let mut node = self.father();
        while let Some(current) = node.get() {
            if address_of(current) == address {
                return true;
            }
            node = current.father();
        }
        false
    }
    /// `IsFather(of)`: this node is the father of `of`.
    pub fn is_father(&self, of: &Handle<Self>) -> Result<bool, Failure> {
        let of = live(of, "TDataStd_TreeNode::IsFather")?;
        Ok(of.father().object_address() == address_of(self))
    }
    /// `IsChild(of)`: `of` is the father of this node.
    pub fn is_child(&self, of: &Handle<Self>) -> bool {
        self.father().object_address() == of.object_address()
    }
    /// `IsRoot`: no father, no brothers.
    pub fn is_root(&self) -> bool {
        !self.has_father() && !self.has_previous() && !self.has_next()
    }
    /// `Root`: the topmost father (this node for a root); `Standard_NullObject` for an
    /// unbound root node.
    pub fn root(&self) -> Result<Handle<Self>, Failure> {
        let mut node = self.this_node();
        let mut father = self.father();
        while let Some(current) = father.get() {
            let up = current.father();
            node = Ok(father.clone());
            father = up;
        }
        node
    }
    /// `HasFather`.
    pub fn has_father(&self) -> bool {
        self.father().is_some()
    }
    /// `Father`: null for a root.
    pub fn father(&self) -> Handle<Self> {
        self.linked(|state| &state.father)
    }
    /// `HasNext`.
    pub fn has_next(&self) -> bool {
        self.next().is_some()
    }
    /// `Next`: null for the last brother.
    pub fn next(&self) -> Handle<Self> {
        self.linked(|state| &state.next)
    }
    /// `HasPrevious`.
    pub fn has_previous(&self) -> bool {
        self.previous().is_some()
    }
    /// `Previous`: null for the first brother.
    pub fn previous(&self) -> Handle<Self> {
        self.linked(|state| &state.previous)
    }
    /// `HasFirst`.
    pub fn has_first(&self) -> bool {
        self.first().is_some()
    }
    /// `First`: the first child.
    pub fn first(&self) -> Handle<Self> {
        self.linked(|state| &state.first)
    }
    /// `HasLast`: the cached last child is set.
    pub fn has_last(&self) -> bool {
        self.linked(|state| &state.last).is_some()
    }
    /// `Last`: the cached last child when it is still a child of this node, else
    /// `FindLast` (the stale cache is dropped without backup, as natively).
    pub fn last(&self) -> Handle<Self> {
        let cached = self.linked(|state| &state.last);
        if cached
            .get()
            .is_some_and(|last| last.father().object_address() != address_of(self))
        {
            self.state.write().last = WeakHandle::default();
            return self.find_last();
        }
        if cached.is_null() {
            return self.find_last();
        }
        cached
    }
    /// `FindLast`: the last child, by walking the brothers.
    pub fn find_last(&self) -> Handle<Self> {
        let mut last = self.first();
        while let Some(next) = last.get().map(Self::next).filter(Handle::is_some) {
            last = next;
        }
        last
    }
    /// `SetTreeID` (no backup, as natively).
    pub fn set_tree_id(&self, explicit_id: &Guid) {
        self.state.write().tree_id = *explicit_id;
    }
    /// Backs up, then sets a link; every link but `myLast` also drops the cached last child.
    fn set_link(
        &self,
        node: &Handle<Self>,
        slot: fn(&mut TreeNodeState) -> &mut WeakHandle<Self>,
        drops_last: bool,
    ) -> Result<(), Failure> {
        self.backup()?;
        let mut state = self.state.write();
        *slot(&mut state) = node.downgrade();
        if drops_last {
            state.last = WeakHandle::default();
        }
        Ok(())
    }
    /// `SetFather`.
    pub fn set_father(&self, node: &Handle<Self>) -> Result<(), Failure> {
        self.set_link(node, |state| &mut state.father, true)
    }
    /// `SetNext`.
    pub fn set_next(&self, node: &Handle<Self>) -> Result<(), Failure> {
        self.set_link(node, |state| &mut state.next, true)
    }
    /// `SetPrevious`.
    pub fn set_previous(&self, node: &Handle<Self>) -> Result<(), Failure> {
        self.set_link(node, |state| &mut state.previous, true)
    }
    /// `SetFirst`.
    pub fn set_first(&self, node: &Handle<Self>) -> Result<(), Failure> {
        self.set_link(node, |state| &mut state.first, true)
    }
    /// `SetLast`.
    pub fn set_last(&self, node: &Handle<Self>) -> Result<(), Failure> {
        self.set_link(node, |state| &mut state.last, false)
    }
}
