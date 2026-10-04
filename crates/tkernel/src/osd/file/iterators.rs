//! `OSD_DirectoryIterator` and `OSD_FileIterator` of the non-`_WIN32` build: one
//! `readdir` walk that keeps the entries of one type whose name matches a `*` mask.
use super::directory_stream::SharedDirectoryEntries;
use super::{
    Directory, File,
    calls::{NodeType, PosixFileCalls},
    native_calls,
};
use crate::osd::{CadPath, OsdError, OsdErrorAccess};
use std::sync::Arc;

/// `strcmp_joker`: `*` matches any run of bytes; every other byte matches itself.
pub fn mask_matches(mask: &[u8], name: &[u8]) -> bool {
    let literal: usize = mask
        .iter()
        .position(|&byte| byte == b'*')
        .unwrap_or(mask.len());
    if name.len() < literal || mask[..literal] != name[..literal] {
        // Native compares byte by byte, so a shorter name fails at its terminator.
        return false;
    }
    let (mask, name) = (&mask[literal..], &name[literal..]);
    if mask.is_empty() {
        return name.is_empty();
    }
    let rest: &[u8] = &mask[mask
        .iter()
        .position(|&byte| byte != b'*')
        .unwrap_or(mask.len())..];
    if rest.is_empty() {
        return true;
    }
    // Native tries every suffix except the empty one.
    (0..name.len()).any(|start| mask_matches(rest, &name[start..]))
}

/// The entries one iterator kind keeps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EntryKind {
    /// Directories, `.` and `..` included.
    Directories,
    /// Regular files; `.` and `..` are skipped before the type test.
    Files,
}

/// The shared `readdir` state (`myPlace`, `myMask`, `myDescr`, `myEntry`, `myFlag`, `myInit`).
#[derive(Clone)]
struct DirectoryWalk {
    kind: EntryKind,
    place: Vec<u8>,
    mask: Vec<u8>,
    open: Option<SharedDirectoryEntries>,
    entry: Option<Vec<u8>>,
    flag: bool,
    init: bool,
    error: OsdError,
    calls: Arc<dyn PosixFileCalls>,
}

impl DirectoryWalk {
    fn new(kind: EntryKind, calls: Arc<dyn PosixFileCalls>) -> Self {
        Self {
            kind,
            place: Vec::new(),
            mask: Vec::new(),
            open: None,
            entry: None,
            flag: false,
            init: false,
            error: OsdError::new(),
            calls,
        }
    }
    fn initialize(&mut self, place: &CadPath, mask: &[u8]) {
        self.flag = false;
        self.place = place.system_name().as_bytes().to_vec();
        if self.place.is_empty() {
            self.place = b".".to_vec();
        }
        self.mask = mask.to_vec();
        self.open = None;
        self.init = true;
    }
    fn more(&mut self) -> bool {
        if self.init {
            self.init = false;
            if let Ok(names) = self.calls.open_directory(&self.place) {
                self.open = Some(SharedDirectoryEntries::new(names));
                self.flag = true;
                self.next();
            }
        }
        self.flag
    }
    /// Advances to the next kept entry, closing the directory after the last one.
    /// Natively `Next` on a closed directory calls `readdir(NULL)`; here it ends.
    fn next(&mut self) {
        self.flag = false;
        // Natively a failing stat leaves the previous entry's stat buffer in place.
        let mut last_type: Option<NodeType> = None;
        loop {
            let Some(name) = self
                .open
                .as_ref()
                .and_then(SharedDirectoryEntries::next_name)
            else {
                self.entry = None;
                self.open = None;
                return;
            };
            if self.kind == EntryKind::Files && (name == b"." || name == b"..") {
                self.entry = Some(name);
                continue;
            }
            let mut full: Vec<u8> = self.place.clone();
            full.push(b'/');
            full.extend_from_slice(&name);
            last_type = self
                .calls
                .stat(&full)
                .map(|status| status.node_type)
                .ok()
                .or(last_type);
            let wanted: NodeType = match self.kind {
                EntryKind::Directories => NodeType::Directory,
                EntryKind::Files => NodeType::Regular,
            };
            let keep: bool = last_type == Some(wanted) && mask_matches(&self.mask, &name);
            self.entry = Some(name);
            if keep {
                self.flag = true;
                return;
            }
        }
    }
    /// The current entry as a path with name before, and extension from, its first dot.
    fn value_path(&self) -> CadPath {
        let name: &[u8] = self.entry.as_deref().unwrap_or_default();
        let dot: usize = name
            .iter()
            .position(|&byte| byte == b'.')
            .unwrap_or(name.len());
        CadPath::from_values("", "", "", "", "", &name[..dot], &name[dot..])
    }
}

