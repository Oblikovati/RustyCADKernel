# Martensite genuine allocator and independent storage evidence

Local oracle only, pinned OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
Native derived dispatch4 + forwarding2 + valid Heap retirement6 + construction5 records.
The construction replay checks stack0, owned1, original-pointer source2 and released1;
the final static-reference count control is native-only, paired with Rust exact slot-reference tests.
Assets are under crates/tkernel/tests/fixtures/allocator-*-native.txt;
tools/oracle/martensite_allocator_*.cxx are their actual native sources.

The Heap retirement regression was genuinely red with retained allocator
HandleObserver backing. Native count-zero destructor ran with Size1; Rust
postponed it and current Free observed retired=false. The corrected independent
memory-provider lease has weak allocator identity and zero intrusive ownership.
Current Free receives original raw allocation provenance outside record guards.

No renderer submission change, GPU timing or baseline acceptance is involved.
The allocator class remains partial; no new denominator exclusions, private
helper credit, source-retirement class completion or custom-pool native assets
are fabricated. Subsequent checked Sequence integration is owned by the clipping
worker and will test actual collection metadata/argument callbacks.

A final consumer run exposed eleven real no-scratch PackedMap rehash failures:
constructing a temporary owning façade allocated an extra size48/align8 Box.
The raw Standard seam now borrows the actual persistent CommonBaseAllocator
handle and invokes its real source directly. All eleven strict width cases pass
without changing their allocation trace gate. Rejected and fixed logs are retained.

Final checks:9,089 five-toolkit tests, five-crate Clippy -Dwarnings,46 Miri
allocator cases, global formatter, canonical discovery and13 discovery
regressions pass. Five existing visible bindings are refreshed; constructor
now maps the actual zero-count protected base. No class completion is claimed.

The final one-crate compiler capture is retained as rustdoc-kernel.json.gz,
rust-api-inventory.json.gz and capture-manifest.json.gz; claim-preflight.json.gz
verifies all five exact current source bindings. This scoped TKernel capture
is not a whole-workspace inventory or class-completion claim.

Temporary dispatch observer lifetime during owner loss inside an active virtual
callback remains open. The native retirement fixture loses the caller owner
after Allocate returns; the original four texture-retirement bindings remain partial.
