// Rust port of the state machine of LDOM_XmlReader::ReadRecord and of isName
// (ApplicationFramework/TKCDF/LDOM/LDOM_XmlReader.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! One step of `ReadRecord` per parser state (ocaf-design.md section 9).
use super::basic_string::LdomBasicString;
use super::c_text::c_str;
use super::char_reference::LdomCharReference;
use super::os_stream::LdomOSStream;
use super::xml_reader::{LdomXmlReader, LdomXmlRecordType, RecordScan, ScanState, Step};
use rustycad_tkernel::tcollection::AsciiString;

/// XML white space.
fn is_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r')
}

impl LdomXmlReader {
    /// One step of the state machine.
    pub(crate) fn step(&mut self, scan: &mut RecordScan, data: &mut LdomOSStream) -> Step {
        match scan.state {
            ScanState::Waiting => self.waiting(scan, data),
            ScanState::Header => self.header(scan, data),
            ScanState::Doctype => self.doctype(scan, data),
            ScanState::DoctypeMarkup => self.doctype_markup(scan, data),
            ScanState::Comment => self.comment(scan, data),
            ScanState::Text => self.text(scan, data),
            ScanState::Cdata => self.cdata(scan, data),
            ScanState::Element => self.element_name(scan, data),
            ScanState::AttributeName => self.attribute_name(scan, data),
            ScanState::AttributeEqual => self.attribute_equal(scan),
            ScanState::AttributeValue => self.attribute_value(scan, data),
            ScanState::ElementEnd => self.element_end(scan, data),
        }
    }
    /// Fails the record with `message`.
    fn fail(&mut self, message: &str) -> Step {
        self.error = AsciiString::new(message);
        Step::Return(LdomXmlRecordType::XmlUnknown)
    }
    /// Copies `start..to` of the buffer to `data`.
    fn put(&self, data: &mut LdomOSStream, start: usize, to: usize) {
        if to > start {
            let _ = data.rdbuf().xsputn(&self.buffer[start..to]);
        }
    }
    /// Waits at the end of the buffer for more input (`myPtr` set back by `keep` bytes).
    fn wait_more(&mut self, scan: &mut RecordScan, keep: usize) -> Step {
        self.ptr = self.end.saturating_sub(keep);
        scan.has_read = false;
        Step::Continue
    }
    /// `STATE_WAITING`: white space is skipped, `<` starts markup, anything else is text.
    fn waiting(&mut self, scan: &mut RecordScan, data: &mut LdomOSStream) -> Step {
        match self.byte(self.ptr) {
            b' ' | b'\t' | b'\n' | b'\r' => self.ptr += 1,
            b'<' => return self.markup(scan),
            // Natively an embedded NUL at the end of input loops forever; the port skips it.
            0 if self.eof => self.ptr += 1,
            _ => return self.text_start(scan, data),
        }
        Step::Continue
    }
    /// Text in `STATE_WAITING`: up to the next `<`, or the whole buffer then `STATE_TEXT`.
    fn text_start(&mut self, scan: &mut RecordScan, data: &mut LdomOSStream) -> Step {
        if let Some(open) = self.find(self.ptr, b'<', self.end as isize - self.ptr as isize) {
            self.put(data, self.ptr, open);
            self.ptr = open;
            return Step::Return(LdomXmlRecordType::XmlText);
        }
        scan.state = ScanState::Text;
        scan.start = Some(self.ptr);
        self.ptr = self.end;
        scan.has_read = false;
        Step::Continue
    }
    /// `<` in `STATE_WAITING`: a declaration, an end tag, a comment, a DOCTYPE, a CDATA
    /// section or an element name; "Unknown XML object" otherwise.
    fn markup(&mut self, scan: &mut RecordScan) -> Step {
        let (state, skip) = match self.byte(self.ptr + 1) {
            b'?' => (ScanState::Header, 2),
            b'/' => (ScanState::ElementEnd, 2),
            b'!' => match self.bang_markup() {
                Some(found) => found,
                None => return self.unknown_object(),
            },
            _ => return self.element_start(scan),
        };
        scan.state = state;
        self.ptr += skip;
        scan.start = Some(self.ptr);
        Step::Continue
    }
    /// `<!`: a comment, a DOCTYPE followed by white space, or a CDATA section.
    fn bang_markup(&self) -> Option<(ScanState, usize)> {
        let at = |offset: usize| self.byte(self.ptr + offset);
        if at(2) == b'-' && at(3) == b'-' {
            return Some((ScanState::Comment, 4));
        }
        let follows =
            |pattern: &[u8]| (0..pattern.len()).all(|index| at(2 + index) == pattern[index]);
        if follows(b"DOCTYPE") {
            return is_space(at(9)).then_some((ScanState::Doctype, 10));
        }
        follows(b"[CDATA[").then_some((ScanState::Cdata, 9))
    }
    /// The "Unknown XML object" error with the next `XML_MIN_BUFFER` bytes.
    fn unknown_object(&mut self) -> Step {
        let shown = self
            .buffer
            .get(self.ptr..(self.ptr + super::xml_reader::XML_MIN_BUFFER).min(self.buffer.len()))
            .unwrap_or_default();
        self.error = AsciiString::new("Unknown XML object: ");
        self.error.assign_cat(AsciiString::from_bytes(shown));
        Step::Return(LdomXmlRecordType::XmlUnknown)
    }
    /// `<name`: the element is created when the name ends in the buffer, else
    /// `STATE_ELEMENT` waits for the rest of the name.
    fn element_start(&mut self, scan: &mut RecordScan) -> Step {
        if !self.is_name(self.ptr + 1, scan) {
            return self.unknown_object();
        }
        let start = self.ptr + 1;
        scan.start = Some(start);
        self.ptr = scan.name_end;
        if self.ptr >= self.end {
            scan.state = ScanState::Element;
            return Step::Continue;
        }
        let name = self.buffer[start..self.ptr].to_vec();
        self.create_element(&name, name.len() as i32);
        self.last_child = None;
        scan.state = ScanState::AttributeName;
        scan.start = None;
        Step::Continue
    }
    /// `STATE_HEADER`: the XML declaration up to `?>`.
    fn header(&mut self, scan: &mut RecordScan, data: &mut LdomOSStream) -> Step {
        let start = scan.start.unwrap_or(self.ptr);
        let Some(mark) = self.find(start, b'?', self.end as isize - 1 - start as isize) else {
            return self.wait_more(scan, 1);
        };
        if self.byte(mark + 1) != b'>' {
            return self.fail("Character '>' is expected in the end of XML declaration");
        }
        self.put(data, start, mark);
        self.ptr = mark + 2;
        Step::Return(LdomXmlRecordType::XmlHeader)
    }
    /// `STATE_DOCTYPE`: up to `>` (the byte before it is left out, as natively), or `[`
    /// that starts the markup declarations.
    fn doctype(&mut self, scan: &mut RecordScan, data: &mut LdomOSStream) -> Step {
        let start = scan.start.unwrap_or(self.ptr);
        for at in start..self.end.saturating_sub(1) {
            if self.buffer[at] == b'[' {
                scan.state = ScanState::DoctypeMarkup;
                scan.start = Some(at + 1);
                return self.doctype_markup(scan, data);
            }
            if self.buffer[at] == b'>' {
                self.put(data, start, at.saturating_sub(1).max(start));
                self.ptr = at + 1;
                return Step::Return(LdomXmlRecordType::XmlDoctype);
            }
        }
        self.wait_more(scan, 1)
    }
    /// `STATE_DOCTYPE_MARKUP`: up to `]>`.
    fn doctype_markup(&mut self, scan: &mut RecordScan, data: &mut LdomOSStream) -> Step {
        let start = scan.start.unwrap_or(self.ptr);
        let Some(close) = self.find(start, b']', self.end as isize - 1 - start as isize) else {
            return self.wait_more(scan, 1);
        };
        if self.byte(close + 1) != b'>' {
            return self.fail("Character '>' is expected in the end of DOCTYPE declaration");
        }
        self.put(data, start, close);
        self.ptr = close + 2;
        Step::Return(LdomXmlRecordType::XmlDoctype)
    }
    /// `STATE_COMMENT`: up to `-->`; `--` without `>` is an error.
    fn comment(&mut self, scan: &mut RecordScan, data: &mut LdomOSStream) -> Step {
        let start = scan.start.unwrap_or(self.ptr);
        let mut at = start;
        while let Some(dash) = self.find(at, b'-', self.end as isize - 2 - at as isize) {
            if self.byte(dash + 1) != b'-' {
                at = dash + 1;
                continue;
            }
            if self.byte(dash + 2) != b'>' {
                return self.fail("Character '>' is expected in the end of comment");
            }
            self.put(data, start, dash);
            self.ptr = dash + 3;
            return Step::Return(LdomXmlRecordType::XmlComment);
        }
        self.wait_more(scan, 2)
    }
    /// `STATE_TEXT`: up to the next `<`.
    fn text(&mut self, scan: &mut RecordScan, data: &mut LdomOSStream) -> Step {
        let start = scan.start.unwrap_or(self.ptr);
        let Some(open) = self.find(start, b'<', self.end as isize - start as isize) else {
            return self.wait_more(scan, 0);
        };
        self.put(data, start, open);
        self.ptr = open;
        Step::Return(LdomXmlRecordType::XmlText)
    }
    /// `STATE_CDATA`: up to `]]`; the `>` after it is left for the next record, and a
    /// single `]` is an error, as natively.
    fn cdata(&mut self, scan: &mut RecordScan, data: &mut LdomOSStream) -> Step {
        let start = scan.start.unwrap_or(self.ptr);
        let Some(close) = self.find(start, b']', self.end as isize - 1 - start as isize) else {
            return self.wait_more(scan, 1);
        };
        if self.byte(close + 1) != b']' {
            return self.fail("Characters ']]' are expected in the end of CDATA");
        }
        self.put(data, start, close);
        self.ptr = close + 2;
        Step::Return(LdomXmlRecordType::XmlCdata)
    }
    /// `STATE_ELEMENT`: the rest of an element name after a buffer refill.
    fn element_name(&mut self, scan: &mut RecordScan, data: &mut LdomOSStream) -> Step {
        if !self.is_name(self.ptr, scan) && (data.length() == 0 || scan.name_end != self.ptr) {
            return self.fail("Invalid tag name");
        }
        self.put(data, scan.start.unwrap_or(self.ptr), scan.name_end);
        let name = data.str();
        self.create_element(&name, data.length());
        data.clear();
        self.last_child = None;
        scan.state = ScanState::AttributeName;
        scan.start = None;
        self.ptr = scan.name_end;
        Step::Continue
    }
    /// `STATE_ELEMENT_END`: the name of an end tag up to `>`.
    fn element_end(&mut self, scan: &mut RecordScan, data: &mut LdomOSStream) -> Step {
        let start = scan.start.unwrap_or(self.ptr);
        let Some(close) = self.find(start, b'>', self.end as isize - start as isize) else {
            return self.wait_more(scan, 0);
        };
        self.put(data, start, close);
        self.ptr = close + 1;
        Step::Return(LdomXmlRecordType::XmlEndElement)
    }
    /// `STATE_ATTRIBUTE_NAME`: white space, the end of the start tag (`>` or `/>`) or an
    /// attribute name.
    fn attribute_name(&mut self, scan: &mut RecordScan, data: &mut LdomOSStream) -> Step {
        match self.byte(self.ptr) {
            byte if is_space(byte) => {
                if scan.start.is_some() {
                    return self.attribute_name_from_data(scan, data);
                }
                self.ptr += 1;
                Step::Continue
            }
            b'/' if scan.start.is_some() => self.fail("Inexpected end of attribute"),
            b'/' if self.byte(self.ptr + 1) != b'>' => {
                self.fail("Improper element tag termination")
            }
            b'/' => {
                self.ptr += 2;
                Step::Return(LdomXmlRecordType::XmlFullElement)
            }
            b'>' if scan.start.is_some() => self.fail("Inexpected end of attribute"),
            b'>' => {
                self.ptr += 1;
                Step::Return(LdomXmlRecordType::XmlStartElement)
            }
            _ => self.attribute_name_text(scan, data),
        }
    }
    /// An attribute name in `STATE_ATTRIBUTE_NAME`: kept for the next refill when it reaches
    /// the buffer end.
    fn attribute_name_text(&mut self, scan: &mut RecordScan, data: &mut LdomOSStream) -> Step {
        if !self.is_name(self.ptr, scan) && (data.length() == 0 || scan.name_end != self.ptr) {
            return self.fail("Invalid attribute name");
        }
        if scan.name_end >= self.end {
            scan.start = Some(self.ptr);
            self.ptr = scan.name_end;
            return Step::Continue;
        }
        if data.length() > 0 {
            self.put(data, self.ptr, scan.name_end);
            return self.attribute_name_from_data(scan, data);
        }
        let name = &self.buffer[self.ptr..scan.name_end];
        scan.attribute_name =
            LdomBasicString::with_length(Some(name), name.len() as i32, &self.document);
        self.attribute_name_done(scan)
    }
    /// `attr_name:`: the name collected in `data`. Native quirk: `myPtr` then takes the
    /// name end of the last name test, even when that test was before a buffer refill.
    fn attribute_name_from_data(&mut self, scan: &mut RecordScan, data: &mut LdomOSStream) -> Step {
        let name = data.str();
        data.clear();
        scan.attribute_name = LdomBasicString::with_document(Some(&name), &self.document);
        self.attribute_name_done(scan)
    }
    /// The attribute name is known: wait for `=`.
    fn attribute_name_done(&mut self, scan: &mut RecordScan) -> Step {
        scan.start = None;
        scan.state = ScanState::AttributeEqual;
        self.ptr = scan.name_end;
        Step::Continue
    }
    /// `STATE_ATTRIBUTE_EQUAL`: white space and `=`.
    fn attribute_equal(&mut self, scan: &mut RecordScan) -> Step {
        match self.byte(self.ptr) {
            b'=' => scan.state = ScanState::AttributeValue,
            byte if is_space(byte) => {}
            _ => return self.fail("Equal sign expected in attribute definition"),
        }
        self.ptr += 1;
        Step::Continue
    }
    /// `STATE_ATTRIBUTE_VALUE`: the quoted value (the native switch with case labels inside
    /// its `if` blocks: white space before the quote is skipped, another byte is an error
    /// unless a value is open).
    fn attribute_value(&mut self, scan: &mut RecordScan, data: &mut LdomOSStream) -> Step {
        let byte = self.byte(self.ptr);
        let quote = byte == b'"' || byte == b'\'';
        if is_space(byte) && scan.start.is_none() {
            self.ptr += 1;
            return Step::Continue;
        }
        if !is_space(byte) && !quote && scan.delimiter == 0 {
            return self.fail("Expected an attribute value");
        }
        if quote && scan.start.is_none() {
            scan.start = Some(self.ptr + 1);
            scan.delimiter = byte;
        }
        let start = scan.start.unwrap_or(self.ptr);
        match self.find(start, scan.delimiter, self.end as isize - start as isize) {
            Some(close) => self.store_attribute(scan, data, start, close),
            None => self.wait_more(scan, 0),
        }
    }
    /// The value ends at `close`: decoded, read as an integer when it is one, and added to
    /// the element.
    fn store_attribute(
        &mut self,
        scan: &mut RecordScan,
        data: &mut LdomOSStream,
        start: usize,
        close: usize,
    ) -> Step {
        self.buffer[close] = 0;
        scan.delimiter = 0;
        let (mut text, end) = if data.length() > 0 {
            self.put(data, start, close);
            let text = data.str();
            let end = c_str(&text).len();
            (text, end)
        } else {
            (self.buffer[start..=close].to_vec(), close - start)
        };
        let Some(value) = self.decoded_value(&mut text, end) else {
            // Natively the null result of Decode is dereferenced.
            return self.fail("Invalid character reference in attribute value");
        };
        data.clear();
        let name = std::mem::take(&mut scan.attribute_name);
        self.add_attribute(&name, &value);
        scan.attribute_name = name;
        self.ptr = close + 1;
        scan.start = None;
        scan.state = ScanState::AttributeName;
        Step::Continue
    }
    /// `Decode` of a value, then an integer (`getInteger` against the undecoded end) or a
    /// document string of the decoded length.
    pub(crate) fn decoded_value(&self, text: &mut [u8], end: usize) -> Option<LdomBasicString> {
        let mut length = 0;
        if !LdomCharReference::decode(text, &mut length) {
            return None;
        }
        let mut value = LdomBasicString::new();
        if text.first().is_some_and(u8::is_ascii_digit) && !Self::get_integer(&mut value, text, end)
        {
            return Some(value);
        }
        Some(LdomBasicString::with_length(
            Some(text),
            length,
            &self.document,
        ))
    }
    /// `isName(aString, aStringEnd, aNameEnd)`: a letter, `_` or `:` then name bytes up to a
    /// delimiter or the buffer end (true), or up to another byte (false); the end goes to
    /// `scan.name_end`.
    pub(crate) fn is_name(&self, from: usize, scan: &mut RecordScan) -> bool {
        let first = self.byte(from);
        if !(first.is_ascii_alphabetic() || first == b'_' || first == b':') {
            scan.name_end = from;
            return false;
        }
        let mut at = from + 1;
        while at < self.end {
            match self.buffer[at] {
                b' ' | b'\n' | b'\r' | b'\t' | b'=' | 0 | b'/' | b'>' => break,
                byte if byte.is_ascii_alphanumeric()
                    || matches!(byte, b'.' | b'-' | b'_' | b':') =>
                {
                    at += 1
                }
                _ => {
                    scan.name_end = at;
                    return false;
                }
            }
        }
        scan.name_end = at;
        true
    }
}