macro_rules! walk_error_access {
    ($type:ty) => {
        impl OsdErrorAccess for $type {
            fn osd_error(&self) -> &OsdError {
                &self.walk.error
            }
            fn osd_error_mut(&mut self) -> &mut OsdError {
                &mut self.walk.error
            }
        }
    };
}

/// `OSD_DirectoryIterator`: the subdirectories (with `.` and `..`) whose name matches a
/// mask, in `readdir` order; `let mut it = DirectoryIterator::new(&dir, "*"); while it.more() { it.next(); }`.
#[derive(Clone)]
pub struct DirectoryIterator {
    walk: DirectoryWalk,
}
walk_error_access!(DirectoryIterator);

/// `OSD_FileIterator`: the regular files whose name matches a mask, in `readdir` order;
/// `let mut it = FileIterator::new(&dir, "*.step")`.
#[derive(Clone)]
pub struct FileIterator {
    walk: DirectoryWalk,
}
walk_error_access!(FileIterator);

macro_rules! iterator_members {
    ($type:ident, $kind:expr, $value:ty, $value_new:path) => {
        impl Default for $type {
            fn default() -> Self {
                Self::empty()
            }
        }
        impl $type {
            /// The default constructor: nothing to iterate until `initialize`; `Self::empty()`.
            pub fn empty() -> Self {
                Self {
                    walk: DirectoryWalk::new($kind, native_calls()),
                }
            }
            /// The (where, mask) constructor over the native calls; `Self::new(&dir, "*")`.
            pub fn new(place: &CadPath, mask: impl AsRef<[u8]>) -> Self {
                Self::with_calls(place, mask, native_calls())
            }
            /// The (where, mask) constructor over injected calls; `Self::with_calls(&dir, "*", calls)`.
            pub fn with_calls(
                place: &CadPath,
                mask: impl AsRef<[u8]>,
                calls: Arc<dyn PosixFileCalls>,
            ) -> Self {
                let mut iterator: Self = Self {
                    walk: DirectoryWalk::new($kind, calls),
                };
                iterator.initialize(place, mask);
                iterator
            }
            /// `Destroy`: a no-op kept for Windows compatibility (the directory stays open);
            /// `iterator.destroy()`.
            pub fn destroy(&mut self) {}
            /// `Initialize`: restarts on `place` (its system name, "." when empty) with a
            /// new mask; the directory is opened by the next `more`; `iterator.initialize(&dir, "*")`.
            pub fn initialize(&mut self, place: &CadPath, mask: impl AsRef<[u8]>) {
                self.walk.initialize(place, mask.as_ref());
            }
            /// `More`: opens the directory on first use (an unreadable one has no entries)
            /// and tells whether an entry is selected; `iterator.more()`.
            pub fn more(&mut self) -> bool {
                self.walk.more()
            }
            /// `Next`: selects the next matching entry; `iterator.next()`.
            #[allow(clippy::should_implement_trait)] // The native iterator protocol.
            pub fn next(&mut self) {
                self.walk.next();
            }
            /// `Values`: the selected entry as a node whose path holds only its name and
            /// extension (split at the first dot); `iterator.values()`.
            pub fn values(&self) -> $value {
                $value_new(self.walk.value_path(), Arc::clone(&self.walk.calls))
            }
        }
    };
}

iterator_members!(
    DirectoryIterator,
    EntryKind::Directories,
    Directory,
    Directory::with_calls
);
iterator_members!(FileIterator, EntryKind::Files, File, File::with_calls);
