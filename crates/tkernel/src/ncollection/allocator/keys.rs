//! Protected key and hasher value types from `NCollection_AccAllocator`.

/// `NCollection_AccAllocator::Key`, containing a block's address-divided key value.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AccBlockKey(usize);

impl AccBlockKey {
    /// Creates a block key value; `AccBlockKey::new(42).value() == 42`.
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    /// Returns `Key::Value`; `AccBlockKey::new(42).value() == 42`.
    pub const fn value(self) -> usize {
        self.0
    }

    /// Updates public native `Key::Value`; `let mut key = AccBlockKey::new(1); key.set_value(2)`.
    pub fn set_value(&mut self, value: usize) {
        self.0 = value;
    }
}

/// `NCollection_AccAllocator::Hasher`, which hashes a key to its stored value.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AccBlockKeyHasher;

impl AccBlockKeyHasher {
    /// Returns the identity hash used by OCCT; `AccBlockKeyHasher.hash(AccBlockKey::new(42))`.
    pub const fn hash(self, key: AccBlockKey) -> usize {
        key.value()
    }

    /// Compares key values; `AccBlockKeyHasher.equal(AccBlockKey::new(4), AccBlockKey::new(4))`.
    pub const fn equal(self, first: AccBlockKey, second: AccBlockKey) -> bool {
        first.value() == second.value()
    }
}
