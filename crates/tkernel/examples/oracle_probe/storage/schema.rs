//! Data, schema and binary-stream scripts (twin of `tkernel_storage_schema_probe.cxx`).
use super::common::{
    ascii, ascii_hex, driver, ext, file_name, header_text, hex, mode, roots_text, split, thrown,
    types_text, unhex,
};
use rustycad_tkernel::fsd::{BinaryFile, MemoryFileSystem};
use rustycad_tkernel::standard::{Failure, Handle, StandardPersistent};
use rustycad_tkernel::storage::{
    BucketIterator, BucketOfPersistent, Data, DefaultCallBack, DriverBase, HeaderSections,
    IoStream, OpenMode, Schema,
};
use rustycad_tkernel::tcollection::AsciiString;
use std::rc::Rc;
use std::str::SplitWhitespace;
use std::sync::Arc;

/// The output of a schema-level operation, or `None` when `op` belongs elsewhere.
pub fn evaluate(op: &str, input: &mut SplitWhitespace<'_>) -> Option<String> {
    Some(match op {
        "stor_schema" => schema_script(input),
        "stor_data" => data_script(input),
        "stor_bucket" => bucket_script(input),
        "stor_cb" => callback_script(input),
        "stor_complete" => complete_script(input),
        "stor_bstream" => super::bstream::script(input),
        _ => return None,
    })
}

pub fn data_text(data: &Handle<Data>) -> String {
    let set = data.get().expect("data");
    format!(
        " hdr={} rdt={} tdt={}",
        header_text(&set.header_data()),
        roots_text(&set.root_data()),
        types_text(&set.type_data())
    )
}

fn configure(token: &str, set: &Data, schema: &mut Schema) {
    let (key, value) = token.split_once('=').unwrap_or((token, ""));
    match key {
        "app" => set.set_application_name(&ext(value)),
        "ver" => set.set_application_version(&ascii(value)),
        "type" => set.set_data_type(&ext(value)),
        "info" => set.add_to_user_info(&ascii(value)),
        "com" => set.add_to_comments(&ext(value)),
        "root" => set.add_named_root(&ascii(value), &Handle::default()),
        "aroot" => set.add_root(&Handle::default()),
        "oroot" => set.add_named_root(&ascii(value), &Handle::new(StandardPersistent::default())),
        "sname" => schema.set_name(&ascii(value)),
        "sver" => schema.set_version(&ascii(value)),
        _ => {}
    }
}

fn schema_script(input: &mut SplitWhitespace<'_>) -> String {
    let kind = input
        .next()
        .expect("kind")
        .chars()
        .next()
        .expect("kind letter");
    let letter = input
        .next()
        .expect("mode")
        .chars()
        .next()
        .expect("mode letter");
    let data = Handle::new(Data::default());
    let mut schema = Schema::default();
    for token in input.by_ref() {
        configure(token, data.get().expect("data"), &mut schema);
    }
    let files = MemoryFileSystem::default();
    let mut out;
    {
        let mut driver = driver(kind, &files);
        let opened = driver.open(&file_name(), mode(letter)).expect("open");
        out = format!("open={}", opened.ordinal());
        out += &match schema.write(driver.as_mut(), &data) {
            Ok(()) => " write=ok".into(),
            Err(failure) => format!(" write{}", thrown(&failure)),
        };
        if driver.open_mode() != OpenMode::None {
            let _ = driver.close();
        }
    }
    let set = data.get().expect("data");
    let file = files
        .contents(&file_name())
        .map_or_else(|| "absent".into(), |bytes| hex(&bytes));
    format!(
        "{out} st={},{}{} F:{file}",
        set.error_status().ordinal(),
        ascii_hex(&set.error_status_extension()),
        data_text(&data)
    )
}

fn data_script(input: &mut SplitWhitespace<'_>) -> String {
    let data = Handle::new(Data::default());
    let set = data.get().expect("data");
    set.clear_error_status();
    let mut out = String::new();
    for token in input.by_ref() {
        let (op, args) = match token.split_once('=') {
            Some((op, args)) => (op, split(args, ',')),
            None => (token, Vec::new()),
        };
        out += &match data_op(op, &args, &data) {
            Ok(result) => format!(" {op}={result}"),
            Err(failure) => format!(" {op}{}", thrown(&failure)),
        };
    }
    out
}

