# `NCollection_IndexedMap::Add(const TheKeyType&)` probe

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The probe uses a custom key copy constructor and hasher/equality policy. The first
Add observes hash before key copy (`HK`); the duplicate lookup observes one hash
and one equality check without a copy (`HE`). The allocation-trace companion
confirms native node allocation precedes key copying (`HAK`), duplicates do not
allocate (`HE`), and throwing copy construction calls placement delete (`HAKF`).
Rust allocates uninitialized node storage before invoking `Clone` and releases it
if `Clone` panics. Bucket-array allocator callbacks, direct construction into the
key subobject's final address, and arbitrary C++ subobject/destructor behavior
remain outside this safe value-semantic mapping.
