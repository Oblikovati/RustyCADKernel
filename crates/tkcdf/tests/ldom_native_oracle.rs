//! The LDOM port against the native oracle: `tests/data/ldom_native_oracle.txt` is the
//! output of a native OCCT program that runs the same cases (parse and write, the DOM
//! quirks, the output stream and the character references); this test prints the same
//! `key|value` lines from the port and compares them.
use rustycad_tkcdf::ldom::{
    LdomCharReference, LdomDocument, LdomElement, LdomOSStream, LdomParser, LdomString,
    LdomXmlWriter,
};
use rustycad_tkernel::tcollection::AsciiString;

/// The native `esc`: new lines, NULs and `|` made visible.
fn esc(bytes: &[u8]) -> String {
    let mut text = String::new();
    for byte in bytes {
        match byte {
            b'\n' => text.push_str("\\n"),
            0 => text.push_str("\\0"),
            b'|' => text.push_str("\\p"),
            _ => text.push(char::from(*byte)),
        }
    }
    text
}

/// A string argument of the native calls.
fn s(text: &str) -> LdomString {
    LdomString::from_c_str(Some(text.as_bytes()))
}

/// `LDOM_XmlWriter` with an indentation, into bytes.
fn written(document: &LdomDocument, indent: i32) -> Vec<u8> {
    let mut output = Vec::new();
    let mut writer = LdomXmlWriter::new(None);
    writer.set_indentation(indent);
    writer
        .write_document(&mut output, document)
        .expect("writing to memory");
    output
}

/// One parse of the oracle: the error, the written document and the byte order mark.
fn parse_case(lines: &mut Vec<String>, key: &str, xml: &[u8], per_step: bool, no_root: bool) {
    let mut parser = LdomParser::new();
    let mut input = xml;
    let failed = parser.parse(&mut input, per_step, no_root);
    let mut data = AsciiString::default();
    let error = parser.get_error(&mut data).clone();
    lines.push(format!(
        "{key}.err|{}|{}|{}",
        u8::from(failed),
        esc(error.as_bytes()),
        esc(data.as_bytes())
    ));
    if !failed {
        lines.push(format!(
            "{key}.out0|{}",
            esc(&written(&parser.get_document(), 0))
        ));
        lines.push(format!(
            "{key}.out2|{}",
            esc(&written(&parser.get_document(), 2))
        ));
    }
    lines.push(format!("{key}.bom|{}", parser.get_bom() as i32));
}

/// The parse cases of the oracle.
fn parse_cases(lines: &mut Vec<String>) {
    let document = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!-- head -->\n\
<root a=\"1\" b=\"x&amp;y\" c=\"007\" d=\"3000000000\" e=\"\" f='q\"t'>\n  \
<child n=\"-5\">text &lt;here&gt; &#65;&#x42;</child>\n  <empty/>\n  <!-- inner -->\n  \
<num>42</num><z>0</z><w>012</w>\n  <cd><![CDATA[raw <x> & y]]></cd>\n</root>\n";
    parse_case(lines, "doc", document, false, false);
    parse_case(lines, "step", document, true, false);
    parse_case(lines, "noroot", b"<a/><b>t</b>", false, true);
    let cases: [(&str, &[u8]); 20] = [
        ("bom", b"\xEF\xBB\xBF<r x=\"1\"/>"),
        ("mismatch", b"<a><b></a>"),
        ("eof", b"<a><b>"),
        ("trunc", b"<a x=\"1\""),
        ("hdr_late", b"<a/><?xml version=\"1.0\"?>"),
        ("text_after", b"<a/>tail"),
        ("doctype", b"<!DOCTYPE r [ <!ELEMENT r ANY> ]><r/>"),
        ("doctype2", b"<!DOCTYPE r SYSTEM \"r.dtd\"><r/>"),
        ("badname", b"<a 1x=\"2\"/>"),
        ("noeq", b"<a x \"2\"/>"),
        ("noquote", b"<a x=2/>"),
        ("badslash", b"<a x=\"1\"/ >"),
        ("cdata_single", b"<a><![CDATA[x]y]]></a>"),
        ("comment_dd", b"<a><!-- a--b --></a>"),
        ("unknown", b"<a><!X></a>"),
        ("dup_attr", b"<a x=\"1\" x=\"2\" y=\"3\"/>"),
        ("ws_value", b"<a x = \"1\"   y=  '2' />"),
        ("ctrl", b"<a x=\"&#1;&#9;\">&#2;</a>"),
        ("empty", b""),
        ("only_ws", b"  \n "),
    ];
    for (key, xml) in cases {
        parse_case(lines, key, xml, false, false);
    }
}

/// A document larger than the read buffer.
fn big_case(lines: &mut Vec<String>) {
    let mut big = String::from("<r>");
    for index in 0..3000 {
        big.push_str(&format!(
            "<item idx=\"{index}\" name=\"n{index}\">v{index}</item>"
        ));
    }
    big.push_str("</r>");
    let mut parser = LdomParser::new();
    let failed = parser.parse(&mut big.as_bytes(), false, false);
    let output = written(&parser.get_document(), 0);
    lines.push(format!("big.err|{}", u8::from(failed)));
    lines.push(format!(
        "big.len|{}|{}",
        output.len(),
        esc(&output[output.len() - 60..])
    ));
    let items = parser
        .get_document()
        .get_elements_by_tag_name(&s("item"))
        .get_length();
    lines.push(format!("big.items|{items}"));
}

