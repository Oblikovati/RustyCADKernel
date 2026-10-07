// Rust port of ApplicationFramework/TKCDF/LDOM/LDOM_BasicNode, LDOM_BasicElement,
// LDOM_BasicAttribute and LDOM_BasicText (.hxx and .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The basic nodes: views of one node record of a document (ocaf-design.md section 9).
//! A native `LDOM_BasicNode&` is the document's handle and the node's arena id.
use super::LdomMemManager;
use super::arena::{LdomNodeId, LdomNodeType, NodeKind, NodeRecord};
use super::basic_string::{LdomBasicString, LdomStringType};
use super::store::DocumentStore;
use rustycad_tkernel::standard::Handle;
use std::ops::Deref;

/// `LDOM_BasicNode`: a node of a document; `element.get_sibling()`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LdomBasicNode {
    pub(crate) document: Handle<LdomMemManager>,
    pub(crate) id: LdomNodeId,
}

impl LdomBasicNode {
    /// The view of node `id` of `document`.
    pub(crate) fn at(document: &Handle<LdomMemManager>, id: LdomNodeId) -> Self {
        Self {
            document: document.clone(),
            id,
        }
    }
    /// `LDOM_BasicNode(aType)`: a new plain node of `document` (`UNKNOWN` by default).
    pub fn new(document: &Handle<LdomMemManager>, node_type: LdomNodeType) -> Self {
        let record = NodeRecord {
            node_type,
            sibling: None,
            kind: NodeKind::Basic,
        };
        let id = with_store(document, LdomNodeId(usize::MAX), |store| {
            store.arena.push(record)
        });
        Self::at(document, id)
    }
    /// The arena id of the node.
    pub fn id(&self) -> LdomNodeId {
        self.id
    }
    /// The document of the node.
    pub fn document(&self) -> &Handle<LdomMemManager> {
        &self.document
    }
    /// `isNull`: the node type is `UNKNOWN`.
    pub fn is_null(&self) -> bool {
        self.get_node_type() == LdomNodeType::Unknown
    }
    /// `getNodeType`.
    pub fn get_node_type(&self) -> LdomNodeType {
        self.read(LdomNodeType::Unknown, |store| {
            store.arena.node_type(self.id)
        })
    }
    /// `GetSibling`: the next non-null sibling; the null siblings passed are unlinked.
    pub fn get_sibling(&self) -> Option<LdomBasicNode> {
        self.write(None, |store| store.arena.get_sibling(self.id))
            .map(|id| Self::at(&self.document, id))
    }
    /// `SetSibling(anOther)`: the raw sibling link.
    pub fn set_sibling(&self, other: Option<&LdomBasicNode>) {
        self.write((), |store| {
            store.arena.set_sibling(self.id, other.map(|node| node.id))
        });
    }
    /// `operator=(LDOM_NullPtr*)` of the base class: only the type becomes `UNKNOWN`.
    pub fn set_null(&self) {
        self.write((), |store| {
            store.arena.node_mut(self.id).node_type = LdomNodeType::Unknown
        });
    }
    /// Reads the document storage, or returns `default` for a node of a null document.
    pub(crate) fn read<R>(&self, default: R, f: impl FnOnce(&DocumentStore) -> R) -> R {
        self.document
            .get()
            .map_or(default, |manager| f(&manager.read()))
    }
    /// Writes the document storage, or returns `default` for a node of a null document.
    pub(crate) fn write<R>(&self, default: R, f: impl FnOnce(&mut DocumentStore) -> R) -> R {
        with_store(&self.document, default, f)
    }
}

/// Writes the storage of `document`, or returns `default` for a null handle.
pub(crate) fn with_store<R>(
    document: &Handle<LdomMemManager>,
    default: R,
    f: impl FnOnce(&mut DocumentStore) -> R,
) -> R {
    document
        .get()
        .map_or(default, |manager| f(&mut manager.write()))
}

/// A hashed name of the document as a string (`const char*` of the hash table).
fn hashed_name(text: Option<std::sync::Arc<super::basic_string::LdomChars>>) -> LdomBasicString {
    let mut name = LdomBasicString::new();
    if let Some(text) = text {
        name.set_direct(LdomStringType::LdomAsciiHashed, text);
    }
    name
}

