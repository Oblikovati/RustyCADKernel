//! Rust port of the OCCT TKXCAF toolkit: the extended data exchange (XDE) document.
//!
//! - [`xcaf_dim_tol_objects`]: the GD&T description objects (`XCAFDimTolObjects`).
//! - [`xcaf_note_objects`]: the note description object (`XCAFNoteObjects`).
//! - [`xcaf_view`]: the view description object (`XCAFView`).
//! - [`xcaf_doc`]: the XDE attributes and tools of a document (`XCAFDoc`).
//! - [`xcaf_app`]: the XDE application (`XCAFApp`).
//! - [`xcaf_prs`]: the presentation of XDE documents (`XCAFPrs`).
//!
//! The design rules are in `docs/xcaf-design.md`.
mod json;
pub mod xcaf_dim_tol_objects;
pub mod xcaf_doc;
pub mod xcaf_note_objects;
pub mod xcaf_prs;
pub mod xcaf_view;

/// The native enumerations of the toolkit: their enumerators in value order and the native
/// integer cast (`(Enum)value`), which the attributes need when they read a stored integer.
pub trait NativeEnum: Copy + 'static {
    /// Every enumerator in value order (`ALL[i] as i32 == i`).
    const ALL: &'static [Self];
    /// The enumerator of a value, or `None` outside the enumeration (an invalid enumerator
    /// natively).
    ///
    /// ```
    /// use rustycad_tkxcaf::NativeEnum;
    /// use rustycad_tkxcaf::xcaf_dim_tol_objects::XCAFDimTolObjectsAngularQualifier as Q;
    /// assert_eq!(Q::from_i32(2), Some(Q::Large));
    /// assert_eq!(Q::from_i32(4), None);
    /// ```
    fn from_i32(value: i32) -> Option<Self> {
        usize::try_from(value)
            .ok()
            .and_then(|index| Self::ALL.get(index).copied())
    }
}
