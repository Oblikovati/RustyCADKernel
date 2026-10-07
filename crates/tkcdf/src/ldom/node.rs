// Rust port of ApplicationFramework/TKCDF/LDOM/LDOM_Node.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LDOM_Node`: the DOM value that points to a basic node of a document (ocaf-design.md
//! section 9). It caches the last child of an element, as natively.
use super::LdomMemManager;
use super::arena::{LdomNodeId, LdomNodeType};
use super::basic_nodes::{LdomBasicNode, with_store};
use super::basic_string::LdomStringType;
use super::string::LdomString;
use crate::live;
use rustycad_tkernel::standard::{Failure, Handle};
use std::cell::Cell;

/// `LDOM_Node`; `LdomNode::new().is_null()`.
///
/// ```
/// use rustycad_tkcdf::ldom::{LdomDocument, LdomString};
/// let document = LdomDocument::create_document(&LdomString::from_c_str(Some(b"root")));
/// let root = document.get_document_element();
/// let child = document.create_element(&LdomString::from_c_str(Some(b"child")));
/// root.append_child(&child);
/// assert!(root.has_child_nodes());
/// assert_eq!(root.get_first_child(), *child);
/// ```
#[derive(Clone, Debug, Default)]
pub struct LdomNode {
    /// `myDocument`.
    pub(crate) document: Handle<LdomMemManager>,
    /// `myOrigin`.
    pub(crate) origin: Option<LdomNodeId>,
    /// `myLastChild`: the cached last child of an element (written by const members).
    pub(crate) last_child: Cell<Option<LdomNodeId>>,
}

impl PartialEq for LdomNode {
    /// `operator==`: two null nodes, or the same basic node.
    fn eq(&self, other: &Self) -> bool {
        if self.is_null() {
            return other.is_null();
        }
        self.origin == other.origin && self.document == other.document
    }
}

