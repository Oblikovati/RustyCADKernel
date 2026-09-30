//! `Storage_Schema::CheckTypeMigration`: the old-to-new persistent type name table.
use crate::ncollection::DataMap;
use crate::tcollection::AsciiString;
use std::sync::OnceLock;

/// The table file named by `CSF_MIGRATION_TYPES`, as the native loader finds it.
pub enum MigrationFile {
    /// No such file (or the variable is unset): the built-in table applies.
    Missing,
    /// The path exists but cannot be opened for reading: the table stays empty.
    Unreadable,
    /// The file's bytes.
    Contents(Vec<u8>),
}

/// Environment and file-system seam of the migration table; inject a named fake in tests.
pub trait MigrationSource {
    /// Resolves `CSF_MIGRATION_TYPES` and reads the file it names; `source.migration_file()`.
    fn migration_file(&self) -> MigrationFile;
}

/// Reads `CSF_MIGRATION_TYPES` and the host file system; `TypeMigration::load(&NativeMigrationSource)`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeMigrationSource;

impl MigrationSource for NativeMigrationSource {
    fn migration_file(&self) -> MigrationFile {
        let path = std::env::var_os("CSF_MIGRATION_TYPES").unwrap_or_default();
        if path.is_empty() || !std::path::Path::new(&path).exists() {
            return MigrationFile::Missing;
        }
        std::fs::read(&path).map_or(MigrationFile::Unreadable, MigrationFile::Contents)
    }
}

/// Built-in renames used when no table file exists (`TDataStd` to `TDataXtd` and friends).
const BUILT_IN: [(&str, &str); 19] = [
    ("TDataStd_Shape", "TDataXtd_Shape"),
    ("TDataStd_Constraint", "TDataXtd_Constraint"),
    ("TDataStd_Geometry", "TDataXtd_Geometry"),
    ("TDataStd_Axis", "TDataXtd_Axis"),
    ("TDataStd_Point", "TDataXtd_Point"),
    ("TDataStd_Plane", "TDataXtd_Plane"),
    ("TDataStd_Position", "TDataXtd_Position"),
    ("TDataStd_Placement", "TDataXtd_Placement"),
    ("TDataStd_PatternStd", "TDataXtd_PatternStd"),
    ("TPrsStd_AISPresentation", "TDataXtd_Presentation"),
    ("PDataStd_Shape", "PDataXtd_Shape"),
    ("PDataStd_Constraint", "PDataXtd_Constraint"),
    ("PDataStd_Geometry", "PDataXtd_Geometry"),
    ("PDataStd_Axis", "PDataXtd_Axis"),
    ("PDataStd_Point", "PDataXtd_Point"),
    ("PDataStd_Plane", "PDataXtd_Plane"),
    ("PDataStd_Position", "PDataXtd_Position"),
    ("PDataStd_Placement", "PDataXtd_Placement"),
    ("PDataStd_PatternStd", "PDataXtd_PatternStd"),
];

/// `OSD_File::ReadLine(buffer, 80, n)` on POSIX: `fgets` of at most 79 bytes, keeping the
/// newline, stopping at the end of data or at a line whose C string is empty.
fn table_lines(contents: &[u8]) -> Vec<&[u8]> {
    const LINE_BYTES: usize = 79;
    let mut lines = Vec::new();
    let mut rest = contents;
    while !rest.is_empty() {
        let limit = rest.len().min(LINE_BYTES);
        let length = rest[..limit]
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(limit, |newline| newline + 1);
        let line = &rest[..length];
        if line.first().is_none_or(|byte| *byte == 0) {
            break;
        }
        lines.push(line);
        rest = &rest[length..];
    }
    lines
}

/// The loaded migration table.
///
/// ```
/// use rustycad_tkernel::storage::{MigrationFile, MigrationSource, TypeMigration};
/// use rustycad_tkernel::tcollection::AsciiString;
/// struct NoTableFile;
/// impl MigrationSource for NoTableFile {
///     fn migration_file(&self) -> MigrationFile { MigrationFile::Missing }
/// }
/// let table = TypeMigration::load(&NoTableFile);
/// let mut renamed = AsciiString::default();
/// assert!(table.check(&AsciiString::new("TDataStd_Shape"), &mut renamed));
/// assert_eq!(renamed.as_bytes(), b"TDataXtd_Shape");
/// ```
#[derive(Default)]
pub struct TypeMigration {
    renames: DataMap<AsciiString, AsciiString>,
}

impl TypeMigration {
    /// Loads the table as the first native `CheckTypeMigration` call does.
    /// Example: `TypeMigration::load(&NativeMigrationSource)`.
    pub fn load(source: &dyn MigrationSource) -> Self {
        let mut table = Self::default();
        match source.migration_file() {
            MigrationFile::Missing => table.bind_built_in(),
            MigrationFile::Unreadable => {}
            MigrationFile::Contents(bytes) => table.bind_lines(&bytes),
        }
        table
    }

    fn bind_built_in(&mut self) {
        for (old, new) in BUILT_IN {
            self.renames
                .bind(AsciiString::new(old), AsciiString::new(new));
        }
    }

    fn bind_lines(&mut self, contents: &[u8]) {
        for line in table_lines(contents) {
            let line = AsciiString::from_bytes(line);
            let key = line.token(" \t", 1);
            let value = line.token(" \t\n\r", 2);
            self.renames.bind(key, value);
        }
    }

    /// The process-wide table, loaded once from the host environment, like the native
    /// function-local statics; `TypeMigration::process_wide()`.
    pub fn process_wide() -> &'static Self {
        static TABLE: OnceLock<TypeMigration> = OnceLock::new();
        TABLE.get_or_init(|| Self::load(&NativeMigrationSource))
    }

    /// `CheckTypeMigration`: replaces `new_name` and returns true when `old_name` is renamed.
    /// Example: `table.check(&old, &mut new)`.
    pub fn check(&self, old_name: &AsciiString, new_name: &mut AsciiString) -> bool {
        match self.renames.seek(old_name) {
            Some(renamed) => {
                *new_name = renamed.clone();
                true
            }
            None => false,
        }
    }
}
