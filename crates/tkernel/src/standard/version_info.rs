// Rust translation of FoundationClasses/TKernel/Standard/Standard_VersionInfo.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Standard_VersionInfo: the version of OCCT this crate ports.
//!
//! The C++ values come from the generated `Standard_Version.hxx`; CMake sets
//! `OCC_VERSION_DEVELOPMENT` to `dev1-<short commit>` for the pinned development build, so
//! the port returns the values of the pinned revision 3d097a0328.

/// `OCC_VERSION_DEVELOPMENT` of the pinned build.
const DEVELOPMENT: &str = "dev1-3d097a0328";

/// `OCCT_DevelopmentVersion()`: the development tag ("" for an official release).
///
/// ```
/// assert!(rustycad_tkernel::standard::version_info::occt_development_version().starts_with("dev"));
/// ```
pub const fn occt_development_version() -> &'static str {
    DEVELOPMENT
}

/// `OCCT_Version_Double()`: `OCC_VERSION`, "major.minor" as a double.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::version_info::occt_version_double(), 8.1);
/// ```
pub const fn occt_version_double() -> f64 {
    8.1
}

/// `OCCT_Version_String()`: `OCC_VERSION_STRING`, "major.minor".
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::version_info::occt_version_string(), "8.1");
/// ```
pub const fn occt_version_string() -> &'static str {
    "8.1"
}

/// `OCCT_Version_String_Complete()`: `OCC_VERSION_COMPLETE`, "major.minor.maintenance".
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::version_info::occt_version_string_complete(), "8.1.0");
/// ```
pub const fn occt_version_string_complete() -> &'static str {
    "8.1.0"
}

/// `OCCT_Version_String_Extended()`: `OCC_VERSION_STRING_EXT`, the complete version and the
/// development tag joined by a dot.
///
/// ```
/// use rustycad_tkernel::standard::version_info::occt_version_string_extended;
/// assert_eq!(occt_version_string_extended(), "8.1.0.dev1-3d097a0328");
/// ```
pub const fn occt_version_string_extended() -> &'static str {
    "8.1.0.dev1-3d097a0328"
}
