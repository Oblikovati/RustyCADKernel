// Copyright (c) 1998-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! `OSD_Process`: identity, date and working directory of the running process.
//!
//! Every system query goes through a [`ProcessSource`]; [`NativeProcess`] reads the real
//! process, tests inject named fakes. Failures are recorded in the object's `OSD_Error`
//! with the native context messages, as the Unix build does.
use super::{CadPath, OsdError, OsdErrorAccess, WhoAmI};
use crate::{Error, quantity::Date, tcollection::AsciiString};

mod environment;
mod host;
mod mem_info;
mod native;
mod shared_library;
mod source;
pub use environment::{Environment, EnvironmentSource, NativeEnvironment};
pub use host::{Host, HostSource, NativeHost, SystemNames, machine_type_of, system_id_of};
pub use mem_info::{MemCounter, MemInfo, MemInfoSource, NativeMemInfo};
pub use native::NativeProcess;
pub use shared_library::{SharedLibrary, SymbolAddress};
pub use source::{LocalTime, ProcessSource};

/// `OSD_Process`; `Process::new().current_directory()`.
pub struct Process<S: ProcessSource = NativeProcess> {
    source: S,
    error: OsdError,
}
impl Default for Process<NativeProcess> {
    fn default() -> Self {
        Self::new()
    }
}
impl Process<NativeProcess> {
    /// Queries the running process; `Process::new().process_id()`.
    pub fn new() -> Self {
        Self::with_source(NativeProcess)
    }
    /// `OSD_Process::ExecutablePath`: the absolute path of the running executable, or
    /// empty when the host cannot tell; `Process::executable_path()`.
    ///
    /// Linux reads the `/proc/<pid>/exe` link; macOS and Windows resolve the executable
    /// through the standard library (native: `_NSGetExecutablePath` + `realpath`,
    /// `GetModuleFileNameW`).
    pub fn executable_path() -> AsciiString {
        AsciiString::from_raw(&NativeProcess.executable_path())
    }
    /// `OSD_Process::ExecutableFolder`: [`Process::executable_path`] up to and including its
    /// last separator; `Process::executable_folder()`.
    pub fn executable_folder() -> AsciiString {
        executable_folder_of(&Self::executable_path())
    }
}
impl<S: ProcessSource> Process<S> {
    /// Queries an injected source; `Process::with_source(fake)`.
    pub fn with_source(source: S) -> Self {
        Self {
            source,
            error: OsdError::new(),
        }
    }
    /// `OSD_Process::TerminalType`; `process.terminal_type()`.
    ///
    /// Native quirk (Unix): the method reads the `TERM` variable but returns the name of
    /// the variable, "TERM", not its value. Windows returns "WIN32 console".
    pub fn terminal_type(&mut self) -> AsciiString {
        if cfg!(windows) {
            return AsciiString::from("WIN32 console");
        }
        let mut term: Environment = Environment::with_value("TERM", "")
            .expect("TERM and an empty value are valid variable spellings");
        term.value();
        term.name()
    }
    /// `OSD_Process::SystemDate`: the local date and time, to the second on Unix and to the
    /// millisecond on Windows; `process.system_date()?`.
    ///
    /// When the clock cannot be read the error is recorded ("GetSystem") and, as natively,
    /// building the all-zero date fails with `Quantity_DateDefinitionError`.
    pub fn system_date(&mut self) -> Result<Date, Error> {
        let time: LocalTime = self.source.local_time().unwrap_or_else(|errno| {
            self.error
                .set_value(errno, WhoAmI::Process as i32, "GetSystem");
            LocalTime::default()
        });
        Date::new(
            time.month,
            time.day,
            time.year,
            time.hour,
            time.minute,
            time.second,
            time.millisecond,
            0,
        )
    }
    /// `OSD_Process::UserName`: the login name, or empty when unknown; `process.user_name()`.
    pub fn user_name(&mut self) -> AsciiString {
        match self.source.user_name() {
            Ok(name) => AsciiString::from_raw(&name),
            Err(errno) => {
                self.error
                    .set_value(errno, WhoAmI::Process as i32, "UserName");
                AsciiString::default()
            }
        }
    }
    /// `OSD_Process::IsSuperUser`: Unix compares the user id with 0; `process.is_super_user()`.
    pub fn is_super_user(&mut self) -> bool {
        self.source.is_super_user()
    }
    /// `OSD_Process::ProcessId`; `process.process_id()`.
    pub fn process_id(&mut self) -> i32 {
        self.source.process_id()
    }
    /// `OSD_Process::CurrentDirectory`: the working directory as a directory path (Unix
    /// appends the separator so the whole name becomes the trek); `process.current_directory()`.
    ///
    /// On failure the error is recorded ("Where") and the empty path is returned.
    pub fn current_directory(&mut self) -> CadPath {
        match self.source.current_directory() {
            Ok(mut name) => {
                if !cfg!(windows) {
                    name.push(b'/');
                }
                CadPath::new(name)
            }
            Err(errno) => {
                self.error.set_value(errno, WhoAmI::Process as i32, "Where");
                CadPath::default()
            }
        }
    }
    /// `OSD_Process::SetCurrentDirectory`: changes to the system spelling of `path`;
    /// `process.set_current_directory(&CadPath::new("/tmp/"))`.
    pub fn set_current_directory(&mut self, path: &CadPath) {
        let name: AsciiString = path.system_name();
        if let Err(errno) = self.source.set_current_directory(name.as_bytes()) {
            let context: &str = "Move to directory";
            self.error.set_value(errno, WhoAmI::Process as i32, context);
        }
    }
}
impl<S: ProcessSource> OsdErrorAccess for Process<S> {
    fn osd_error(&self) -> &OsdError {
        &self.error
    }
    fn osd_error_mut(&mut self) -> &mut OsdError {
        &mut self.error
    }
}

/// The folder part of an executable path: everything up to and including the last host
/// separator (`\` on Windows, `/` elsewhere), or empty without one;
/// `executable_folder_of(&"/usr/bin/tool".into())` is "/usr/bin/".
pub fn executable_folder_of(path: &AsciiString) -> AsciiString {
    let separator: u8 = if cfg!(windows) { b'\\' } else { b'/' };
    path.as_bytes()
        .iter()
        .rposition(|&byte| byte == separator)
        .map_or_else(AsciiString::default, |index| {
            AsciiString::from_raw(&path.as_bytes()[..=index])
        })
}
