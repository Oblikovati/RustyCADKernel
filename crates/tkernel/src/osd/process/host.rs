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

//! `OSD_Host`: name, system, address and vendor of the machine.
//!
//! The Unix build asks `uname`, `gethostname` and `gethostbyname` on every call. The
//! Windows build fills a process-wide cache from Win32/Winsock calls that have no
//! standard-library counterpart; there only `SystemId` and `MachineType` are ported.
use crate::osd::{OemType, OsdError, OsdErrorAccess, SysType, WhoAmI};
use crate::tcollection::AsciiString;

mod native;
pub use native::NativeHost;

/// The `uname` fields OSD_Host reads.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SystemNames {
    /// `utsname::sysname`, e.g. "Linux".
    pub sysname: Vec<u8>,
    /// `utsname::release`, e.g. "6.8.0".
    pub release: Vec<u8>,
}

/// Machine queries behind `OSD_Host`; implement for deterministic tests.
pub trait HostSource {
    /// The `uname` names; `source.system_names()`.
    fn system_names(&self) -> SystemNames;
    /// The host name (`gethostname` into a 64-byte buffer), or the native error number;
    /// `source.host_name()`.
    fn host_name(&self) -> Result<Vec<u8>, i32>;
    /// The first IPv4 address `gethostbyname` gives for `name`; `source.resolve_ipv4(b"host")`.
    fn resolve_ipv4(&self, name: &[u8]) -> Option<[u8; 4]>;
}

/// `OSD_Host`; `Host::new().host_name()`.
pub struct Host<S: HostSource = NativeHost> {
    source: S,
    error: OsdError,
}
impl Default for Host<NativeHost> {
    fn default() -> Self {
        Self::new()
    }
}
impl Host<NativeHost> {
    /// Queries the real machine; `Host::new()`.
    pub fn new() -> Self {
        Self::with_source(NativeHost)
    }
}
impl<S: HostSource> Host<S> {
    /// Queries an injected source; `Host::with_source(fake)`.
    pub fn with_source(source: S) -> Self {
        Self {
            source,
            error: OsdError::new(),
        }
    }
    /// `OSD_Host::SystemVersion`: "sysname release" on Unix; `host.system_version()`.
    pub fn system_version(&mut self) -> AsciiString {
        if cfg!(windows) {
            return AsciiString::default(); // GetVersionExW cache: pending.
        }
        let names: SystemNames = self.source.system_names();
        let mut text: Vec<u8> = names.sysname;
        text.push(b' ');
        text.extend_from_slice(&names.release);
        AsciiString::from_raw(&text)
    }
    /// `OSD_Host::SystemId`: the `OSD_SysType` of the `uname` system name (WindowsNt on
    /// Windows); `host.system_id()`.
    pub fn system_id(&self) -> SysType {
        if cfg!(windows) {
            return SysType::WindowsNt;
        }
        system_id_of(&self.source.system_names().sysname)
    }
    /// `OSD_Host::HostName`; records "Host Name" on failure; `host.host_name()`.
    ///
    /// Native Unix then returns whatever the buffer held; Rust returns the empty name.
    pub fn host_name(&mut self) -> AsciiString {
        if cfg!(windows) {
            return AsciiString::default(); // GetComputerNameA cache: pending.
        }
        match self.source.host_name() {
            Ok(name) => AsciiString::from_raw(&name),
            Err(errno) => {
                self.error
                    .set_value(errno, WhoAmI::Host as i32, "Host Name");
                AsciiString::default()
            }
        }
    }
    /// `OSD_Host::AvailableMemory`: always 0 on Unix (native "A faire"); `host.available_memory()`.
    pub fn available_memory(&mut self) -> i32 {
        // Windows reads dwAvailPageFile from GlobalMemoryStatus: pending, 0 as on Unix.
        0
    }
    /// `OSD_Host::InternetAddress`: the dotted IPv4 address of the host name, falling back
    /// to "localhost"; `host.internet_address()`.
    ///
    /// Natively a name that resolves nowhere dereferences null; Rust returns the empty text.
    pub fn internet_address(&mut self) -> AsciiString {
        if cfg!(windows) {
            return AsciiString::default(); // Winsock gethostbyname cache: pending.
        }
        let host: AsciiString = self.host_name();
        let address = self
            .source
            .resolve_ipv4(host.as_bytes())
            .or_else(|| self.source.resolve_ipv4(b"localhost"));
        address.map_or_else(AsciiString::default, |[a, b, c, d]| {
            AsciiString::from(format!("{a}.{b}.{c}.{d}").as_str())
        })
    }
    /// `OSD_Host::MachineType`: the `OSD_OEMType` of the `uname` system name (Pc on
    /// Windows); `host.machine_type()`.
    pub fn machine_type(&mut self) -> OemType {
        if cfg!(windows) {
            return OemType::Pc;
        }
        machine_type_of(&self.source.system_names().sysname)
    }
}
impl<S: HostSource> OsdErrorAccess for Host<S> {
    fn osd_error(&self) -> &OsdError {
        &self.error
    }
    fn osd_error_mut(&mut self) -> &mut OsdError {
        &mut self.error
    }
}

/// The native `OSD_Host::SystemId` mapping; `system_id_of(b"Linux") == SysType::LinuxRedHat`.
///
/// Exact names except the prefixes "Linux", "IRIX" and "OSF", tested in native order.
pub fn system_id_of(sysname: &[u8]) -> SysType {
    use SysType::*;
    let table: [(&[u8], bool, SysType); 10] = [
        (b"SunOS", true, UnixBsd),
        (b"ULTRIX", true, UnixBsd),
        (b"FreeBSD", true, UnixBsd),
        (b"Linux", false, LinuxRedHat),
        (b"IRIX", false, UnixSystemV),
        (b"OSF", false, Osf),
        (b"AIX", true, Aix),
        (b"UNIX_System_V", true, UnixSystemV),
        (b"VMS_POSIX", true, Vms),
        (b"Darwin", true, MacOs),
    ];
    first_match(sysname, &table).unwrap_or(Unknown)
}

/// The native `OSD_Host::MachineType` mapping; `machine_type_of(b"Darwin") == OemType::Mac`.
///
/// Exact names except the prefixes "IRIX", "OSF", "Linux" and "AIX", in native order.
pub fn machine_type_of(sysname: &[u8]) -> OemType {
    use OemType::*;
    let table: [(&[u8], bool, OemType); 11] = [
        (b"SunOS", true, Sun),
        (b"ULTRIX", true, Dec),
        (b"IRIX", false, Sgi),
        (b"HP-UX", true, Hp),
        (b"UNIX_System_V", true, Nec),
        (b"VMS_POSIX", true, Vax),
        (b"OSF", false, Dec),
        (b"Linux", false, Lin),
        (b"FreeBSD", true, Lin),
        (b"AIX", false, Aix),
        (b"Darwin", true, Mac),
    ];
    first_match(sysname, &table).unwrap_or(Unavailable)
}

/// `strcmp` (exact) or `strncmp` (prefix) rows, first match wins.
fn first_match<T: Copy>(sysname: &[u8], table: &[(&[u8], bool, T)]) -> Option<T> {
    table
        .iter()
        .find(|(name, exact, _)| {
            if *exact {
                sysname == *name
            } else {
                sysname.starts_with(name)
            }
        })
        .map(|&(_, _, value)| value)
}
