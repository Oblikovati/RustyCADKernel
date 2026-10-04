// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT/MAT_TList.gxx/.lxx, instantiated by MAT_ListOfBisector.hxx/_0.cxx and MAT_ListOfEdge.hxx/_0.cxx (docs/topalgo-design.md section 3) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MAT_TList`: the doubly linked list of MAT with a current position, ported once
//! as a generic over its item.
mod edits;
mod node;

pub use node::MatTListNode;

use rustycad_tkernel::standard::{
    Failure, Handle, RuntimeType, RuntimeTyped, SharedState, StandardTransient, Transient,
    project_concrete,
};
use rustycad_tkg2d::geom2d::errors;
use std::any::{Any, TypeId};

/// The item of a [`MatTList`]: the native class names of the instance and the
/// `Dump` the list calls on each item.
pub trait MatTListItem: RuntimeTyped + Transient + Sized {
    /// The class name of the list instance (`MAT_ListOfBisector`).
    const LIST_NAME: &'static str;
    /// The class name of the node instance (`MAT_TListNodeOfListOfBisector`).
    const NODE_NAME: &'static str;
    /// `Item->Dump(ashift, alevel)`; `item.dump_item(0, 1, &mut text)?`.
    fn dump_item(&self, ashift: i32, alevel: i32, out: &mut String) -> Result<(), Failure>;
}

pub(super) struct ListState<I: MatTListItem> {
    pub first: Handle<MatTListNode<I>>,
    pub last: Handle<MatTListNode<I>>,
    pub current: Handle<MatTListNode<I>>,
    pub index: i32,
    pub count: i32,
}

/// `MAT_TList` (`MAT_ListOfBisector`, `MAT_ListOfEdge`): a transient doubly linked list
/// of handles with a current node and its index (1-based, 0 before the first
/// `First`). Members that dereference the current node fail with
/// Standard_NullObject where the native code dereferences a null handle.
/// ```
/// use rustycad_tktopalgo::mat::{MatEdge, MatListOfEdge};
/// use rustycad_tkernel::standard::Handle;
/// let list = MatListOfEdge::new();
/// list.back_add(&Handle::new(MatEdge::new()));
/// list.first();
/// assert!(list.more() && list.number() == 1 && list.index() == 1);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct MatTList<I: MatTListItem> {
    parent: StandardTransient,
    state: SharedState<ListState<I>>,
}

impl<I: MatTListItem> Default for MatTList<I> {
    fn default() -> Self {
        Self::new()
    }
}

// The node a handle holds, or the native null dereference as Standard_NullObject.
pub(super) fn node<'a, I: MatTListItem>(
    handle: &'a Handle<MatTListNode<I>>,
    member: &str,
) -> Result<&'a MatTListNode<I>, Failure> {
    handle.get().ok_or_else(|| {
        errors::null_object(&format!(
            "{}::{member}: no node there; expected a current node",
            I::LIST_NAME
        ))
    })
}

