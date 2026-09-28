//! Driver scripts (twin of `tkernel_storage_drivers_probe.cxx`).
use super::common::{
    ascii, ascii_hex, ascii_list, double, driver, ext, ext_hex, ext_list, file_name, float,
    header_text, hex, mode, roots_text, split, thrown, types_text, unhex,
};
use rustycad_tkernel::fsd::{BinaryFile, CmpFile, DriverFileSystem, File, MemoryFileSystem};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::{BaseDriver, HeaderData, OpenMode, RootData, StorageError, TypeData};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
use std::rc::Rc;
use std::str::SplitWhitespace;

/// The out-parameters a script reads into, persistent across its operations.
#[derive(Default)]
pub struct DriverValues {
    pub count: i32,
    pub int1: i32,
    pub int2: i32,
    pub version: AsciiString,
    pub date: AsciiString,
    pub schema: AsciiString,
    pub schema_version: AsciiString,
    pub app_version: AsciiString,
    pub text1: AsciiString,
    pub text2: AsciiString,
    pub app_name: ExtendedString,
    pub data_type: ExtendedString,
    pub info: Sequence<AsciiString>,
    pub comments: Sequence<ExtendedString>,
    pub byte: u8,
    pub unit: u16,
    pub flag: bool,
    pub real: f64,
    pub short_real: f32,
    pub header: Handle<HeaderData>,
    pub roots: Handle<RootData>,
    pub types: Handle<TypeData>,
}

impl DriverValues {
    pub fn new() -> Self {
        Self {
            header: Handle::new(HeaderData::default()),
            roots: Handle::new(RootData::default()),
            types: Handle::new(TypeData::default()),
            ..Self::default()
        }
    }
}

/// The output of a driver operation, or `None` when `op` belongs elsewhere.
pub fn evaluate(op: &str, input: &mut SplitWhitespace<'_>) -> Option<String> {
    match op {
        "stor_drv" => Some(driver_script(input)),
        "stor_good" => Some(good_file_type(input)),
        _ => None,
    }
}

fn good_file_type(input: &mut SplitWhitespace<'_>) -> String {
    let kind = input.next().expect("kind");
    let bytes = input.next().expect("file");
    let files = MemoryFileSystem::default();
    if bytes != "-" {
        files.insert(&file_name(), unhex(bytes));
    }
    let system: Rc<dyn DriverFileSystem> = Rc::new(files);
    let status = match kind {
        "b" => BinaryFile::is_good_file_type(system, &file_name()),
        "c" => CmpFile::is_good_file_type(system, &file_name()),
        _ => File::is_good_file_type(system, &file_name()),
    };
    status.expect("IsGoodFileType").ordinal().to_string()
}

fn driver_script(input: &mut SplitWhitespace<'_>) -> String {
    let kind = input.next().expect("kind").chars().next().expect("kind letter");
    let bytes = input.next().expect("file");
    let files = MemoryFileSystem::default();
    if bytes != "-" {
        files.insert(&file_name(), unhex(bytes));
    }
    let mut values = DriverValues::new();
    let mut out = String::new();
    {
        let mut driver = driver(kind, &files);
        for token in input.by_ref() {
            out += &driver_op(token, driver.as_mut(), &mut values);
        }
        if driver.open_mode() != OpenMode::None {
            let _ = driver.close();
        }
    }
    let file = files
        .contents(&file_name())
        .map_or_else(|| "absent".into(), |bytes| hex(&bytes));
    format!(
        "{out} hdr={} rdt={} tdt={} F:{file}",
        header_text(&values.header),
        roots_text(&values.roots),
        types_text(&values.types)
    )
}

/// ` op=result`, ` op!Class`, or for reads ` op!Class=values`.
pub fn driver_op(token: &str, driver: &mut dyn BaseDriver, values: &mut DriverValues) -> String {
    let (op, arguments) = match token.split_once('=') {
        Some((op, arguments)) => (op, split(arguments, ',')),
        None => (token, Vec::new()),
    };
    let reported = get_op(op, driver, values).or_else(|| read_op(op, driver, values));
    if let Some((outcome, shown)) = reported {
        let prefix = outcome.err().map_or_else(|| "=".to_string(), |f| format!("{}=", thrown(&f)));
        return format!(" {op}{prefix}{shown}");
    }
    match other_op(op, &arguments, driver, values) {
        Ok(result) => format!(" {op}={result}"),
        Err(failure) => format!(" {op}{}", thrown(&failure)),
    }
}

