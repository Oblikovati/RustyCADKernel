//! Native outcomes of interactive-context selection actions.
/// Selection result; `AisStatusOfPick::NothingSelected as i32 == 1`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AisStatusOfPick {
    /// `AIS_SOP_Error`.
    Error = 0,
    /// `AIS_SOP_NothingSelected`.
    NothingSelected = 1,
    /// `AIS_SOP_Removed`.
    Removed = 2,
    /// `AIS_SOP_OneSelected`.
    OneSelected = 3,
    /// `AIS_SOP_SeveralSelected`.
    SeveralSelected = 4,
}
