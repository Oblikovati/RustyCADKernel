//! Hashing and equality are one injected policy, as in OCCT's NCollection maps.
use crate::standard::Failure;
/// A key's OCCT hash representation; `42_i32.occt_hash()`.
pub trait HashKey {
    /// Returns the native-width hash used by NCollection.
    fn occt_hash(&self) -> usize;
    /// Native `std::equal_to` dispatch, independently of Rust operators; `first.occt_equal(&second)`.
    fn occt_equal(&self, other: &Self) -> bool
    where
        Self: PartialEq,
    {
        self == other
    }
}
/// Combined hash/equality policy; implement both methods for stateful key semantics.
pub trait HashPolicy<K> {
    /// Returns a raw hash before bucket reduction; `policy.hash(&key)`.
    fn hash(&self, key: &K) -> usize;
    /// Compares keys consistently with their hash; `policy.equal(&first,&second)`.
    fn equal(&self, first: &K, second: &K) -> bool;
}
/// Stateless native OCCT hashing; `DataMap::<i32,i32>::default()` uses this policy.
#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultHashPolicy;
impl<K: HashKey + PartialEq> HashPolicy<K> for DefaultHashPolicy {
    fn hash(&self, key: &K) -> usize {
        key.occt_hash()
    }
    fn equal(&self, first: &K, second: &K) -> bool {
        first.occt_equal(second)
    }
}
macro_rules! integer_hash {
    ($($type:ty),*) => { $(impl HashKey for $type { fn occt_hash(&self)->usize { *self as usize } })* };
}
integer_hash!(
    bool, u8, i8, u16, i16, u32, i32, u64, i64, usize, isize, char
);
impl HashKey for crate::tcollection::AsciiString {
    fn occt_hash(&self) -> usize {
        self.hash_code()
    }
}
/// `NCollection_Primes::NextPrimeForMap`, the first table step above `size`; `next_map_prime(100) == 101`.
pub fn next_map_prime(size: usize) -> usize {
    const STEPS: [usize; 24] = [
        101, 1009, 2003, 5003, 10007, 20011, 37003, 57037, 65003, 100019, 209953, 472393, 995329,
        2359297, 4478977, 9437185, 17915905, 35831809, 71663617, 150994945, 301989889, 573308929,
        1019215873, 2038431745,
    ];
    // Native `std::lower_bound(table, theN + 1)` in size_t arithmetic: `theN + 1` wraps to 0
    // for SIZE_MAX, so that request yields the first table entry, 101.
    let target = size.wrapping_add(1);
    STEPS
        .into_iter()
        .find(|&prime| prime >= target)
        .unwrap_or(target)
}

/// `NCollection_BaseMap::nbBucketsFromInt`, the check behind every `int` bucket-count
/// overload (`NCollection_DataMap(int)`, `ReSize(int)`); `buckets_from_int(-1)` is an error.
///
/// ```
/// use rustycad_tkernel::ncollection::buckets_from_int;
/// assert_eq!(buckets_from_int(7).unwrap(), 7);
/// assert_eq!(buckets_from_int(-1).unwrap_err().exception_type(), "Standard_OutOfRange");
/// ```
pub fn buckets_from_int(count: i32) -> Result<usize, Failure> {
    usize::try_from(count).map_err(|_| {
        Failure::with_type(
            "Standard_OutOfRange",
            &format!("NCollection_BaseMap: negative bucket count {count}; expected >= 0"),
            "",
        )
    })
}

// NCollection_DefaultHasher<T*> hashes the address itself and compares pointers.
impl<T> HashKey for *const T {
    fn occt_hash(&self) -> usize {
        *self as usize
    }
}
impl<T> HashKey for *mut T {
    fn occt_hash(&self) -> usize {
        *self as usize
    }
}

// libstdc++ hash<float/double> in the pinned Linux oracle profile; signed zeros share a bucket.
impl HashKey for f32 {
    fn occt_hash(&self) -> usize {
        if *self == 0.0 {
            return 0;
        }
        libstdcpp_float_bytes(&self.to_ne_bytes())
    }
}
impl HashKey for f64 {
    fn occt_hash(&self) -> usize {
        if *self == 0.0 {
            return 0;
        }
        libstdcpp_float_bytes(&self.to_ne_bytes())
    }
}

fn libstdcpp_float_bytes(bytes: &[u8]) -> usize {
    if usize::BITS == 64 {
        crate::standard::hashing::murmur64(bytes, 0xc70f_6907) as usize
    } else {
        crate::standard::hashing::murmur32(bytes, 0xc70f_6907) as usize
    }
}

/// Selects OCCT's `static_cast<size_t>` enum hashing for fieldless `Copy` enums.
/// Equality uses the enum's `PartialEq`, as native enum keys use `operator==`.
///
/// ```
/// use rustycad_tkernel::{impl_enum_hash_key, ncollection::HashKey};
/// #[derive(Clone, Copy, PartialEq)]
/// #[repr(i32)]
/// enum Direction { Reverse = -1, Forward = 1 }
/// impl_enum_hash_key!(Direction);
/// assert_eq!(Direction::Reverse.occt_hash(), usize::MAX);
/// ```
#[macro_export]
macro_rules! impl_enum_hash_key {
    ($($enum_type:ty),+ $(,)?) => {
        $(impl $crate::ncollection::HashKey for $enum_type {
            fn occt_hash(&self) -> usize {
                *self as usize
            }
        })+
    };
}
