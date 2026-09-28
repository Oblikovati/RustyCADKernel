//! OSD_Process and OSD_Host: every public member, driven through named fake sources, plus
//! native smoke checks on the running host.
use rustycad_tkernel::osd::{
    CadPath, Host, HostSource, LocalTime, OemType, OsdErrorAccess, Process, ProcessSource, SysType,
    SystemNames, WhoAmI, executable_folder_of, machine_type_of, system_id_of,
};
use rustycad_tkernel::tcollection::AsciiString;
use std::cell::RefCell;

/// A process whose every answer is fixed; failures are toggled per query.
struct FakeProcess {
    clock_errno: Option<i32>,
    directory: Result<Vec<u8>, i32>,
    moved_to: RefCell<Vec<Vec<u8>>>,
    move_errno: Option<i32>,
}
impl FakeProcess {
    fn healthy() -> Self {
        Self {
            clock_errno: None,
            directory: Ok(b"/work/space".to_vec()),
            moved_to: RefCell::new(Vec::new()),
            move_errno: None,
        }
    }
}
impl ProcessSource for FakeProcess {
    fn process_id(&self) -> i32 {
        4242
    }
    fn is_super_user(&self) -> bool {
        true
    }
    fn user_name(&self) -> Result<Vec<u8>, i32> {
        Ok(b"designer".to_vec())
    }
    fn local_time(&self) -> Result<LocalTime, i32> {
        if let Some(errno) = self.clock_errno {
            return Err(errno);
        }
        Ok(LocalTime {
            month: 2,
            day: 29,
            year: 2024,
            hour: 13,
            minute: 5,
            second: 59,
            millisecond: 0,
        })
    }
    fn current_directory(&self) -> Result<Vec<u8>, i32> {
        self.directory.clone()
    }
    fn set_current_directory(&self, path: &[u8]) -> Result<(), i32> {
        self.moved_to.borrow_mut().push(path.to_vec());
        self.move_errno.map_or(Ok(()), Err)
    }
    fn executable_path(&self) -> Vec<u8> {
        b"/opt/cad/bin/tool".to_vec()
    }
}

#[test]
fn process_reports_injected_identity_and_date() {
    let mut process = Process::with_source(FakeProcess::healthy());
    assert_eq!(process.process_id(), 4242);
    assert!(process.is_super_user());
    assert_eq!(process.user_name().as_bytes(), b"designer");
    let date = process.system_date().unwrap();
    assert_eq!(date.values(), (2, 29, 2024, 13, 5, 59, 0, 0));
    assert!(!process.failed());
}

#[cfg(unix)]
#[test]
fn process_terminal_type_returns_the_variable_name_on_unix() {
    // Native quirk: TerminalType reads TERM but hands back the variable name.
    assert_eq!(Process::new().terminal_type().as_bytes(), b"TERM");
}

#[cfg(unix)]
#[test]
fn process_clock_failure_records_get_system_and_rejects_the_zero_date() {
    let mut source = FakeProcess::healthy();
    source.clock_errno = Some(libc::EFAULT);
    let mut process = Process::with_source(source);
    let error = process.system_date().unwrap_err();
    assert_eq!(error.exception_type(), "Quantity_DateDefinitionError");
    assert!(process.failed());
    assert_eq!(process.osd_error().message().as_bytes(), b"GetSystem");
    assert_eq!(process.osd_error().who(), Some(WhoAmI::Process));
    assert!(process.perror().is_err());
}

#[cfg(unix)]
#[test]
fn process_current_directory_appends_the_separator_and_records_failures() {
    let mut process = Process::with_source(FakeProcess::healthy());
    let path: CadPath = process.current_directory();
    assert_eq!(path.trek().as_bytes(), b"|work|space|");
    assert_eq!(path.system_name().as_bytes(), b"/work/space/");
    let mut source = FakeProcess::healthy();
    source.directory = Err(libc::EACCES);
    let mut failing = Process::with_source(source);
    assert!(failing.current_directory().system_name().is_empty());
    assert_eq!(failing.osd_error().message().as_bytes(), b"Where");
    failing.reset();
    assert!(!failing.failed());
}

#[test]
fn process_set_current_directory_passes_the_system_name() {
    let mut source = FakeProcess::healthy();
    source.move_errno = Some(2);
    let mut process = Process::with_source(source);
    process.set_current_directory(&CadPath::new("/tmp/cad/"));
    assert!(process.failed());
    assert_eq!(
        process.osd_error().message().as_bytes(),
        b"Move to directory"
    );
    assert_eq!(process.osd_error().errno(), 2);
}

