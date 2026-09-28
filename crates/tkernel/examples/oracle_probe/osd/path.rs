//! OSD path probe operations (OSD_Path); twin of `tkernel_osd_path_probe.cxx`.
use super::{bit, integer};
use rustycad_tkernel::{
    osd::{CadPath, PathComponents, SysType},
    standard::Failure,
    tcollection::AsciiString,
};
use std::str::SplitWhitespace;

/// The output of a path operation, or `None` when `op` belongs to another family.
pub fn evaluate(op: &str, input: &mut SplitWhitespace<'_>) -> Option<String> {
    Some(match op {
        "osd_path_parse" => parse(input),
        "osd_path_values" => {
            let [node, user, password, disk, trek, name, extension] = components(input);
            report(&CadPath::from_values(
                node, user, password, disk, trek, name, extension,
            ))
        }
        "osd_path_setters" => setters(input),
        "osd_path_trek" => trek(input),
        "osd_path_rel" | "osd_path_abs" => relative_or_absolute(op, input),
        "osd_path_split" => split(input),
        "osd_path_kind" => kind(&hex(input)),
        _ => return None,
    })
}

fn hex(input: &mut SplitWhitespace<'_>) -> Vec<u8> {
    let token: &str = input.next().expect("hex token");
    if token == "-" {
        return Vec::new();
    }
    (0..token.len() / 2)
        .map(|index| u8::from_str_radix(&token[2 * index..2 * index + 2], 16).expect("hex"))
        .collect()
}

fn components(input: &mut SplitWhitespace<'_>) -> [Vec<u8>; 7] {
    std::array::from_fn(|_| hex(input))
}

fn text(bytes: &[u8]) -> String {
    let mut out: String = String::from("[");
    for &byte in bytes {
        if !(0x20..=0x7e).contains(&byte) || matches!(byte, b'\\' | b'[' | b']') {
            out.push_str(&format!("\\x{byte:02x}"));
        } else {
            out.push(char::from(byte));
        }
    }
    out + "]"
}

fn ascii(value: &AsciiString) -> String {
    text(value.as_bytes())
}

fn failure(error: &Failure) -> String {
    format!(
        "EXC {}{}",
        error.exception_type(),
        text(error.message().as_bytes())
    )
}

fn report(path: &CadPath) -> String {
    let PathComponents {
        node,
        user_name,
        password,
        disk,
        trek,
        name,
        extension,
    } = path.values();
    let mut out: String = [node, user_name, password, disk, trek, name, extension]
        .iter()
        .map(ascii)
        .collect();
    out += &format!(" {}", path.trek_length());
    let types: [SysType; 7] = [
        SysType::Default,
        SysType::UnixBsd,
        SysType::Vms,
        SysType::Os2,
        SysType::MacOs,
        SysType::WindowsNt,
        SysType::Unknown,
    ];
    for sys_type in types {
        out.push(' ');
        out += &path
            .system_name_as(sys_type)
            .map_or_else(|error| failure(&error), |full| ascii(&full));
    }
    out
}

fn parse(input: &mut SplitWhitespace<'_>) -> String {
    let sys_type: SysType = SysType::from_raw(integer(input) as i32).expect("OSD_SysType");
    let path_text: Vec<u8> = hex(input);
    let valid: String = format!(
        "{}{} ",
        bit(CadPath::is_valid(&path_text, sys_type)),
        bit(CadPath::is_valid(&path_text, SysType::Default))
    );
    valid
        + &CadPath::with_sys_type(&path_text, sys_type)
            .map_or_else(|error| failure(&error), |path| report(&path))
}

fn setters(input: &mut SplitWhitespace<'_>) -> String {
    let [node, user, password, disk, trek, name, extension] = components(input);
    let mut path: CadPath = CadPath::default();
    path.set_node(node);
    path.set_user_name(user);
    path.set_password(password);
    path.set_disk(disk);
    path.set_trek(trek);
    path.set_name(name);
    path.set_extension(extension);
    let mut expanded: AsciiString = AsciiString::new("x");
    path.expanded_name(&mut expanded);
    format!(
        "{}{}{}{}{}{}{} {} {}",
        ascii(&path.node()),
        ascii(&path.user_name()),
        ascii(&path.password()),
        ascii(&path.disk()),
        ascii(&path.trek()),
        ascii(&path.name()),
        ascii(&path.extension()),
        ascii(&expanded),
        report(&path)
    )
}

fn trek_operation(
    path: &mut CadPath,
    op: &str,
    input: &mut SplitWhitespace<'_>,
) -> Result<String, Failure> {
    match op {
        "up" => path.up_trek(),
        "down" => path.down_trek(hex(input)),
        "rm" => path.remove_a_trek(integer(input) as i32)?,
        "rmname" => path.remove_a_trek_named(hex(input))?,
        "value" => return Ok(ascii(&path.trek_value(integer(input) as i32)?)),
        _ => {
            let name: Vec<u8> = hex(input);
            path.insert_a_trek(name, integer(input) as i32)?;
        }
    }
    Ok(String::new())
}

fn trek(input: &mut SplitWhitespace<'_>) -> String {
    let mut path: CadPath = CadPath::default();
    path.set_values("", "", "", "", hex(input), "n", ".e");
    let op: &str = input.next().expect("trek operation");
    let result: String = match trek_operation(&mut path, op, input) {
        Ok(value) => value + "ok",
        Err(error) => failure(&error),
    };
    format!("{result} {} {}", ascii(&path.trek()), path.trek_length())
}

fn relative_or_absolute(op: &str, input: &mut SplitWhitespace<'_>) -> String {
    let (dir, file) = (hex(input), hex(input));
    let result: Result<AsciiString, Failure> = if op == "osd_path_rel" {
        CadPath::relative_path(dir, file)
    } else {
        CadPath::absolute_path(dir, file)
    };
    result.map_or_else(|error| failure(&error), |path| ascii(&path))
}

fn split(input: &mut SplitWhitespace<'_>) -> String {
    let path_text: Vec<u8> = hex(input);
    let (folder, file) = CadPath::folder_and_file_from_path(&path_text);
    let (name, extension) = CadPath::file_name_and_extension(&path_text);
    [folder, file, name, extension].iter().map(ascii).collect()
}

fn kind(path: &[u8]) -> String {
    [
        CadPath::is_dos_path(path),
        CadPath::is_nt_extended_path(path),
        CadPath::is_unc_path(path),
        CadPath::is_unc_extended_path(path),
        CadPath::is_unix_path(path),
        CadPath::is_content_protocol_path(path),
        CadPath::is_remote_protocol_path(path),
        CadPath::is_relative_path(path),
        CadPath::is_absolute_path(path),
    ]
    .map(|value| bit(value).to_string())
    .concat()
}
