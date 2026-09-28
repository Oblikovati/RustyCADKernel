//! OSD files probe operations (OSD_FileNode, OSD_File, OSD_Directory,
//! OSD_DirectoryIterator, OSD_FileIterator); twin of `tkernel_osd_files_probe.cxx`.
use super::{bit, escape, integer};
use rustycad_tkernel::{
    Error,
    osd::{
        CadPath, Directory, DirectoryIterator, File, FileIterator, FileNode, FromWhere, LockType,
        OpenMode, OsdErrorAccess, Protection, SingleProtection,
    },
    quantity::Date,
    standard::Failure,
    tcollection::AsciiString,
};
use std::str::SplitWhitespace;

type Input<'a, 'b> = &'a mut SplitWhitespace<'b>;

/// The output of a files operation, or `None` when `op` belongs to another family.
pub fn evaluate(op: &str, input: Input<'_, '_>) -> Option<String> {
    if op != "osd_files" {
        return None;
    }
    let _ = std::fs::remove_dir_all("osdf");
    let _ = std::fs::create_dir("osdf");
    let out: String = run(input);
    let _ = std::fs::remove_dir_all("osdf");
    Some(out)
}

fn hex_bytes(input: Input<'_, '_>) -> Vec<u8> {
    let token: &str = input.next().expect("hex token");
    if token == "-" {
        return Vec::new();
    }
    (0..token.len() / 2)
        .map(|index| u8::from_str_radix(&token[2 * index..2 * index + 2], 16).expect("hex"))
        .collect()
}

fn name(input: Input<'_, '_>) -> String {
    let name: &str = input.next().expect("name");
    match name {
        "-" => String::new(),
        absolute if absolute.starts_with('/') => absolute.to_string(),
        relative => format!("osdf/{relative}"),
    }
}

fn int(input: Input<'_, '_>) -> i32 {
    integer(input) as i32
}

fn single(value: i32) -> SingleProtection {
    SingleProtection::from_raw(value).expect("protection")
}

fn protection(input: Input<'_, '_>) -> Protection {
    let (s, u, g, w) = (int(input), int(input), int(input), int(input));
    Protection::with_values(single(s), single(u), single(g), single(w))
}

fn protection_text(protection: &Protection) -> String {
    let (s, u, g, w) = protection.values();
    format!("{},{},{},{}", s as i32, u as i32, g as i32, w as i32)
}

fn date_text(date: Result<Date, Error>) -> Result<String, Failure> {
    let date: Date =
        date.map_err(|error| Failure::with_type(error.exception_type(), error.message(), ""))?;
    let (mm, dd, yy, hh, mn, ss, _, _) = date.values();
    Ok(format!("{yy}-{mm}-{dd}T{hh}:{mn}:{ss}"))
}

fn lossy(bytes: &[u8]) -> String {
    escape(&String::from_utf8_lossy(bytes))
}

fn failure_text(failure: &Failure) -> String {
    format!(
        "EXC {}{}",
        failure.exception_type(),
        escape(failure.message())
    )
}

/// The node commands shared by files and directories; `None` when not a node command.
fn node_command(
    node: &mut FileNode,
    command: &str,
    input: Input<'_, '_>,
) -> Option<Result<String, Failure>> {
    Some(Ok(match command {
        "failed" => bit(node.failed()).to_string(),
        "error" => node.error().to_string(),
        "perror" => return Some(node.perror().map(|()| "ok".to_string())),
        "reset" => {
            node.reset();
            "ok".to_string()
        }
        "exists" => bit(node.exists()).to_string(),
        "remove" => {
            node.remove();
            bit(node.failed()).to_string()
        }
        "move" | "copy" => {
            let target: CadPath = CadPath::new(name(input));
            if command == "move" {
                node.move_to(&target)
            } else {
                node.copy_to(&target)
            }
            bit(node.failed()).to_string()
        }
        "prot" => protection_text(&node.protection()),
        "setprot" => {
            node.set_protection(&protection(input));
            bit(node.failed()).to_string()
        }
        "moment" => {
            let created: Result<String, Failure> = date_text(node.creation_moment());
            let accessed: Result<String, Failure> = date_text(node.access_moment());
            return Some(created.and_then(|created| Ok(format!("{created}/{}", accessed?))));
        }
        "path" => lossy(node.path().system_name().as_bytes()),
        _ => return None,
    }))
}

