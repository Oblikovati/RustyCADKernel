// Rust port of the allocation side of ApplicationFramework/TKCDF/LDOM/LDOM_MemManager.cxx
// (HashTable) and the Create members of LDOM_BasicElement, LDOM_BasicAttribute and
// LDOM_BasicText, at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The storage of one document: its node arena, its hash table of names and its root
//! element (ocaf-design.md section 9).
use super::arena::{Arena, LdomNodeId, LdomNodeType, NodeKind, NodeRecord};
use super::basic_string::{LdomBasicString, LdomChars};
use super::c_text::c_str;
use std::sync::Arc;

/// `HASH_MASK`: the hash table has 256 buckets.
const HASH_MASK: u32 = 255;

/// `LDOM_MemManager::HashTable::Hash`: the CRC-16 of the bytes, masked to a bucket;
/// `crc_hash(b"a") < 256`.
pub(crate) fn crc_hash(text: &[u8]) -> i32 {
    const LOW: [u32; 16] = [
        0o000000, 0o140301, 0o140601, 0o000500, 0o141401, 0o001700, 0o001200, 0o141101, 0o143001,
        0o003300, 0o003600, 0o143501, 0o002400, 0o142701, 0o142201, 0o002100,
    ];
    const HIGH: [u32; 16] = [
        0o000000, 0o146001, 0o154001, 0o012000, 0o170001, 0o036000, 0o024000, 0o162001, 0o120001,
        0o066000, 0o074000, 0o132001, 0o050000, 0o116001, 0o104001, 0o043000,
    ];
    let mut crc: u32 = 0;
    for byte in text {
        let mixed = crc ^ u32::from(*byte);
        crc = ((crc >> 8) ^ LOW[(mixed & 0x0F) as usize]) ^ HIGH[((mixed >> 4) & 0x0F) as usize];
    }
    (crc & HASH_MASK) as i32
}

/// `LDOM_MemManager::HashTable`: 256 buckets of names, each in insertion order.
#[derive(Debug)]
pub(crate) struct HashTable {
    buckets: Vec<Vec<Arc<LdomChars>>>,
}

impl Default for HashTable {
    fn default() -> Self {
        Self {
            buckets: vec![Vec::new(); (HASH_MASK + 1) as usize],
        }
    }
}

impl HashTable {
    /// `AddString(theString, theLen, theHashIndex)`: the stored copy of the first `len`
    /// bytes (added when absent) and its bucket.
    pub(crate) fn add_string(&mut self, text: &[u8]) -> (Arc<LdomChars>, i32) {
        let hash = crc_hash(text);
        let bucket = &mut self.buckets[hash as usize];
        if let Some(found) = bucket
            .iter()
            .find(|stored| table_match(&stored.bytes, text))
        {
            return (Arc::clone(found), hash);
        }
        let stored = LdomChars::new(text, Some(hash as u8));
        bucket.push(Arc::clone(&stored));
        (stored, hash)
    }
}

/// The `compareStrings` of the hash table: the first `text.len()` stored bytes are `text`
/// and the stored byte after them is the terminator.
fn table_match(stored: &[u8], text: &[u8]) -> bool {
    stored.starts_with(text) && stored.get(text.len()).is_none_or(|byte| *byte == 0)
}

/// The document storage behind an `LDOM_MemManager`.
#[derive(Debug, Default)]
pub(crate) struct DocumentStore {
    pub(crate) arena: Arena,
    /// `myHashTable`, created by the first hashed allocation.
    pub(crate) table: Option<HashTable>,
    /// `myRootElement`.
    pub(crate) root: Option<LdomNodeId>,
}

impl DocumentStore {
    /// `HashedAllocate(theString, theLen, theHash)`: the hashed copy of `text` (the first
    /// `length` bytes) and its hash.
    pub(crate) fn hashed_allocate(&mut self, text: &[u8]) -> (Arc<LdomChars>, i32) {
        self.table
            .get_or_insert_with(HashTable::default)
            .add_string(text)
    }
    /// `LDOM_BasicElement::Create(aName, aLength, aDoc)`: a new element with the hashed
    /// tag. A null name gives the native static void element: here a new, unattached
    /// `UNKNOWN` element.
    pub(crate) fn create_element(&mut self, name: Option<&[u8]>) -> LdomNodeId {
        let Some(name) = name else {
            return self.arena.push(element_record(LdomNodeType::Unknown, None));
        };
        let (tag, _) = self.hashed_allocate(name);
        self.arena
            .push(element_record(LdomNodeType::ElementNode, Some(tag)))
    }
    /// `LDOM_BasicAttribute::Create(theName, theDoc, theHashIndex)`: a new attribute with
    /// the hashed name (up to its C terminator) and a null value, and the name's hash.
    pub(crate) fn create_attribute(&mut self, name: &LdomBasicString) -> (LdomNodeId, i32) {
        let (name, hash) = self.hashed_allocate(c_str(name.get_string()));
        let record = NodeRecord {
            node_type: LdomNodeType::AttributeNode,
            sibling: None,
            kind: NodeKind::Attribute {
                name: Some(name),
                value: LdomBasicString::new(),
            },
        };
        (self.arena.push(record), hash)
    }
    /// `LDOM_BasicText::Create(aType, aData, aDoc)`: a new text node of that type.
    pub(crate) fn create_text(
        &mut self,
        node_type: LdomNodeType,
        data: LdomBasicString,
    ) -> LdomNodeId {
        let record = NodeRecord {
            node_type,
            sibling: None,
            kind: NodeKind::Text { value: data },
        };
        self.arena.push(record)
    }
}

/// A new element record without attribute mask or children.
fn element_record(node_type: LdomNodeType, tag_name: Option<Arc<LdomChars>>) -> NodeRecord {
    NodeRecord {
        node_type,
        sibling: None,
        kind: NodeKind::Element {
            tag_name,
            attribute_mask: 0,
            first_child: None,
        },
    }
}
