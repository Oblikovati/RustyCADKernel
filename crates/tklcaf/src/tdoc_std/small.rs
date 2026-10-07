// Rust port of ApplicationFramework/TKLCAF/TDocStd/TDocStd_FormatVersion.hxx,
// TDocStd_PathParser (.hxx, .cxx), TDocStd_Context (.hxx, .cxx) and TDocStd_CompoundDelta
// (.hxx, .cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The small TDocStd classes: format versions, path parsing, update context and compound
//! deltas.
use crate::tdf::TdfDelta;
use rustycad_tkernel::{impl_transient, tcollection::ExtendedString};

/// `TDocStd_FormatVersion`: the OCAF storage format versions; `value as i32` is native.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum TDocStdFormatVersion {
    /// `TDocStd_FormatVersion_VERSION_2`: first supported version.
    Version2 = 2,
    /// `TDocStd_FormatVersion_VERSION_3`: OCCT 6.3.0.
    Version3,
    /// `TDocStd_FormatVersion_VERSION_4`: OCCT 6.3.1.
    Version4,
    /// `TDocStd_FormatVersion_VERSION_5`: OCCT 6.3.1.
    Version5,
    /// `TDocStd_FormatVersion_VERSION_6`: OCCT 6.5.0.
    Version6,
    /// `TDocStd_FormatVersion_VERSION_7`: OCCT 6.7.0.
    Version7,
    /// `TDocStd_FormatVersion_VERSION_8`: OCCT 7.0.0.
    Version8,
    /// `TDocStd_FormatVersion_VERSION_9`: OCCT 7.1.0.
    Version9,
    /// `TDocStd_FormatVersion_VERSION_10`: OCCT 7.2.0.
    Version10,
    /// `TDocStd_FormatVersion_VERSION_11`: OCCT 7.6.0.
    Version11,
    /// `TDocStd_FormatVersion_VERSION_12`: OCCT 7.6.0.
    Version12,
}
impl TDocStdFormatVersion {
    /// `TDocStd_FormatVersion_CURRENT` (= VERSION_12).
    pub const CURRENT: Self = Self::Version12;
}
/// `TDocStd_FormatVersion_LOWER` (= VERSION_2).
pub const TDOCSTD_FORMAT_VERSION_LOWER: i32 = TDocStdFormatVersion::Version2 as i32;
/// `TDocStd_FormatVersion_UPPER` (= VERSION_12).
pub const TDOCSTD_FORMAT_VERSION_UPPER: i32 = TDocStdFormatVersion::Version12 as i32;

/// `TDocStd_PathParser`: splits a path into trek, name and extension.
///
/// ```
/// use rustycad_tklcaf::tdoc_std::TDocStdPathParser;
/// use rustycad_tkernel::tcollection::ExtendedString;
/// let parser = TDocStdPathParser::new(&ExtendedString::new("/tmp/doc.cbf"));
/// assert_eq!(parser.trek(), ExtendedString::new("/tmp"));
/// assert_eq!(parser.name(), ExtendedString::new("doc"));
/// assert_eq!(parser.extension(), ExtendedString::new("cbf"));
/// ```
#[derive(Clone, Debug, Default)]
pub struct TDocStdPathParser {
    trek: ExtendedString,
    name: ExtendedString,
    extension: ExtendedString,
    path: ExtendedString,
}
impl TDocStdPathParser {
    /// `TDocStd_PathParser(path)`: parses at once.
    pub fn new(path: &ExtendedString) -> Self {
        let mut parser = Self {
            path: path.clone(),
            ..Self::default()
        };
        parser.parse();
        parser
    }
    /// `Parse`: the extension after the last dot, the name after the last separator and the
    /// trek before it (`./` without separator, `.\` on Windows); nothing without a dot.
    pub fn parse(&mut self) {
        let mut temp = self.path.clone();
        let dot = self.path.search_from_end(ExtendedString::new("."));
        if dot <= 0 {
            return;
        }
        let Ok(extension) = temp.split(dot) else {
            return;
        };
        self.extension = extension;
        if temp.trunc(dot - 1).is_err() {
            return;
        }
        let is_file_name = temp.length() != 0;
        let mut separator = -1;
        if cfg!(windows) {
            separator = temp.search_from_end(ExtendedString::new("\\"));
        }
        if separator <= 0 {
            separator = temp.search_from_end(ExtendedString::new("/"));
        }
        self.split_name(temp, separator, is_file_name);
    }
    /// The name/trek part of `Parse`.
    fn split_name(&mut self, mut temp: ExtendedString, separator: i32, is_file_name: bool) {
        if separator <= 0 {
            if !is_file_name {
                return;
            }
            self.name = temp;
            self.trek = ExtendedString::new(if cfg!(windows) { ".\\" } else { "./" });
            return;
        }
        if let Ok(name) = temp.split(separator) {
            self.name = name;
        }
        if temp.trunc(separator - 1).is_ok() {
            self.trek = temp;
        }
    }
    /// `Extension`.
    pub fn extension(&self) -> ExtendedString {
        self.extension.clone()
    }
    /// `Name`.
    pub fn name(&self) -> ExtendedString {
        self.name.clone()
    }
    /// `Trek`.
    pub fn trek(&self) -> ExtendedString {
        self.trek.clone()
    }
    /// `Path`.
    pub fn path(&self) -> ExtendedString {
        self.path.clone()
    }
    /// `Length`: of the whole path.
    pub fn length(&self) -> i32 {
        self.path.length()
    }
}

/// `TDocStd_Context`: the update context of `TDocStd_Document::Update`.
#[derive(Clone, Copy, Debug, Default)]
pub struct TDocStdContext {
    modified_references: bool,
}
impl TDocStdContext {
    /// `TDocStd_Context()`: no modified references.
    pub fn new() -> Self {
        Self::default()
    }
    /// `SetModifiedReferences`.
    pub fn set_modified_references(&mut self, modified: bool) {
        self.modified_references = modified;
    }
    /// `ModifiedReferences`.
    pub fn modified_references(&self) -> bool {
        self.modified_references
    }
}

/// `TDocStd_CompoundDelta`: a `TDF_Delta` that gathers the deltas of nested transactions;
/// its handle casts to `Handle<TdfDelta>` through the embedded delta.
#[derive(Default)]
pub struct TDocStdCompoundDelta {
    delta: TdfDelta,
}
impl_transient!(
    TDocStdCompoundDelta,
    "TDocStd_CompoundDelta",
    TdfDelta,
    delta
);
impl TDocStdCompoundDelta {
    /// `TDocStd_CompoundDelta()`: an empty delta.
    pub fn new() -> Self {
        Self::default()
    }
    /// The embedded `TDF_Delta` (the native base class).
    pub fn delta(&self) -> &TdfDelta {
        &self.delta
    }
}
