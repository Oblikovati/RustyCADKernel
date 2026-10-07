//! Test-only recovery of native leaked pairs without inventing a production repair API.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::BaseAllocator;
use std::{marker::PhantomData, mem::MaybeUninit, ptr::NonNull};
/// Retains real Acc cursor provenance before the next scalar-aligned pair allocation.
pub struct PairRepair<'a, N> {
    cursor: NonNull<MaybeUninit<[N; 2]>>,
    allocator: &'a BaseAllocator,
    ownership: PhantomData<N>,
}
impl<'a, N> PairRepair<'a, N> {
    /// Captures a warmed Acc cursor; validate success with `repair.matches(first_child)`.
    pub fn capture(allocator: &'a BaseAllocator) -> Self {
        let cursor = allocator.allocate_raw(0).unwrap();
        // SAFETY: this balances only the zero-byte raw cursor observation's allocation count.
        unsafe { allocator.free_raw(cursor).unwrap() };
        Self {
            // Acc allocates downward. These scalar fixtures reserve one exact pair below
            // the observed cursor; wrapping arithmetic retains the backing block provenance.
            cursor: NonNull::new(cursor.as_ptr().wrapping_sub(std::mem::size_of::<[N; 2]>()))
                .unwrap()
                .cast(),
            allocator,
            ownership: PhantomData,
        }
    }
    /// Verifies the cursor against an actual successful allocation; `repair.matches(child)`.
    pub fn matches(&self, first: &N) -> bool {
        std::ptr::eq(self.cursor.as_ptr().cast::<N>(), first)
    }
    /// Repairs only an abandoned pair with exactly `initialized` completed payloads.
    /// # Safety
    /// The warmed Acc block must have sufficient aligned room below the captured cursor.
    /// This cursor must identify the original typed pair allocated next, with no competing
    /// owner, reset or intervening allocation. Only slots below `initialized` may be alive;
    /// their destructors must not panic. Example: `unsafe { repair.reclaim(1) }`.
    pub unsafe fn reclaim(self, initialized: usize) {
        assert!(
            initialized <= 2,
            "initialized {initialized}; expected 0..=2"
        );
        // SAFETY: caller guarantees the exact abandoned allocation and excluded reset/free.
        let allocation = unsafe { self.allocator.adopt_abandoned_value(self.cursor) };
        for index in 0..initialized {
            // SAFETY: only completed, uniquely owned slots are dropped; no links are followed.
            unsafe { std::ptr::drop_in_place(self.cursor.as_ptr().cast::<N>().add(index)) };
        }
        drop(allocation);
    }
}
/// Counts completed native children for a prepublication failure; `initialized_slots("NO2")`.
pub fn initialized_slots(rejection: &str) -> usize {
    match rejection {
        "NB1" | "NO1" => 0,
        "NB2" | "NO2" => 1,
        "AB1" | "AO1" | "AO2" | "AB2" => 2,
        _ => panic!("failure {rejection}; expected unpublished child failure"),
    }
}
