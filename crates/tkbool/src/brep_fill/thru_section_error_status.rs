// Rust translation of ModelingAlgorithms/TKBool/BRepFill/BRepFill_ThruSectionErrorStatus.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFill_ThruSectionErrorStatus`: the outcome of a thru-section construction.

/// The errors that can occur in the thru-section algorithm, with the native
/// discriminants; `ThruSectionErrorStatus::NotDone as i32 == 1`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ThruSectionErrorStatus {
    /// `BRepFill_ThruSectionErrorStatus_Done`: the algorithm is done.
    Done = 0,
    /// `BRepFill_ThruSectionErrorStatus_NotDone`: the algorithm is not done.
    #[default]
    NotDone = 1,
    /// `BRepFill_ThruSectionErrorStatus_NotSameTopology`: the profiles are not all
    /// closed or all open.
    NotSameTopology = 2,
    /// `BRepFill_ThruSectionErrorStatus_ProfilesInconsistent`: the profiles are
    /// inconsistent.
    ProfilesInconsistent = 3,
    /// `BRepFill_ThruSectionErrorStatus_WrongUsage`: wrong usage of punctual sections.
    WrongUsage = 4,
    /// `BRepFill_ThruSectionErrorStatus_Null3DCurve`: an edge has no 3d curve.
    Null3DCurve = 5,
    /// `BRepFill_ThruSectionErrorStatus_Failed`: the algorithm has failed.
    Failed = 6,
}

#[cfg(test)]
mod tests {
    use super::ThruSectionErrorStatus;

    #[test]
    fn discriminants_follow_the_native_declaration_order() {
        assert_eq!(ThruSectionErrorStatus::Done as i32, 0);
        assert_eq!(ThruSectionErrorStatus::NotDone as i32, 1);
        assert_eq!(ThruSectionErrorStatus::Null3DCurve as i32, 5);
        assert_eq!(ThruSectionErrorStatus::Failed as i32, 6);
        assert_eq!(
            ThruSectionErrorStatus::default(),
            ThruSectionErrorStatus::NotDone
        );
    }
}
