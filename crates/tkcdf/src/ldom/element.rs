// Rust port of ApplicationFramework/TKCDF/LDOM/LDOM_Element.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LDOM_Element`: the DOM value of an element (ocaf-design.md section 9).
use super::LdomMemManager;
use super::arena::{LdomNodeId, LdomNodeType};
use super::basic_nodes::{LdomBasicElement, with_store};
use super::basic_string::LdomBasicString;
use super::node::LdomNode;
use super::node_list::LdomNodeList;
use super::string::LdomString;
use super::values::LdomAttr;
use rustycad_tkernel::standard::Handle;
use std::ops::Deref;

/// `LDOM_Element`; `LdomElement::new().is_null()`.
///
/// ```
/// use rustycad_tkcdf::ldom::{LdomDocument, LdomString};
/// let document = LdomDocument::create_document(&LdomString::from_c_str(Some(b"root")));
/// let root = document.get_document_element();
/// root.set_attribute(&LdomString::from_c_str(Some(b"n")), &LdomString::from_integer(3));
/// let mut value = 0;
/// assert!(root.get_attribute(&LdomString::from_c_str(Some(b"n"))).get_integer(&mut value));
/// assert_eq!(value, 3);
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LdomElement {
    node: LdomNode,
}

impl Deref for LdomElement {
    type Target = LdomNode;
    fn deref(&self) -> &LdomNode {
        &self.node
    }
}