/// Declares a view class that derefs to [`LdomBasicNode`].
macro_rules! basic_view {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name {
            node: LdomBasicNode,
        }
        impl Deref for $name {
            type Target = LdomBasicNode;
            fn deref(&self) -> &LdomBasicNode {
                &self.node
            }
        }
        impl $name {
            /// The view of a node (the native `(Class&)` cast).
            pub fn from_node(node: LdomBasicNode) -> Self {
                Self { node }
            }
            /// `operator=(LDOM_NullPtr*)`: drops the names and values (an element also
            /// nulls its children and attributes) and becomes `UNKNOWN`.
            pub fn set_null(&self) {
                self.node.write((), |store| store.arena.nullify(self.node.id));
            }
        }
    };
}

basic_view!(
    /// `LDOM_BasicElement`: an element, its children then its attributes.
    LdomBasicElement
);
basic_view!(
    /// `LDOM_BasicAttribute`: an attribute name and value.
    LdomBasicAttribute
);
basic_view!(
    /// `LDOM_BasicText`: the data of a text, comment or CDATA node.
    LdomBasicText
);

impl LdomBasicElement {
    /// `LDOM_BasicElement()`: a new `UNKNOWN` element of `document`.
    pub fn new(document: &Handle<LdomMemManager>) -> Self {
        let id = with_store(document, LdomNodeId(usize::MAX), |store| {
            store.create_element(None)
        });
        Self::from_node(LdomBasicNode::at(document, id))
    }
    /// `Create(aName, aLength, aDoc)`: a new element with the hashed tag (the first
    /// `length` bytes); a null name gives a new `UNKNOWN` element (natively a static one).
    pub fn create(name: Option<&[u8]>, length: i32, document: &Handle<LdomMemManager>) -> Self {
        let length = usize::try_from(length).unwrap_or(0);
        let name = name.map(|bytes| &bytes[..length.min(bytes.len())]);
        let id = with_store(document, LdomNodeId(usize::MAX), |store| {
            store.create_element(name)
        });
        Self::from_node(LdomBasicNode::at(document, id))
    }
    /// `GetTagName`: the hashed tag (null for an `UNKNOWN` element).
    pub fn get_tag_name(&self) -> LdomBasicString {
        hashed_name(self.read(None, |store| store.arena.name(self.id).cloned()))
    }
    /// `GetFirstChild`: the raw first child (possibly an attribute).
    pub fn get_first_child(&self) -> Option<LdomBasicNode> {
        self.read(None, |store| store.arena.first_child(self.id))
            .map(|id| LdomBasicNode::at(&self.document, id))
    }
    /// `GetLastChild`: the last node before the attributes.
    pub fn get_last_child(&self) -> Option<LdomBasicNode> {
        self.read(None, |store| store.arena.last_child(self.id))
            .map(|id| LdomBasicNode::at(&self.document, id))
    }
    /// `GetAttribute(aName, aLastCh)`: the attribute named `name` after `last_child`
    /// (natively a static null attribute when there is none).
    pub fn get_attribute(
        &self,
        name: &LdomBasicString,
        last_child: Option<&LdomBasicNode>,
    ) -> Option<LdomBasicAttribute> {
        let last = last_child.map(|node| node.id);
        let found = self.write(None, |store| {
            store.arena.get_attribute(self.id, name.get_string(), last)
        });
        found.map(|id| LdomBasicAttribute::from_node(LdomBasicNode::at(&self.document, id)))
    }
    /// `AddAttribute(anAttrName, anAttrValue, aDoc, aLastCh)`: adds or replaces an
    /// attribute; returns the last child (found on the way when `last_child` is none).
    pub fn add_attribute(
        &self,
        name: &LdomBasicString,
        value: &LdomBasicString,
        last_child: Option<&LdomBasicNode>,
    ) -> Option<LdomBasicNode> {
        let last = last_child.map(|node| node.id);
        let found = self.write(None, |store| {
            store.add_attribute(self.id, name, value, last)
        });
        found.map(|id| LdomBasicNode::at(&self.document, id))
    }
    /// `RemoveAttribute(aName, aLastCh)`: as natively, the attribute stays; returns the
    /// last child.
    pub fn remove_attribute(
        &self,
        name: &LdomBasicString,
        last_child: Option<&LdomBasicNode>,
    ) -> Option<LdomBasicNode> {
        let last = last_child.map(|node| node.id);
        let found = self.write(None, |store| {
            store.remove_attribute(self.id, name.get_string(), last)
        });
        found.map(|id| LdomBasicNode::at(&self.document, id))
    }
    /// `RemoveChild(aChild)`: unlinks the child and makes it `UNKNOWN`.
    pub fn remove_child(&self, child: &LdomBasicNode) {
        self.write((), |store| store.arena.remove_child(self.id, child.id));
    }
    /// `AppendChild(aChild, aLastCh)`: links the child after `last_child`, which becomes it.
    pub fn append_child(&self, child: &LdomBasicNode, last_child: &mut Option<LdomBasicNode>) {
        let mut last = last_child.as_ref().map(|node| node.id);
        self.write((), |store| {
            store.arena.append_child(self.id, child.id, &mut last)
        });
        *last_child = last.map(|id| LdomBasicNode::at(&self.document, id));
    }
}

