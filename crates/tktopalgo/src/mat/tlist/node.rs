// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT/MAT_TListNodeOfListOfBisector.hxx/_0.cxx and MAT_TListNodeOfListOfEdge.hxx/_0.cxx (one generic, docs/topalgo-design.md section 3) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The node of the doubly linked lists of MAT: an item and its two neighbours.
use super::MatTListItem;
use rustycad_tkernel::standard::{
    Handle, RuntimeType, RuntimeTyped, SharedState, StandardTransient, Transient, project_concrete,
};
use std::any::{Any, TypeId};

struct NodeLinks<I: MatTListItem> {
    next: Handle<MatTListNode<I>>,
    previous: Handle<MatTListNode<I>>,
    item: Handle<I>,
}

/// `MAT_TListNodeOfListOfBisector` and `MAT_TListNodeOfListOfEdge`: a node of a
/// [`super::MatTList`], a transient that holds an item and strong handles to its next
/// and previous nodes (the list breaks the cycles when it is dropped).
/// ```
/// use rustycad_tktopalgo::mat::{MatEdge, MatTListNodeOfListOfEdge};
/// use rustycad_tkernel::standard::Handle;
/// let node = MatTListNodeOfListOfEdge::with_item(&Handle::new(MatEdge::new()));
/// assert!(node.next().is_null() && node.get_item().is_some());
/// ```
pub struct MatTListNode<I: MatTListItem> {
    parent: StandardTransient,
    links: SharedState<NodeLinks<I>>,
}

impl<I: MatTListItem> Default for MatTListNode<I> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I: MatTListItem> MatTListNode<I> {
    /// `MAT_TListNode()`: no item, no neighbours; `MatTListNodeOfListOfEdge::new()`.
    pub fn new() -> Self {
        Self::with_item(&Handle::default())
    }
    /// `MAT_TListNode(anitem)`: a node of `anitem`; `MatTListNodeOfListOfEdge::with_item(&edge)`.
    pub fn with_item(anitem: &Handle<I>) -> Self {
        Self {
            parent: StandardTransient::default(),
            links: SharedState::new(NodeLinks {
                next: Handle::default(),
                previous: Handle::default(),
                item: anitem.clone(),
            }),
        }
    }
    /// `GetItem()`; `node.get_item()`.
    pub fn get_item(&self) -> Handle<I> {
        self.links.read().item.clone()
    }
    /// `Next()`: the next node; `node.next()`.
    pub fn next(&self) -> Handle<Self> {
        self.links.read().next.clone()
    }
    /// `Previous()`: the previous node; `node.previous()`.
    pub fn previous(&self) -> Handle<Self> {
        self.links.read().previous.clone()
    }
    /// `SetItem(anitem)`; `node.set_item(&edge)`.
    pub fn set_item(&self, anitem: &Handle<I>) {
        self.links.write().item = anitem.clone();
    }
    /// `Next(atlistnode)`: sets the next node; `node.set_next(&other)`.
    pub fn set_next(&self, atlistnode: &Handle<Self>) {
        self.links.write().next = atlistnode.clone();
    }
    /// `Previous(atlistnode)`: sets the previous node; `node.set_previous(&other)`.
    pub fn set_previous(&self, atlistnode: &Handle<Self>) {
        self.links.write().previous = atlistnode.clone();
    }
    /// `Dummy()`: does nothing; `node.dummy()`.
    pub fn dummy(&self) {}
}

impl<I: MatTListItem> RuntimeTyped for MatTListNode<I> {
    const TYPE_NAME: &'static str = I::NODE_NAME;
    fn static_type() -> &'static RuntimeType {
        // The registry is keyed by the Rust type, so each instance keeps its own
        // descriptor (a static in a generic function would be shared).
        RuntimeType::retain_class_descriptor::<Self>(RuntimeType::register::<Self>(
            I::NODE_NAME,
            std::mem::size_of::<Self>(),
            Some(StandardTransient::static_type()),
        ))
    }
    fn project_from(object: &dyn Transient) -> Option<&Self> {
        project_concrete::<Self>(object)
    }
}

impl<I: MatTListItem> Transient for MatTListNode<I> {
    fn dynamic_type(&self) -> &'static RuntimeType {
        Self::static_type()
    }
    fn project(&self, requested: TypeId) -> Option<&dyn Any> {
        if requested == TypeId::of::<Self>() {
            return Some(self);
        }
        self.parent.project(requested)
    }
    fn transient_base(&self) -> &StandardTransient {
        &self.parent
    }
}
