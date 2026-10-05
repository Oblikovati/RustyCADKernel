# `NCollection_IndexedMap::Add(const TheKeyType&)` probe

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The probe uses a custom key copy constructor and hasher/equality policy. The first
Add observes hash before key copy (`HK`); the duplicate lookup observes one hash
and one equality check without a copy (`HE`). This covers observable ordering for
value-semantic keys. OCCT allocates one combined node before constructing its key,
while Rust clones the key before its separate allocator call. Arbitrary C++
constructor, allocator-failure, and destructor effects therefore remain partial.
