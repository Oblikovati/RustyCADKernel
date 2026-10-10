// Rust translation of ModelingAlgorithms/TKBool/BRepFill/BRepFill_TransitionStyle.hxx and
// BRepFill_TypeOfContact.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The transition modes of a sweep at the corners of its spine, and the contact
//! kinds of a pipe section with its spine.

/// `BRepFill_TransitionStyle`: how the sweep treats the discontinuities of the spine;
/// `TransitionStyle::Round as i32 == 2`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum TransitionStyle {
    /// `BRepFill_Modified`: the discontinuities are treated by modification of the
    /// sweeping mode.
    #[default]
    Modified = 0,
    /// `BRepFill_Right`: right corners (the surfaces are extended and intersected).
    Right = 1,
    /// `BRepFill_Round`: round corners (a pipe of revolution fills the gaps).
    Round = 2,
}

/// `BRepFill_TypeOfContact`: the contact of a section with its spine;
/// `TypeOfContact::ContactOnBorder as i32 == 2`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum TypeOfContact {
    /// `BRepFill_NoContact`: the section is placed as it is.
    #[default]
    NoContact = 0,
    /// `BRepFill_Contact`: the section is moved to touch the spine.
    Contact = 1,
    /// `BRepFill_ContactOnBorder`: the section is moved to touch the spine at its border.
    ContactOnBorder = 2,
}

#[cfg(test)]
mod tests {
    use super::{TransitionStyle, TypeOfContact};

    #[test]
    fn discriminants_follow_the_native_declaration_order() {
        assert_eq!(TransitionStyle::Modified as i32, 0);
        assert_eq!(TransitionStyle::Right as i32, 1);
        assert_eq!(TransitionStyle::Round as i32, 2);
        assert_eq!(TypeOfContact::NoContact as i32, 0);
        assert_eq!(TypeOfContact::Contact as i32, 1);
        assert_eq!(TypeOfContact::ContactOnBorder as i32, 2);
    }
}
