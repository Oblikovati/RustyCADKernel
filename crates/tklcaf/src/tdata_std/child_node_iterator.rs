// Rust port of ApplicationFramework/TKLCAF/TDataStd/TDataStd_ChildNodeIterator.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDataStd_ChildNodeIterator`: the children of a tree node, directly or at all levels
//! (depth first).
use super::TDataStdTreeNode;
use crate::live;
use rustycad_tkernel::standard::{Failure, Handle};

/// `TDataStd_ChildNodeIterator`; `TDataStdChildNodeIterator::new(&node, true)`.
///
/// ```
/// use rustycad_tklcaf::{tdata_std::{TDataStdChildNodeIterator, TDataStdTreeNode}, tdf::TdfData};
/// let data = TdfData::new();
/// let root = TDataStdTreeNode::set(&data.root()).unwrap();
/// root.append(&TDataStdTreeNode::set(&data.root().find_child(1, true).unwrap()).unwrap()).unwrap();
/// let mut children = TDataStdChildNodeIterator::new(&root, false).unwrap();
/// assert!(children.more());
/// children.next().unwrap();
/// assert!(!children.more());
/// ```
#[derive(Clone, Default)]
pub struct TDataStdChildNodeIterator {
    node: Handle<TDataStdTreeNode>,
    first_level: i32,
}

impl TDataStdChildNodeIterator {
    /// `TDataStd_ChildNodeIterator()`: nothing to iterate.
    pub fn empty() -> Self {
        Self::default()
    }
    /// The constructor `(aTreeNode, allLevels)`: the first child of the node;
    /// `Standard_NullObject` for a null node.
    pub fn new(tree_node: &Handle<TDataStdTreeNode>, all_levels: bool) -> Result<Self, Failure> {
        let mut iterator = Self::empty();
        iterator.initialize(tree_node, all_levels)?;
        Ok(iterator)
    }
    /// `Initialize(aTreeNode, allLevels)`: restarts on the first child; the depth of the
    /// node bounds an all-levels walk (-1 iterates the direct children only).
    pub fn initialize(
        &mut self,
        tree_node: &Handle<TDataStdTreeNode>,
        all_levels: bool,
    ) -> Result<(), Failure> {
        let node = live(tree_node, "TDataStd_ChildNodeIterator::Initialize")?;
        self.node = node.first();
        self.first_level = if all_levels { node.depth() } else { -1 };
        Ok(())
    }
    /// `More`.
    pub fn more(&self) -> bool {
        self.node.is_some()
    }
    // The native OCCT iterator protocol (More/Next/Value), not std::iter::Iterator.
    #[allow(clippy::should_implement_trait)]
    /// `Next`: the next brother, or (at all levels) the first child, else the next brother
    /// of the nearest father below the start level; `Standard_NullObject` past the end.
    pub fn next(&mut self) -> Result<(), Failure> {
        let current = live(&self.node, "TDataStd_ChildNodeIterator::Next")?;
        if self.first_level == -1 {
            self.node = current.next();
            return Ok(());
        }
        let first = current.first();
        if first.is_some() {
            self.node = first;
            return Ok(());
        }
        self.up_to_brother();
        Ok(())
    }
    /// `NextBrother`: the next brother, else as `Next` past a last child;
    /// `Standard_NullObject` past the end.
    pub fn next_brother(&mut self) -> Result<(), Failure> {
        let next = live(&self.node, "TDataStd_ChildNodeIterator::NextBrother")?.next();
        if next.is_some() {
            self.node = next;
            return Ok(());
        }
        self.up_to_brother();
        Ok(())
    }
    /// `Value`.
    pub fn value(&self) -> Handle<TDataStdTreeNode> {
        self.node.clone()
    }
    /// The native `ChildNodeIterator_UpToBrother` macro: climbs while the node is below the
    /// start level and has no next brother, then steps to the next brother, or ends.
    fn up_to_brother(&mut self) {
        while let Some(node) = self
            .node
            .get()
            .filter(|n| n.depth() > self.first_level && !n.has_next())
        {
            self.node = node.father();
        }
        let below = self
            .node
            .get()
            .filter(|n| n.depth() > self.first_level && n.has_father());
        self.node = below.map(TDataStdTreeNode::next).unwrap_or_default();
    }
}
