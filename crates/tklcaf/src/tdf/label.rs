// Rust port of ApplicationFramework/TKLCAF/TDF/TDF_Label.hxx, .lxx and .cxx (tree members)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDF_Label`: a value that designates one node of a data framework's label tree
//! (ocaf-design.md section 3).
use super::label_node::{FLAGS_MASK, TdfLabelNode};
use super::{TdfChildIterator, TdfData, TdfTagSource};
use rustycad_tkernel::{
    ncollection::HashKey,
    standard::{Failure, Handle, exceptions},
};
use std::sync::Arc;

/// `TDF_Label`; `TdfLabel::default()` is the null label.
///
/// ```
/// use rustycad_tklcaf::tdf::TdfData;
/// let data = TdfData::new();
/// let child = data.root().find_child(3, true).unwrap();
/// assert_eq!((child.tag(), child.depth().unwrap()), (3, 1));
/// assert!(child.father() == data.root());
/// ```
#[derive(Clone, Default)]
pub struct TdfLabel {
    node: Option<Arc<TdfLabelNode>>,
}

impl TdfLabel {
    /// `TDF_Label()`: the null label.
    pub fn new() -> Self {
        Self::default()
    }
    pub(crate) fn from_node(node: Option<Arc<TdfLabelNode>>) -> Self {
        Self { node }
    }
    /// The designated node (the native private `myLabelNode`), `None` for a null label.
    pub fn node(&self) -> Option<&Arc<TdfLabelNode>> {
        self.node.as_ref()
    }
    /// The node, or the native `Standard_NullObject` with `message`.
    pub(crate) fn live_node(&self, message: &str) -> Result<&Arc<TdfLabelNode>, Failure> {
        self.node
            .as_ref()
            .ok_or_else(|| exceptions::null_object(message))
    }
    /// The node address: the native `myLabelNode` pointer value.
    pub(crate) fn address(&self) -> usize {
        self.node
            .as_ref()
            .map_or(0, |node| Arc::as_ptr(node) as usize)
    }
    /// `Nullify`; `label.nullify()`.
    pub fn nullify(&mut self) {
        self.node = None;
    }
    /// `Data`: the owning framework; null for a null or detached label (ocaf-design.md
    /// section 3).
    pub fn data(&self) -> Handle<TdfData> {
        self.node
            .as_ref()
            .map(|node| node.state.read().data.upgrade())
            .unwrap_or_default()
    }
    /// `Tag`; 0 for a null label (a crash natively).
    pub fn tag(&self) -> i32 {
        self.node.as_ref().map_or(0, |node| node.tag())
    }
    /// `Father`: null for a root or a null label.
    pub fn father(&self) -> TdfLabel {
        Self::from_node(self.node.as_ref().and_then(|node| node.father()))
    }
    /// `IsNull`; `label.is_null()`.
    pub fn is_null(&self) -> bool {
        self.node.is_none()
    }
    /// `Imported(aStatus)`: sets the flag on the label and its whole sub-tree, when it
    /// changes; `Standard_NullObject` for a null label.
    pub fn imported(&self, status: bool) -> Result<(), Failure> {
        let node = self.live_node("A null Label has no status.")?;
        if node.is_imported() == status {
            return Ok(());
        }
        node.set_imported(status);
        let mut children = TdfChildIterator::new(self, true)?;
        while children.more() {
            if let Some(child) = children.value().node() {
                child.set_imported(status);
            }
            children.next();
        }
        Ok(())
    }
    /// `IsImported`; false for a null label.
    pub fn is_imported(&self) -> bool {
        self.node.as_ref().is_some_and(|node| node.is_imported())
    }
    /// `IsEqual` (`operator==`): same node.
    pub fn is_equal(&self, other: &TdfLabel) -> bool {
        self.address() == other.address()
    }
    /// `IsDifferent` (`operator!=`).
    pub fn is_different(&self, other: &TdfLabel) -> bool {
        !self.is_equal(other)
    }
    /// `IsRoot`; false for a null label.
    pub fn is_root(&self) -> bool {
        self.node.as_ref().is_some_and(|node| node.is_root())
    }
    /// `MayBeModified`; false for a null label.
    pub fn may_be_modified(&self) -> bool {
        self.node
            .as_ref()
            .is_some_and(|node| node.may_be_modified())
    }
    /// `AttributesModified`; false for a null label.
    pub fn attributes_modified(&self) -> bool {
        self.node
            .as_ref()
            .is_some_and(|node| node.attributes_modified())
    }
    /// `Depth`: 0 for a root; `Standard_NullObject` for a null label.
    pub fn depth(&self) -> Result<i32, Failure> {
        Ok(self.live_node("A null Label has no depth.")?.depth())
    }
    /// `IsDescendant(aLabel)`: this label is `label` or below it; false for null labels.
    pub fn is_descendant(&self, label: &TdfLabel) -> bool {
        let (Some(ancestor), Some(mut current)) = (label.node.clone(), self.node.clone()) else {
            return false;
        };
        let ancestor_depth = ancestor.depth();
        while current.depth() > ancestor_depth && !Arc::ptr_eq(&current, &ancestor) {
            let Some(father) = current.father() else {
                return false;
            };
            current = father;
        }
        Arc::ptr_eq(&current, &ancestor)
    }
    /// `Root`: the root of the framework; `Standard_NullObject` for a null label.
    pub fn root(&self) -> Result<TdfLabel, Failure> {
        let node = self.live_node("A null Label has no root.")?;
        let core = node.core();
        Ok(Self::from_node(core.map(|core| core.root_node().clone())))
    }
    /// `HasChild`; false for a null label.
    pub fn has_child(&self) -> bool {
        self.node
            .as_ref()
            .is_some_and(|node| node.first_child().is_some())
    }
    /// `NbChildren`: the direct children; `Standard_NullObject` for a null label.
    pub fn nb_children(&self) -> Result<i32, Failure> {
        let node = self.live_node("A null Label has no children.")?;
        let mut count = 0;
        let mut child = node.first_child();
        while let Some(current) = child {
            count += 1;
            child = current.brother();
        }
        Ok(count)
    }
    /// `FindChild(aTag, create)`: the child of that tag, created (in increasing tag order)
    /// when asked; `Standard_NullObject` for a null label, `Standard_OutOfRange` when the
    /// new depth would overflow the depth bits.
    pub fn find_child(&self, tag: i32, create: bool) -> Result<TdfLabel, Failure> {
        let node = self.live_node("A null Label has no child.")?;
        if create && ((node.depth() + 1) & FLAGS_MASK) != 0 {
            return Err(exceptions::out_of_range(&format!(
                "Depth value out of range: {} exceeds the depth bits",
                node.depth() + 1
            )));
        }
        Ok(Self::from_node(super::label_children::find_or_add_child(
            node, tag, create,
        )))
    }
    /// `NewChild`: a child with the next tag of the label's `TDF_TagSource`.
    pub fn new_child(&self) -> Result<TdfLabel, Failure> {
        TdfTagSource::new_child_of(self)
    }
    /// `Transaction`: the current transaction of the framework; `Standard_NullObject` for
    /// a null label.
    pub fn transaction(&self) -> Result<i32, Failure> {
        let node = self.live_node("A null Label has no transaction.")?;
        let core = node.core().ok_or_else(|| {
            exceptions::null_object("TDF_Label::Transaction: the label has no data framework")
        })?;
        Ok(core.transaction())
    }
    /// `HasLowerNode`: compares node addresses, as natively.
    pub fn has_lower_node(&self, other: &TdfLabel) -> bool {
        self.address() < other.address()
    }
    /// `HasGreaterNode`: compares node addresses, as natively.
    pub fn has_greater_node(&self, other: &TdfLabel) -> bool {
        self.address() > other.address()
    }
}

impl PartialEq for TdfLabel {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other)
    }
}
impl Eq for TdfLabel {}
/// `std::hash<TDF_Label>`: the node address.
impl std::hash::Hash for TdfLabel {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.address().hash(state);
    }
}
impl HashKey for TdfLabel {
    fn occt_hash(&self) -> usize {
        self.address()
    }
}
impl std::fmt::Debug for TdfLabel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut entry = rustycad_tkernel::tcollection::AsciiString::default();
        super::TdfTool::entry(self, &mut entry);
        write!(
            formatter,
            "TdfLabel({})",
            String::from_utf8_lossy(entry.as_bytes())
        )
    }
}