type Reported = Option<(Result<(), Failure>, String)>;

fn get_op(op: &str, driver: &mut dyn BaseDriver, v: &mut DriverValues) -> Reported {
    let outcome = match op {
        "gr" => driver.get_reference(&mut v.int1),
        "gc" => driver.get_character(&mut v.byte),
        "ge" => driver.get_ext_character(&mut v.unit),
        "gi" => driver.get_integer(&mut v.int1),
        "gb" => driver.get_boolean(&mut v.flag),
        "gd" => driver.get_real(&mut v.real),
        "gf" => driver.get_short_real(&mut v.short_real),
        _ => return None,
    };
    let shown = match op {
        "gr" | "gi" => v.int1.to_string(),
        "gc" => v.byte.to_string(),
        "ge" => v.unit.to_string(),
        "gb" => u8::from(v.flag).to_string(),
        "gd" => format!("{:016x}", v.real.to_bits()),
        _ => format!("{:08x}", v.short_real.to_bits()),
    };
    Some((outcome, shown))
}

fn read_op(op: &str, driver: &mut dyn BaseDriver, v: &mut DriverValues) -> Reported {
    let outcome = match op {
        "ri" => driver.read_info(
            &mut v.count,
            &mut v.version,
            &mut v.date,
            &mut v.schema,
            &mut v.schema_version,
            &mut v.app_name,
            &mut v.app_version,
            &mut v.data_type,
            &mut v.info,
        ),
        "rc" => driver.read_comment(&mut v.comments),
        "rt" => driver.read_type_informations(&mut v.int1, &mut v.text1),
        "rr" => driver.read_root(&mut v.text1, &mut v.int1, &mut v.text2),
        "rf" => driver.read_reference_type(&mut v.int1, &mut v.int2),
        "roh" => driver.read_persistent_object_header(&mut v.int1, &mut v.int2),
        _ => return None,
    };
    let shown = match op {
        "ri" => info_text(v),
        "rc" => ext_list(&v.comments),
        "rt" => format!("{},{}", v.int1, ascii_hex(&v.text1)),
        "rr" => format!("{},{},{}", ascii_hex(&v.text1), v.int1, ascii_hex(&v.text2)),
        _ => format!("{},{}", v.int1, v.int2),
    };
    Some((outcome, shown))
}

fn info_text(v: &DriverValues) -> String {
    format!(
        "{},{},{},{},{},{},{},{},{}",
        v.count,
        ascii_hex(&v.version),
        ascii_hex(&v.date),
        ascii_hex(&v.schema),
        ascii_hex(&v.schema_version),
        ext_hex(&v.app_name),
        ascii_hex(&v.app_version),
        ext_hex(&v.data_type),
        ascii_list(&v.info)
    )
}

fn status(result: Result<StorageError, Failure>) -> Result<String, Failure> {
    result.map(|status| status.ordinal().to_string())
}

fn other_op(
    op: &str,
    args: &[String],
    driver: &mut dyn BaseDriver,
    v: &mut DriverValues,
) -> Result<String, Failure> {
    if let Some(letter) = op.strip_prefix('o').filter(|rest| rest.len() == 1) {
        let letter = letter.chars().next().expect("mode");
        return status(driver.open(&file_name(), mode(letter)));
    }
    match op {
        "ie" => driver.is_end().map(|end| u8::from(end).to_string()),
        "tl" => driver.tell().map(|position| position.to_string()),
        "hr" => read_bool(v.header.get().expect("header").read(driver)),
        "rdr" => read_bool(v.roots.get().expect("roots").read(driver)),
        "tdr" => read_bool(v.types.get().expect("types").read(driver)),
        _ => table_op(op, args, driver),
    }
}

fn read_bool(result: Result<bool, Failure>) -> Result<String, Failure> {
    result.map(|done| u8::from(done).to_string())
}

fn number(text: &str) -> i32 {
    text.parse().expect("integer argument")
}

fn ascii_items(text: &str) -> Sequence<AsciiString> {
    let mut items = Sequence::default();
    if text != "-" {
        for line in split(text, ';') {
            items.append(ascii(&line));
        }
    }
    items
}