impl LdomBasicAttribute {
    /// `LDOM_BasicAttribute()`: a new `UNKNOWN` attribute of `document`.
    pub fn new(document: &Handle<LdomMemManager>) -> Self {
        let kind = NodeKind::Attribute {
            name: None,
            value: LdomBasicString::new(),
        };
        let record = NodeRecord {
            node_type: LdomNodeType::Unknown,
            sibling: None,
            kind,
        };
        let id = with_store(document, LdomNodeId(usize::MAX), |store| {
            store.arena.push(record)
        });
        Self::from_node(LdomBasicNode::at(document, id))
    }
    /// `Create(theName, theDoc, theHashIndex)`: a new attribute with the hashed name; the
    /// name's hash goes to `hash`.
    pub fn create(
        name: &LdomBasicString,
        document: &Handle<LdomMemManager>,
        hash: &mut i32,
    ) -> Self {
        let (id, bucket) = with_store(document, (LdomNodeId(usize::MAX), 0), |store| {
            store.create_attribute(name)
        });
        *hash = bucket;
        Self::from_node(LdomBasicNode::at(document, id))
    }
    /// `GetName`: the hashed name (null for a null attribute).
    pub fn get_name(&self) -> LdomBasicString {
        hashed_name(self.read(None, |store| store.arena.name(self.id).cloned()))
    }
    /// `GetValue`: the value (a copy of the string; a document string shares its text).
    pub fn get_value(&self) -> LdomBasicString {
        self.read(None, |store| store.arena.value(self.id).cloned())
            .unwrap_or_default()
    }
    /// `SetValue(aValue, aDoc)`: the value becomes a copy into the document.
    pub fn set_value(&self, value: &LdomBasicString) {
        self.write((), |store| store.set_value(self.id, value.document_copy()));
    }
}

impl LdomBasicText {
    /// `LDOM_BasicText()`: a new `UNKNOWN` text node of `document`.
    pub fn new(document: &Handle<LdomMemManager>) -> Self {
        Self::create(LdomNodeType::Unknown, &LdomBasicString::new(), document)
    }
    /// `Create(aType, aData, aDoc)`: a new node of that type with the data (copied as the
    /// native copy constructor does).
    pub fn create(
        node_type: LdomNodeType,
        data: &LdomBasicString,
        document: &Handle<LdomMemManager>,
    ) -> Self {
        let id = with_store(document, LdomNodeId(usize::MAX), |store| {
            store.create_text(node_type, data.clone())
        });
        Self::from_node(LdomBasicNode::at(document, id))
    }
    /// `GetData`: the data.
    pub fn get_data(&self) -> LdomBasicString {
        self.read(None, |store| store.arena.value(self.id).cloned())
            .unwrap_or_default()
    }
    /// `SetData(aValue, aDoc)`: the data becomes a copy into the document.
    pub fn set_data(&self, value: &LdomBasicString) {
        self.write((), |store| store.set_value(self.id, value.document_copy()));
    }
}
