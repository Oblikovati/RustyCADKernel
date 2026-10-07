// Rust port of ApplicationFramework/TKCDF/LDOM/LDOM_XmlReader.hxx and .cxx (the reader,
// its buffer and the byte order marks) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LDOM_XmlReader`: reads one XML record at a time into an [`LdomOSStream`]
//! (ocaf-design.md section 9). The native pointers into the read buffer are indices; the
//! state machine is in `xml_states.rs`.
use super::LdomMemManager;
use super::arena::LdomNodeId;
use super::basic_nodes::{LdomBasicElement, LdomBasicNode, with_store};
use super::basic_string::LdomBasicString;
use super::c_text::strtol;
use super::istream::LdomIStream;
use super::os_stream::{LdomBomType, LdomOSStream};
use rustycad_tkernel::{standard::Handle, tcollection::AsciiString};

/// `XML_BUFFER_SIZE`: the bytes one read fills.
pub const XML_BUFFER_SIZE: usize = 20480;
/// `XML_MIN_BUFFER`: fewer unread bytes than this make the reader read more.
pub(crate) const XML_MIN_BUFFER: usize = 10;

/// `LDOM_XmlReader::RecordType`: what `ReadRecord` read.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum LdomXmlRecordType {
    /// `XML_UNKNOWN`: an error (see the error string).
    #[default]
    XmlUnknown,
    /// `XML_HEADER`.
    XmlHeader,
    /// `XML_DOCTYPE`.
    XmlDoctype,
    /// `XML_COMMENT`.
    XmlComment,
    /// `XML_START_ELEMENT`.
    XmlStartElement,
    /// `XML_END_ELEMENT`.
    XmlEndElement,
    /// `XML_FULL_ELEMENT`: an empty element `<a/>`.
    XmlFullElement,
    /// `XML_TEXT`.
    XmlText,
    /// `XML_CDATA`.
    XmlCdata,
    /// `XML_EOF`.
    XmlEof,
}

/// `ParserState`: where `ReadRecord` is inside a record.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ScanState {
    #[default]
    Waiting,
    Header,
    Doctype,
    DoctypeMarkup,
    Element,
    ElementEnd,
    AttributeName,
    AttributeEqual,
    AttributeValue,
    Comment,
    Cdata,
    Text,
}

/// The local variables of one `ReadRecord` call.
#[derive(Debug, Default)]
pub(crate) struct RecordScan {
    pub(crate) state: ScanState,
    /// `aStartData`.
    pub(crate) start: Option<usize>,
    /// `aNameEnd`: written by every name test, read again after a buffer refill.
    pub(crate) name_end: usize,
    pub(crate) attribute_name: LdomBasicString,
    /// `anAttDelimiter`.
    pub(crate) delimiter: u8,
    /// `aHasRead`.
    pub(crate) has_read: bool,
}

/// What a state step does next.
pub(crate) enum Step {
    Continue,
    Return(LdomXmlRecordType),
}

/// `LDOM_XmlReader`; `LdomXmlReader::new(&document, false)`.
///
/// ```
/// use rustycad_tkcdf::ldom::{LdomIStream, LdomMemManager, LdomOSStream, LdomXmlReader, LdomXmlRecordType};
/// use rustycad_tkernel::standard::Handle;
/// let document = Handle::new(LdomMemManager::new(1000));
/// let mut reader = LdomXmlReader::new(&document, false);
/// let mut source: &[u8] = b"<a x=\"1\"/>";
/// let mut stream = LdomIStream::new(&mut source);
/// let (mut data, mut start) = (LdomOSStream::new(64), true);
/// assert_eq!(reader.read_record(&mut stream, &mut data, &mut start), LdomXmlRecordType::XmlFullElement);
/// assert_eq!(reader.get_element().unwrap().get_tag_name().get_string(), b"a");
/// ```
pub struct LdomXmlReader {
    /// `myEOF`.
    pub(crate) eof: bool,
    /// `myError` (natively a reference to the parser's error string).
    pub(crate) error: AsciiString,
    /// `myDocument`.
    pub(crate) document: Handle<LdomMemManager>,
    /// `myElement`.
    pub(crate) element: Option<LdomNodeId>,
    /// `myLastChild`.
    pub(crate) last_child: Option<LdomNodeId>,
    /// `myPtr` and `myEndPtr`.
    pub(crate) ptr: usize,
    pub(crate) end: usize,
    /// `myBuffer`.
    pub(crate) buffer: Vec<u8>,
    /// `myTagPerStep`.
    tag_per_step: bool,
    /// `myBOM`.
    bom: LdomBomType,
}