fn ext_items(text: &str) -> Sequence<ExtendedString> {
    let mut items = Sequence::default();
    if text != "-" {
        for line in split(text, ';') {
            items.append(ext(&line));
        }
    }
    items
}

fn table_op(op: &str, args: &[String], driver: &mut dyn BaseDriver) -> Result<String, Failure> {
    let done = |result: Result<(), Failure>| result.map(|()| String::new());
    match op {
        "wi" => done(driver.write_info(
            number(&args[0]),
            &ascii(&args[1]),
            &ascii(&args[2]),
            &ascii(&args[3]),
            &ascii(&args[4]),
            &ext(&args[5]),
            &ascii(&args[6]),
            &ext(&args[7]),
            &ascii_items(&args[8]),
        )),
        "wc" => done(driver.write_comment(&ext_items(&args[0]))),
        "sts" => done(driver.set_type_section_size(number(&args[0]))),
        "ssr" => done(driver.set_root_section_size(number(&args[0]))),
        "ssf" => done(driver.set_ref_section_size(number(&args[0]))),
        "ts" => driver.type_section_size().map(|size| size.to_string()),
        "rs" => driver.root_section_size().map(|size| size.to_string()),
        "fs" => driver.ref_section_size().map(|size| size.to_string()),
        "wt" => done(driver.write_type_informations(number(&args[0]), &ascii(&args[1]))),
        "wr" => done(driver.write_root(&ascii(&args[0]), number(&args[1]), &ascii(&args[2]))),
        "wf" => done(driver.write_reference_type(number(&args[0]), number(&args[1]))),
        "woh" => done(driver.write_persistent_object_header(number(&args[0]), number(&args[1]))),
        _ => put_or_frame_op(op, args, driver),
    }
}

fn put_or_frame_op(op: &str, args: &[String], driver: &mut dyn BaseDriver) -> Result<String, Failure> {
    let outcome = match op {
        "pr" => driver.put_reference(number(&args[0])),
        "pc" => driver.put_character(number(&args[0]) as u8),
        "pe" => driver.put_ext_character(number(&args[0]) as u16),
        "pi" => driver.put_integer(number(&args[0])),
        "pb" => driver.put_boolean(args[0] == "1"),
        "pd" => driver.put_real(double(&args[0])),
        "pf" => driver.put_short_real(float(&args[0])),
        "bwp" => driver.begin_write_persistent_object_data(),
        "bwo" => driver.begin_write_object_data(),
        "ewo" => driver.end_write_object_data(),
        "ewp" => driver.end_write_persistent_object_data(),
        "brp" => driver.begin_read_persistent_object_data(),
        "bro" => driver.begin_read_object_data(),
        "ero" => driver.end_read_object_data(),
        "erp" => driver.end_read_persistent_object_data(),
        "sk" => driver.skip_object(),
        _ => return status(section_op(op, driver)),
    };
    outcome.map(|()| String::new())
}

fn section_op(op: &str, driver: &mut dyn BaseDriver) -> Result<StorageError, Failure> {
    match op {
        "bwi" => driver.begin_write_info_section(),
        "ewi" => driver.end_write_info_section(),
        "bri" => driver.begin_read_info_section(),
        "eri" => driver.end_read_info_section(),
        "bwc" => driver.begin_write_comment_section(),
        "ewc" => driver.end_write_comment_section(),
        "brc" => driver.begin_read_comment_section(),
        "erc" => driver.end_read_comment_section(),
        "bwt" => driver.begin_write_type_section(),
        "ewt" => driver.end_write_type_section(),
        "brt" => driver.begin_read_type_section(),
        "ert" => driver.end_read_type_section(),
        "bwr" => driver.begin_write_root_section(),
        "ewr" => driver.end_write_root_section(),
        "brr" => driver.begin_read_root_section(),
        "err" => driver.end_read_root_section(),
        "bwf" => driver.begin_write_ref_section(),
        "ewf" => driver.end_write_ref_section(),
        "brf" => driver.begin_read_ref_section(),
        "erf" => driver.end_read_ref_section(),
        "bwd" => driver.begin_write_data_section(),
        "ewd" => driver.end_write_data_section(),
        "brd" => driver.begin_read_data_section(),
        "erd" => driver.end_read_data_section(),
        "cl" => driver.close(),
        other => panic!("unknown storage driver operation {other}"),
    }
}