fn data_op(op: &str, args: &[String], data: &Handle<Data>) -> Result<String, Failure> {
    let set = data.get().expect("data");
    // `tn` takes a number, so the name is decoded only by the operations that use one.
    let text = || {
        args.first()
            .map_or_else(AsciiString::default, |arg| ascii(arg))
    };
    let types = set.type_data();
    let types = types.get().expect("types");
    Ok(match op {
        "ar" => {
            set.add_named_root(&text(), &Handle::default());
            String::new()
        }
        "au" => {
            set.add_root(&Handle::default());
            String::new()
        }
        "ao" => {
            set.add_named_root(&text(), &Handle::new(StandardPersistent::default()));
            String::new()
        }
        "rm" => {
            set.remove_root(&text());
            String::new()
        }
        "fd" => find_text(set, &text()),
        "ir" => u8::from(set.is_root(&text())).to_string(),
        "ur" => {
            let roots = set.root_data();
            let object = Handle::new(StandardPersistent::default());
            roots.get().expect("roots").update_root(&text(), &object)?;
            String::new()
        }
        "at" => {
            types.add_type(&text(), args[1].parse().expect("number"));
            String::new()
        }
        "tn" => ascii_hex(&types.type_name(args[0].parse().expect("number"))?),
        "tt" => types.type_number(&text())?.to_string(),
        "it" => u8::from(set.is_type(&text())).to_string(),
        "ai" => {
            set.add_to_user_info(&text());
            String::new()
        }
        "ac" => {
            set.add_to_comments(&ext(&args[0]));
            String::new()
        }
        "cl" => {
            set.clear();
            String::new()
        }
        "ce" => {
            set.clear_error_status();
            String::new()
        }
        "st" => format!(
            "{},{}",
            set.error_status().ordinal(),
            ascii_hex(&set.error_status_extension())
        ),
        "dump" => data_text(data),
        _ => String::new(),
    })
}

fn find_text(set: &Data, name: &AsciiString) -> String {
    let root = set.find(name);
    match root.get() {
        None => "null".into(),
        Some(root) => format!(
            "{}:{}:{}:{}",
            ascii_hex(&root.name()),
            root.reference(),
            ascii_hex(&root.type_name()),
            u8::from(root.object().is_some())
        ),
    }
}

fn bucket_script(input: &mut SplitWhitespace<'_>) -> String {
    let size: i32 = input.next().expect("size").parse().expect("integer");
    let mut bucket = BucketOfPersistent::new(size, 2);
    let mut objects: Vec<Handle<StandardPersistent>> = Vec::new();
    let id = |objects: &[Handle<StandardPersistent>], object: &Handle<StandardPersistent>| {
        objects
            .iter()
            .position(|known| std::ptr::eq(known.as_ptr(), object.as_ptr()) && object.is_some())
            .map_or(0, |index| index + 1)
    };
    let mut out = String::new();
    for op in input.by_ref() {
        out += &format!(" {op}");
        match op {
            "a" => {
                objects.push(Handle::new(StandardPersistent::default()));
                bucket.append(objects.last().expect("object"));
            }
            "n" => bucket.append(&Handle::default()),
            "len" => out += &format!("={}", bucket.length()),
            "clr" => bucket.clear(),
            "it" => {
                out += "=";
                let mut walk = BucketIterator::new(Some(&bucket));
                while walk.more() {
                    out += &format!("{};", id(&objects, &walk.value()));
                    walk.next();
                }
            }
            value => {
                let index: i32 = value[1..].parse().expect("index");
                out += &format!("={}", id(&objects, &bucket.value(index)));
            }
        }
    }
    out
}

fn callback_script(input: &mut SplitWhitespace<'_>) -> String {
    let mut schema = Schema::default();
    let mut out = String::new();
    for token in input.by_ref() {
        let (op, value) = token.split_once('=').unwrap_or((token, "-"));
        let name = ascii(value);
        out += &format!(" {op}");
        match op {
            "add" => schema
                .add_read_unknown_type_callback(&name, Some(Arc::new(DefaultCallBack::default()))),
            "addn" => schema.add_read_unknown_type_callback(&name, None),
            "rm" => schema.remove_read_unknown_type_callback(&name),
            "ls" => {
                let list = schema.installed_callback_list();
                let names: String = list.get().expect("list").sequence(|items| {
                    items
                        .iter()
                        .map(|name| format!("{};", ascii_hex(name)))
                        .collect()
                });
                out += &format!("={names}");
            }
            "clr" => schema.clear_callback_list(),
            "use" => schema.use_default_callback(),
            "dont" => schema.dont_use_default_callback(),
            "is" => out += &format!("={}", u8::from(schema.is_using_default_callback())),
            "rst" => schema.reset_default_callback(),
            "setn" => schema.set_default_callback(None),
            _ => out += &format!("={}", u8::from(schema.default_callback().is_some())),
        }
    }
    out
}

fn complete_script(input: &mut SplitWhitespace<'_>) -> String {
    let mut stream = IoStream::from_bytes(unhex(input.next().expect("bytes")));
    let mut data: Handle<Data> = Handle::default();
    let mut driver = BinaryFile::new(Rc::new(MemoryFileSystem::default()));
    let magic = ascii_hex(&DriverBase::read_magic_number(&mut stream));
    let mut out = match driver.read_complete_info(&mut stream, &mut data) {
        Ok(()) => format!("{magic} ok"),
        Err(failure) => format!("{magic} {}", thrown(&failure)),
    };
    if data.is_some() {
        out += &data_text(&data);
    }
    out
}