impl LdomXmlReader {
    /// `LDOM_XmlReader(aDocument, anErrorString, theTagPerStep)`: a reader that creates its
    /// elements in `document`; with `tag_per_step` it reads up to one `>` at a time.
    pub fn new(document: &Handle<LdomMemManager>, tag_per_step: bool) -> Self {
        Self {
            eof: false,
            error: AsciiString::default(),
            document: document.clone(),
            element: None,
            last_child: None,
            ptr: 0,
            end: 0,
            buffer: vec![0; XML_BUFFER_SIZE + 4],
            tag_per_step,
            bom: LdomBomType::BomUndefined,
        }
    }
    /// The error of the last record (`myError`).
    pub fn error(&self) -> &AsciiString {
        &self.error
    }
    /// `GetElement`: the element of the last start or full element record.
    pub fn get_element(&self) -> Option<LdomBasicElement> {
        self.element.map(|id| {
            LdomBasicElement::from_node(LdomBasicNode {
                document: self.document.clone(),
                id,
            })
        })
    }
    /// `CreateElement(theName, theLen)`: the current element becomes a new element.
    pub fn create_element(&mut self, name: &[u8], length: i32) {
        self.element = Some(LdomBasicElement::create(Some(name), length, &self.document).id);
    }
    /// `GetBOM`: the byte order mark found at the start of the document.
    pub fn get_bom(&self) -> LdomBomType {
        self.bom
    }
    /// `getInteger(theValue, theStart, theEnd)`: false when the C string `text` is a
    /// decimal integer that ends at `end` (`theValue` becomes it, truncated to `int` as
    /// natively); a multi-digit number with a leading `0` is not one. True otherwise.
    pub fn get_integer(value: &mut LdomBasicString, text: &[u8], end: usize) -> bool {
        if end != 1 && text.first() == Some(&b'0') {
            return true;
        }
        let parsed = strtol(text, 10);
        if parsed.consumed != end || parsed.overflow {
            return true;
        }
        *value = LdomBasicString::from_integer(parsed.value as i32);
        false
    }
    /// `ReadRecord(theIStream, theData, theDocStart)`: reads the next record; its text goes
    /// to `data`. At the document start a byte order mark is read first.
    pub fn read_record(
        &mut self,
        stream: &mut LdomIStream,
        data: &mut LdomOSStream,
        doc_start: &mut bool,
    ) -> LdomXmlRecordType {
        data.clear();
        self.error.clear();
        let mut scan = RecordScan::default();
        loop {
            if !self.refill(stream, data, &mut scan) {
                break;
            }
            if *doc_start && !self.eof {
                *doc_start = false;
                self.read_bom();
                if self.bom != LdomBomType::BomUndefined {
                    continue;
                }
            }
            if let Step::Return(record) = self.step(&mut scan, data) {
                return record;
            }
        }
        if scan.state != ScanState::Waiting {
            self.error = AsciiString::new("Unexpected end of file");
            return LdomXmlRecordType::XmlUnknown;
        }
        LdomXmlRecordType::XmlEof
    }
    /// Reads more input when fewer than `XML_MIN_BUFFER` bytes are left: the pending
    /// record data goes to `data`, the rest moves to the buffer start. False at the end.
    fn refill(
        &mut self,
        stream: &mut LdomIStream,
        data: &mut LdomOSStream,
        scan: &mut RecordScan,
    ) -> bool {
        let rest = self.end.saturating_sub(self.ptr);
        if rest >= XML_MIN_BUFFER || (!self.eof && self.tag_per_step && scan.has_read) {
            return true;
        }
        if self.eof {
            return rest > 0;
        }
        if let Some(start) = scan.start {
            if self.ptr > start {
                let _ = data.rdbuf().xsputn(&self.buffer[start..self.ptr]);
            }
            scan.start = Some(0);
        }
        self.buffer.copy_within(self.ptr..self.ptr + rest, 0);
        self.ptr = 0;
        self.read_block(stream, rest, scan);
        true
    }
    /// One `read` (or `getline` up to `>`) after the `rest` kept bytes.
    fn read_block(&mut self, stream: &mut LdomIStream, rest: usize, scan: &mut RecordScan) {
        let room = &mut self.buffer[rest..XML_BUFFER_SIZE];
        if self.tag_per_step {
            stream.getline(room, b'>');
            scan.has_read = true;
        } else {
            stream.read(room);
        }
        let count = stream.gcount();
        if count == 0 {
            self.eof = true;
        } else if self.tag_per_step {
            // getline drops the '>' (or, at the end of input, this overwrites the last byte).
            self.buffer[rest + count - 1] = b'>';
        }
        self.end = rest + count;
        self.buffer[rest + count] = 0;
    }
    /// The byte at `index` (0 beyond the buffer).
    pub(crate) fn byte(&self, index: usize) -> u8 {
        self.buffer.get(index).copied().unwrap_or(0)
    }
    /// C `memchr` of `byte` in `length` bytes from `from` (a negative length finds nothing).
    pub(crate) fn find(&self, from: usize, byte: u8, length: isize) -> Option<usize> {
        let length = usize::try_from(length).ok()?;
        let region = self
            .buffer
            .get(from..(from + length).min(self.buffer.len()))?;
        region
            .iter()
            .position(|candidate| *candidate == byte)
            .map(|at| from + at)
    }
    /// Adds an attribute to the current element (`AddAttribute` with `myLastChild`).
    pub(crate) fn add_attribute(&mut self, name: &LdomBasicString, value: &LdomBasicString) {
        let (Some(element), last) = (self.element, self.last_child) else {
            return;
        };
        self.last_child = with_store(&self.document, last, |store| {
            store.add_attribute(element, name, value, last)
        });
    }
    /// Detects a byte order mark at `myPtr` and skips it.
    fn read_bom(&mut self) {
        let bytes: [u8; 5] = std::array::from_fn(|offset| self.byte(self.ptr + offset));
        if let Some((bom, length)) = bom_of(&bytes) {
            self.bom = bom;
            self.ptr += length;
        }
    }
}