impl<I: MatTListItem> MatTList<I> {
    /// `MAT_TList()`: an empty list; `MatListOfEdge::new()`.
    pub fn new() -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(ListState {
                first: Handle::default(),
                last: Handle::default(),
                current: Handle::default(),
                index: 0,
                count: 0,
            }),
        }
    }
    /// `First()`: the first node becomes current; `list.first()`.
    pub fn first(&self) {
        let mut state = self.state.write();
        state.current = state.first.clone();
        state.index = 1;
    }
    /// `Last()`: the last node becomes current; `list.last()`.
    pub fn last(&self) {
        let mut state = self.state.write();
        state.current = state.last.clone();
        state.index = state.count;
    }
    /// `Init(anitem)`: from the first node, the first node of `anitem` becomes current
    /// (the list runs past its end when the item is absent; a looped list then loops
    /// forever, as natively); `list.init(&edge)?`.
    pub fn init(&self, anitem: &Handle<I>) -> Result<(), Failure> {
        self.first();
        while self.more() {
            if *anitem == self.current()? {
                break;
            }
            self.next()?;
        }
        Ok(())
    }
    /// `Next()`: the next node becomes current (nothing on an empty list);
    /// `list.next()?`.
    pub fn next(&self) -> Result<(), Failure> {
        let mut state = self.state.write();
        if state.count == 0 {
            return Ok(());
        }
        state.current = node(&state.current, "Next")?.next();
        state.index = (state.index % state.count) + 1;
        Ok(())
    }
    /// `Previous()`: the previous node becomes current (nothing on an empty list);
    /// `list.previous()?`.
    pub fn previous(&self) -> Result<(), Failure> {
        let mut state = self.state.write();
        if state.count == 0 {
            return Ok(());
        }
        state.current = node(&state.current, "Previous")?.previous();
        state.index = ((state.index + state.count - 2) % state.count) + 1;
        Ok(())
    }
    /// `More()`: true while there is a current node; `list.more()`.
    pub fn more(&self) -> bool {
        self.state.read().current.is_some()
    }
    /// `Current()`: the current item; `list.current()?`.
    pub fn current(&self) -> Result<Handle<I>, Failure> {
        Ok(node(&self.state.read().current, "Current")?.get_item())
    }
    /// `Current(anitem)`: replaces the current item; `list.set_current(&edge)?`.
    pub fn set_current(&self, anitem: &Handle<I>) -> Result<(), Failure> {
        node(&self.state.read().current, "Current")?.set_item(anitem);
        Ok(())
    }
    /// `FirstItem()`; `list.first_item()?`.
    pub fn first_item(&self) -> Result<Handle<I>, Failure> {
        Ok(node(&self.state.read().first, "FirstItem")?.get_item())
    }
    /// `LastItem()`; `list.last_item()?`.
    pub fn last_item(&self) -> Result<Handle<I>, Failure> {
        Ok(node(&self.state.read().last, "LastItem")?.get_item())
    }
    /// `PreviousItem()`: the item before the current node; `list.previous_item()?`.
    pub fn previous_item(&self) -> Result<Handle<I>, Failure> {
        let previous = node(&self.state.read().current, "PreviousItem")?.previous();
        Ok(node(&previous, "PreviousItem")?.get_item())
    }
    /// `NextItem()`: the item after the current node; `list.next_item()?`.
    pub fn next_item(&self) -> Result<Handle<I>, Failure> {
        let next = node(&self.state.read().current, "NextItem")?.next();
        Ok(node(&next, "NextItem")?.get_item())
    }
    /// `Number()`: the number of items; `list.number()`.
    pub fn number(&self) -> i32 {
        self.state.read().count
    }
    /// `Index()`: the index of the current node; `list.index()`.
    pub fn index(&self) -> i32 {
        self.state.read().index
    }
    /// `IsEmpty()`; `list.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.state.read().count == 0
    }
    /// `Brackets(anindex)` (`operator()`): moves the current node to `anindex` step
    /// by step and returns its item; `list.brackets(2)?`.
    pub fn brackets(&self, anindex: i32) -> Result<Handle<I>, Failure> {
        let mut state = self.state.write();
        while state.index > anindex {
            state.index -= 1;
            state.current = node(&state.current, "Brackets")?.previous();
        }
        while state.index < anindex {
            state.index += 1;
            state.current = node(&state.current, "Brackets")?.next();
        }
        Ok(node(&state.current, "Brackets")?.get_item())
    }
    /// `Dump(ashift, alevel)`: the Dump of every item from the first one (the current
    /// node ends past the last); `list.dump(0, 1, &mut text)?`.
    pub fn dump(&self, ashift: i32, alevel: i32, out: &mut String) -> Result<(), Failure> {
        self.first();
        while self.more() {
            self.current()?.dump_item(ashift, alevel, out)?;
            self.next()?;
        }
        Ok(())
    }
}

impl<I: MatTListItem> Drop for MatTList<I> {
    // The destructor unlinks every node from the first one, so the cycles of the
    // strong links (and of a looped list) are broken.
    fn drop(&mut self) {
        let state = self.state.write();
        let mut a_node: Handle<MatTListNode<I>> = state.first.clone();
        drop(state);
        while let Some(current) = a_node.get() {
            let a_next = current.next();
            current.set_next(&Handle::default());
            current.set_previous(&Handle::default());
            a_node = a_next;
        }
    }
}

impl<I: MatTListItem> RuntimeTyped for MatTList<I> {
    const TYPE_NAME: &'static str = I::LIST_NAME;
    fn static_type() -> &'static RuntimeType {
        // The registry is keyed by the Rust type, so each instance keeps its own
        // descriptor (a static in a generic function would be shared).
        RuntimeType::retain_class_descriptor::<Self>(RuntimeType::register::<Self>(
            I::LIST_NAME,
            std::mem::size_of::<Self>(),
            Some(StandardTransient::static_type()),
        ))
    }
    fn project_from(object: &dyn Transient) -> Option<&Self> {
        project_concrete::<Self>(object)
    }
}

impl<I: MatTListItem> Transient for MatTList<I> {
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
