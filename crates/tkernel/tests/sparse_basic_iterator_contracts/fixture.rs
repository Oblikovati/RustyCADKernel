#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::{
    BasicSparseArraySource, BasicSparseArraySourceMut, SparseArrayBase, SparseArrayBlock,
};
use std::{cell::Cell, ffi::c_void};
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ProjectedPair {
    pub marker: i32,
    pub value: i32,
}
#[repr(C)]
pub struct CustomSparse {
    pub prefix: u64,
    pub base: SparseArrayBase,
    pub pointers: Cell<usize>,
    pub capacities: Cell<usize>,
    pub end_override: Cell<Option<usize>>,
}
impl CustomSparse {
    pub fn new(values: &[(usize, i32)]) -> Self {
        let source = Self {
            prefix: 42,
            base: SparseArrayBase::new(size_of::<ProjectedPair>(), 4, discard),
            pointers: Cell::new(0),
            capacities: Cell::new(0),
            end_override: Cell::new(None),
        };
        for &(index, value) in values {
            source.put(index, value);
        }
        source
    }
    pub fn put(&self, index: usize, value: i32) {
        let mut pair = ProjectedPair {
            marker: 100 + index as i32,
            value,
        };
        unsafe {
            self.base.set_value(
                index,
                std::ptr::from_mut(&mut pair).cast(),
                copy_pair,
                copy_pair,
            );
        }
    }
}
unsafe fn discard(_: *mut c_void) {}
unsafe fn copy_pair(target: *mut c_void, source: *mut c_void) {
    unsafe {
        target
            .cast::<ProjectedPair>()
            .write(*source.cast::<ProjectedPair>());
    }
}
// SAFETY: stable raw storage owns initialized pairs; projection identifies their aligned value fields.
unsafe impl BasicSparseArraySource<i32> for CustomSparse {
    fn sparse_storage(&self) -> &SparseArrayBase {
        &self.base
    }
    fn sparse_capacity(&self) -> usize {
        self.capacities.set(self.capacities.get() + 1);
        self.end_override
            .get()
            .unwrap_or_else(|| self.base.capacity())
    }
    unsafe fn sparse_item(&self, block: SparseArrayBlock, offset: usize) -> *mut i32 {
        self.pointers.set(self.pointers.get() + 1);
        let pair = unsafe { self.base.get_item(block, offset).cast::<ProjectedPair>() };
        unsafe { std::ptr::addr_of_mut!((*pair).value) }
    }
}
// SAFETY: values are inside raw interior storage and writable under the iterator's exclusive access contract.
unsafe impl BasicSparseArraySourceMut<i32> for CustomSparse {}
pub fn oracle_row(actual: &str) {
    let rows = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tools/oracle/tkernel_sparse_basic_iterator_contracts.expected"
    ));
    assert!(
        rows.lines().any(|row| row == actual),
        "row {actual:?}; expected exact native output"
    );
}

pub struct PlainSparse {
    pub base: SparseArrayBase,
}
impl PlainSparse {
    pub fn new() -> Self {
        Self {
            base: SparseArrayBase::new(size_of::<i32>(), 4, discard),
        }
    }
    pub fn put(&self, index: usize, mut value: i32) {
        unsafe {
            self.base.set_value(
                index,
                std::ptr::from_mut(&mut value).cast(),
                copy_integer,
                copy_integer,
            );
        }
    }
}
unsafe fn copy_integer(target: *mut c_void, source: *mut c_void) {
    unsafe {
        target.cast::<i32>().write(*source.cast::<i32>());
    }
}
// SAFETY: every occupied raw slot contains an initialized aligned i32.
unsafe impl BasicSparseArraySource<i32> for PlainSparse {
    fn sparse_storage(&self) -> &SparseArrayBase {
        &self.base
    }
}
// SAFETY: erased raw i32 objects are writable when the caller excludes aliases.
unsafe impl BasicSparseArraySourceMut<i32> for PlainSparse {}

pub type MutableCursor<'a> =
    rustycad_tkernel::ncollection::BaseSparseArrayBasicIterator<'a, CustomSparse, i32, true>;
pub type ConstCursor<'a> =
    rustycad_tkernel::ncollection::BaseSparseArrayBasicIterator<'a, CustomSparse, i32>;
pub fn observe_restart(cursor: &mut MutableCursor<'_>, source: &CustomSparse) {
    unsafe {
        cursor.advance();
        cursor.restart();
    }
    oracle_row(&format!(
        "restart {} {}",
        cursor.key(),
        source.capacities.get()
    ));
}
pub fn observe_ends(
    mutable: &MutableCursor<'_>,
    readonly: &ConstCursor<'_>,
    end: &MutableCursor<'_>,
) {
    let detached = ConstCursor::default();
    oracle_row(&format!(
        "ends {} {} {} {} {}",
        u8::from(*mutable == *readonly),
        end.key(),
        u8::from(*end == detached),
        u8::from(detached.more()),
        detached.index()
    ));
}
pub fn observe_short_end(cursor: &ConstCursor<'_>) {
    oracle_row(&format!(
        "short_end {} {} {}",
        u8::from(cursor.more()),
        cursor.key(),
        unsafe { *cursor.value() }
    ));
}
pub fn observe_short_exhaustion(cursor: &mut ConstCursor<'_>) {
    unsafe {
        cursor.advance();
    }
    oracle_row(&format!(
        "short_end_exhausted {} {}",
        u8::from(cursor.more()),
        cursor.key()
    ));
}

pub struct NoncloneValue(pub i32);
pub struct NoncloneSource {
    base: SparseArrayBase,
}
impl NoncloneSource {
    pub fn new() -> Self {
        let base = SparseArrayBase::new(size_of::<NoncloneValue>(), 4, destroy_nonclone);
        let mut original = NoncloneValue(7);
        unsafe {
            base.set_value(
                1,
                std::ptr::from_mut(&mut original).cast(),
                copy_nonclone,
                copy_nonclone,
            );
        }
        Self { base }
    }
}
unsafe fn copy_nonclone(target: *mut c_void, source: *mut c_void) {
    unsafe {
        target
            .cast::<NoncloneValue>()
            .write(NoncloneValue((*source.cast::<NoncloneValue>()).0));
    }
}
unsafe fn destroy_nonclone(item: *mut c_void) {
    unsafe {
        item.cast::<NoncloneValue>().drop_in_place();
    }
}
// SAFETY: the raw engine owns initialized aligned NoncloneValue objects until owner drop.
unsafe impl BasicSparseArraySource<NoncloneValue> for NoncloneSource {
    fn sparse_storage(&self) -> &SparseArrayBase {
        &self.base
    }
}
