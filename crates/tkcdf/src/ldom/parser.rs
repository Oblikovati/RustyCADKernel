// Rust port of ApplicationFramework/TKCDF/LDOM/LDOMParser.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LDOMParser`: builds an LDOM document from XML (ocaf-design.md section 9). The
//! protected virtual `startElement` and `endElement` are the [`LdomParserHooks`] trait.
use super::LdomMemManager;
use super::arena::{LdomNodeId, LdomNodeType};
use super::basic_nodes::with_store;
use super::basic_string::LdomBasicString;
use super::c_text::c_str;
use super::char_reference::LdomCharReference;
use super::document::LdomDocument;
use super::element::LdomElement;
use super::istream::{LdomIStream, SharedIStreamReader};
use super::os_stream::{LdomBomType, LdomOSStream};
use super::xml_reader::{LdomXmlReader, LdomXmlRecordType};
use rustycad_tkernel::osd::{IosOpenMode, default_file_system};
use rustycad_tkernel::{standard::Handle, tcollection::AsciiString};
use std::io::Read;

/// The virtual members of `LDOMParser` a subclass overrides; returning true aborts the
/// parse ("User abort at ...").
pub trait LdomParserHooks {
    /// `startElement`: called when an element was read with its attributes.
    fn start_element(&mut self, current: &LdomElement) -> bool {
        let _ = current;
        false
    }
    /// `endElement`: called after the children of an element were read.
    fn end_element(&mut self, current: &LdomElement) -> bool {
        let _ = current;
        false
    }
}

/// The base-class `startElement` and `endElement`: they never abort.
struct DefaultParserHooks;
impl LdomParserHooks for DefaultParserHooks {}

/// The flags of `ParseDocument`.
struct DocumentScan {
    is_element: bool,
    is_doctype: bool,
    fictitious_root: bool,
    doc_start: bool,
}

/// `LDOMParser`; `LdomParser::new().parse(&mut source, false, false)` is false on success.
///
/// ```
/// use rustycad_tkcdf::ldom::{LdomParser, LdomString};
/// let mut parser = LdomParser::new();
/// let mut source: &[u8] = b"<?xml version=\"1.0\"?><a n=\"5\"><b>text</b></a>";
/// assert!(!parser.parse(&mut source, false, false));
/// let root = parser.get_document().get_document_element();
/// let mut value = 0;
/// assert!(root.get_attribute(&LdomString::from_c_str(Some(b"n"))).get_integer(&mut value));
/// assert_eq!(value, 5);
/// ```
pub struct LdomParser {
    /// `myReader`.
    reader: Option<LdomXmlReader>,
    /// `myDocument`.
    document: Handle<LdomMemManager>,
    /// `myCurrentData`.
    current_data: LdomOSStream,
    /// `myError`.
    error: AsciiString,
    /// The overriding `startElement` and `endElement`.
    hooks: Box<dyn LdomParserHooks>,
}

