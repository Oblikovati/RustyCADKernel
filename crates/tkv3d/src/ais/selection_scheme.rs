//! `AIS_SelectionScheme`: how a detection changes the current selection.

/// `AIS_SelectionScheme`: how a detection changes the current selection.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AisSelectionScheme {
    /// `AIS_SelectionScheme_UNKNOWN`: undefined.
    Unknown = -1,
    /// `AIS_SelectionScheme_Replace`: the detected objects replace the selection.
    Replace = 0,
    /// `AIS_SelectionScheme_Add`: the detected objects are added.
    Add = 1,
    /// `AIS_SelectionScheme_Remove`: the detected objects are removed.
    Remove = 2,
    /// `AIS_SelectionScheme_XOR`: the detected objects are toggled.
    Xor = 3,
    /// `AIS_SelectionScheme_Clear`: the selection is cleared.
    Clear = 4,
    /// `AIS_SelectionScheme_ReplaceExtra`: replace, keeping a selected detected element when
    /// the replacement would be empty.
    ReplaceExtra = 5,
}
