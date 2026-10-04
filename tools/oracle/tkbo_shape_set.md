# BOPTools occurrence set

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
392 native cases combine seven compound contents, two traversal kinds, and four
root orientations. They compare cardinalities, equality in both directions,
shape occurrence retention, clone/assignment, clear and normalized hash-sum
formula. Degenerate edges are skipped only for edge traversal.
Replay: `cargo test -p rustycad-tkbo --test shape_set`.

Native IsEqual is intentionally asymmetric for some equal-length duplicate
lists: [A,B] considers [A,A] equal but the reverse is false. The Rust port
therefore exposes is_equal without implementing Rust Eq/PartialEq. Clear retains
the source shape; INTERNAL occurrences expand into FORWARD/REVERSED entries.

Hash values incorporate process-local topology identities. Absolute native/Rust
numbers cannot be compared across processes; both implementations are checked
against the native normalization formula using their respective input hashes.
Custom/null allocator behavior, exhaustion, platform-width extremes, arbitrary
locations and complete legacy/API parity remain unproven. These mappings are
partial. The storage uses the already ported allocator-aware DynamicArray.
