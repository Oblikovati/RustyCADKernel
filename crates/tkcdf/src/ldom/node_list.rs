// Rust port of ApplicationFramework/TKCDF/LDOM/LDOM_NodeList.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LDOM_NodeList`: the basic nodes a query found, in document order (ocaf-design.md
//! section 9). The native `LDOM_BasicNodeSequence` linked list is a `Vec`.
use super::LdomMemManager;
use super::arena::LdomNodeId;
use super::node::LdomNode;
use rustycad_tkernel::standard::Handle;

/// `LDOM_NodeList`; `LdomNodeList::new().get_length() == 0`.
///
/// ```
/// use rustycad_tkcdf::ldom::{LdomDocument, LdomString};
/// let document = LdomDocument::create_document(&LdomString::from_c_str(Some(b"a")));
/// let list = document.get_elements_by_tag_name(&LdomString::from_c_str(Some(b"a")));
/// assert_eq!(list.get_length(), 1);
/// assert!(list.item(1).is_null());
/// ```
#[derive(Clone, Debug, Default)]
pub struct LdomNodeList {
    /// `myDoc`.
    document: Handle<LdomMemManager>,
    /// `mySeq`.
    sequence: Vec<LdomNodeId>,
}

impl LdomNodeList {
    /// `LDOM_NodeList()`: an empty list without document.
    pub fn new() -> Self {
        Self::default()
    }
    /// `LDOM_NodeList(aDoc)`: an empty list of the document.
    pub fn with_document(document: &Handle<LdomMemManager>) -> Self {
        Self {
            document: document.clone(),
            sequence: Vec::new(),
        }
    }
    /// The list of `document` over `nodes`.
    pub(crate) fn from_ids(document: &Handle<LdomMemManager>, nodes: Vec<LdomNodeId>) -> Self {
        Self {
            document: document.clone(),
            sequence: nodes,
        }
    }
    /// `Append(aNode)`.
    pub(crate) fn append(&mut self, node: LdomNodeId) {
        self.sequence.push(node);
    }
    /// `operator=(LDOM_NullPtr*)`: no document and no node.
    pub fn set_null(&mut self) {
        *self = Self::default();
    }
    /// `operator==(LDOM_NullPtr*)`: no document or no node.
    pub fn is_null(&self) -> bool {
        self.document.is_null() || self.sequence.is_empty()
    }
    /// `item(anIndex)`: the node at the 0-based index; the null node out of range or
    /// without document.
    pub fn item(&self, index: i32) -> LdomNode {
        let found = usize::try_from(index)
            .ok()
            .and_then(|index| self.sequence.get(index));
        match found {
            Some(id) if self.document.is_some() => LdomNode::at(&self.document, Some(*id)),
            _ => LdomNode::new(),
        }
    }
    /// `getLength`.
    pub fn get_length(&self) -> i32 {
        self.sequence.len() as i32
    }
}
