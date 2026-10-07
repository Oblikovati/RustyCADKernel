// Rust port of the members of ApplicationFramework/TKCDF/LDOM/LDOM_BasicElement.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The list surgery of an element (ocaf-design.md section 9): its children, then its
//! attributes, in one sibling chain that starts at `myFirstChild`.
use super::arena::{Arena, LdomNodeId, LdomNodeType, Link, NodeKind, OwnedElementFields};
use super::basic_string::{LdomBasicString, LdomChars};
use super::c_text::c_str;
use super::store::{DocumentStore, crc_hash};

/// The bit of `myAttributeMask` for a hash: `1 << (hash & 63)` on a 32-bit `int`, which
/// x86-64 shifts by `hash & 31` and sign-extends into the 64-bit `unsigned long`.
pub(crate) fn mask_bit(hash: i32) -> u64 {
    let shift = (hash as u32) & 63;
    (1i32.wrapping_shl(shift)) as i64 as u64
}

/// `LDOM_MemManager::CompareStrings`: the hash byte stored before the hashed string is
/// `hash` and the C strings match.
pub(crate) fn compare_hashed(text: &[u8], hash: i32, hashed: &LdomChars) -> bool {
    hashed.hash == Some(hash as u8) && c_str(&hashed.bytes) == c_str(text)
}

impl Arena {
    /// `GetLastChild`: the last node before the attributes (raw links, null nodes included).
    pub(crate) fn last_child(&self, id: LdomNodeId) -> Option<LdomNodeId> {
        let mut node = self.first_child(id)?;
        if self.node_type(node) == LdomNodeType::AttributeNode {
            return None;
        }
        while let Some(next) = self.raw_sibling(node) {
            if self.node_type(next) == LdomNodeType::AttributeNode {
                break;
            }
            node = next;
        }
        Some(node)
    }
    /// `GetAttribute(aName, aLastCh)`: the attribute of that name after the children.
    pub(crate) fn get_attribute(
        &mut self,
        id: LdomNodeId,
        name: &[u8],
        last_child: Option<LdomNodeId>,
    ) -> Option<LdomNodeId> {
        let mut node = match last_child {
            Some(last) => self.get_sibling(last),
            None => self.first_child(id),
        };
        while let Some(current) = node {
            let is_match = self.node_type(current) == LdomNodeType::AttributeNode
                && self
                    .name(current)
                    .is_some_and(|stored| *stored.bytes == *name);
            if is_match {
                return Some(current);
            }
            node = self.raw_sibling(current);
        }
        None
    }
    /// `GetFirstAttribute(theLastCh, thePrevNode)`: the first attribute and the link that
    /// points to it; without a known last child, the last non-null child is found on the way.
    pub(crate) fn first_attribute(
        &self,
        id: LdomNodeId,
        last_child: &mut Option<LdomNodeId>,
    ) -> (Option<LdomNodeId>, Link) {
        let (mut node, mut link) = match *last_child {
            Some(last) => (self.raw_sibling(last), Link::Sibling(last)),
            None => (self.first_child(id), Link::FirstChild(id)),
        };
        let search_last = last_child.is_none();
        while let Some(current) = node {
            if self.node_type(current) == LdomNodeType::AttributeNode {
                break;
            }
            if search_last && !self.is_null(current) {
                *last_child = Some(current);
            }
            link = Link::Sibling(current);
            node = self.raw_sibling(current);
        }
        (node, link)
    }
    /// `RemoveChild(aChild)`: unlinks the child (before the attributes) and makes it a null
    /// basic node (only its type changes, as the native base-class assignment does).
    pub(crate) fn remove_child(&mut self, id: LdomNodeId, child: LdomNodeId) {
        let mut node = self.first_child(id);
        let mut link = Link::FirstChild(id);
        while let Some(current) = node {
            if self.node_type(current) == LdomNodeType::AttributeNode {
                break;
            }
            if current == child {
                let next = self.get_sibling(current);
                self.set_link(link, next);
                self.node_mut(child).node_type = LdomNodeType::Unknown;
                break;
            }
            link = Link::Sibling(current);
            node = self.get_sibling(current);
        }
    }
    /// `AppendChild(aChild, aLastChild)`: links the child after the last child (or before the
    /// attributes) and makes it the last child.
    pub(crate) fn append_child(
        &mut self,
        id: LdomNodeId,
        child: LdomNodeId,
        last_child: &mut Option<LdomNodeId>,
    ) {
        if let Some(last) = *last_child {
            let next = self.raw_sibling(last);
            self.set_sibling(child, next);
            self.set_sibling(last, Some(child));
        } else {
            let mut node = self.first_child(id);
            let mut link = Link::FirstChild(id);
            while let Some(current) = node {
                if self.node_type(current) == LdomNodeType::AttributeNode {
                    self.set_sibling(child, Some(current));
                    break;
                }
                link = Link::Sibling(current);
                node = self.raw_sibling(current);
            }
            self.set_link(link, Some(child));
        }
        *last_child = Some(child);
    }
    /// `AddElementsByTagName(aList, aTagName)`: every descendant element of that tag, in
    /// document order.
    pub(crate) fn add_elements_by_tag_name(
        &mut self,
        id: LdomNodeId,
        tag: &[u8],
        list: &mut Vec<LdomNodeId>,
    ) {
        let mut node = self.first_child(id);
        while let Some(current) = node {
            match self.node_type(current) {
                LdomNodeType::AttributeNode => break,
                LdomNodeType::ElementNode => {
                    if self.name(current).is_some_and(|name| *name.bytes == *tag) {
                        list.push(current);
                    }
                    self.add_elements_by_tag_name(current, tag, list);
                }
                _ => {}
            }
            node = self.get_sibling(current);
        }
    }
    /// `AddAttributes(aList, aLastChild)`: every attribute after the children.
    pub(crate) fn add_attributes(
        &mut self,
        id: LdomNodeId,
        last_child: Option<LdomNodeId>,
        list: &mut Vec<LdomNodeId>,
    ) {
        let mut node = match last_child {
            Some(last) => self.get_sibling(last),
            None => self.first_child(id),
        };
        while let Some(current) = node {
            if self.node_type(current) == LdomNodeType::AttributeNode {
                list.push(current);
            }
            node = self.get_sibling(current);
        }
    }
}

