//! ChFi3d.cxx orientation propagation at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use rustycad_tkg3d::top_abs::{Orientation, reverse};

/// Propagate two saved sides and return the next fillet choice; `next_side(&mut a,&mut b,sa,sb,2)`.
pub fn next_side(
    first: &mut Orientation,
    second: &mut Orientation,
    saved_first: Orientation,
    saved_second: Orientation,
    saved_choice: i32,
) -> i32 {
    *first = relative_side(*first, saved_first, Orientation::Forward);
    *second = relative_side(*second, saved_second, Orientation::Forward);
    let choice = match (
        *first == Orientation::Forward,
        *second == Orientation::Forward,
    ) {
        (true, true) => 1,
        (false, false) => 5,
        (true, false) => {
            if saved_choice < 0 {
                3
            } else {
                7
            }
        }
        (false, true) => {
            if saved_choice < 0 {
                7
            } else {
                3
            }
        }
    };
    // Remainder preserves native abs(choice)%2 without abs overflow for i32::MIN.
    choice + i32::from(saved_choice % 2 == 0)
}
/// Propagate one saved side relative to its face; `next_side_on_face(&mut side,saved,face)`.
pub fn next_side_on_face(side: &mut Orientation, saved: Orientation, face: Orientation) {
    *side = relative_side(*side, saved, face);
}
/// Compare two saved sides after their face orientations; `same_side(side,sa,sb,fa,fb)`.
pub fn same_side(
    side: Orientation,
    saved_first: Orientation,
    saved_second: Orientation,
    first_face: Orientation,
    second_face: Orientation,
) -> bool {
    relative_side(side, saved_first, first_face) == relative_side(side, saved_second, second_face)
}
fn relative_side(side: Orientation, saved: Orientation, face: Orientation) -> Orientation {
    if side == face { saved } else { reverse(saved) }
}