impl LdomNode {
    /// `LDOM_Node()`: the null node.
    pub fn new() -> Self {
        Self::default()
    }
    /// `LDOM_Node(anOrig, aDoc)`: the node of a basic node.
    pub fn from_basic(origin: &LdomBasicNode) -> Self {
        Self::at(&origin.document, Some(origin.id))
    }
    /// The node of `origin` in `document`.
    pub(crate) fn at(document: &Handle<LdomMemManager>, origin: Option<LdomNodeId>) -> Self {
        Self {
            document: document.clone(),
            origin,
            last_child: Cell::new(None),
        }
    }
    /// `getOwnerDocument`: `Standard_NullObject` for a null node (natively a null
    /// dereference).
    pub fn get_owner_document(&self) -> Result<&LdomMemManager, Failure> {
        live(&self.document, "LDOM_Node::getOwnerDocument")
    }
    /// The document handle (`myDocument`).
    pub fn document(&self) -> &Handle<LdomMemManager> {
        &self.document
    }
    /// `Origin`: the basic node (none for a null node; natively a static null node).
    pub fn origin(&self) -> Option<LdomBasicNode> {
        self.origin.map(|id| LdomBasicNode::at(&self.document, id))
    }
    /// `operator=(LDOM_NullPtr*)`: becomes the null node.
    pub fn set_null(&mut self) {
        *self = Self::default();
    }
    /// `isNull`: no basic node, or an `UNKNOWN` one.
    pub fn is_null(&self) -> bool {
        self.get_node_type() == LdomNodeType::Unknown
    }
    /// `getNodeType`.
    pub fn get_node_type(&self) -> LdomNodeType {
        self.origin()
            .map_or(LdomNodeType::Unknown, |node| node.get_node_type())
    }
    /// `getNodeName`: the tag of an element or the name of an attribute, as a hashed string
    /// of the document; null for another node.
    pub fn get_node_name(&self) -> LdomString {
        let kind = self.get_node_type();
        if !matches!(
            kind,
            LdomNodeType::ElementNode | LdomNodeType::AttributeNode
        ) {
            return LdomString::new();
        }
        let name = self.read_store(None, |store, id| store.arena.name(id).cloned());
        name.map_or_else(LdomString::new, |text| {
            LdomString::create_direct_string(&text, &self.document)
        })
    }
    /// `getNodeValue`: the value of an attribute or the data of a text, comment or CDATA
    /// node, owned by the document; null for another node.
    pub fn get_node_value(&self) -> LdomString {
        if !self.has_value() {
            return LdomString::new();
        }
        let value = self.read_store(None, |store, id| store.arena.value(id).cloned());
        value.map_or_else(LdomString::new, |value| {
            LdomString::with_owner(&value, &self.document)
        })
    }
    /// The node is an attribute, a text, a comment or a CDATA section.
    fn has_value(&self) -> bool {
        matches!(
            self.get_node_type(),
            LdomNodeType::AttributeNode
                | LdomNodeType::TextNode
                | LdomNodeType::CdataSectionNode
                | LdomNodeType::CommentNode
        )
    }
    /// `getFirstChild`: the first child of an element unless it is an attribute.
    pub fn get_first_child(&self) -> LdomNode {
        if self.get_node_type() != LdomNodeType::ElementNode {
            return LdomNode::new();
        }
        let first = self.read_store(None, |store, id| store.arena.first_child(id));
        let first = first.filter(|child| {
            self.read_store(LdomNodeType::Unknown, |store, _| {
                store.arena.node_type(*child)
            }) != LdomNodeType::AttributeNode
        });
        first.map_or_else(LdomNode::new, |child| Self::at(&self.document, Some(child)))
    }
    /// `getLastChild`: the cached last child of an element, found and cached when unknown.
    pub fn get_last_child(&self) -> LdomNode {
        if self.get_node_type() != LdomNodeType::ElementNode {
            return LdomNode::new();
        }
        if self.last_child.get().is_none() {
            self.last_child
                .set(self.read_store(None, |store, id| store.arena.last_child(id)));
        }
        Self::at(&self.document, self.last_child.get())
    }
    /// `getNextSibling`: the raw next sibling unless it is an attribute; a null node has no
    /// sibling (natively a null dereference).
    pub fn get_next_sibling(&self) -> LdomNode {
        let sibling = self.read_store(None, |store, id| store.arena.raw_sibling(id));
        let sibling = sibling.filter(|next| {
            self.read_store(LdomNodeType::Unknown, |store, _| {
                store.arena.node_type(*next)
            }) != LdomNodeType::AttributeNode
        });
        sibling.map_or_else(LdomNode::new, |next| Self::at(&self.document, Some(next)))
    }
    /// `removeChild(aChild)`: unlinks a child of an element (it becomes `UNKNOWN`) and drops
    /// the cached last child when it is that child.
    pub fn remove_child(&self, child: &LdomNode) {
        if self.get_node_type() != LdomNodeType::ElementNode {
            return;
        }
        if let (false, Some(child_id), Some(id)) = (child.is_null(), child.origin, self.origin) {
            with_store(&self.document, (), |store| {
                store.arena.remove_child(id, child_id)
            });
        }
        if child.origin == self.last_child.get() {
            self.last_child.set(None);
        }
    }
    /// `appendChild(aChild)`: links the child after the cached last child (or through the
    /// element), and caches it as the last child.
    pub fn append_child(&self, child: &LdomNode) {
        let (Some(id), Some(child_id)) = (self.origin, child.origin) else {
            return;
        };
        if self.get_node_type() != LdomNodeType::ElementNode || child.is_null() {
            return;
        }
        let mut last = self.last_child.get();
        with_store(&self.document, (), |store| match last {
            Some(cached) => {
                let next = store.arena.raw_sibling(cached);
                store.arena.set_sibling(child_id, next);
                store.arena.set_sibling(cached, Some(child_id));
            }
            None => store.arena.append_child(id, child_id, &mut last),
        });
        self.last_child.set(Some(child_id));
    }
    /// `hasChildNodes`: an element whose first child is not null.
    pub fn has_child_nodes(&self) -> bool {
        if self.get_node_type() != LdomNodeType::ElementNode {
            return false;
        }
        let first = self.read_store(None, |store, id| store.arena.first_child(id));
        first.is_some_and(|child| !self.read_store(true, |store, _| store.arena.is_null(child)))
    }
    /// `SetValueClear`: a document string value of an attribute or a text node becomes
    /// `LDOM_AsciiDocClear` (its characters need no escaping).
    pub fn set_value_clear(&self) {
        let (true, Some(id)) = (self.has_value(), self.origin) else {
            return;
        };
        with_store(&self.document, (), |store| {
            if let Some(value) = store
                .arena
                .value_mut(id)
                .filter(|value| value.string_type() == LdomStringType::LdomAsciiDoc)
            {
                value.retype(LdomStringType::LdomAsciiDocClear);
            }
        });
    }
    /// Reads the document storage with the origin id, or returns `default`.
    pub(crate) fn read_store<R>(
        &self,
        default: R,
        f: impl FnOnce(&super::store::DocumentStore, LdomNodeId) -> R,
    ) -> R {
        match (self.document.get(), self.origin) {
            (Some(manager), Some(id)) => f(&manager.read(), id),
            _ => default,
        }
    }
}
