// Rust translation of Blend_Status.hxx and Blend_DecrochStatus.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The status of one step of a blend walking and the unhooking of a restriction walk.

/// The result of one step of a walking (`Blend_Status`); `Status::Ok as i32 == 7`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Status {
    /// The step is too large for the deflection.
    StepTooLarge = 0,
    /// The step is below the minimal step.
    StepTooSmall = 1,
    /// The walking goes backward.
    Backward = 2,
    /// Both extremities of the section coincide.
    SamePoints = 3,
    /// The section reaches the first restriction.
    OnRst1 = 4,
    /// The section reaches the second restriction.
    OnRst2 = 5,
    /// The section reaches both restrictions.
    OnRst12 = 6,
    /// The step succeeded.
    Ok = 7,
}

/// Which restriction a section leaves (`Blend_DecrochStatus`);
/// `DecrochStatus::DecrochBoth as i32 == 3`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DecrochStatus {
    /// The section stays on both restrictions.
    NoDecroch = 0,
    /// The section leaves the first restriction.
    DecrochRst1 = 1,
    /// The section leaves the second restriction.
    DecrochRst2 = 2,
    /// The section leaves both restrictions.
    DecrochBoth = 3,
}