impl LdomElement {
    /// `LDOM_Element()`: the null element.
    pub fn new() -> Self {
        Self::default()
    }
    /// `LDOM_Element(anElem, aDoc)`: the element of a basic element.
    pub fn from_basic(element: &LdomBasicElement) -> Self {
        Self::from_node(LdomNode::from_basic(element))
    }
    /// The element of a node (the native `(LDOM_Element&)` cast).
    pub fn from_node(node: LdomNode) -> Self {
        Self { node }
    }
    /// The element of `id` in `document`.
    pub(crate) fn at(document: &Handle<LdomMemManager>, id: Option<LdomNodeId>) -> Self {
        Self::from_node(LdomNode::at(document, id))
    }
    /// `operator=(LDOM_NullPtr*)`.
    pub fn set_null(&mut self) {
        self.node.set_null();
    }
    /// `getTagName`.
    pub fn get_tag_name(&self) -> LdomString {
        self.get_node_name()
    }
    /// `getAttribute(aName)`: the value of the attribute, owned by the document; null when
    /// there is none. Without a cached last child, the last child before the attributes is
    /// found and cached, as natively.
    pub fn get_attribute(&self, name: &LdomString) -> LdomString {
        let Some(id) = self.origin.filter(|_| !self.is_null()) else {
            return LdomString::new();
        };
        if self.last_child.get().is_none() && !self.cache_last_child(id, false) {
            return LdomString::new();
        }
        let last = self.last_child.get();
        let value = with_store(&self.document, None, |store| {
            let attribute = store.arena.get_attribute(id, name.get_string(), last)?;
            store.arena.value(attribute).cloned()
        });
        value.map_or_else(LdomString::new, |value| {
            LdomString::with_owner(&value, &self.document)
        })
    }
    /// `getAttributeNode(aName)`: the attribute node. Native quirk: the cache takes the first
    /// attribute (not the last child), so the search starts after it.
    pub fn get_attribute_node(&self, name: &LdomString) -> LdomAttr {
        let Some(id) = self.origin.filter(|_| !self.is_null()) else {
            return LdomAttr::new();
        };
        if self.last_child.get().is_none() && !self.cache_last_child(id, true) {
            return LdomAttr::new();
        }
        let last = self.last_child.get();
        let found = with_store(&self.document, None, |store| {
            store.arena.get_attribute(id, name.get_string(), last)
        });
        LdomAttr::from_node(LdomNode::at(&self.document, found))
    }
    /// The search of `getAttribute` (`first_attribute` false) and `getAttributeNode`
    /// (true): when the first child is not an attribute, walk to the node before the first
    /// attribute and cache it (or the attribute); false when no attribute follows.
    fn cache_last_child(&self, id: LdomNodeId, first_attribute: bool) -> bool {
        with_store(&self.document, true, |store| {
            let Some(mut node) = store
                .arena
                .first_child(id)
                .filter(|first| store.arena.node_type(*first) != LdomNodeType::AttributeNode)
            else {
                return true;
            };
            loop {
                let Some(sibling) = store.arena.get_sibling(node) else {
                    return false;
                };
                if store.arena.node_type(sibling) == LdomNodeType::AttributeNode {
                    self.last_child
                        .set(Some(if first_attribute { sibling } else { node }));
                    return true;
                }
                node = sibling;
            }
        })
    }
    /// `getElementsByTagName(aName)`: this element when its tag matches, then every
    /// matching descendant element.
    pub fn get_elements_by_tag_name(&self, name: &LdomString) -> LdomNodeList {
        let mut list = LdomNodeList::with_document(&self.document);
        if let Some(id) = self.origin.filter(|_| !self.is_null()) {
            for found in with_store(&self.document, Vec::new(), |store| {
                store.elements_by_tag_name(id, name.get_string())
            }) {
                list.append(found);
            }
        }
        list
    }
    /// `setAttribute(aName, aValue)`: adds or replaces the attribute with a copy of the
    /// value into the document; caches the last child the search found.
    pub fn set_attribute(&self, name: &LdomString, value: &LdomString) {
        let Some(id) = self.origin.filter(|_| !self.is_null()) else {
            return;
        };
        let copy = value.document_copy();
        let last = self.last_child.get();
        let last = with_store(&self.document, last, |store| {
            store.add_attribute(id, name, &copy, last)
        });
        self.last_child.set(last);
    }
    /// `setAttributeNode(aNewAttr)`: `setAttribute` with its name and value.
    pub fn set_attribute_node(&self, attribute: &LdomAttr) {
        self.set_attribute(&attribute.get_name(), &attribute.get_value());
    }
    /// `removeAttribute(aName)`: as natively, the attribute is found but stays.
    pub fn remove_attribute(&self, name: &LdomString) {
        let Some(id) = self.origin.filter(|_| !self.is_null()) else {
            return;
        };
        let last = self.last_child.get();
        with_store(&self.document, None, |store| {
            store.remove_attribute(id, name.get_string(), last)
        });
    }
    /// `GetChildByTagName(aTagName)`: the first child element with that tag.
    pub fn get_child_by_tag_name(&self, tag_name: &LdomString) -> LdomElement {
        if self.is_null() || tag_name.is_null() {
            return LdomElement::new();
        }
        self.first_element_from(self.get_first_child(), tag_name)
    }
    /// `GetSiblingByTagName`: the next sibling element with the same tag.
    pub fn get_sibling_by_tag_name(&self) -> LdomElement {
        if self.is_null() {
            return LdomElement::new();
        }
        self.first_element_from(self.get_next_sibling(), &self.get_tag_name())
    }
    /// The first element from `node` on (through `getNextSibling`) whose tag equals `tag`.
    fn first_element_from(&self, mut node: LdomNode, tag: &LdomBasicString) -> LdomElement {
        while !node.is_null() {
            match node.get_node_type() {
                LdomNodeType::AttributeNode => break,
                LdomNodeType::ElementNode if node.get_node_name().equals(tag) => {
                    return LdomElement::from_node(node);
                }
                _ => {}
            }
            node = node.get_next_sibling();
        }
        LdomElement::new()
    }
    /// `ReplaceElement(anOther)`: in the same document, this element shares the other's
    /// tag, mask and children (and its cached last child); otherwise it takes copies of
    /// them into its document.
    pub fn replace_element(&self, other: &LdomElement) {
        let Some(id) = self.origin else {
            return;
        };
        if self.document == other.document {
            let fields = other.read_store(None, |store, other_id| {
                store
                    .arena
                    .element(other_id)
                    .map(|(tag, mask, first)| (tag.cloned(), mask, first))
            });
            with_store(&self.document, (), |store| store.share_element(id, fields));
            self.last_child.set(other.last_child.get());
            return;
        }
        let snapshot = match (other.document.get(), other.origin) {
            (Some(manager), Some(other_id)) => manager.write().arena.snapshot_element(other_id),
            _ => Default::default(),
        };
        with_store(&self.document, (), |store| {
            store.replace_element(id, &snapshot)
        });
        self.last_child.set(None);
    }
    /// `GetAttributesList`: the attributes after the cached last child.
    pub fn get_attributes_list(&self) -> LdomNodeList {
        let last = self.last_child.get();
        let nodes = self.origin.map_or_else(Vec::new, |id| {
            with_store(&self.document, Vec::new(), |store| {
                let mut nodes = Vec::new();
                store.arena.add_attributes(id, last, &mut nodes);
                nodes
            })
        });
        LdomNodeList::from_ids(&self.document, nodes)
    }
}