impl Default for LdomParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LdomParser {
    /// `LDOMParser()`: a parser with the base-class hooks.
    pub fn new() -> Self {
        Self::with_hooks(Box::new(DefaultParserHooks))
    }
    /// A parser of a subclass that overrides `startElement` and `endElement`.
    pub fn with_hooks(hooks: Box<dyn LdomParserHooks>) -> Self {
        Self {
            reader: None,
            document: Handle::default(),
            current_data: LdomOSStream::new(16384),
            error: AsciiString::default(),
            hooks,
        }
    }
    /// `getDocument`: the document of the last parse (null before any parse).
    pub fn get_document(&self) -> LdomDocument {
        LdomDocument::from_mem_manager(&self.document)
    }
    /// `parse(aFileName)`: parses a file of the default OSD file system; true (with
    /// "Fatal XML error: Cannot open XML file") when it cannot be opened.
    pub fn parse_file(&mut self, file_name: &[u8]) -> bool {
        let opened = default_file_system().open_istream(
            &AsciiString::from_bytes(file_name),
            IosOpenMode::IN,
            0,
            None,
        );
        let stream = opened
            .ok()
            .flatten()
            .filter(|stream| stream.lock().is_ok_and(|locked| locked.good()));
        let Some(stream) = stream else {
            self.error = AsciiString::new("Fatal XML error: Cannot open XML file");
            return true;
        };
        self.parse(&mut SharedIStreamReader(stream), false, false)
    }
    /// `parse(anInput, theTagPerStep, theWithoutRoot)`: parses into a new document; with
    /// `without_root` the records go under a fictitious `document` root. True on error.
    pub fn parse(&mut self, input: &mut dyn Read, tag_per_step: bool, without_root: bool) -> bool {
        self.document = Handle::new(LdomMemManager::new(20000));
        self.error.clear();
        self.reader = Some(LdomXmlReader::new(&self.document, tag_per_step));
        let mut stream = LdomIStream::new(input);
        self.parse_document(&mut stream, without_root)
    }
    /// `GetError(aData)`: the error; `data` takes the record text read last.
    pub fn get_error(&self, data: &mut AsciiString) -> &AsciiString {
        *data = AsciiString::from_bytes(self.current_data.str());
        &self.error
    }
    /// `GetBOM`: the byte order mark of the last parse.
    pub fn get_bom(&self) -> LdomBomType {
        self.reader
            .as_ref()
            .map_or(LdomBomType::BomUndefined, LdomXmlReader::get_bom)
    }
    /// `getCurrentElement`: the element read last.
    pub fn get_current_element(&self) -> LdomElement {
        let element = self.reader.as_ref().and_then(|reader| reader.element);
        LdomElement::at(&self.document, element)
    }
    /// `startElement`: the hook; true aborts.
    pub fn start_element(&mut self) -> bool {
        let current = self.get_current_element();
        self.hooks.start_element(&current)
    }
    /// `endElement`: the hook; true aborts.
    pub fn end_element(&mut self) -> bool {
        let current = self.get_current_element();
        self.hooks.end_element(&current)
    }
    /// `ReadRecord` of the reader; the reader's error is the parser's (natively one string).
    fn read_record(&mut self, stream: &mut LdomIStream, doc_start: &mut bool) -> LdomXmlRecordType {
        let Some(reader) = self.reader.as_mut() else {
            return LdomXmlRecordType::XmlUnknown;
        };
        let record = reader.read_record(stream, &mut self.current_data, doc_start);
        self.error = reader.error().clone();
        record
    }
    /// Fails with `message`.
    fn fail(&mut self, message: &str) -> Option<bool> {
        self.error = AsciiString::new(message);
        Some(true)
    }
    /// `ParseDocument(theIStream, theWithoutRoot)`: the prolog, one document element and
    /// what follows it. True on error.
    fn parse_document(&mut self, stream: &mut LdomIStream, without_root: bool) -> bool {
        let mut scan = DocumentScan {
            is_element: false,
            is_doctype: false,
            fictitious_root: false,
            doc_start: true,
        };
        loop {
            let record = if without_root && !scan.fictitious_root {
                LdomXmlRecordType::XmlStartElement
            } else {
                self.read_record(stream, &mut scan.doc_start)
            };
            if let Some(is_error) = self.document_record(record, stream, &mut scan, without_root) {
                return is_error;
            }
        }
    }
    /// One record of `ParseDocument`: none to go on, or the result.
    fn document_record(
        &mut self,
        record: LdomXmlRecordType,
        stream: &mut LdomIStream,
        scan: &mut DocumentScan,
        without_root: bool,
    ) -> Option<bool> {
        match record {
            LdomXmlRecordType::XmlHeader if scan.is_doctype || scan.is_element => {
                self.fail("Unexpected XML declaration")
            }
            LdomXmlRecordType::XmlDoctype if scan.is_element => {
                self.fail("Unexpected DOCTYPE declaration")
            }
            LdomXmlRecordType::XmlHeader | LdomXmlRecordType::XmlComment => None,
            LdomXmlRecordType::XmlDoctype => {
                scan.is_doctype = true;
                None
            }
            LdomXmlRecordType::XmlFullElement | LdomXmlRecordType::XmlStartElement
                if scan.is_element =>
            {
                self.fail("Expected comment or end-of-file")
            }
            LdomXmlRecordType::XmlFullElement => self.document_full_element(scan),
            LdomXmlRecordType::XmlStartElement => {
                self.document_start_element(stream, scan, without_root)
            }
            LdomXmlRecordType::XmlEndElement => self
                .end_element()
                .then(|| self.fail("User abort at endElement()"))
                .flatten()
                .or(Some(false)),
            LdomXmlRecordType::XmlEof => Some(false),
            LdomXmlRecordType::XmlUnknown if !scan.is_element => Some(true),
            _ => self.fail("Unexpected data beyond the Document Element"),
        }
    }
    /// The document element is an empty element.
    fn document_full_element(&mut self, scan: &mut DocumentScan) -> Option<bool> {
        scan.is_element = true;
        self.set_root();
        if self.start_element() {
            return self.fail("User abort at startElement()");
        }
        if self.end_element() {
            return self.fail("User abort at endElement()");
        }
        None
    }
    /// The document element starts (under a fictitious `document` root when asked).
    fn document_start_element(
        &mut self,
        stream: &mut LdomIStream,
        scan: &mut DocumentScan,
        without_root: bool,
    ) -> Option<bool> {
        scan.is_element = true;
        if without_root && !scan.fictitious_root {
            scan.fictitious_root = true;
            if let Some(reader) = self.reader.as_mut() {
                reader.create_element(b"document", 8);
            }
        }
        self.set_root();
        if self.start_element() {
            return self.fail("User abort at startElement()");
        }
        self.parse_element(stream, &mut scan.doc_start)
            .then_some(true)
    }
    /// `myDocument->myRootElement = &myReader->GetElement()`.
    fn set_root(&mut self) {
        let element = self.reader.as_ref().and_then(|reader| reader.element);
        with_store(&self.document, (), |store| store.root = element);
    }
    /// `ParseElement(theIStream, theDocStart)`: the children of the current element up to
    /// its end tag. True on error.
    fn parse_element(&mut self, stream: &mut LdomIStream, doc_start: &mut bool) -> bool {
        let parent = self.reader.as_ref().and_then(|reader| reader.element);
        let mut last_child = None;
        loop {
            let record = self.read_record(stream, doc_start);
            if let Some(is_error) =
                self.element_record(record, parent, &mut last_child, stream, doc_start)
            {
                return is_error;
            }
        }
    }
    /// One record of `ParseElement`: none to go on, or the result.
    fn element_record(
        &mut self,
        record: LdomXmlRecordType,
        parent: Option<LdomNodeId>,
        last_child: &mut Option<LdomNodeId>,
        stream: &mut LdomIStream,
        doc_start: &mut bool,
    ) -> Option<bool> {
        match record {
            LdomXmlRecordType::XmlUnknown => Some(true),
            LdomXmlRecordType::XmlFullElement | LdomXmlRecordType::XmlStartElement => {
                let child = self.reader.as_ref().and_then(|reader| reader.element);
                self.append_child(parent, child, last_child);
                if self.start_element() {
                    return self.fail("User abort at startElement()");
                }
                if record == LdomXmlRecordType::XmlStartElement {
                    return self.parse_element(stream, doc_start).then_some(true);
                }
                self.end_element()
                    .then(|| self.fail("User abort at endElement()"))
                    .flatten()
            }
            LdomXmlRecordType::XmlEndElement => Some(self.end_tag(parent)),
            LdomXmlRecordType::XmlText
            | LdomXmlRecordType::XmlComment
            | LdomXmlRecordType::XmlCdata => self.text_record(record, parent, last_child),
            LdomXmlRecordType::XmlEof => self.fail("Inexpected end of file"),
            _ => None,
        }
    }
    /// The end tag must name the parent; then `endElement`. True on error.
    fn end_tag(&mut self, parent: Option<LdomNodeId>) -> bool {
        let name = parent
            .and_then(|id| {
                with_store(&self.document, None, |store| {
                    store.arena.name(id).map(|name| c_str(&name.bytes).to_vec())
                })
            })
            .unwrap_or_default();
        if c_str(&self.current_data.str()) != name.as_slice() {
            let mut message = AsciiString::new("Expected end tag '");
            message.assign_cat(AsciiString::from_bytes(&name));
            message.assign_cat(AsciiString::new("'"));
            self.error = message;
            return true;
        }
        self.end_element() && self.fail("User abort at endElement()").is_some()
    }
    /// A text, comment or CDATA record becomes a child node. A text is decoded and read as
    /// an integer when it is one; a comment is decoded; a CDATA section is kept as read.
    fn text_record(
        &mut self,
        record: LdomXmlRecordType,
        parent: Option<LdomNodeId>,
        last_child: &mut Option<LdomNodeId>,
    ) -> Option<bool> {
        let mut text = self.current_data.str();
        let (node_type, value) = match record {
            LdomXmlRecordType::XmlText => {
                (LdomNodeType::TextNode, self.decoded_text(&mut text, true))
            }
            LdomXmlRecordType::XmlComment => (
                LdomNodeType::CommentNode,
                self.decoded_text(&mut text, false),
            ),
            _ => (
                LdomNodeType::CdataSectionNode,
                Some(LdomBasicString::with_length(
                    Some(&text),
                    self.current_data.length(),
                    &self.document,
                )),
            ),
        };
        let Some(value) = value else {
            // Natively the null result of Decode is dereferenced.
            return self.fail("Invalid character reference in character data");
        };
        let node = with_store(&self.document, None, |store| {
            Some(store.create_text(node_type, value))
        });
        self.append_child(parent, node, last_child);
        None
    }
    /// `Decode` of a text, then an integer (for a text, when it is one) or a document
    /// string of the decoded length; none for an invalid character reference.
    fn decoded_text(&self, text: &mut [u8], integer: bool) -> Option<LdomBasicString> {
        let mut length = 0;
        if !LdomCharReference::decode(text, &mut length) {
            return None;
        }
        let mut value = LdomBasicString::new();
        let is_number = integer && text.first().is_some_and(u8::is_ascii_digit);
        if is_number && !LdomXmlReader::get_integer(&mut value, text, length as usize) {
            return Some(value);
        }
        Some(LdomBasicString::with_length(
            Some(text),
            length,
            &self.document,
        ))
    }
    /// `aParent->AppendChild(aChild, aLastChild)`.
    fn append_child(
        &self,
        parent: Option<LdomNodeId>,
        child: Option<LdomNodeId>,
        last_child: &mut Option<LdomNodeId>,
    ) {
        if let (Some(parent), Some(child)) = (parent, child) {
            with_store(&self.document, (), |store| {
                store.arena.append_child(parent, child, last_child)
            });
        }
    }
}
