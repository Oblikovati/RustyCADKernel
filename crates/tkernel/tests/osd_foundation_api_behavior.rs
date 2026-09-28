//! OSD enumerations, OSD_Error, OSD_Protection and exception classes: every public
//! member, with the native quirks each keeps.
// Most cases exercise the Linux native profile; other hosts compile the shared fakes only.
#![cfg_attr(not(target_os = "linux"), allow(unused_imports, dead_code))]
use SingleProtection::*;
use rustycad_tkernel::osd::{
    self, OsdError, OsdErrorAccess, Protection, SignalMode, SingleProtection, SysType, WhoAmI,
    error_list::*, windows_error_code, windows_perror_header,
};

#[test]
fn enumerations_keep_native_order_and_reject_foreign_values() {
    assert_eq!(SysType::from_raw(9), Some(SysType::WindowsNt));
    assert_eq!(SysType::from_raw(12), Option::None);
    assert_eq!(SingleProtection::from_raw(-1), Option::None);
    assert_eq!(WhoAmI::ALL.len(), 15);
    assert_eq!(SignalMode::Unset as i32, 3);
    assert_eq!(osd::OemType::Aix as i32, 11);
    assert_eq!(osd::KindFile::from_raw(4), Some(osd::KindFile::Unknown));
}

#[cfg(unix)]
#[test]
fn unix_protection_defaults_setters_and_quirk_tables() {
    let mut protection: Protection = Protection::new();
    assert_eq!(protection.values(), (R, Rwd, R, R));
    protection.set_values(Rwxd, None, X, W);
    assert_eq!(protection.values(), (Rwxd, None, X, W));
    protection.set_system(R);
    protection.set_user(Rw);
    protection.set_group(Rx);
    protection.set_world(D);
    assert_eq!(
        (
            protection.system(),
            protection.user(),
            protection.group(),
            protection.world()
        ),
        (R, Rw, Rx, D)
    );
    let mut right: SingleProtection = Wx;
    protection.add(&mut right, W);
    assert_eq!(right, Rwx, "native TabProtAdd[WX][W] quirk");
    protection.add(&mut right, D);
    assert_eq!(right, Rwx, "native TabProtAdd[RWX][D] quirk");
    let mut right: SingleProtection = X;
    protection.sub(&mut right, Rw);
    assert_eq!(right, None, "native TabProtSub[X][RW] quirk");
    let mut right: SingleProtection = Rwxd;
    protection.sub(&mut right, Wx);
    assert_eq!(right, Rd);
    assert_eq!(Protection::with_values(R, Rwd, R, R), Protection::default());
}

#[cfg(unix)]
#[test]
fn unix_perror_classifies_errno_by_reporting_class() {
    let mut error: OsdError = OsdError::new();
    assert!(!error.failed());
    assert_eq!(error.perror(), Ok(()));
    error.set_value(libc::ENOENT, WhoAmI::File as i32, "Open");
    assert!(error.failed());
    assert_eq!(error.error(), 0, "Unix Error() is only set by Perror");
    let failure = error.perror().unwrap_err();
    assert_eq!(failure.exception_type(), "OSD_OSDError");
    assert_eq!(failure.message(), " :\n Invalid path (empty string).\n\n");
    assert_eq!(error.error(), ERR_NOENT);
    error.reset();
    assert!(!error.failed());
    assert_eq!(error.error(), ERR_NOENT, "Reset keeps the portable code");
    error.set_value(libc::ENOENT, WhoAmI::FileNode as i32, "Remove");
    let failure = error.perror().unwrap_err();
    assert_eq!(
        failure.message(),
        " :\n File doesn't exist orInvalid path (empty string).\n\n"
    );
    assert_eq!(error.who(), Some(WhoAmI::FileNode));
    assert_eq!(error.errno(), libc::ENOENT);
    assert_eq!(error.message().as_bytes(), b"Remove");
}