fn file_io(file: &mut File, command: &str, input: Input<'_, '_>) -> Result<String, Failure> {
    Ok(match command {
        "write" => {
            let bytes: Vec<u8> = hex_bytes(input);
            file.write(&AsciiString::from_raw(&bytes), int(input))?;
            bit(file.failed()).to_string()
        }
        "read" => {
            let mut text: AsciiString = AsciiString::new("old");
            file.read(&mut text, int(input))?;
            lossy(text.as_bytes())
        }
        "readb" => {
            let count: i32 = int(input);
            let mut buffer: Vec<u8> = vec![0; count.max(1) as usize];
            let read: i32 = file.read_bytes(&mut buffer, count)?;
            format!("{read}{}", lossy(&buffer[..read.max(0) as usize]))
        }
        "readline" => {
            let mut text: AsciiString = AsciiString::new("old");
            let read: i32 = file.read_line(&mut text, int(input))?;
            format!("{read}{}", lossy(text.as_bytes()))
        }
        "lastline" => {
            let mut text: AsciiString = AsciiString::new("old");
            let delay: i32 = int(input);
            let found: bool = file.read_last_line(&mut text, delay, int(input))?;
            format!("{}{}", bit(found), lossy(text.as_bytes()))
        }
        "seek" => {
            let offset: i32 = int(input);
            file.seek(offset, FromWhere::from_raw(int(input)).expect("whence"))?;
            bit(file.failed()).to_string()
        }
        other => panic!("unknown files command {other}"),
    })
}

fn open_command(file: &mut File, command: &str, input: Input<'_, '_>) -> Result<String, Failure> {
    let mode: OpenMode = OpenMode::from_raw(int(input)).expect("open mode");
    let rights: Protection = if command == "buildp" {
        protection(input)
    } else {
        Protection::new()
    };
    match command {
        "open" => file.open(mode, &rights)?,
        "append" => file.append(mode, &rights)?,
        _ => file.build(mode, &rights)?,
    }
    Ok(format!("{}{}", bit(file.is_open()), bit(file.failed())))
}

fn lock_state(file: &File) -> String {
    format!(
        "{}{}{}",
        file.lock() as i32,
        bit(file.is_locked()),
        bit(file.failed())
    )
}

fn file_command(file: &mut File, command: &str, input: Input<'_, '_>) -> Result<String, Failure> {
    Ok(match command {
        "build" | "open" | "append" | "buildp" => return open_command(file, command, input),
        "rewind" => {
            file.rewind();
            "ok".to_string()
        }
        "close" => {
            file.close()?;
            bit(file.failed()).to_string()
        }
        "atend" => bit(file.is_at_end()?).to_string(),
        "size" => {
            let size: usize = file.size()?;
            format!("{size}{}", bit(file.failed()))
        }
        "kind" => (file.kind_of_file() as i32).to_string(),
        "isopen" => bit(file.is_open()).to_string(),
        "readable" => format!(
            "{}{}{}",
            bit(file.is_readable()),
            bit(file.is_writeable()),
            bit(file.is_executable())
        ),
        "lock" => {
            file.set_lock(LockType::from_raw(int(input)).expect("lock"))?;
            lock_state(file)
        }
        "unlock" => {
            file.unlock()?;
            lock_state(file)
        }
        "tempfile" => {
            file.build_temporary()?;
            let full: AsciiString = file.path().system_name();
            let temporary: bool = full.as_bytes().starts_with(b"/tmp/CSF");
            format!("{}{}{}", bit(file.is_open()), bit(temporary), full.len())
        }
        _ => return file_io(file, command, input),
    })
}

