// Rust port of ApplicationFramework/TKCDF/LDOM/LDOM_Attr, LDOM_CharacterData (.hxx and
// .cxx), LDOM_Text.hxx, LDOM_Comment.hxx and LDOM_CDATASection.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The DOM values of attributes and character data (ocaf-design.md section 9).
use super::basic_nodes::{LdomBasicAttribute, LdomBasicText, with_store};
use super::node::LdomNode;
use super::string::LdomString;
use std::cell::Cell;
use std::ops::Deref;

/// `LDOM_Attr`: an attribute node; `LdomAttr::new().is_null()`.
///
/// ```
/// use rustycad_tkcdf::ldom::{LdomDocument, LdomString};
/// let document = LdomDocument::create_document(&LdomString::from_c_str(Some(b"r")));
/// let root = document.get_document_element();
/// root.set_attribute(&LdomString::from_c_str(Some(b"a")), &LdomString::from_c_str(Some(b"1")));
/// let attribute = root.get_attribute_node(&LdomString::from_c_str(Some(b"a")));
/// attribute.set_value(&LdomString::from_c_str(Some(b"2")));
/// assert_eq!(attribute.get_value().get_string(), b"2");
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LdomAttr {
    node: LdomNode,
}

impl Deref for LdomAttr {
    type Target = LdomNode;
    fn deref(&self) -> &LdomNode {
        &self.node
    }
}

impl LdomAttr {
    /// `LDOM_Attr()`: the null attribute.
    pub fn new() -> Self {
        Self::default()
    }
    /// `LDOM_Attr(anAttr, aDoc)`: the node of a basic attribute.
    pub fn from_basic(attribute: &LdomBasicAttribute) -> Self {
        Self::from_node(LdomNode::from_basic(attribute))
    }
    /// The attribute of a node (the native cast).
    pub fn from_node(node: LdomNode) -> Self {
        Self { node }
    }
    /// `operator=(LDOM_NullPtr*)`.
    pub fn set_null(&mut self) {
        self.node.set_null();
    }
    /// `getName`.
    pub fn get_name(&self) -> LdomString {
        self.get_node_name()
    }
    /// `getValue`.
    pub fn get_value(&self) -> LdomString {
        self.get_node_value()
    }
    /// `setValue(aValue)`: the value becomes a copy into the document.
    pub fn set_value(&self, value: &LdomString) {
        set_origin_value(&self.node, value);
    }
}

/// `SetValue`/`SetData` of the basic node behind `node`: a copy into its document.
fn set_origin_value(node: &LdomNode, value: &LdomString) {
    if let Some(id) = node.origin {
        with_store(&node.document, (), |store| {
            store.set_value(id, value.document_copy())
        });
    }
}

/// `LDOM_CharacterData`: a text, comment or CDATA node, with its cached length.
#[derive(Debug, Default)]
pub struct LdomCharacterData {
    node: LdomNode,
    /// `myLength`: -1 until `getLength` computes it.
    length: Cell<i32>,
}

impl Clone for LdomCharacterData {
    /// The copy constructor: the length is computed again.
    fn clone(&self) -> Self {
        Self::from_node(self.node.clone())
    }
}

impl PartialEq for LdomCharacterData {
    /// The inherited `LDOM_Node::operator==`.
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl Deref for LdomCharacterData {
    type Target = LdomNode;
    fn deref(&self) -> &LdomNode {
        &self.node
    }
}

impl LdomCharacterData {
    /// `LDOM_CharacterData()`: the null node with an unknown length.
    pub fn new() -> Self {
        Self::from_node(LdomNode::new())
    }
    /// `LDOM_CharacterData(aText, aDoc)`: the node of a basic text.
    pub fn from_basic(text: &LdomBasicText) -> Self {
        Self::from_node(LdomNode::from_basic(text))
    }
    /// The character data of a node (the native cast).
    pub fn from_node(node: LdomNode) -> Self {
        Self {
            node,
            length: Cell::new(-1),
        }
    }
    /// `operator=(LDOM_NullPtr*)`: the null node; the length is unknown again.
    pub fn set_null(&mut self) {
        self.node.set_null();
        self.length.set(-1);
    }
    /// `operator=(anOther)`: the defaulted assignment copies the cached length too.
    pub fn assign(&mut self, other: &Self) {
        self.node = other.node.clone();
        self.length.set(other.length.get());
    }
    /// `getData`.
    pub fn get_data(&self) -> LdomString {
        self.get_node_value()
    }
    /// `setData(aValue)`: the data becomes a copy into the document; the length is reset.
    pub fn set_data(&self, value: &LdomString) {
        set_origin_value(&self.node, value);
        self.length.set(-1);
    }
    /// `getLength`: the C length of the data, computed once.
    pub fn get_length(&self) -> i32 {
        if self.length.get() < 0 {
            self.length
                .set(self.get_node_value().get_string().len() as i32);
        }
        self.length.get()
    }
}

/// Declares a character-data class over another one.
macro_rules! character_value {
    ($(#[$doc:meta])* $name:ident: $base:ident) => {
        $(#[$doc])*
        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct $name {
            base: $base,
        }
        impl Deref for $name {
            type Target = $base;
            fn deref(&self) -> &$base {
                &self.base
            }
        }
        impl $name {
            /// The null node.
            pub fn new() -> Self {
                Self::default()
            }
            /// The node of a basic text (the protected native constructor).
            pub fn from_basic(text: &LdomBasicText) -> Self {
                Self::from_node(LdomNode::from_basic(text))
            }
            /// The value of a node (the native cast).
            pub fn from_node(node: LdomNode) -> Self {
                Self { base: $base::from_node(node) }
            }
            /// `operator=(LDOM_NullPtr*)`.
            pub fn set_null(&mut self) {
                self.base.set_null();
            }
        }
    };
}

character_value!(
    /// `LDOM_Text`: a text node.
    LdomText: LdomCharacterData
);
character_value!(
    /// `LDOM_Comment`: a comment node.
    LdomComment: LdomCharacterData
);
character_value!(
    /// `LDOM_CDATASection`: a CDATA section node.
    LdomCdataSection: LdomText
);