#[cfg(unix)]
#[test]
fn unix_perror_keeps_the_surprise_unknown_and_eloop_codes() {
    let mut error: OsdError = OsdError::new();
    error.set_value(libc::ELOOP, WhoAmI::Directory as i32, "");
    let failure = error.perror().unwrap_err();
    assert_eq!(failure.message(), " :\n Too many symbolic links.\n\n");
    assert_eq!(
        error.error(),
        ERR_SURPRISE,
        "native ELOOP arm never sets extCode"
    );
    error.set_value(libc::EBADF, WhoAmI::Host as i32, "");
    assert_eq!(error.perror().unwrap_err().message(), " :\n .\n\n");
    assert_eq!(error.error(), ERR_SURPRISE);
    error.set_value(1000, 99, "");
    assert_eq!(
        error.perror().unwrap_err().message(),
        " :\n Unknowm error #1000.\n\n"
    );
    assert_eq!(error.error(), ERR_UNKNOWN);
    assert_eq!(error.who(), Option::None);
}

#[test]
fn windows_tables_prefer_the_class_table_then_the_common_one() {
    assert_eq!(windows_error_code(6, WhoAmI::File as i32), ERR_FBADF);
    assert_eq!(windows_error_code(6, WhoAmI::Host as i32), ERR_INVAL);
    assert_eq!(windows_error_code(9, WhoAmI::Host as i32), ERR_NOMEM);
    assert_eq!(windows_error_code(5, WhoAmI::FileNode as i32), ERR_ACCESS);
    assert_eq!(
        windows_error_code(145, WhoAmI::FileNode as i32),
        ERR_FNNOTEMPTY
    );
    assert_eq!(windows_error_code(145, WhoAmI::File as i32), ERR_SURPRISE);
    assert_eq!(windows_error_code(2, WhoAmI::Directory as i32), ERR_NOENT);
    assert_eq!(
        windows_error_code(-1, WhoAmI::Directory as i32),
        ERR_SURPRISE
    );
}

#[test]
fn windows_perror_header_names_the_class_within_its_buffer() {
    assert_eq!(
        windows_perror_header(WhoAmI::File as i32),
        "Error ( OSD_File )"
    );
    assert_eq!(
        windows_perror_header(WhoAmI::Process as i32),
        "Error ( OSD_Environment )"
    );
    assert_eq!(
        windows_perror_header(WhoAmI::DirectoryIterator as i32),
        "Error ( OSD_DirectoryIterator )"
    );
    assert_eq!(
        windows_perror_header(WhoAmI::EnvironmentIterator as i32),
        "Error ( OSD_EnvironmentIterator"
    );
    assert_eq!(windows_perror_header(-5), "Error ( Unknown )");
}

struct ErrorHolderFixture {
    error: OsdError,
}
impl OsdErrorAccess for ErrorHolderFixture {
    fn osd_error(&self) -> &OsdError {
        &self.error
    }
    fn osd_error_mut(&mut self) -> &mut OsdError {
        &mut self.error
    }
}

#[test]
fn error_access_trait_forwards_to_the_recorded_error() {
    let mut holder = ErrorHolderFixture {
        error: OsdError::default(),
    };
    assert!(!holder.failed());
    assert_eq!(holder.error(), 0);
    holder
        .osd_error_mut()
        .set_value(13, WhoAmI::Package as i32, "");
    assert!(holder.failed());
    let _ = holder.perror();
    holder.reset();
    assert!(!holder.failed());
}

#[test]
fn exception_constructors_carry_the_native_class_names() {
    assert_eq!(osd::sigsegv("x").exception_type(), "OSD_SIGSEGV");
    assert_eq!(osd::exception_ctrl_break("y").message(), "y");
    assert_eq!(osd::osd_osd_error("z").exception_type(), "OSD_OSDError");
    assert_eq!(osd::OSD_EXCEPTION_TYPES.len(), 21);
    assert_eq!(osd::OSD_EXCEPTION_TYPES[12], "OSD_Signal");
}
