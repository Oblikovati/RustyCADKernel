// Rust port of LDOM_BasicElement::ReplaceElement (ApplicationFramework/TKCDF/LDOM/
// LDOM_BasicElement.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The copy of an element of another document (ocaf-design.md section 9). The native code
//! reads the other document while it allocates in this one; the port first takes a
//! snapshot of the other element under its document's lock, then builds the copy.
use super::arena::{Arena, LdomNodeId, LdomNodeType, NodeKind};
use super::basic_string::{LdomBasicString, LdomChars};
use super::store::DocumentStore;
use std::sync::Arc;

/// What `ReplaceElement` reads of the other element.
#[derive(Debug, Default)]
pub(crate) struct ElementSnapshot {
    tag_name: Option<Arc<LdomChars>>,
    attribute_mask: u64,
    children: Vec<ChildSnapshot>,
    attributes: Vec<(Option<Arc<LdomChars>>, LdomBasicString)>,
}

/// One non-null child before the attributes.
#[derive(Debug)]
enum ChildSnapshot {
    Element(ElementSnapshot),
    Text(LdomNodeType, LdomBasicString),
}

impl Arena {
    /// Reads an element for `ReplaceElement`; the walk unlinks null siblings as natively
    /// (`GetSibling`). A node that is not an element reads as an element without children.
    pub(crate) fn snapshot_element(&mut self, id: LdomNodeId) -> ElementSnapshot {
        let Some((tag_name, attribute_mask, first)) = self
            .element(id)
            .map(|(tag, mask, first)| (tag.cloned(), mask, first))
        else {
            return ElementSnapshot::default();
        };
        let mut snapshot = ElementSnapshot {
            tag_name,
            attribute_mask,
            ..ElementSnapshot::default()
        };
        let mut node = first;
        while let Some(current) = node {
            if self.node_type(current) == LdomNodeType::AttributeNode {
                break;
            }
            if let Some(child) = self.snapshot_child(current) {
                snapshot.children.push(child);
            }
            node = self.get_sibling(current);
        }
        self.snapshot_attributes(node, &mut snapshot);
        snapshot
    }
    /// A child of the children loop: null and unknown nodes are skipped.
    fn snapshot_child(&mut self, id: LdomNodeId) -> Option<ChildSnapshot> {
        match self.node_type(id) {
            LdomNodeType::ElementNode => Some(ChildSnapshot::Element(self.snapshot_element(id))),
            kind @ (LdomNodeType::TextNode
            | LdomNodeType::CommentNode
            | LdomNodeType::CdataSectionNode) => Some(ChildSnapshot::Text(
                kind,
                self.value(id).cloned().unwrap_or_default(),
            )),
            _ => None,
        }
    }
    /// The attribute loop: every non-null node from the first attribute on is read as an
    /// attribute, as natively.
    fn snapshot_attributes(&mut self, first: Option<LdomNodeId>, snapshot: &mut ElementSnapshot) {
        let mut node = first;
        while let Some(current) = node {
            if !self.is_null(current) {
                let name = self.name(current).cloned();
                snapshot
                    .attributes
                    .push((name, self.value(current).cloned().unwrap_or_default()));
            }
            node = self.get_sibling(current);
        }
    }
}

impl DocumentStore {
    /// `ReplaceElement(anOtherElem, aDocument)`: this element takes the tag (the other
    /// document's string), the attribute mask, and copies of the children and attributes;
    /// only its sibling link is kept.
    pub(crate) fn replace_element(&mut self, id: LdomNodeId, other: &ElementSnapshot) {
        if let NodeKind::Element {
            tag_name,
            attribute_mask,
            first_child,
        } = &mut self.arena.node_mut(id).kind
        {
            *tag_name = other.tag_name.clone();
            *attribute_mask = other.attribute_mask;
            *first_child = None;
        }
        let mut last = None;
        for child in &other.children {
            let created = self.copy_child(child);
            self.link_after(id, last, created);
            last = Some(created);
        }
        for (name, value) in &other.attributes {
            let created = self.copy_attribute(name.as_deref(), value);
            self.link_after(id, last, created);
            last = Some(created);
        }
    }
    /// A new node of this document for a child of the snapshot.
    fn copy_child(&mut self, child: &ChildSnapshot) -> LdomNodeId {
        match child {
            ChildSnapshot::Element(element) => {
                let tag = element.tag_name.as_deref().map(|text| text.bytes.to_vec());
                let created = self.create_element(Some(tag.as_deref().unwrap_or_default()));
                self.replace_element(created, element);
                created
            }
            ChildSnapshot::Text(kind, value) => self.create_text(*kind, value.document_copy()),
        }
    }
    /// A new attribute of this document: the name hashed here, the value a document copy.
    fn copy_attribute(&mut self, name: Option<&LdomChars>, value: &LdomBasicString) -> LdomNodeId {
        let name = LdomBasicString::from_c_str(Some(name.map_or(&b""[..], |text| &text.bytes)));
        let (created, _) = self.create_attribute(&name);
        self.set_value(created, value.document_copy());
        created
    }
    /// Links `created` after `last`, or as the first child.
    fn link_after(&mut self, id: LdomNodeId, last: Option<LdomNodeId>, created: LdomNodeId) {
        match last {
            Some(last) => self.arena.set_sibling(last, Some(created)),
            None => self.arena.set_first_child(id, Some(created)),
        }
    }
}
