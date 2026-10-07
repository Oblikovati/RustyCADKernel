//! The content-buffer, validation and resource-file cases of the native oracle.
use super::memory_stream::{memory_istream, memory_ostream};
use super::oracle_lines::{Lines, b, esc, s};
use super::oracle_registry::registry;
use super::probe_node::probe_node;
use rustycad_tkde::de::{
    DeConfigurationContext, DeConfigurationNode, DeProvider, DeProviderReadStreamList,
    DeProviderReadStreamNode, DeProviderWriteStreamList, DeProviderWriteStreamNode,
    DeShapeFixConfigurationNode, DeValidationUtils, DeWrapper,
};
use rustycad_tkernel::{
    ncollection::{Buffer, List},
    osd::{IStream, SeekDir},
    standard::{Handle, RuntimeTyped},
    tcollection::AsciiString,
};
use rustycad_tklcaf::tdoc_std::TDocStdDocument;

pub fn stream_lookups(lines: &mut Lines) {
    let handle = registry();
    let wrapper = handle.get().expect("a wrapper");
    let mut step = memory_istream(b"ISO-10303-21;\nHEADER;\n");
    let mut found: Handle<dyn DeProvider> = Handle::default();
    let found_step = wrapper.find_read_provider_from_stream(&s("model.dat"), &mut step, &mut found);
    let vendor = esc(&found.get().expect("a provider").get_vendor());
    lines.line(
        "content.stream",
        format!(
            "{} {vendor} {}",
            b(found_step.expect("a lookup")),
            step.tellg()
        ),
    );
    let mut other = memory_istream(b"solid x\n");
    let mut none: Handle<dyn DeProvider> = Handle::default();
    let found_other =
        wrapper.find_read_provider_from_stream(&s("model.dat"), &mut other, &mut none);
    lines.line("content.other", b(found_other.expect("a lookup")));
}

pub fn content_cases(lines: &mut Lines) {
    stream_lookups(lines);
    let mut long = memory_istream(&[b'a'; 3000]);
    long.seekg(5, SeekDir::Beg);
    let mut buffer: Handle<Buffer> = Handle::default();
    let created =
        DeValidationUtils::create_content_buffer(&mut long, &mut buffer).expect("a buffer");
    let bytes = buffer.get().expect("a buffer").as_slice();
    let text = format!(
        "{} {} {} {} {}",
        b(created),
        bytes.len(),
        long.tellg(),
        bytes[2047],
        bytes[2046]
    );
    lines.line("content.buffer", text);
    let mut short = memory_istream(b"abc");
    let created =
        DeValidationUtils::create_content_buffer(&mut short, &mut buffer).expect("a buffer");
    let terminator = buffer.get().expect("a buffer").as_slice()[3];
    lines.line(
        "content.short",
        format!("{} {terminator} {}", b(created), short.tellg()),
    );
    let missing =
        DeValidationUtils::create_content_buffer_from_path(&s("/no/such/file.stp"), &mut buffer);
    lines.line(
        "content.missing",
        format!("{}{}", b(missing.expect("no failure")), b(buffer.is_null())),
    );
}

pub fn stream_validation(lines: &mut Lines) {
    let context = s("ctx");
    let (mut good, mut bad) = (memory_istream(b"x"), memory_istream(b""));
    bad.read(&mut [0_u8; 1]);
    let empty = DeProviderReadStreamList::default();
    let mut good_list = DeProviderReadStreamList::default();
    good_list.append(DeProviderReadStreamNode::new(&s("a"), &mut good));
    let mut bad_list = DeProviderReadStreamList::default();
    bad_list.append(DeProviderReadStreamNode::new(&s(""), &mut bad));
    let read = [&empty, &good_list, &bad_list].map(|streams| {
        b(DeValidationUtils::validate_read_stream_list(
            streams, &context, false,
        ))
    });
    lines.line(
        "valid.streams",
        format!("{}{}{}", read[0], read[1], read[2]),
    );
    let (mut out, mut bad_out) = (memory_ostream(), memory_ostream());
    rustycad_tkernel::osd::OStream::seekp(&mut bad_out, -1, SeekDir::Beg);
    let mut write_list = DeProviderWriteStreamList::default();
    write_list.append(DeProviderWriteStreamNode::new(&s("w"), &mut out));
    let mut bad_write = DeProviderWriteStreamList::default();
    bad_write.append(DeProviderWriteStreamNode::new(&s("w"), &mut bad_out));
    let write = [
        &DeProviderWriteStreamList::default(),
        &write_list,
        &bad_write,
    ]
    .map(|streams| {
        b(DeValidationUtils::validate_write_stream_list(
            streams, &context, false,
        ))
    });
    lines.line(
        "valid.wstreams",
        format!("{}{}{}", write[0], write[1], write[2]),
    );
}

