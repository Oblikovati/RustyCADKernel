//! Verifies safe borrowed Mat3/Mat4 views preserve native buffer aliasing and layout.

use rustycad_tkernel::ncollection::{Mat3, Mat4};
use std::cell::Cell;

#[test]
fn mat3_const_view_tracks_interior_mutation() {
    let values: [Cell<i32>; 9] = std::array::from_fn(|index| Cell::new(index as i32));
    let backing = values.as_ptr();
    let view = Mat3::map(&values);
    assert_eq!(view.as_slice().as_ptr(), backing);
    assert_eq!(view.as_slice().len(), 9);
    assert_eq!(view.element(1, 2).get(), 7);
    values[7].set(41);
    assert_eq!(view.element(1, 2).get(), 41);
}

#[test]
fn mat3_mut_view_writes_column_major_storage() {
    let mut values = std::array::from_fn::<_, 9, _>(|index| index as i32);
    let backing = values.as_ptr();
    let mut view = Mat3::map_mut(&mut values);
    assert_eq!(view.as_slice().as_ptr(), backing);
    assert_eq!(view.get_value(1, 2), 7);
    view.set_value(1, 2, 41);
    assert_eq!(view.as_slice()[7], 41);
    assert_eq!(view.as_view().get_value(1, 2), 41);
}

#[test]
fn mat4_const_view_tracks_interior_mutation() {
    let values: [Cell<i32>; 16] = std::array::from_fn(|index| Cell::new(index as i32));
    let backing = values.as_ptr();
    let view = Mat4::map(&values);
    assert_eq!(view.as_slice().as_ptr(), backing);
    assert_eq!(view.as_slice().len(), 16);
    assert_eq!(view.element(1, 2).get(), 9);
    values[9].set(52);
    assert_eq!(view.element(1, 2).get(), 52);
}

#[test]
fn mat4_mut_view_writes_column_major_storage() {
    let mut values = std::array::from_fn::<_, 16, _>(|index| index as i32);
    let backing = values.as_ptr();
    let mut view = Mat4::map_mut(&mut values);
    assert_eq!(view.as_slice().as_ptr(), backing);
    assert_eq!(view.get_value(1, 2), 9);
    view.set_value(1, 2, 52);
    assert_eq!(view.as_slice()[9], 52);
    assert_eq!(view.as_view().get_value(1, 2), 52);
    view.as_mut_slice()[10] = 74;
    assert_eq!(view.get_value(2, 2), 74);
}
