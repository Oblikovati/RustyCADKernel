//! Existing safe reserve retains Rust cleanup; the opt-in token owns native lifetime holes.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::LinearVector;
use std::{
    cell::Cell,
    panic::{AssertUnwindSafe, catch_unwind},
};
struct SafeDrop<'a> {
    value: i32,
    drops: &'a Cell<usize>,
    failing: &'a Cell<bool>,
}
impl Drop for SafeDrop<'_> {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
        if self.value == 11 && self.failing.replace(false) {
            panic!("drop value 11: injected failure");
        }
    }
}
#[test]
fn ordinary_safe_reserve_keeps_its_sound_cleanup_and_published_replacement() {
    let drops = Cell::new(0);
    let failing = Cell::new(true);
    let mut vector = LinearVector::with_capacity(4);
    for value in 10..13 {
        vector.append(SafeDrop {
            value,
            drops: &drops,
            failing: &failing,
        });
    }
    let before = vector.data_ptr();
    assert!(
        catch_unwind(AssertUnwindSafe(|| vector.reserve_with_relocation(
            5,
            |item| SafeDrop {
                value: item.value,
                drops: &drops,
                failing: &failing
            }
        )))
        .is_err()
    );
    assert_eq!((vector.len(), vector.capacity(), drops.get()), (3, 8, 3));
    assert_ne!(vector.data_ptr(), before);
    assert!(vector.iter().map(|item| item.value).eq([10, 11, 12]));
    drop(vector);
    assert_eq!(drops.get(), 6);
}
#[test]
fn successful_token_transfers_the_same_live_values_back_to_a_vector() {
    let mut vector = LinearVector::with_capacity(2);
    vector.append(Box::new(17));
    vector.append(Box::new(23));
    let identities: Vec<*const i32> = vector
        .iter()
        .map(|item| std::ptr::from_ref(&**item))
        .collect();
    let mut token = vector.into_growth_token();
    // SAFETY: no live element/payload references are retained, only raw identity addresses.
    unsafe { token.reserve(5, |item| std::mem::replace(item, Box::new(0))) };
    assert_eq!((token.progress().size, token.progress().capacity), (2, 5));
    assert!(token.progress().dead_slots.is_empty());
    let vector = token.try_into_vector().ok().unwrap();
    assert!(
        vector
            .iter()
            .map(|item| std::ptr::from_ref(&**item))
            .eq(identities)
    );
}
