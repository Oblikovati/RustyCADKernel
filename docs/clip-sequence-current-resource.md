# Martensite sequence current-resource retirement

`SequenceNodeRetirement::retire_using` is the allocation-retirement prerequisite
for clipping collection callbacks. The unique original node owner transfers its
full allocation provenance to a shared unwind-cleanup guard. The selector runs
only after the real payload destructor has returned, then forwards the original
raw pointer through the currently selected compatible allocator's virtual `Free`.
Ordinary node destruction still uses its original allocator through the same guard.

The unsafe caller contract requires the selected resource to accept the original
allocation and exact layout. Callers must release collection adapter guards before
retirement; resource lookup occurs after payload callbacks and outside those guards.
For example, a detached node can be retired with
`unsafe { node.retire_using(|| sequence.read().allocator().storage_lease()) }`.
The resource lease preserves physical allocation storage without adding a native
allocator reference.

The Rust-only native replay uses actual `Sequence<i32>` nodes, `SequencePreparedNode`,
and controlled clear traversal. Its real derived allocator replaces the sequence's
resource during `Allocate`; the original allocator then reaches reference count
zero while the node remains alive. Current `Free` observes the actual old sequence
size, the current object's real reference count, and the original object's destructor
flag. All six rows of `allocator-retirement-native.txt` must match byte-for-byte.
Provenance remains in `base-allocator-dispatch-2026-10-12/native-probes.json` and its
independently verified `main-integrated` receipts, at pinned OCCT commit
`3d097a0328e71b826377d4814ab05ec3c3d23871`.

Two further regressions select a replacement allocator from the actual payload's
destructor and verify that Rust unwind cleanup still calls its selected `Free`.
The throwing-payload case preserves Rust's existing cleanup contract; it does not
claim parity for a throwing C++ destructor. Both allocators remain strongly anchored
through these callbacks, avoiding an unproved last-owner loss during active dispatch.

This checkpoint does not complete the clipping collection, restore its audited API
claims, or claim class/constructor completion. The production collection and view
transport still require live-node/reference migration, full native callback replay,
and both-device rendering proof.
