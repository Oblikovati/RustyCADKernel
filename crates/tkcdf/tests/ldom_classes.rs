//! Unit tests of the LDOM classes (the native behaviour each port keeps).
use rustycad_tkcdf::ldom::{
    LdomBasicAttribute, LdomBasicElement, LdomBasicNode, LdomBasicString, LdomBasicText,
    LdomBomType, LdomCharacterData, LdomDocument, LdomElement, LdomIStream, LdomMemManager,
    LdomNode, LdomNodeList, LdomNodeType, LdomOSStream, LdomParser, LdomParserHooks, LdomSBuffer,
    LdomString, LdomStringType, LdomXmlReader, LdomXmlRecordType,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::AsciiString;

fn s(text: &str) -> LdomString {
    LdomString::from_c_str(Some(text.as_bytes()))
}

fn manager() -> Handle<LdomMemManager> {
    Handle::new(LdomMemManager::new(1000))
}

#[test]
fn basic_string_copies_compares_and_converts() {
    let free = LdomBasicString::from_c_str(Some(b"12"));
    assert!(
        free != free.clone(),
        "a free string copy is a new allocation"
    );
    assert!(free.equals(&free.clone()));
    let mut value = 0;
    assert!(free.get_integer(&mut value) && value == 12);
    assert!(!LdomBasicString::from_c_str(Some(b"x")).get_integer(&mut value));
    assert!(LdomBasicString::from_c_str(None).is_null());
    let document = manager();
    let short = LdomBasicString::with_length(Some(b"abcdef"), 3, &document);
    assert_eq!(
        (short.string_type(), short.get_string()),
        (LdomStringType::LdomAsciiDoc, &b"abc"[..])
    );
    assert!(LdomBasicString::with_length(Some(b"abc"), 0, &document).is_null());
    assert_eq!(
        LdomBasicString::from_integer(-4)
            .to_ascii_string()
            .as_bytes(),
        b"-4"
    );
    let unicode = LdomBasicString::from_c_str(Some(b"##feff00410042"));
    assert_eq!(
        unicode.to_extended_string(),
        rustycad_tkernel::tcollection::ExtendedString::from_c_string(b"AB", false)
    );
}

#[test]
fn string_keeps_its_document() {
    let document = manager();
    let copied = LdomString::copied_into(&LdomBasicString::from_c_str(Some(b"v")), &document);
    assert_eq!(copied.string_type(), LdomStringType::LdomAsciiDoc);
    assert!(copied.get_owner_document().is_ok());
    let mut nulled = copied.clone();
    nulled.set_null();
    assert!(nulled.is_null() && nulled.get_owner_document().is_ok());
    let free = s("x");
    assert_ne!(free, free.clone(), "a free string copy is a new allocation");
    let shared = LdomString::with_owner(copied.basic(), &document);
    assert_eq!(shared, copied, "a document string copy shares its text");
}

#[test]
fn mem_manager_hashes_and_shares_names() {
    let document = manager();
    let (mut first, mut second) = (0, 0);
    let name = document.hashed_allocate(b"name", 4, &mut first);
    let again = document.hashed_allocate(b"name!", 4, &mut second);
    assert!(name == again, "equal names share one hashed allocation");
    assert_eq!((first, LdomMemManager::hash(b"name", 4)), (second, first));
    assert!(LdomMemManager::compare_strings(b"name", first, &name));
    assert!(!LdomMemManager::compare_strings(
        b"name",
        (first + 1) & 255,
        &name
    ));
    let mut into = LdomBasicString::new();
    document.hashed_allocate_into(b"other", 5, &mut into);
    assert_eq!(into.string_type(), LdomStringType::LdomAsciiHashed);
    assert_eq!(LdomMemManager::new(10).block_size(), 3);
    assert!(document.root_element().is_none() && std::ptr::eq(document.self_ref(), &*document));
}

#[test]
fn basic_nodes_link_and_null() {
    let document = manager();
    let element = LdomBasicElement::create(Some(b"e"), 1, &document);
    let text = LdomBasicText::create(
        LdomNodeType::TextNode,
        &LdomBasicString::from_c_str(Some(b"t")),
        &document,
    );
    let other = LdomBasicText::create(
        LdomNodeType::CommentNode,
        &LdomBasicString::from_integer(2),
        &document,
    );
    let mut last = None;
    element.append_child(&text, &mut last);
    element.append_child(&other, &mut last);
    assert_eq!(
        element.get_last_child().map(|node| node.id()),
        Some(other.id())
    );
    let last_child = element.add_attribute(
        &LdomBasicString::from_c_str(Some(b"a")),
        &LdomBasicString::from_integer(1),
        None,
    );
    assert_eq!(last_child.map(|node| node.id()), Some(other.id()));
    let attribute = element
        .get_attribute(&LdomBasicString::from_c_str(Some(b"a")), None)
        .expect("attribute a");
    assert_eq!(attribute.get_name().get_string(), b"a");
    attribute.set_value(&LdomBasicString::from_c_str(Some(b"z")));
    assert_eq!(attribute.get_value().get_string(), b"z");
    other.set_null();
    assert!(
        other.is_null() && text.get_sibling().map(|node| node.id()) == Some(attribute.id()),
        "GetSibling unlinks a null sibling"
    );
    element.remove_child(&text);
    assert_eq!(text.get_node_type(), LdomNodeType::Unknown);
    assert!(
        LdomBasicAttribute::new(&document).is_null() && LdomBasicElement::new(&document).is_null()
    );
    assert!(
        LdomBasicNode::new(&document, LdomNodeType::TextNode).get_node_type()
            == LdomNodeType::TextNode
    );
}

#[test]
fn node_caches_its_last_child_and_clears_values() {
    let document = LdomDocument::create_document(&s("r"));
    let root = document.get_document_element();
    assert!(root.get_last_child().is_null());
    let text = document.create_text_node(&s("a"));
    root.append_child(&text);
    assert_eq!(root.get_last_child(), **text);
    assert!(LdomNode::new() == LdomNode::new() && LdomNode::new() != **text);
    text.set_value_clear();
    assert_eq!(
        text.get_node_value().string_type(),
        LdomStringType::LdomAsciiDocClear
    );
    assert!(text.get_owner_document().is_ok() && LdomNode::new().get_owner_document().is_err());
    let mut copy = root.clone();
    copy.set_null();
    assert!(copy.is_null() && !root.is_null());
}

#[test]
fn node_list_and_character_data() {
    let document = LdomDocument::create_document(&s("r"));
    let list = document.get_elements_by_tag_name(&s("r"));
    assert!(!list.is_null() && list.item(-1).is_null() && !list.item(0).is_null());
    assert!(LdomNodeList::new().is_null());
    let text = document.create_text_node(&s("four"));
    assert_eq!(text.get_length(), 4);
    text.set_data(&s("seven!!"));
    assert_eq!(text.get_length(), 7);
    let mut data = LdomCharacterData::new();
    data.assign(&text);
    assert_eq!(data.get_data().get_string(), b"seven!!");
    data.set_null();
    assert!(data.is_null());
}

#[test]
fn element_finds_siblings_by_tag() {
    let document = LdomDocument::create_document(&s("r"));
    let root = document.get_document_element();
    for tag in ["a", "b", "a"] {
        root.append_child(&document.create_element(&s(tag)));
    }
    let first = root.get_child_by_tag_name(&s("a"));
    let next = first.get_sibling_by_tag_name();
    assert!(!next.is_null() && next != first);
    assert!(next.get_sibling_by_tag_name().is_null());
    assert!(root.get_child_by_tag_name(&LdomString::new()).is_null());
    assert_eq!(root.get_elements_by_tag_name(&s("a")).get_length(), 2);
    assert!(LdomElement::new().get_attribute(&s("x")).is_null());
}

#[test]
fn document_null_states() {
    let mut document = LdomDocument::create_document(&s("r"));
    assert!(!document.is_null() && !document.eq_null());
    let same = LdomDocument::from_mem_manager(document.mem_manager());
    assert!(same == document);
    document.set_null();
    assert!(document.eq_null() && same != document);
}

#[test]
fn reader_reads_records_and_integers() {
    let document = manager();
    let mut reader = LdomXmlReader::new(&document, false);
    let mut source: &[u8] = b"<?xml?><a b='1'>t<!--c--></a>";
    let mut stream = LdomIStream::new(&mut source);
    let (mut data, mut start) = (LdomOSStream::new(64), true);
    let mut records = Vec::new();
    loop {
        let record = reader.read_record(&mut stream, &mut data, &mut start);
        records.push(record);
        if matches!(
            record,
            LdomXmlRecordType::XmlEof | LdomXmlRecordType::XmlUnknown
        ) {
            break;
        }
    }
    use LdomXmlRecordType::*;
    assert_eq!(
        records,
        [
            XmlHeader,
            XmlStartElement,
            XmlText,
            XmlComment,
            XmlEndElement,
            XmlEof
        ]
    );
    assert_eq!(reader.get_bom(), LdomBomType::BomUndefined);
    let mut value = LdomBasicString::new();
    assert!(!LdomXmlReader::get_integer(&mut value, b"42", 2));
    assert!(
        LdomXmlReader::get_integer(&mut value, b"042", 3),
        "a leading zero is not an integer"
    );
    assert!(LdomXmlReader::get_integer(&mut value, b"4x", 2));
    reader.create_element(b"made", 4);
    assert_eq!(
        reader
            .get_element()
            .expect("element")
            .get_tag_name()
            .get_string(),
        b"made"
    );
}

#[test]
fn stream_buffer_and_istream() {
    let mut buffer = LdomSBuffer::new(8);
    assert_eq!(
        (buffer.overflow(i32::from(b'x')), buffer.underflow()),
        (i32::from(b'x'), -1)
    );
    buffer.clear();
    assert_eq!(buffer.length(), 0);
    let mut source: &[u8] = b"abcdef>gh";
    let mut stream = LdomIStream::new(&mut source);
    let mut line = [0u8; 4];
    stream.getline(&mut line, b'>');
    assert_eq!(
        (stream.gcount(), &line, stream.good()),
        (3, b"abc\0", false),
        "a full line sets fail"
    );
    let mut block = [0u8; 2];
    stream.read(&mut block);
    assert_eq!(stream.gcount(), 0, "a failed stream reads nothing");
}

/// A parser subclass that aborts at its second element.
struct AbortSecond(u32);
impl LdomParserHooks for AbortSecond {
    fn start_element(&mut self, current: &LdomElement) -> bool {
        self.0 += 1;
        self.0 == 2 && current.get_tag_name().get_string() == b"b"
    }
}

#[test]
fn parser_hooks_files_and_errors() {
    let mut parser = LdomParser::with_hooks(Box::new(AbortSecond(0)));
    assert!(parser.parse(&mut &b"<a><b/></a>"[..], false, false));
    let mut data = AsciiString::default();
    assert_eq!(
        parser.get_error(&mut data).as_bytes(),
        b"User abort at startElement()"
    );
    assert_eq!(
        parser.get_current_element().get_tag_name().get_string(),
        b"b"
    );
    let mut parser = LdomParser::new();
    assert!(parser.parse_file(b"/nonexistent/ldom-input.xml"));
    assert_eq!(
        parser.get_error(&mut data).as_bytes(),
        b"Fatal XML error: Cannot open XML file"
    );
    let path = std::env::temp_dir().join(format!("ldom-parser-{}.xml", std::process::id()));
    std::fs::write(&path, b"<file v=\"1\"/>").expect("temporary file");
    assert!(!parser.parse_file(path.to_string_lossy().as_bytes()));
    let _ = std::fs::remove_file(&path);
    assert_eq!(
        parser
            .get_document()
            .get_document_element()
            .get_tag_name()
            .get_string(),
        b"file"
    );
    assert!(!parser.start_element() && !parser.end_element());
}
