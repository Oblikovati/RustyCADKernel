//! Actual allocator-api2 layouts exercise the same typed reservations as canonical map nodes.
use allocator_api2::alloc::{Allocator, Layout};
use rustycad_tkernel::ncollection::BaseAllocator;
use std::{fmt::Write, ptr::NonNull};

pub struct ReservationTrace {
    pub arena: BaseAllocator,
    row: String,
    owned: Vec<(NonNull<u8>, Layout)>,
}
impl ReservationTrace {
    pub fn new(label: &str, block_size: usize) -> Self {
        Self {
            arena: BaseAllocator::accumulating(block_size).unwrap(),
            row: label.into(),
            owned: Vec::new(),
        }
    }
    pub fn request(&mut self, size: usize, align: usize) {
        let layout = Layout::from_size_align(size, align).unwrap();
        let pointer = Allocator::allocate(&self.arena, layout)
            .unwrap()
            .cast::<u8>();
        assert_eq!(pointer.as_ptr().addr() % align, 0);
        // SAFETY: the layout owns exactly size writable bytes; distinct reservations may not overlap.
        unsafe { pointer.write_bytes(self.owned.len() as u8, size) };
        self.owned.push((pointer, layout));
        self.observe(pointer);
    }
    fn observe(&mut self, pointer: NonNull<u8>) {
        let address = pointer.as_ptr().addr();
        let state = self.arena.accumulating_block(address).unwrap();
        write!(
            self.row,
            " {}:{}:{}:{}",
            address - state.address,
            state.free_size,
            state.alloc_count,
            self.arena.accumulating_chain().len()
        )
        .unwrap();
    }
    fn release_all(&mut self) {
        for (index, (pointer, layout)) in self.owned.drain(..).enumerate() {
            // SAFETY: every initialized reservation remains owned until this loop releases it.
            let bytes = unsafe { std::slice::from_raw_parts(pointer.as_ptr(), layout.size()) };
            assert!(bytes.iter().all(|&value| value == index as u8));
            // SAFETY: matching original pointer/layout released exactly once via originating owner.
            unsafe { Allocator::deallocate(&self.arena, pointer, layout) };
        }
    }
    fn observe_empty(&mut self) {
        let chain = self.arena.accumulating_chain();
        write!(
            self.row,
            " empty {}:{}:{}",
            chain[0].free_size,
            chain[0].alloc_count,
            chain.len()
        )
        .unwrap();
    }
    pub fn assert_native(mut self) {
        self.release_all();
        self.observe_empty();
        let expected = include_str!(
            "../../../../tools/oracle/tkernel_acc_allocator_aligned_reservation.expected"
        );
        let label = self.row.split_whitespace().next().unwrap();
        let native = expected
            .lines()
            .find(|row| row.split_whitespace().next() == Some(label));
        assert_eq!(
            Some(self.row.as_str()),
            native,
            "native allocation row {label}"
        );
    }
}