/// The attribute quirks: `getAttributeNode` skips the first attribute of an element with
/// children and `removeAttribute` removes nothing.
fn attribute_cases(lines: &mut Vec<String>, document: &LdomDocument, root: &LdomElement) {
    root.append_child(&document.create_element(&s("c")));
    root.set_attribute(&s("first"), &s("1"));
    root.set_attribute(&s("second"), &s("2"));
    root.set_attribute(&s("third"), &LdomString::from_integer(3));
    root.set_attribute(&s("second"), &s("two"));
    let null_node = |name: &str| {
        u8::from(
            document
                .get_document_element()
                .get_attribute_node(&s(name))
                .is_null(),
        )
    };
    lines.push(format!("attrnode.first|{}", null_node("first")));
    lines.push(format!("attrnode.third|{}", null_node("third")));
    let fresh = document.get_document_element();
    lines.push(format!(
        "attr.first|{}",
        esc(fresh.get_attribute(&s("first")).get_string())
    ));
    fresh.remove_attribute(&s("first"));
    lines.push(format!(
        "removed.first|{}",
        esc(fresh.get_attribute(&s("first")).get_string())
    ));
    lines.push(format!(
        "attrs.count|{}",
        fresh.get_attributes_list().get_length()
    ));
    lines.push(format!("dom.out|{}", esc(&written(document, 0))));
}

/// Children of every kind, `removeChild`, the tag searches and `ReplaceElement`.
fn child_cases(lines: &mut Vec<String>, document: &LdomDocument, root: &LdomElement) {
    let first = root.get_child_by_tag_name(&s("c"));
    let second = document.create_element(&s("c2"));
    root.append_child(&second);
    root.append_child(&document.create_text_node(&s("t&t")));
    root.append_child(&document.create_comment(&s("cm")));
    root.append_child(&document.create_cdata_section(&s("cd<")));
    lines.push(format!("dom.out2|{}", esc(&written(document, 2))));
    root.remove_child(&first);
    lines.push(format!(
        "removed.child|{}|{}",
        u8::from(first.is_null()),
        esc(&written(document, 0))
    ));
    let missing = |name: &str| u8::from(root.get_child_by_tag_name(&s(name)).is_null());
    lines.push(format!("child.by.tag|{}|{}", missing("c2"), missing("c")));
    lines.push(format!(
        "has.children|{}|{}",
        u8::from(root.has_child_nodes()),
        u8::from(second.has_child_nodes())
    ));
    lines.push(format!(
        "text.len|{}",
        document.create_text_node(&s("hello")).get_length()
    ));
    replace_cases(lines, document, root, &second);
}

/// `ReplaceElement` from another document and inside one document.
fn replace_cases(
    lines: &mut Vec<String>,
    document: &LdomDocument,
    root: &LdomElement,
    second: &LdomElement,
) {
    let other = LdomDocument::create_document(&s("src"));
    let source = other.get_document_element();
    source.set_attribute(&s("k"), &s("v"));
    source.set_attribute(&s("n"), &LdomString::from_integer(7));
    let inner = other.create_element(&s("s1"));
    source.append_child(&inner);
    inner.append_child(&other.create_text_node(&s("inner")));
    let target = document.create_element(&s("target"));
    root.append_child(&target);
    target.replace_element(&source);
    lines.push(format!("replace.other|{}", esc(&written(document, 0))));
    let same = document.create_element(&s("same"));
    root.append_child(&same);
    same.replace_element(second);
    lines.push(format!("replace.same|{}", esc(&written(document, 0))));
}

/// `LDOM_OSStream` chunks and `LDOM_CharReference`.
fn text_cases(lines: &mut Vec<String>) {
    let mut stream = LdomOSStream::new(4);
    stream.rdbuf().xsputn(b"ab\0cd").expect("small write");
    stream.rdbuf().xsputn(b"efgh").expect("small write");
    lines.push(format!("os.len|{}|{}", stream.length(), esc(&stream.str())));
    let decodes: [&[u8]; 8] = [
        b"a&lt;b&#x41;&#66;&amp;&quot;&apos;&gt;",
        b"x&unknown;y&",
        b"&#300;",
        b"&#0;",
        b"&#x4G;",
        b"&# 65;",
        b"&#x0x41;",
        b"q&amp",
    ];
    for input in decodes {
        let mut text = input.to_vec();
        let mut length = -1;
        let decoded = if LdomCharReference::decode(&mut text, &mut length) {
            esc(&text[..length as usize])
        } else {
            "NULL".to_owned()
        };
        lines.push(format!(
            "decode|{}|{decoded}|{}",
            esc(input),
            if decoded == "NULL" { -1 } else { length }
        ));
    }
    let encodes: [&[u8]; 3] = [b"plain", b"a<b>&\"c'", b"\x01\x7f\x80\xff\t\n\r"];
    for input in encodes {
        for attribute in [false, true] {
            let mut length = -1;
            let encoded = LdomCharReference::encode(input, &mut length, attribute);
            lines.push(format!(
                "encode|{}|{}|{length}",
                u8::from(attribute),
                esc(&encoded)
            ));
        }
    }
}

#[test]
fn ldom_matches_the_native_oracle() {
    let mut lines = Vec::new();
    parse_cases(&mut lines);
    big_case(&mut lines);
    let document = LdomDocument::create_document(&s("root"));
    let root = document.get_document_element();
    attribute_cases(&mut lines, &document, &root);
    child_cases(&mut lines, &document, &root);
    text_cases(&mut lines);
    let expected = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/ldom_native_oracle.txt"
    ))
    .expect("oracle file");
    let expected = String::from_utf8_lossy(&expected);
    let expected: Vec<&str> = expected.lines().collect();
    for (index, (port, native)) in lines.iter().zip(&expected).enumerate() {
        assert_eq!(
            port.as_bytes(),
            native.as_bytes(),
            "line {} differs",
            index + 1
        );
    }
    assert_eq!(lines.len(), expected.len(), "number of oracle lines");
}
