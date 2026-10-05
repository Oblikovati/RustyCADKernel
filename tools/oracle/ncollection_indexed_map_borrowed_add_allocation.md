# `NCollection_IndexedMap::Add(const TheKeyType&)` allocation trace

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The custom allocator is attached before insertion; its constructor-time events are
cleared so the first row records only `Add` behavior.

Insertion logs `HAK`: hash/search, placement allocation, then key copy. A duplicate
logs `HE`: no node allocation or key copy. When the key copy constructor throws,
the placement delete calls allocator `Free`, producing `HAKF`; the map remains
empty. This is the source evidence for allocating a Rust `MaybeUninit` owner before
calling `Clone`, then writing the completed value into it.
