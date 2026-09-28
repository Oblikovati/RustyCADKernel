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

//! Resource_Manager::Save and the static GetResourcePath.
use super::{DEBUG, ResourceEnvironment, ResourceManager, lines, paths};
use crate::tcollection::AsciiString;
use std::{io::Write, sync::atomic::Ordering};
impl ResourceManager {
    /// Writes the user map, sorted by key, to `$CSF_<name>UserDefaults/<name>`, creating
    /// directories as needed (native `Save`); `manager.save()`.
    ///
    /// The destination variable is read at save time, and write errors after a successful
    /// open are ignored, as natively.
    pub fn save(&self) -> bool {
        let variable = format!("CSF_{}UserDefaults", self.name_text());
        let directory = self
            .host
            .environment
            .directory(&variable)
            .map(|path| paths::path_bytes(&path))
            .unwrap_or_default();
        if directory.is_empty() {
            self.verbose_trace(&[
                b"Resource Manager Warning: environment variable \"",
                variable.as_bytes(),
                b"\" not set.  Cannot save resources.\n",
            ]);
            return false;
        }
        let directory = AsciiString::from_raw(&directory);
        if self
            .host
            .io
            .create_directories(&paths::host_path(&directory))
            .is_err()
        {
            self.verbose_trace(&[
                b"Resource Manager: Error opening or creating directory \"",
                directory.as_bytes(),
                b"\". Permission denied. Cannot save resources.\n",
            ]);
            return false;
        }
        let file = paths::resource_file(directory.as_bytes(), &self.name_text());
        let Ok(mut output) = self.host.io.open_write(&paths::host_path(&file)) else {
            self.verbose_trace(&[
                b"Resource Manager: Error opening or creating file \"",
                file.as_bytes(),
                b"\". Permission denied. Cannot save resources.\n",
            ]);
            return false;
        };
        self.write_user_map(output.as_mut(), &file);
        true
    }
    fn write_user_map(&self, output: &mut dyn Write, file: &AsciiString) {
        if self.user.is_empty() {
            return;
        }
        let mut entries: Vec<_> = self.user.items().collect();
        // std::sort with TCollection_AsciiString::operator< orders keys bytewise.
        entries.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
        for (key, value) in entries {
            let line = lines::saved_line(key, value);
            if DEBUG.load(Ordering::Relaxed) {
                self.host
                    .console
                    .write(&[b"Line = '".as_slice(), &line, b"'\n"].concat());
            }
            let _ = output.write_all(&line);
        }
        self.verbose_trace(&[
            b"Resource Manager: Resources saved in file ",
            file.as_bytes(),
            b"\n",
        ]);
    }
    fn verbose_trace(&self, parts: &[&[u8]]) {
        if self.verbose {
            self.host.console.write(&parts.concat());
        }
    }
    fn name_text(&self) -> String {
        String::from_utf8_lossy(self.name.as_bytes()).into_owned()
    }
    /// Native static `GetResourcePath` with the process environment: the path of
    /// `$CSF_<name>[User]Defaults/<name>`, or an empty string when the variable is unset;
    /// `ResourceManager::resource_path("App", false)`.
    pub fn resource_path(name: &str, user_defaults: bool) -> AsciiString {
        Self::resource_path_with(name, user_defaults, &super::NativeResourceEnvironment)
    }
    /// `resource_path` over an injected environment; `ResourceManager::resource_path_with("App", true, &env)`.
    pub fn resource_path_with(
        name: &str,
        user_defaults: bool,
        environment: &dyn ResourceEnvironment,
    ) -> AsciiString {
        let name = name.split('\0').next().unwrap_or_default();
        let kind = if user_defaults {
            "UserDefaults"
        } else {
            "Defaults"
        };
        let directory = environment
            .directory(&format!("CSF_{name}{kind}"))
            .map(|path| paths::path_bytes(&path))
            .unwrap_or_default();
        if directory.is_empty() {
            return AsciiString::default();
        }
        paths::resource_file(&directory, name)
    }
}
