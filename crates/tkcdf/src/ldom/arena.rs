// Rust port of the node storage of ApplicationFramework/TKCDF/LDOM (LDOM_BasicNode,
// LDOM_BasicElement, LDOM_BasicAttribute and LDOM_BasicText fields, LDOM_BasicNode.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The arena of a document's nodes (ocaf-design.md section 9): every native
//! `LDOM_BasicNode*` is a [`LdomNodeId`] into it.
use super::basic_string::{LdomBasicString, LdomChars};
use std::sync::Arc;

/// A node of a document: the index of its record in the document's arena (the native
/// `LDOM_BasicNode*`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LdomNodeId(pub(crate) usize);

/// `LDOM_Node::NodeType`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum LdomNodeType {
    /// `UNKNOWN` (0): a null node.
    #[default]
    Unknown = 0,
    /// `ELEMENT_NODE` (1).
    ElementNode = 1,
    /// `ATTRIBUTE_NODE` (2).
    AttributeNode = 2,
    /// `TEXT_NODE` (3).
    TextNode = 3,
    /// `CDATA_SECTION_NODE` (4).
    CdataSectionNode = 4,
    /// `COMMENT_NODE` (8).
    CommentNode = 8,
}

/// The fields of the concrete basic node classes.
#[derive(Clone, Debug)]
pub(crate) enum NodeKind {
    /// A plain `LDOM_BasicNode` (natively only the static null node).
    Basic,
    /// `LDOM_BasicElement`: `myTagName`, `myAttributeMask`, `myFirstChild`.
    Element {
        tag_name: Option<Arc<LdomChars>>,
        attribute_mask: u64,
        first_child: Option<LdomNodeId>,
    },
    /// `LDOM_BasicAttribute`: `myName`, `myValue`.
    Attribute {
        name: Option<Arc<LdomChars>>,
        value: LdomBasicString,
    },
    /// `LDOM_BasicText`: `myValue`.
    Text { value: LdomBasicString },
}

/// One basic node: `myNodeType`, `mySibling` and the fields of its class.
#[derive(Clone, Debug)]
pub(crate) struct NodeRecord {
    pub(crate) node_type: LdomNodeType,
    pub(crate) sibling: Option<LdomNodeId>,
    pub(crate) kind: NodeKind,
}

/// Where a link to a node lives: an element's `myFirstChild` or a node's `mySibling` (the
/// native `const LDOM_BasicNode**`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Link {
    FirstChild(LdomNodeId),
    Sibling(LdomNodeId),
}

/// The fields of an element: its tag, its attribute mask and its first child.
pub(crate) type ElementFields<'a> = (Option<&'a Arc<LdomChars>>, u64, Option<LdomNodeId>);
/// [`ElementFields`] that own the tag.
pub(crate) type OwnedElementFields = (Option<Arc<LdomChars>>, u64, Option<LdomNodeId>);

/// The nodes of one document, in allocation order.
#[derive(Debug, Default)]
pub(crate) struct Arena {
    nodes: Vec<NodeRecord>,
}

