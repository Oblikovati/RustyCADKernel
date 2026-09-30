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
        let Some(directory) = self.save_directory() else {
            return false;
        };
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
    /// `$CSF_<name>UserDefaults`, created when missing; `None` after the native warning.
    fn save_directory(&self) -> Option<AsciiString> {
        let variable = format!("CSF_{}UserDefaults", self.name_text());
        let environment = self.host.environment.directory(&variable);
        let directory = environment.map_or_else(Vec::new, |path| paths::path_bytes(&path));
        if directory.is_empty() {
            self.verbose_trace(&[
                b"Resource Manager Warning: environment variable \"",
                variable.as_bytes(),
                b"\" not set.  Cannot save resources.\n",
            ]);
            return None;
        }
        let directory = AsciiString::from_raw(&directory);
        let created = self
            .host
            .io
            .create_directories(&paths::host_path(&directory));
        if created.is_err() {
            self.verbose_trace(&[
                b"Resource Manager: Error opening or creating directory \"",
                directory.as_bytes(),
                b"\". Permission denied. Cannot save resources.\n",
            ]);
            return None;
        }
        Some(directory)
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
