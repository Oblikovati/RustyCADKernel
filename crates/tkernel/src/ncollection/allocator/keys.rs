//! Protected key and hasher value types from `NCollection_AccAllocator`.

/// `NCollection_AccAllocator::Key`, containing a block's address-divided key value.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AccBlockKey {
    /// Address-divided block key; may be borrowed directly, as native `Key::Value`.
    ///
    /// ```
    /// use rustycad_tkernel::ncollection::AccBlockKey;
    /// let mut key = AccBlockKey::new(41);
    /// let value = &mut key.key_value;
    /// *value += 1;
    /// assert_eq!(key.value(), 42);
    /// ```
    pub key_value: usize,
}

impl AccBlockKey {
    /// Creates a block key value; `AccBlockKey::new(42).value() == 42`.
    pub const fn new(value: usize) -> Self {
        Self { key_value: value }
    }

    /// Returns `Key::Value`; `AccBlockKey::new(42).value() == 42`.
    pub const fn value(self) -> usize {
        self.key_value
    }

    /// Updates public native `Key::Value`; `let mut key = AccBlockKey::new(1); key.set_value(2)`.
    pub fn set_value(&mut self, value: usize) {
        self.key_value = value;
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
