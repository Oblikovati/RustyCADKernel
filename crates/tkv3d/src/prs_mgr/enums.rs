//! The PrsMgr enumerations with their native discriminants.

/// `PrsMgr_DisplayStatus`: where an interactive object is displayed.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrsMgrDisplayStatus {
    /// `PrsMgr_DisplayStatus_Displayed`: displayed in the main viewer.
    Displayed = 0,
    /// `PrsMgr_DisplayStatus_Erased`: hidden in the main viewer.
    Erased = 1,
    /// `PrsMgr_DisplayStatus_None`: displayed nowhere.
    None = 2,
}

impl PrsMgrDisplayStatus {
    /// `AIS_DS_Displayed`, the legacy alias of [`PrsMgrDisplayStatus::Displayed`].
    pub const AIS_DS_DISPLAYED: Self = Self::Displayed;
    /// `AIS_DS_Erased`, the legacy alias of [`PrsMgrDisplayStatus::Erased`].
    pub const AIS_DS_ERASED: Self = Self::Erased;
    /// `AIS_DS_None`, the legacy alias of [`PrsMgrDisplayStatus::None`].
    pub const AIS_DS_NONE: Self = Self::None;
}

/// `PrsMgr_TypeOfPresentation3d`: whether the presentation depends on the view projector.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrsMgrTypeOfPresentation3d {
    /// `PrsMgr_TOP_AllView`: one presentation for every view.
    AllView = 0,
    /// `PrsMgr_TOP_ProjectorDependent`: computed per view projector (hidden lines).
    ProjectorDependent = 1,
}

/// `PrsMgr_TOP_ProjectorDependant`, the deprecated spelling of
/// [`PrsMgrTypeOfPresentation3d::ProjectorDependent`].
pub const PRS_MGR_TOP_PROJECTOR_DEPENDANT: PrsMgrTypeOfPresentation3d =
    PrsMgrTypeOfPresentation3d::ProjectorDependent;
