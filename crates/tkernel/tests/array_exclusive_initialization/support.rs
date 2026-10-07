use rustycad_tkernel::ncollection::{Array1, Array2};
use std::cell::Cell;
use std::mem::MaybeUninit;

pub struct DropCell<'a> {
    pub value: usize,
    pub deaths: &'a Cell<usize>,
}

impl Drop for DropCell<'_> {
    fn drop(&mut self) {
        let previous = self.deaths.get();
        self.deaths.set(previous + 1);
    }
}

pub fn initialize_slots<'a>(slots: &mut [MaybeUninit<DropCell<'a>>], deaths: &'a Cell<usize>) {
    for (value, slot) in slots.iter_mut().enumerate() {
        slot.write(DropCell { value, deaths });
    }
}

#[allow(unsafe_code)]
pub unsafe fn destroy_slots(slots: &mut [MaybeUninit<DropCell<'_>>]) {
    for slot in slots {
        unsafe { slot.assume_init_drop() };
    }
}

pub fn check_linear(values: &Array1<'_, DropCell<'_>>, pointer: *const DropCell<'_>, owned: bool) {
    assert_eq!(values.data_ptr(), pointer);
    assert_eq!(values.lower(), -2);
    assert_eq!(values.is_deletable(), owned);
    assert_eq!(
        values.iter().map(|cell| cell.value).collect::<Vec<_>>(),
        [0, 1, 2]
    );
}

pub fn rectangle<T>(values: &Array2<'_, T>) -> (i32, i32, usize, usize, i32) {
    (
        values.lower_row(),
        values.lower_col(),
        values.nb_rows(),
        values.nb_columns(),
        values.array1().lower(),
    )
}

#[allow(unsafe_code)]
pub fn changed_matrix_case(length: usize) {
    let deaths = Cell::new(0);
    let mut slots = Array2::<DropCell<'_>>::with_uninit_bounds(-2, -1, 3, 4).unwrap();
    *slots.change_array1() = Array1::<DropCell<'_>>::with_uninit_size(length);
    slots.change_array1().update_lower_bound(17);
    initialize_slots(slots.as_mut_slice(), &deaths);
    let (pointer, bounds) = (slots.data_ptr().cast::<DropCell<'_>>(), rectangle(&slots));
    let values = unsafe { slots.try_assume_init_exclusive() }.unwrap_or_else(|_| panic!("shared"));
    assert_eq!((values.data_ptr(), rectangle(&values)), (pointer, bounds));
    assert_eq!(values.len(), length);
    assert_eq!(deaths.get(), 0);
    drop(values);
    assert_eq!(deaths.get(), length);
}

#[allow(unsafe_code)]
pub fn copy_source_slots<'a>(
    source: &[DropCell<'a>],
    copies: &Cell<usize>,
) -> Array1<'a, MaybeUninit<DropCell<'a>>> {
    assert_eq!(source.len(), 3, "expected exactly three donor slots");
    let mut slots = Array1::<DropCell<'a>>::with_uninit_bounds(-2, 0).unwrap();
    unsafe {
        Array1::copy_construct_nontrivial_raw(slots.data_mut_ptr(), source.as_ptr(), 3, |cell| {
            copies.set(copies.get() + 1);
            DropCell {
                value: cell.value,
                deaths: cell.deaths,
            }
        });
    }
    slots
}
