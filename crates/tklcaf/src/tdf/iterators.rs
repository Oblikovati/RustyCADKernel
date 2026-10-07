// Rust port of ApplicationFramework/TKLCAF/TDF/TDF_AttributeIterator, TDF_ChildIterator and
// TDF_ChildIDIterator (.hxx and .cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The iterators over the attributes of a label and over its children.
use super::{TdfAttribute, TdfAttributeState, TdfLabel, label_node::TdfLabelNode};
use rustycad_tkernel::standard::{Failure, Guid, Handle};
use std::sync::Arc;

/// `TDF_AttributeIterator`: the attributes of a label, skipping forgotten ones unless asked.
/// It walks the native `myNext` chain, so it survives the removal of the current attribute.
///
/// ```
/// use rustycad_tklcaf::tdf::{TdfAttributeIterator, TdfData, TdfTagSource};
/// let data = TdfData::new();
/// TdfTagSource::set(&data.root()).unwrap();
/// let mut iterator = TdfAttributeIterator::new(&data.root(), true).unwrap();
/// assert!(iterator.more());
/// iterator.next();
/// assert!(!iterator.more());
/// ```
#[derive(Default)]
pub struct TdfAttributeIterator {
    value: Handle<dyn TdfAttribute>,
    without_forgotten: bool,
}
impl TdfAttributeIterator {
    /// `TDF_AttributeIterator(aLabel, withoutForgotten)`; `Standard_NullObject` for a null
    /// label (a crash natively).
    pub fn new(label: &TdfLabel, without_forgotten: bool) -> Result<Self, Failure> {
        let mut iterator = Self::default();
        iterator.initialize(label, without_forgotten)?;
        Ok(iterator)
    }
    /// `TDF_AttributeIterator(aLabelNode, withoutForgotten)`.
    pub(crate) fn from_node(node: &Arc<TdfLabelNode>, without_forgotten: bool) -> Self {
        let mut iterator = Self {
            value: Handle::default(),
            without_forgotten,
        };
        iterator.go_to_next(node.first_attribute());
        iterator
    }
    /// `Initialize(aLabel, withoutForgotten)`.
    pub fn initialize(&mut self, label: &TdfLabel, without_forgotten: bool) -> Result<(), Failure> {
        let node = label.live_node("TDF_AttributeIterator: a null Label has no attribute.")?;
        self.without_forgotten = without_forgotten;
        self.value = Handle::default();
        self.go_to_next(node.first_attribute());
        Ok(())
    }
    /// `More`; `iterator.more()`.
    pub fn more(&self) -> bool {
        self.value.is_some()
    }
    /// `Next`: the following attribute through `myNext`.
    pub fn next(&mut self) {
        let next = self.value.get().map(|value| value.attribute_base().next());
        match next {
            Some(next) => self.go_to_next(next),
            None => self.value = Handle::default(),
        }
    }
    /// `Value` (and `PtrValue`): the current attribute, null past the end.
    pub fn value(&self) -> Handle<dyn TdfAttribute> {
        self.value.clone()
    }
    /// Private `goToNext`: `attribute`, or the first not forgotten one after it.
    fn go_to_next(&mut self, attribute: Handle<dyn TdfAttribute>) {
        let mut current = attribute;
        while self.without_forgotten && current.get().is_some_and(|value| value.is_forgotten()) {
            current = current.get().map(|value| value.attribute_base().next()).unwrap_or_default();
        }
        self.value = current;
    }
}

