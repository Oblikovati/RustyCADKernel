//! `FSD_BinaryFile` stream overloads (the `stor_bstream` part of
//! `tkernel_storage_schema_probe.cxx`).
use super::common::{
    ascii, ascii_hex, ext, ext_hex, ext_list, header_text, hex, split, thrown, unhex,
};
use super::drivers::DriverValues;
use rustycad_tkernel::fsd::{BinaryFile, FileHeader, MemoryFileSystem};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkernel::tcollection::AsciiString;
use std::str::SplitWhitespace;

/// Runs one script on a read-write stream over the given bytes, then prints its bytes.
pub fn script(input: &mut SplitWhitespace<'_>) -> String {
    let files = MemoryFileSystem::default();
    let name = AsciiString::new("stream");
    files.insert(&name, unhex(input.next().expect("bytes")));
    let device = rustycad_tkernel::fsd::DriverFileSystem::open(
        &files,
        &name,
        rustycad_tkernel::fsd::FileOpenMode::UPDATE,
    )
    .expect("memory file");
    let mut stream = IoStream::with_device(device, true, true);
    let mut values = DriverValues::new();
    let mut out = String::new();
    for token in input.by_ref() {
        out += &stream_op(token, &mut stream, &mut values);
    }
    drop(stream);
    format!(
        "{out} F:{}",
        hex(&files.contents(&name).unwrap_or_default())
    )
}

fn stream_op(token: &str, stream: &mut IoStream, v: &mut DriverValues) -> String {
    let (op, args) = match token.split_once('=') {
        Some((op, args)) => (op, split(args, ',')),
        None => (token, Vec::new()),
    };
    let only_count = args.last().is_some_and(|last| last == "1");
    if op == "gi" {
        let outcome = BinaryFile::get_integer_stream(stream, &mut v.int1);
        let prefix = outcome
            .err()
            .map_or_else(|| "=".to_string(), |f| format!("{}=", thrown(&f)));
        return format!(" gi{prefix}{}", v.int1);
    }
    match run(op, &args, only_count, stream, v) {
        Ok(result) => format!(" {op}={result}"),
        Err(failure) => format!(" {op}{}", thrown(&failure)),
    }
}

fn number(text: &str) -> i32 {
    text.parse().expect("integer argument")
}

fn write_op(
    op: &str,
    args: &[String],
    only_count: bool,
    stream: &mut IoStream,
) -> Result<String, Failure> {
    let size = match op {
        "pi" => BinaryFile::put_integer_stream(stream, number(&args[0]), only_count)?,
        "wh" => {
            let fields: [i32; 13] = std::array::from_fn(|index| number(&args[index]));
            BinaryFile::write_header_stream(stream, &FileHeader::from_fields(fields), only_count)?
        }
        "wi" => {
            let mut info = Sequence::default();
            if args[8] != "-" {
                for line in split(&args[8], ';') {
                    info.append(ascii(&line));
                }
            }
            BinaryFile::write_info_stream(
                stream,
                number(&args[0]),
                &ascii(&args[1]),
                &ascii(&args[2]),
                &ascii(&args[3]),
                &ascii(&args[4]),
                &ext(&args[5]),
                &ascii(&args[6]),
                &ext(&args[7]),
                &info,
                only_count,
            )?
        }
        _ => {
            let mut comments = Sequence::default();
            if args[0] != "-" {
                for line in split(&args[0], ';') {
                    comments.append(ext(&line));
                }
            }
            BinaryFile::write_comment_stream(stream, &comments, only_count)?
        }
    };
    Ok(size.to_string())
}

fn run(
    op: &str,
    args: &[String],
    only_count: bool,
    stream: &mut IoStream,
    v: &mut DriverValues,
) -> Result<String, Failure> {
    Ok(match op {
        "pi" | "wh" | "wi" | "wc" => write_op(op, args, only_count, stream)?,
        "rs" => {
            BinaryFile::read_string_stream(stream, &mut v.text1)?;
            ascii_hex(&v.text1)
        }
        "re" => {
            BinaryFile::read_extended_string_stream(stream, &mut v.app_name)?;
            ext_hex(&v.app_name)
        }
        "rh" => {
            let mut header = FileHeader::default();
            BinaryFile::read_header_stream(stream, &mut header)?;
            format!(
                "{};{};{}",
                header.test_indian, header.begin_info, header.end_data
            )
        }
        "rhd" => {
            BinaryFile::read_header_data(stream, &v.header)?;
            header_text(&v.header)
        }
        "rc" => {
            BinaryFile::read_comment_stream(stream, &mut v.comments)?;
            ext_list(&v.comments)
        }
        "ts" => BinaryFile::type_section_size_stream(stream)?.to_string(),
        "rt" => {
            BinaryFile::read_type_informations_stream(stream, &mut v.int1, &mut v.text1)?;
            format!("{},{}", v.int1, ascii_hex(&v.text1))
        }
        "rr" => {
            BinaryFile::read_root_stream(stream, &mut v.text1, &mut v.int1, &mut v.text2)?;
            format!("{},{},{}", ascii_hex(&v.text1), v.int1, ascii_hex(&v.text2))
        }
        "rf" => {
            BinaryFile::read_reference_type_stream(stream, &mut v.int1, &mut v.int2)?;
            format!("{},{}", v.int1, v.int2)
        }
        "tg" => stream.tellg().to_string(),
        "tp" => stream.tellp().to_string(),
        _ => String::new(),
    })
}
