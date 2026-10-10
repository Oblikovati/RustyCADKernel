//! Native detection outcomes, including values retained for compatibility.
/// Result of a mouse or axis detection; `AisStatusOfDetection::Nothing as i32 == 1`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AisStatusOfDetection {
    /// `AIS_SOD_Error`.
    Error = 0,
    /// `AIS_SOD_Nothing`.
    Nothing = 1,
    /// `AIS_SOD_AllBad`.
    AllBad = 2,
    /// `AIS_SOD_Selected`.
    Selected = 3,
    /// `AIS_SOD_OnlyOneDetected`.
    OnlyOneDetected = 4,
    /// `AIS_SOD_OnlyOneGood`.
    OnlyOneGood = 5,
    /// `AIS_SOD_SeveralGood`.
    SeveralGood = 6,
}