/// `TDF_ChildIterator`: the children of a label, or (`all_levels`) its whole sub-tree in
/// depth-first order.
#[derive(Clone, Default)]
pub struct TdfChildIterator {
    node: Option<Arc<TdfLabelNode>>,
    first_level: i32,
}
impl TdfChildIterator {
    /// `TDF_ChildIterator(aLabel, allLevels)`; `Standard_NullObject` for a null label.
    pub fn new(label: &TdfLabel, all_levels: bool) -> Result<Self, Failure> {
        let mut iterator = Self::default();
        iterator.initialize(label, all_levels)?;
        Ok(iterator)
    }
    /// `Initialize(aLabel, allLevels)`.
    pub fn initialize(&mut self, label: &TdfLabel, all_levels: bool) -> Result<(), Failure> {
        let node = label.live_node("TDF_ChildIterator: a null Label has no children.")?;
        self.node = node.first_child();
        self.first_level = if all_levels { node.depth() } else { -1 };
        Ok(())
    }
    /// `More`; `iterator.more()`.
    pub fn more(&self) -> bool {
        self.node.is_some()
    }
    /// `Next`: the brother, or (all levels) the first child, else the next brother up.
    pub fn next(&mut self) {
        let Some(node) = self.node.clone() else {
            return;
        };
        if self.first_level == -1 {
            self.node = node.brother();
            return;
        }
        match node.first_child() {
            Some(child) => self.node = Some(child),
            None => self.up_to_brother(),
        }
    }
    /// `NextBrother`: skips the sub-tree of the current label.
    pub fn next_brother(&mut self) {
        let Some(node) = self.node.clone() else {
            return;
        };
        let brother = node.brother();
        if self.first_level == -1 || brother.is_some() {
            self.node = brother;
            return;
        }
        self.up_to_brother();
    }
    /// The native `ChildIterator_UpToBrother` macro.
    fn up_to_brother(&mut self) {
        let mut node = self.node.clone();
        while let Some(current) = node.clone().filter(|n| n.depth() > self.first_level && n.brother().is_none()) {
            node = current.father();
        }
        self.node = node
            .filter(|n| n.depth() > self.first_level && n.father().is_some())
            .and_then(|n| n.brother());
    }
    /// `Value`: the current label.
    pub fn value(&self) -> TdfLabel {
        TdfLabel::from_node(self.node.clone())
    }
}

/// `TDF_ChildIDIterator`: the children (or descendants) that hold an attribute of one ID.
#[derive(Default)]
pub struct TdfChildIdIterator {
    id: Guid,
    iterator: TdfChildIterator,
    attribute: Handle<dyn TdfAttribute>,
}
impl TdfChildIdIterator {
    /// `TDF_ChildIDIterator(aLabel, anID, allLevels)`.
    pub fn new(label: &TdfLabel, id: &Guid, all_levels: bool) -> Result<Self, Failure> {
        let mut iterator = Self::default();
        iterator.initialize(label, id, all_levels)?;
        Ok(iterator)
    }
    /// `Initialize(aLabel, anID, allLevels)`.
    pub fn initialize(&mut self, label: &TdfLabel, id: &Guid, all_levels: bool) -> Result<(), Failure> {
        self.id = *id;
        self.iterator.initialize(label, all_levels)?;
        self.attribute = Handle::default();
        self.find_next(TdfChildIterator::next)
    }
    /// `More`: an attribute is current.
    pub fn more(&self) -> bool {
        self.attribute.is_some()
    }
    /// `Next`; `iterator.next()?`.
    pub fn next(&mut self) -> Result<(), Failure> {
        self.attribute = Handle::default();
        if !self.iterator.more() {
            return Ok(());
        }
        self.iterator.next();
        self.find_next(TdfChildIterator::next)
    }
    /// `NextBrother`: skips the sub-tree of the current label.
    pub fn next_brother(&mut self) -> Result<(), Failure> {
        self.attribute = Handle::default();
        if !self.iterator.more() {
            return Ok(());
        }
        self.iterator.next_brother();
        self.find_next(TdfChildIterator::next_brother)
    }
    /// `Value`: the current attribute.
    pub fn value(&self) -> Handle<dyn TdfAttribute> {
        self.attribute.clone()
    }
    /// The native `ChildIDIterator_FindNext` loop with the given step.
    fn find_next(&mut self, step: fn(&mut TdfChildIterator)) -> Result<(), Failure> {
        while self.iterator.more() && !self.iterator.value().find_attribute(&self.id, &mut self.attribute)? {
            step(&mut self.iterator);
        }
        Ok(())
    }
}