pub fn validation_cases(lines: &mut Lines) {
    let readable_path =
        std::env::temp_dir().join(format!("tkde_oracle_read_{}.txt", std::process::id()));
    std::fs::write(&readable_path, "x").expect("a readable file");
    let readable = s(readable_path.to_str().expect("a UTF-8 path"));
    let context = s("ctx");
    let node = probe_node("STEP", "OCC", "step", true, true, "").cast::<dyn DeConfigurationNode>();
    let shape_fix = <dyn DeShapeFixConfigurationNode as RuntimeTyped>::static_type();
    let any_node = <dyn DeConfigurationNode as RuntimeTyped>::static_type();
    let checks = [
        DeValidationUtils::validate_configuration_node(&node, shape_fix, &context, false),
        DeValidationUtils::validate_configuration_node(
            &node,
            DeWrapper::static_type(),
            &context,
            false,
        ),
        DeValidationUtils::validate_configuration_node(
            &Handle::default(),
            any_node,
            &context,
            false,
        ),
    ];
    lines.line(
        "valid.node",
        checks.map(|check| b(check).to_string()).concat(),
    );
    let read = [
        "",
        "/no/such/file",
        readable_path.to_str().unwrap_or_default(),
    ]
    .map(|path| {
        b(DeValidationUtils::validate_file_for_reading(
            &s(path),
            &context,
            false,
        ))
        .to_string()
    });
    lines.line("valid.read", read.concat());
    let write = [s(""), s("/no/such/dir/file"), readable].map(|path| {
        b(DeValidationUtils::validate_file_for_writing(
            &path, &context, false,
        ))
        .to_string()
    });
    lines.line("valid.write", write.concat());
    std::fs::remove_file(&readable_path).expect("cleanup");
    stream_validation(lines);
    lines.line(
        "valid.unit",
        b(DeValidationUtils::warn_length_unit_not_supported(
            2.0, &context, false,
        )),
    );
    let document: Handle<TDocStdDocument> = Handle::default();
    lines.line(
        "valid.doc",
        b(DeValidationUtils::validate_document(
            &document, &context, false,
        )),
    );
}

pub fn file_cases(lines: &mut Lines) {
    let context = DeConfigurationContext::new();
    lines.line(
        "file.missing",
        b(context
            .load_file(&s("/no/such/dir/conf.txt"))
            .expect("no failure")),
    );
    let node = probe_node("STL", "OCC", "stl", true, true, "");
    let object = node.get().expect("a node");
    object.set_value(13);
    let path_buf =
        std::env::temp_dir().join(format!("tkde_oracle_conf_{}.txt", std::process::id()));
    let path = s(path_buf.to_str().expect("a UTF-8 path"));
    lines.line(
        "file.save",
        b(object.save_resource_file(&path).expect("no failure")),
    );
    lines.line(
        "file.loadfile",
        b(context.load_file(&path).expect("no failure")),
    );
    lines.line(
        "file.value",
        context.integer_val("provider.STL.OCC.value", -1, ""),
    );
    lines.line("file.extent", context.get_internal_map().extent());
    let reader = probe_node("STL", "OCC", "stl", true, true, "");
    let reader_object = reader.get().expect("a node");
    let loaded = reader_object.load_resource_file(&path).expect("no failure");
    lines.line(
        "file.nodeload",
        format!("{} {}", b(loaded), reader_object.value()),
    );
    let dispatched = DeConfigurationContext::new();
    let dispatch = dispatched.load(&path).expect("no failure");
    lines.line(
        "file.load.dispatch",
        format!("{} {}", b(dispatch), dispatched.get_internal_map().extent()),
    );
    let empty = match reader_object.load_resource_file(&s("")) {
        Ok(loaded) => b(loaded).to_string(),
        Err(failure) => format!("throw {}", failure.exception_type()),
    };
    lines.line("file.nodeload.empty", empty);
    let wrapper = DeWrapper::new();
    wrapper.bind(&node.cast());
    let none: List<AsciiString> = List::default();
    lines.line(
        "file.wrapper.save",
        b(wrapper
            .save_to_file(&path, true, &none, &none)
            .expect("no failure")),
    );
    let bad = wrapper.save_to_file(&s("/no/such/dir/x.txt"), true, &none, &none);
    lines.line("file.wrapper.bad", b(bad.expect("no failure")));
    std::fs::remove_file(&path_buf).expect("cleanup");
}