/// The byte order mark the first bytes start with, and its length.
fn bom_of(bytes: &[u8; 5]) -> Option<(LdomBomType, usize)> {
    let utf7 = |third: u8| matches!(third, 43 | 47 | 56 | 57);
    match *bytes {
        [0xEF, 0xBB, 0xBF, ..] => Some((LdomBomType::BomUtf8, 3)),
        [0xFE, 0xFF, ..] => Some((LdomBomType::BomUtf16be, 2)),
        [0xFF, 0xFE, 0, 0, _] => Some((LdomBomType::BomUtf32le, 4)),
        [0xFF, 0xFE, ..] => Some((LdomBomType::BomUtf16le, 2)),
        [0, 0, 0xFE, 0xFF, _] => Some((LdomBomType::BomUtf32be, 4)),
        [0x2B, 47, 118, 56, 45] => Some((LdomBomType::BomUtf7, 5)),
        [0x2B, 47, 118, third, _] if utf7(third) => Some((LdomBomType::BomUtf7, 4)),
        [0xF7, 100, 76, ..] => Some((LdomBomType::BomUtf1, 3)),
        [0xDD, 115, 102, 115, _] => Some((LdomBomType::BomUtfebcdic, 4)),
        [0x0E, 0xFE, 0xFF, ..] => Some((LdomBomType::BomScsu, 3)),
        [0xFB, 0xEE, 40, ..] => Some((LdomBomType::BomBocu1, 3)),
        [0x84, 49, 0x95, 51, _] => Some((LdomBomType::BomGb18030, 4)),
        _ => None,
    }
}
