use rustycad_tkernel::ncollection::{PackedInteger, PackedMap, PackedWord};
use std::marker::PhantomData;
#[derive(Clone, Copy, Default)]
struct BorrowedWord<'a>(u32, PhantomData<&'a u8>);
impl PackedWord for BorrowedWord<'_> {
    fn from_bits(bits: u64) -> Self {
        Self(bits as u32, PhantomData)
    }
    fn bits(self) -> u64 {
        u64::from(self.0)
    }
}
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct BorrowedInteger<'a>(u32, PhantomData<&'a u8>);
impl<'a> PackedInteger for BorrowedInteger<'a> {
    type Word = BorrowedWord<'a>;
    const IS_64_BIT: bool = false;
    const MIN: Self = Self(0, PhantomData);
    const MAX: Self = Self(u32::MAX, PhantomData);
    fn bits(self) -> u64 {
        u64::from(self.0)
    }
    fn from_bits(bits: u64) -> Self {
        Self(bits as u32, PhantomData)
    }
}
pub fn verify<'a>(_lifetime: &'a u8) {
    assert!(BorrowedWord::<'a>::NATIVE_REHASH_READER.is_none());
    let mut map = PackedMap::<BorrowedInteger<'a>>::default();
    for value in [0, 1, 32, 3232, u32::MAX] {
        map.add(BorrowedInteger(value, PhantomData));
    }
    custom_resize(&mut map);
    assert_eq!(map.bucket_count(), 1009);
    assert_eq!(map.len(), 5);
    assert!(map.contains(BorrowedInteger(u32::MAX, PhantomData)));
}

fn custom_resize(map: &mut PackedMap<BorrowedInteger<'_>>) {
    let (_, events) = super::trace::capture(|| map.resize(1000));
    assert!(
        events
            .iter()
            .any(|event| event.align == align_of::<usize>())
    );
}