#[test]
fn executable_folder_keeps_the_last_separator() {
    let separator: &str = if cfg!(windows) { "\\" } else { "/" };
    let path: String = ["", "opt", "cad", "tool"].join(separator);
    let folder = executable_folder_of(&AsciiString::from(path.as_str()));
    assert_eq!(
        folder.as_bytes(),
        ["", "opt", "cad", ""].join(separator).as_bytes()
    );
    assert!(executable_folder_of(&AsciiString::from("tool")).is_empty());
}

#[cfg(target_os = "linux")]
#[test]
fn native_executable_path_names_the_test_binary() {
    let path = Process::executable_path();
    let expected = std::env::current_exe().unwrap();
    assert_eq!(path.as_bytes(), expected.to_str().unwrap().as_bytes());
    let folder = Process::executable_folder();
    assert!(path.as_bytes().starts_with(folder.as_bytes()));
    assert_eq!(folder.as_bytes().last(), Some(&b'/'));
    assert_eq!(Process::new().process_id(), std::process::id() as i32);
}

/// A machine with fixed names; the resolver knows only listed hosts.
struct FakeHost {
    sysname: &'static [u8],
    name: Result<Vec<u8>, i32>,
    known: Vec<(&'static [u8], [u8; 4])>,
}
impl HostSource for FakeHost {
    fn system_names(&self) -> SystemNames {
        SystemNames {
            sysname: self.sysname.to_vec(),
            release: b"6.8.0-generic".to_vec(),
        }
    }
    fn host_name(&self) -> Result<Vec<u8>, i32> {
        self.name.clone()
    }
    fn resolve_ipv4(&self, name: &[u8]) -> Option<[u8; 4]> {
        self.known
            .iter()
            .find(|(host, _)| *host == name)
            .map(|(_, address)| *address)
    }
}

#[cfg(unix)]
#[test]
fn host_reports_injected_names_and_falls_back_to_localhost() {
    let mut host = Host::with_source(FakeHost {
        sysname: b"Linux",
        name: Ok(b"bench".to_vec()),
        known: vec![(b"localhost", [127, 0, 0, 1])],
    });
    assert_eq!(host.system_version().as_bytes(), b"Linux 6.8.0-generic");
    assert_eq!(host.system_id(), SysType::LinuxRedHat);
    assert_eq!(host.machine_type(), OemType::Lin);
    assert_eq!(host.host_name().as_bytes(), b"bench");
    assert_eq!(host.internet_address().as_bytes(), b"127.0.0.1");
    assert_eq!(host.available_memory(), 0);
    assert!(!host.failed());
}

#[cfg(unix)]
#[test]
fn host_name_failure_records_host_name() {
    let mut host = Host::with_source(FakeHost {
        sysname: b"Darwin",
        name: Err(libc::ENAMETOOLONG),
        known: vec![(b"", [10, 1, 2, 3])],
    });
    assert!(host.host_name().is_empty());
    assert!(host.failed());
    assert_eq!(host.osd_error().message().as_bytes(), b"Host Name");
    assert_eq!(host.internet_address().as_bytes(), b"10.1.2.3");
    assert_eq!(host.machine_type(), OemType::Mac);
}

#[test]
fn system_and_machine_tables_follow_native_order_and_prefix_rules() {
    assert_eq!(system_id_of(b"Linux-custom"), SysType::LinuxRedHat);
    assert_eq!(system_id_of(b"SunOS"), SysType::UnixBsd);
    assert_eq!(system_id_of(b"SunOS5"), SysType::Unknown);
    assert_eq!(system_id_of(b"IRIX64"), SysType::UnixSystemV);
    assert_eq!(system_id_of(b"OSF1"), SysType::Osf);
    assert_eq!(system_id_of(b"AIXv7"), SysType::Unknown);
    assert_eq!(system_id_of(b"VMS_POSIX"), SysType::Vms);
    assert_eq!(system_id_of(b"HP-UX"), SysType::Unknown);
    assert_eq!(machine_type_of(b"AIXv7"), OemType::Aix);
    assert_eq!(machine_type_of(b"HP-UX"), OemType::Hp);
    assert_eq!(machine_type_of(b"FreeBSD"), OemType::Lin);
    assert_eq!(machine_type_of(b"UNIX_System_V"), OemType::Nec);
    assert_eq!(machine_type_of(b"ULTRIX"), OemType::Dec);
    assert_eq!(machine_type_of(b"Windows"), OemType::Unavailable);
}

#[cfg(target_os = "linux")]
#[test]
fn native_host_matches_uname_on_linux() {
    let mut host = Host::new();
    assert_eq!(host.system_id(), SysType::LinuxRedHat);
    assert_eq!(host.machine_type(), OemType::Lin);
    assert!(host.system_version().as_bytes().starts_with(b"Linux "));
    assert!(!host.host_name().is_empty());
    assert_eq!(
        host.internet_address()
            .as_bytes()
            .iter()
            .filter(|&&b| b == b'.')
            .count(),
        3
    );
}