impl Arena {
    /// Allocates a node and returns its id.
    pub(crate) fn push(&mut self, record: NodeRecord) -> LdomNodeId {
        self.nodes.push(record);
        LdomNodeId(self.nodes.len() - 1)
    }
    /// The record of a node.
    pub(crate) fn node(&self, id: LdomNodeId) -> &NodeRecord {
        &self.nodes[id.0]
    }
    /// The record of a node, for writing.
    pub(crate) fn node_mut(&mut self, id: LdomNodeId) -> &mut NodeRecord {
        &mut self.nodes[id.0]
    }
    /// `getNodeType`.
    pub(crate) fn node_type(&self, id: LdomNodeId) -> LdomNodeType {
        self.node(id).node_type
    }
    /// `isNull`: the node type is `UNKNOWN`.
    pub(crate) fn is_null(&self, id: LdomNodeId) -> bool {
        self.node_type(id) == LdomNodeType::Unknown
    }
    /// The raw `mySibling` link.
    pub(crate) fn raw_sibling(&self, id: LdomNodeId) -> Option<LdomNodeId> {
        self.node(id).sibling
    }
    /// `SetSibling`.
    pub(crate) fn set_sibling(&mut self, id: LdomNodeId, sibling: Option<LdomNodeId>) {
        self.node_mut(id).sibling = sibling;
    }
    /// `GetSibling`: the next non-null sibling; the null siblings it passes are unlinked
    /// from `mySibling`, as natively.
    pub(crate) fn get_sibling(&mut self, id: LdomNodeId) -> Option<LdomNodeId> {
        while let Some(sibling) = self.raw_sibling(id) {
            if !self.is_null(sibling) {
                break;
            }
            let next = self.raw_sibling(sibling);
            self.set_sibling(id, next);
        }
        self.raw_sibling(id)
    }
    /// The element fields of a node (`None` for another class).
    pub(crate) fn element(&self, id: LdomNodeId) -> Option<ElementFields<'_>> {
        match &self.node(id).kind {
            NodeKind::Element {
                tag_name,
                attribute_mask,
                first_child,
            } => Some((tag_name.as_ref(), *attribute_mask, *first_child)),
            _ => None,
        }
    }
    /// `myFirstChild` of an element (none for another node).
    pub(crate) fn first_child(&self, id: LdomNodeId) -> Option<LdomNodeId> {
        self.element(id).and_then(|(_, _, first)| first)
    }
    /// The value of an attribute or a text node.
    pub(crate) fn value(&self, id: LdomNodeId) -> Option<&LdomBasicString> {
        match &self.node(id).kind {
            NodeKind::Attribute { value, .. } | NodeKind::Text { value } => Some(value),
            NodeKind::Element { .. } | NodeKind::Basic => None,
        }
    }
    /// The value of an attribute or a text node, for writing.
    pub(crate) fn value_mut(&mut self, id: LdomNodeId) -> Option<&mut LdomBasicString> {
        match &mut self.node_mut(id).kind {
            NodeKind::Attribute { value, .. } | NodeKind::Text { value } => Some(value),
            NodeKind::Element { .. } | NodeKind::Basic => None,
        }
    }
    /// The name of an element (its tag) or of an attribute.
    pub(crate) fn name(&self, id: LdomNodeId) -> Option<&Arc<LdomChars>> {
        match &self.node(id).kind {
            NodeKind::Element { tag_name, .. } => tag_name.as_ref(),
            NodeKind::Attribute { name, .. } => name.as_ref(),
            NodeKind::Text { .. } | NodeKind::Basic => None,
        }
    }
    /// Writes a link (`*aPrevNode = value`).
    pub(crate) fn set_link(&mut self, link: Link, value: Option<LdomNodeId>) {
        match link {
            Link::FirstChild(element) => self.set_first_child(element, value),
            Link::Sibling(node) => self.set_sibling(node, value),
        }
    }
    /// Sets `myFirstChild` of an element.
    pub(crate) fn set_first_child(&mut self, id: LdomNodeId, value: Option<LdomNodeId>) {
        if let NodeKind::Element { first_child, .. } = &mut self.node_mut(id).kind {
            *first_child = value;
        }
    }
    /// `operator=(LDOM_NullPtr*)` of a basic node of any class: an element drops its tag
    /// and its children (`RemoveNodes`), an attribute its name and value, a text its value;
    /// the node becomes `UNKNOWN`.
    pub(crate) fn nullify(&mut self, id: LdomNodeId) {
        match &mut self.node_mut(id).kind {
            NodeKind::Element { tag_name, .. } => *tag_name = None,
            NodeKind::Attribute { name, value } => {
                *name = None;
                value.set_null();
            }
            NodeKind::Text { value } => value.set_null(),
            NodeKind::Basic => {}
        }
        if self.element(id).is_some() {
            self.remove_nodes(id);
        }
        self.node_mut(id).node_type = LdomNodeType::Unknown;
    }
    /// `RemoveNodes`: every child and attribute of an element becomes null, then the element
    /// has no first child.
    pub(crate) fn remove_nodes(&mut self, id: LdomNodeId) {
        let mut node = self.first_child(id);
        while let Some(current) = node {
            let next = self.get_sibling(current);
            if !self.is_null(current) {
                self.nullify(current);
            }
            node = next;
        }
        self.set_first_child(id, None);
    }
}