fn iterate(input: Input<'_, '_>) -> String {
    let kind: &str = input.next().expect("iterator kind");
    let mask: Vec<u8> = hex_bytes(input);
    let place: CadPath = CadPath::new(name(input));
    let mut names: Vec<String> = Vec::new();
    let mut push = |path: CadPath| {
        let text: Vec<u8> = [path.name().as_bytes(), b"|", path.extension().as_bytes()].concat();
        names.push(String::from_utf8_lossy(&text).into_owned());
    };
    if kind == "d" {
        let mut iterator: DirectoryIterator = DirectoryIterator::new(&place, &mask);
        while iterator.more() {
            push(iterator.values().path());
            iterator.next();
        }
    } else {
        let mut iterator: FileIterator = FileIterator::new(&place, &mask);
        while iterator.more() {
            push(iterator.values().path());
            iterator.next();
        }
    }
    names.sort();
    let listed: String = names.iter().map(|entry| escape(entry)).collect();
    format!("{}{listed}", names.len())
}

fn fixture(command: &str, input: Input<'_, '_>) -> String {
    let succeeded: bool = match command {
        "fixture" => {
            let path: String = name(input);
            std::fs::write(path, hex_bytes(input)).is_ok()
        }
        other => special_fixture(other, input),
    };
    bit(succeeded).to_string()
}

#[cfg(unix)]
fn special_fixture(command: &str, input: Input<'_, '_>) -> bool {
    use std::os::unix::fs::PermissionsExt;
    match command {
        "mkfifo" => std::process::Command::new("mkfifo")
            .args(["-m", "644", &name(input)])
            .status()
            .is_ok_and(|status| status.success()),
        "chmod" => {
            let path: String = name(input);
            let mode: u32 = u32::from_str_radix(input.next().expect("octal"), 8).expect("octal");
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).is_ok()
        }
        _ => {
            let target: String = input.next().expect("target").to_string();
            std::os::unix::fs::symlink(target, name(input)).is_ok()
        }
    }
}

#[cfg(not(unix))]
fn special_fixture(_command: &str, _input: Input<'_, '_>) -> bool {
    // FIFOs, modes and symlinks are Unix fixtures; the probe runs on Linux only.
    false
}

fn directory_command(
    directory: &mut Directory,
    command: &str,
    input: Input<'_, '_>,
) -> Result<String, Failure> {
    match command {
        "dbuild" | "dbuildp" => {
            let rights: Protection = if command == "dbuildp" {
                protection(input)
            } else {
                Protection::new()
            };
            directory.build(&rights)?;
            Ok(bit(directory.failed()).to_string())
        }
        "tempdir" => {
            *directory = Directory::build_temporary();
            let full: AsciiString = directory.path().system_name();
            let temporary: bool = full.as_bytes().starts_with(b"/tmp/CSF");
            Ok(format!(
                "{}{}{}",
                bit(directory.exists()),
                bit(temporary),
                full.len()
            ))
        }
        _ => node_command(directory, &command[1..], input).expect("directory command"),
    }
}

fn step(
    file: &mut File,
    directory: &mut Directory,
    command: &str,
    input: Input<'_, '_>,
) -> Result<String, Failure> {
    match command {
        "file" => *file = File::new(name(input)),
        "dir" => *directory = Directory::with_path(CadPath::new(name(input))),
        "iter" => return Ok(iterate(input)),
        "fixture" | "mkfifo" | "chmod" | "symlink" => return Ok(fixture(command, input)),
        _ if command.starts_with('d') || command == "tempdir" => {
            return directory_command(directory, command, input);
        }
        _ => {
            if let Some(result) = node_command(file, command, input) {
                return result;
            }
            return file_command(file, command, input);
        }
    }
    Ok(String::new())
}

fn run(input: Input<'_, '_>) -> String {
    let mut file: File = File::empty();
    let mut directory: Directory = Directory::empty();
    let mut out: String = String::new();
    while let Some(command) = input.next() {
        let text: String = step(&mut file, &mut directory, command, input)
            .unwrap_or_else(|failure| failure_text(&failure));
        out += &format!(" {command}={text}");
    }
    // A pending error would make close raise (and the native destructor call it again).
    file.reset();
    if file.is_open() {
        let _ = file.close();
    }
    out
}
