// Rust port of ApplicationFramework/TKCDF/LDOM/LDOM_XmlWriter.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LDOM_XmlWriter`: writes a document or a node as XML (ocaf-design.md section 9).
use super::arena::LdomNodeType;
use super::basic_string::{LdomBasicString, LdomStringType};
use super::c_text::c_str;
use super::char_reference::LdomCharReference;
use super::document::LdomDocument;
use super::element::LdomElement;
use super::node::LdomNode;
use std::io::{self, Write};

/// The 40 spaces and the `<` of the element indentation (`aSpaces`).
const SPACES: &[u8; 41] = b"                                        <";
/// `aMaxNSpaces`.
const MAX_SPACES: i32 = 40;

/// `LDOM_XmlWriter`; `LdomXmlWriter::new(None)` writes UTF-8 declarations.
///
/// ```
/// use rustycad_tkcdf::ldom::{LdomDocument, LdomString, LdomXmlWriter};
/// let document = LdomDocument::create_document(&LdomString::from_c_str(Some(b"root")));
/// let root = document.get_document_element();
/// root.set_attribute(&LdomString::from_c_str(Some(b"a")), &LdomString::from_c_str(Some(b"x<y")));
/// let mut output = Vec::new();
/// LdomXmlWriter::new(None).write_document(&mut output, &document).unwrap();
/// assert_eq!(output, b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<root a=\"x&lt;y\"/>");
/// ```
#[derive(Clone, Debug)]
pub struct LdomXmlWriter {
    /// `myEncodingName`.
    encoding_name: Vec<u8>,
    /// `myIndent`.
    indent: i32,
    /// `myCurIndent`.
    current_indent: i32,
}

