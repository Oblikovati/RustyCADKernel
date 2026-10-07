//! The SelectMgr enumerations, with their native discriminants.

/// `SelectMgr_SelectionType`: the kind of selecting volume.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SelectMgrSelectionType {
    /// `SelectMgr_SelectionType_Unknown`: undefined.
    Unknown = -1,
    /// `SelectMgr_SelectionType_Point`: by point (a frustum with a tolerance or an axis).
    Point = 0,
    /// `SelectMgr_SelectionType_Box`: by rectangle.
    Box = 1,
    /// `SelectMgr_SelectionType_Polyline`: by polygon.
    Polyline = 2,
}

/// `SelectMgr_FilterType`: how a composition filter combines its filters.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SelectMgrFilterType {
    /// `SelectMgr_FilterType_AND`: every filter must accept.
    And = 0,
    /// `SelectMgr_FilterType_OR`: one filter must accept.
    Or = 1,
}

/// `SelectMgr_PickingStrategy`: which detected entity is picked.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SelectMgrPickingStrategy {
    /// `SelectMgr_PickingStrategy_FirstAcceptable`: the first one that passes the filter.
    FirstAcceptable = 0,
    /// `SelectMgr_PickingStrategy_OnlyTopmost`: only the topmost one, when it passes.
    OnlyTopmost = 1,
}

/// `SelectMgr_TypeOfDepthTolerance`: how entities at nearly the same depth compare.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SelectMgrTypeOfDepthTolerance {
    /// `SelectMgr_TypeOfDepthTolerance_Uniform`: a tolerance in world units.
    Uniform = 0,
    /// `SelectMgr_TypeOfDepthTolerance_UniformPixels`: a tolerance in pixels.
    UniformPixels = 1,
    /// `SelectMgr_TypeOfDepthTolerance_SensitivityFactor`: the entity's sensitivity in pixels.
    SensitivityFactor = 2,
}

/// `SelectMgr_StateOfSelection`: the activation state of a selection.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SelectMgrStateOfSelection {
    /// `SelectMgr_SOS_Any`: any state (queries only).
    Any = -2,
    /// `SelectMgr_SOS_Unknown`: never activated.
    Unknown = -1,
    /// `SelectMgr_SOS_Deactivated`: deactivated after an activation.
    Deactivated = 0,
    /// `SelectMgr_SOS_Activated`: activated.
    Activated = 1,
}

/// `SelectMgr_TypeOfUpdate`: the pending update of a selection.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SelectMgrTypeOfUpdate {
    /// `SelectMgr_TOU_Full`: recompute the sensitive entities.
    Full = 0,
    /// `SelectMgr_TOU_Partial`: update the location only.
    Partial = 1,
    /// `SelectMgr_TOU_None`: nothing to do.
    None = 2,
}

/// `SelectMgr_TypeOfBVHUpdate`: the pending update of a selection's BVH.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SelectMgrTypeOfBvhUpdate {
    /// `SelectMgr_TBU_Add`.
    Add = 0,
    /// `SelectMgr_TBU_Remove`.
    Remove = 1,
    /// `SelectMgr_TBU_Renew`.
    Renew = 2,
    /// `SelectMgr_TBU_Invalidate`.
    Invalidate = 3,
    /// `SelectMgr_TBU_None`.
    None = 4,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discriminants_match_the_native_values() {
        assert_eq!(SelectMgrSelectionType::Unknown as i32, -1);
        assert_eq!(SelectMgrSelectionType::Polyline as i32, 2);
        assert_eq!(SelectMgrStateOfSelection::Any as i32, -2);
        assert_eq!(SelectMgrStateOfSelection::Activated as i32, 1);
        assert_eq!(SelectMgrTypeOfBvhUpdate::None as i32, 4);
        assert_eq!(SelectMgrTypeOfUpdate::None as i32, 2);
        assert_eq!(SelectMgrTypeOfDepthTolerance::SensitivityFactor as i32, 2);
        assert_eq!(SelectMgrPickingStrategy::OnlyTopmost as i32, 1);
        assert_eq!(SelectMgrFilterType::Or as i32, 1);
    }
}
