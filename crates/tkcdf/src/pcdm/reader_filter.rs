// Rust port of ApplicationFramework/TKCDF/PCDM/PCDM_ReaderFilter.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `PCDM_ReaderFilter`: which attributes and which label sub-trees a reader loads.
//!
//! The native class keeps a tree of `NCollection_DataMap<int, void*>` maps, each child map
//! holding its father under the key -1 and the mark "in a sub-tree definition" under -2. The
//! port keeps the same tree as an arena of [`TagNode`]s indexed by position; `myCurrent` is
//! the index of the current node. No OCCT class overrides the virtual members, so the class
//! is a plain struct.
use rustycad_tkernel::{
    impl_transient,
    ncollection::Map,
    standard::{RuntimeType, SharedState, StandardTransient},
    tcollection::AsciiString,
};
use std::collections::BTreeMap;

/// `PCDM_ReaderFilter::AppendMode`: how a reader merges into an existing document.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PcdmReaderFilterAppendMode {
    /// `AppendMode_Forbid`: do not allow append (default).
    #[default]
    Forbid = 0,
    /// `AppendMode_Protect`: keep existing attributes, read only new ones.
    Protect = 1,
    /// `AppendMode_Overwrite`: overwrite existing attributes with the loaded ones.
    Overwrite = 2,
}

/// One map of the native iteration tree: the father (key -1), the children by tag and the
/// "sub-tree end" mark (key -2).
#[derive(Clone, Debug, Default)]
struct TagNode {
    father: Option<usize>,
    children: BTreeMap<i32, usize>,
    in_sub_trees: bool,
}

#[derive(Default)]
struct FilterState {
    append: PcdmReaderFilterAppendMode,
    skip: Map<AsciiString>,
    read: Map<AsciiString>,
    sub_trees: Vec<AsciiString>,
    tree: Vec<TagNode>,
    current: Option<usize>,
    current_depth: i32,
}

/// `PCDM_ReaderFilter`: attribute-type and entry filters of a partial read;
/// `PcdmReaderFilter::with_entry(&AsciiString::new("0:1"))`.
///
/// ```
/// use rustycad_tkcdf::pcdm::PcdmReaderFilter;
/// use rustycad_tkernel::tcollection::AsciiString;
/// let filter = PcdmReaderFilter::with_entry(&AsciiString::new("0:1"));
/// assert!(filter.is_passed_entry(&AsciiString::new("0:1:2")));
/// assert!(!filter.is_passed_entry(&AsciiString::new("0:10")));
/// ```
#[derive(Default)]
pub struct PcdmReaderFilter {
    transient: StandardTransient,
    state: SharedState<FilterState>,
}
impl_transient!(
    PcdmReaderFilter,
    "PCDM_ReaderFilter",
    StandardTransient,
    transient
);

impl PcdmReaderFilter {
    /// `PCDM_ReaderFilter()`: no filter, append forbidden; `PcdmReaderFilter::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `PCDM_ReaderFilter(const occ::handle<Standard_Type>&)`: skips one attribute type.
    pub fn with_skipped_type(skipped: &RuntimeType) -> Self {
        let filter = Self::new();
        filter.add_skipped_type(skipped);
        filter
    }
    /// `PCDM_ReaderFilter(const TCollection_AsciiString&)`: reads one sub-tree entry.
    pub fn with_entry(entry_to_read: &AsciiString) -> Self {
        let filter = Self::new();
        filter.add_path(entry_to_read);
        filter
    }
    /// `PCDM_ReaderFilter(const AppendMode)`; `PcdmReaderFilter::with_append_mode(mode)`.
    pub fn with_append_mode(append: PcdmReaderFilterAppendMode) -> Self {
        let filter = Self::new();
        filter.state.write().append = append;
        filter
    }
    /// `AddSkipped(const occ::handle<Standard_Type>&)`: skips the type by its name.
    pub fn add_skipped_type(&self, skipped: &RuntimeType) {
        self.add_skipped(&AsciiString::new(skipped.name()));
    }
    /// `AddSkipped(const TCollection_AsciiString&)`; `filter.add_skipped(&name)`.
    pub fn add_skipped(&self, skipped: &AsciiString) {
        self.state.write().skip.add(skipped.clone());
    }
    /// `AddRead(const occ::handle<Standard_Type>&)`: reads only the listed types.
    pub fn add_read_type(&self, read: &RuntimeType) {
        self.add_read(&AsciiString::new(read.name()));
    }
    /// `AddRead(const TCollection_AsciiString&)`; `filter.add_read(&name)`.
    pub fn add_read(&self, read: &AsciiString) {
        self.state.write().read.add(read.clone());
    }
    /// `AddPath`: appends a sub-tree entry to read; `filter.add_path(&entry)`.
    pub fn add_path(&self, entry_to_read: &AsciiString) {
        self.state.write().sub_trees.push(entry_to_read.clone());
    }
    /// `Clear`: forgets the skipped, read and sub-tree lists (not the iteration tree).
    pub fn clear(&self) {
        let mut state = self.state.write();
        state.skip.clear(true);
        state.read.clear(true);
        state.sub_trees.clear();
    }
    /// `IsPassed(const occ::handle<Standard_Type>&)`: whether an attribute type is read.
    pub fn is_passed_type(&self, attribute_type: &RuntimeType) -> bool {
        self.is_passed_attr(&AsciiString::new(attribute_type.name()))
    }
    /// `IsPassedAttr`: the read list wins over the skip list when it is not empty.
    pub fn is_passed_attr(&self, attribute_type: &AsciiString) -> bool {
        let state = self.state.read();
        if state.read.is_empty() {
            return !state.skip.contains(attribute_type);
        }
        state.read.contains(attribute_type)
    }
    /// `Mode()`: the append mode (the native member returns a modifiable reference, see
    /// [`PcdmReaderFilter::set_mode`]).
    pub fn mode(&self) -> PcdmReaderFilterAppendMode {
        self.state.read().append
    }
    /// Assignment through the reference that native `Mode()` returns.
    pub fn set_mode(&self, append: PcdmReaderFilterAppendMode) {
        self.state.write().append = append;
    }
    /// `IsAppendMode`: any mode but `Forbid`.
    pub fn is_append_mode(&self) -> bool {
        self.mode() != PcdmReaderFilterAppendMode::Forbid
    }
}

#[path = "reader_filter_entries.rs"]
mod entries;