impl LdomXmlWriter {
    /// `LDOM_XmlWriter(theEncoding)`: `UTF-8` without encoding name.
    pub fn new(encoding: Option<&[u8]>) -> Self {
        let encoding_name = c_str(encoding.unwrap_or(b"UTF-8")).to_vec();
        Self {
            encoding_name,
            indent: 0,
            current_indent: 0,
        }
    }
    /// `SetIndentation(theIndent)`: the spaces per element level (0: no line breaks).
    pub fn set_indentation(&mut self, indent: i32) {
        self.indent = indent;
    }
    /// `Write(theOStream, theDoc)`: the XML declaration and the document element.
    pub fn write_document(
        &mut self,
        stream: &mut dyn Write,
        document: &LdomDocument,
    ) -> io::Result<()> {
        stream.write_all(b"<?xml version=\"")?;
        stream.write_all(b"1.0")?;
        stream.write_all(b"\" encoding=\"")?;
        stream.write_all(&self.encoding_name)?;
        stream.write_all(b"\"?>\n")?;
        self.write_node(stream, &document.get_document_element())
    }
    /// `Write(theOStream, theNode)`: a text, an element with its attributes (in the
    /// reverse order of their list, which is the order they were set) and children, a
    /// CDATA section or a comment; another node is reported on standard error.
    pub fn write_node(&mut self, stream: &mut dyn Write, node: &LdomNode) -> io::Result<()> {
        let value = node.get_node_value();
        match node.get_node_type() {
            LdomNodeType::TextNode => write_string(stream, &value),
            LdomNodeType::ElementNode => self.write_element(stream, node),
            LdomNodeType::CdataSectionNode => write_wrapped(stream, b"<![CDATA[", &value, b"]]>"),
            LdomNodeType::CommentNode => write_wrapped(stream, b"<!--", &value, b"-->"),
            other => {
                eprintln!("Unrecognized node type = {}", other as i32);
                Ok(())
            }
        }
    }
    /// The element case of `Write(theOStream, theNode)`.
    fn write_element(&mut self, stream: &mut dyn Write, node: &LdomNode) -> io::Result<()> {
        let name = node.get_node_name();
        let indent = &SPACES[(MAX_SPACES - self.current_indent).clamp(0, MAX_SPACES) as usize..];
        stream.write_all(indent)?;
        stream.write_all(c_str(name.get_string()))?;
        let attributes = LdomElement::from_node(node.clone()).get_attributes_list();
        for index in (0..attributes.get_length()).rev() {
            self.write_attribute(stream, &attributes.item(index))?;
        }
        let first = node.get_first_child();
        if first.is_null() {
            stream.write_all(b"/>")?;
        } else {
            self.write_children(stream, first, indent, name.get_string())?;
        }
        if self.indent > 0 {
            stream.write_all(b"\n")?;
        }
        Ok(())
    }
    /// The children and the end tag; after an element child the end tag is indented.
    fn write_children(
        &mut self,
        stream: &mut dyn Write,
        first: LdomNode,
        indent: &[u8],
        name: &[u8],
    ) -> io::Result<()> {
        stream.write_all(b">")?;
        if first.get_node_type() == LdomNodeType::ElementNode && self.indent > 0 {
            stream.write_all(b"\n")?;
        }
        let mut child = first;
        let mut is_child_element = false;
        while !child.is_null() {
            is_child_element = child.get_node_type() == LdomNodeType::ElementNode;
            let step = if is_child_element { self.indent } else { 0 };
            self.current_indent += step;
            self.write_node(stream, &child)?;
            self.current_indent -= step;
            child = child.get_next_sibling();
        }
        if is_child_element {
            stream.write_all(indent)?;
            stream.write_all(b"/")?;
        } else {
            stream.write_all(b"</")?;
        }
        stream.write_all(c_str(name))?;
        stream.write_all(b">")
    }
    /// `WriteAttribute(theOStream, theAtt)`: ` name="value"`; an integer as `%d`, a clear
    /// document string as is, any other string with its references encoded.
    fn write_attribute(&mut self, stream: &mut dyn Write, attribute: &LdomNode) -> io::Result<()> {
        let name = attribute.get_node_name();
        let value = attribute.get_node_value();
        let mut text = Vec::new();
        let mut integer = 0;
        if value.string_type() == LdomStringType::LdomInteger && value.get_integer(&mut integer) {
            text.extend_from_slice(integer.to_string().as_bytes());
        } else if value.string_type() == LdomStringType::LdomAsciiDocClear {
            text.extend_from_slice(c_str(value.get_string()));
        } else {
            let mut length = 0;
            text = LdomCharReference::encode(value.get_string(), &mut length, true);
        }
        stream.write_all(b" ")?;
        stream.write_all(c_str(name.get_string()))?;
        stream.write_all(b"=\"")?;
        stream.write_all(c_str(&text))?;
        stream.write_all(b"\"")
    }
}

/// `Write(theOStream, theString)`: an integer as decimal, a hashed or clear string as is,
/// a free or document string with its references encoded; the null string writes nothing.
fn write_string(stream: &mut dyn Write, text: &LdomBasicString) -> io::Result<()> {
    let mut integer = 0;
    match text.string_type() {
        LdomStringType::LdomInteger if text.get_integer(&mut integer) => {
            stream.write_all(integer.to_string().as_bytes())
        }
        LdomStringType::LdomAsciiHashed | LdomStringType::LdomAsciiDocClear => {
            stream.write_all(c_str(text.get_string()))
        }
        LdomStringType::LdomAsciiFree | LdomStringType::LdomAsciiDoc => {
            let mut length = 0;
            stream.write_all(&LdomCharReference::encode(
                text.get_string(),
                &mut length,
                false,
            ))
        }
        _ => Ok(()),
    }
}

/// `prefix`, the string, `suffix`.
fn write_wrapped(
    stream: &mut dyn Write,
    prefix: &[u8],
    text: &LdomBasicString,
    suffix: &[u8],
) -> io::Result<()> {
    stream.write_all(prefix)?;
    write_string(stream, text)?;
    stream.write_all(suffix)
}
