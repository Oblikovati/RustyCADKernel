//! Native outcome of a selection-list update.
/// Result of adding/removing an owner; `AisSelectStatus::Added as i32 == 0`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AisSelectStatus {
    /// `AIS_SS_Added`.
    Added = 0,
    /// `AIS_SS_Removed`.
    Removed = 1,
    /// `AIS_SS_NotDone`.
    NotDone = 2,
}
