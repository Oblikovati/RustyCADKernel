// Rust port of ApplicationFramework/TKCDF/LDOM/LDOM_Document.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LDOM_Document`: a document over its storage (ocaf-design.md section 9).
use super::LdomMemManager;
use super::arena::LdomNodeType;
use super::basic_nodes::with_store;
use super::element::LdomElement;
use super::node::LdomNode;
use super::node_list::LdomNodeList;
use super::string::LdomString;
use super::values::{LdomCdataSection, LdomComment, LdomText};
use rustycad_tkernel::standard::Handle;

/// `MEMORY_GRANULE`: the block size of a new document storage.
const MEMORY_GRANULE: i32 = 10000;

/// `LDOM_Document`; `LdomDocument::create_document(&LdomString::from_c_str(Some(b"root")))`.
///
/// ```
/// use rustycad_tkcdf::ldom::{LdomDocument, LdomString};
/// let empty = LdomDocument::new();
/// assert!(empty.is_null() && empty.eq_null());
/// let document = LdomDocument::create_document(&LdomString::new());
/// assert_eq!(document.get_document_element().get_tag_name().get_string(), b"document");
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct LdomDocument {
    /// `myMemManager`.
    mem_manager: Handle<LdomMemManager>,
}

impl Default for LdomDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl LdomDocument {
    /// `LDOM_Document()`: a document over a new, empty storage.
    pub fn new() -> Self {
        Self {
            mem_manager: Handle::new(LdomMemManager::new(MEMORY_GRANULE)),
        }
    }
    /// `LDOM_Document(aMemManager)`: a document over an existing storage.
    pub fn from_mem_manager(mem_manager: &Handle<LdomMemManager>) -> Self {
        Self {
            mem_manager: mem_manager.clone(),
        }
    }
    /// The storage (`myMemManager`).
    pub fn mem_manager(&self) -> &Handle<LdomMemManager> {
        &self.mem_manager
    }
    /// `createDocument(theQualifiedName)`: a new document whose root element has that tag
    /// (`document` for an empty name).
    pub fn create_document(qualified_name: &LdomString) -> Self {
        let document = Self::new();
        let name = Some(qualified_name.get_string())
            .filter(|name| !name.is_empty())
            .unwrap_or(b"document");
        with_store(&document.mem_manager, (), |store| {
            let root = store.create_element(Some(name));
            store.root = Some(root);
        });
        document
    }
    /// `createElement(theTagName)`: a new, unattached element of the document.
    pub fn create_element(&self, tag_name: &LdomString) -> LdomElement {
        let id = with_store(&self.mem_manager, None, |store| {
            Some(store.create_element(Some(tag_name.get_string())))
        });
        LdomElement::at(&self.mem_manager, id)
    }
    /// `createCDATASection(theData)`: a new CDATA node over a copy of the data, marked clear.
    pub fn create_cdata_section(&self, data: &LdomString) -> LdomCdataSection {
        let node =
            LdomCdataSection::from_node(self.text_node(LdomNodeType::CdataSectionNode, data));
        node.set_value_clear();
        node
    }
    /// `createComment(theData)`: a new comment node over a copy of the data.
    pub fn create_comment(&self, data: &LdomString) -> LdomComment {
        LdomComment::from_node(self.text_node(LdomNodeType::CommentNode, data))
    }
    /// `createTextNode(theData)`: a new text node over a copy of the data.
    pub fn create_text_node(&self, data: &LdomString) -> LdomText {
        LdomText::from_node(self.text_node(LdomNodeType::TextNode, data))
    }
    /// A new basic text node of that type over a copy of the data into the document.
    fn text_node(&self, node_type: LdomNodeType, data: &LdomString) -> LdomNode {
        let copy = data.document_copy();
        let id = with_store(&self.mem_manager, None, |store| {
            Some(store.create_text(node_type, copy))
        });
        LdomNode::at(&self.mem_manager, id)
    }
    /// `getDocumentElement`: the root element (null without one).
    pub fn get_document_element(&self) -> LdomElement {
        LdomElement::at(
            &self.mem_manager,
            self.mem_manager
                .get()
                .and_then(LdomMemManager::root_element),
        )
    }
    /// `getElementsByTagName(theTagName)`: the root when its tag matches, then every
    /// matching element below it.
    pub fn get_elements_by_tag_name(&self, tag_name: &LdomString) -> LdomNodeList {
        let nodes = with_store(&self.mem_manager, Vec::new(), |store| match store.root {
            Some(root) => store.elements_by_tag_name(root, tag_name.get_string()),
            None => Vec::new(),
        });
        LdomNodeList::from_ids(&self.mem_manager, nodes)
    }
    /// `isNull`: no root element, or a null one.
    pub fn is_null(&self) -> bool {
        self.get_document_element().is_null()
    }
    /// `operator==(LDOM_NullPtr*)`: no root element.
    pub fn eq_null(&self) -> bool {
        self.mem_manager
            .get()
            .and_then(LdomMemManager::root_element)
            .is_none()
    }
    /// `operator=(LDOM_NullPtr*)`: the document takes a new, empty storage.
    pub fn set_null(&mut self) {
        *self = Self::new();
    }
}
