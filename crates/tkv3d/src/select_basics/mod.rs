//! The SelectBasics package: the pick result and the abstract selecting volume that the
//! Select3D sensitive entities test themselves against.
mod pick_result;
mod selecting_volume_manager;

pub use pick_result::SelectBasicsPickResult;
pub use selecting_volume_manager::{
    SelectBasicsSelectingVolumeManager, SelectBasicsSelectionTypeAliases,
};

/// The package class `SelectBasics`: the owner priority range.
pub struct SelectBasics;

impl SelectBasics {
    /// `MaxOwnerPriority`: 9; `SelectBasics::max_owner_priority()`.
    pub fn max_owner_priority() -> i32 {
        9
    }
    /// `MinOwnerPriority`: 0; `SelectBasics::min_owner_priority()`.
    pub fn min_owner_priority() -> i32 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_priority_range_is_zero_to_nine() {
        assert_eq!(
            (
                SelectBasics::min_owner_priority(),
                SelectBasics::max_owner_priority()
            ),
            (0, 9)
        );
    }
}