impl DocumentStore {
    /// `AddAttribute(anAttrName, anAttrValue, aDoc, aLastCh)`: a new attribute in front of
    /// the attributes, unless the mask says the name may exist and it does: then that
    /// attribute takes the value (a document copy). Returns the last child found.
    pub(crate) fn add_attribute(
        &mut self,
        id: LdomNodeId,
        name: &LdomBasicString,
        value: &LdomBasicString,
        last_child: Option<LdomNodeId>,
    ) -> Option<LdomNodeId> {
        let (attribute, hash) = self.create_attribute(name);
        self.set_value(attribute, value.clone());
        let mut last_child = last_child;
        let (first, link) = self.arena.first_attribute(id, &mut last_child);
        let bit = mask_bit(hash);
        let mask = self.arena.element(id).map_or(0, |(_, mask, _)| mask);
        if mask & bit == 0 {
            self.set_attribute_mask(id, mask | bit);
        } else if let Some(existing) = self.find_hashed(first, name.get_string(), hash) {
            self.set_value(existing, value.document_copy());
            return last_child;
        }
        self.arena.set_link(link, Some(attribute));
        self.arena.set_sibling(attribute, first);
        last_child
    }
    /// The attribute of a hashed name from `first` on (raw links).
    fn find_hashed(&self, first: Option<LdomNodeId>, name: &[u8], hash: i32) -> Option<LdomNodeId> {
        let mut node = first;
        while let Some(current) = node {
            let matches = self.arena.node_type(current) == LdomNodeType::AttributeNode
                && self
                    .arena
                    .name(current)
                    .is_some_and(|stored| compare_hashed(name, hash, stored));
            if matches {
                return Some(current);
            }
            node = self.arena.raw_sibling(current);
        }
        None
    }
    /// Sets `myAttributeMask` of an element.
    fn set_attribute_mask(&mut self, id: LdomNodeId, value: u64) {
        if let NodeKind::Element { attribute_mask, .. } = &mut self.arena.node_mut(id).kind {
            *attribute_mask = value;
        }
    }
    /// `RemoveAttribute(aName, aLastCh)`: finds the attribute when the mask allows it but,
    /// as natively, removes nothing; returns the last child found on the way.
    pub(crate) fn remove_attribute(
        &mut self,
        id: LdomNodeId,
        name: &[u8],
        last_child: Option<LdomNodeId>,
    ) -> Option<LdomNodeId> {
        let hash = crc_hash(name);
        let mask = self.arena.element(id).map_or(0, |(_, mask, _)| mask);
        let mut last_child = last_child;
        if mask & mask_bit(hash) != 0 {
            let (first, _) = self.arena.first_attribute(id, &mut last_child);
            // The native loop finds the attribute and only nulls its local pointer.
            let _found = self.find_hashed(first, name, hash);
        }
        last_child
    }
    /// Writes the value of an attribute or the data of a text node.
    pub(crate) fn set_value(&mut self, id: LdomNodeId, value: LdomBasicString) {
        if let Some(slot) = self.arena.value_mut(id) {
            *slot = value;
        }
    }
    /// `getElementsByTagName` of an element or a document: the element itself when its tag
    /// matches, then `AddElementsByTagName`.
    pub(crate) fn elements_by_tag_name(&mut self, id: LdomNodeId, tag: &[u8]) -> Vec<LdomNodeId> {
        let mut list = Vec::new();
        if self
            .arena
            .name(id)
            .is_some_and(|name| c_str(&name.bytes) == c_str(tag))
        {
            list.push(id);
        }
        self.arena.add_elements_by_tag_name(id, tag, &mut list);
        list
    }
    /// The same-document `LDOM_Element::ReplaceElement`: the tag, the mask and the first
    /// child of the other element are shared.
    pub(crate) fn share_element(&mut self, id: LdomNodeId, fields: Option<OwnedElementFields>) {
        let (tag, mask, first) = fields.unwrap_or_default();
        if let NodeKind::Element {
            tag_name,
            attribute_mask,
            first_child,
        } = &mut self.arena.node_mut(id).kind
        {
            *tag_name = tag;
            *attribute_mask = mask;
            *first_child = first;
        }
    }
}
