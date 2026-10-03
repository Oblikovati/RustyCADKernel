# TKernel refactoring with Rust ownership and standard types

Research date: 2026-10-03. Rust source snapshot: `65fed4cd`.
OCCT oracle: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
This is a plan for later execution; no implementation or dependency changes are included.

## Recommendation

Keep the mapped OCCT compatibility interface while simplifying implementations and offering
an opt-in Rust interface built around borrowed slices, owned values, iterators, `Option`, and
`Result`. Prefer standard types for new private storage. Deprecate redundant **Rust usage
patterns**, then selected convenience wrappers after a consumer migration; do not remove
an OCCT obligation from the inventory because its original motivation was C++.

The port already uses Rust facilities extensively. `Array1` owns `Vec<T>` or borrows slices;
`LinearVector` owns `Vec<T>`; `LocalArray` uses `SmallVec`; `Handle` owns an `Arc`; parallel
execution uses `std::thread`. A wholesale rewrite would duplicate existing modernization.
The opportunity is chiefly less conversion and cloning at callers, bounded lock lifetimes,
and selective removal of compatibility storage from private algorithms. See
[Array1](../crates/tkernel/src/ncollection/array1.rs),
[LinearVector](../crates/tkernel/src/ncollection/linear_vector.rs),
[LocalArray](../crates/tkernel/src/ncollection/local_array.rs),
[Handle](../crates/tkernel/src/standard/handle.rs), and
[parallel execution](../crates/tkernel/src/osd/parallel.rs).

The seam is the existing compatibility interface: a deep module should keep indexing,
errors, ownership, and traversal behavior local to one implementation. Do not expose a
new generic storage-policy parameter to every caller merely to make replacement possible.
Introduce an internal adapter only when a second implementation is actually evaluated.

## Measured scope and confidence

These counts are physical source measurements, not semantic coverage or performance results.
At the snapshot, `crates/tkernel/src/**/*.rs` contains **477 files / 60,133 lines**, including
comments and inline tests; `crates/tkernel/tests/**/*.rs` contains **286 files**.
Fifteen other workspace Cargo manifests directly mention `rustycad-tkernel`.

| Rust source directory | Files | Lines | API ledger rows by corresponding native package |
| --- | ---: | ---: | ---: |
| `ncollection` | 128 | 15,744 | 1,966 |
| `standard` | 68 | 6,957 | 360 |
| `tcollection` | 34 | 3,078 | 435 |
| `osd` | 70 | 9,815 | 393 |
| `message` | 41 | 4,688 | 429 |
| `quantity` | 27 | 5,008 | 130 |
| `storage` + `fsd` | 42 | 6,843 | 462 |
| `units` + `units_api` | 33 | 4,668 | 251 |
| `resource` | 15 | 1,754 | 42 |
| `plugin` + `tcol_std` | 3 | 238 | 8 |

Root-level modules and `c_format`/`stl` account for the remainder. The native package list
also contains `StdFail`, `TShort`, `UnitsMethods`, `FlexLexer`, and `Precision`; lack of a
same-named Rust directory or ledger row is not proof that a package is unnecessary.
Source: pinned `src/FoundationClasses/TKernel/PACKAGES.cmake`; Rust
[exports](../crates/tkernel/src/lib.rs) and [ledger](../porting/parity.json).

The TKernel ledger has **4,511 rows: 4,073 implemented claims, 57 partial, 381 excluded**.
These are claims, not 4,073 proven unique compiler declarations. Existing exclusions include
C++ destructors represented by Rust drop behavior and begin/end overloads represented by
Rust iterators; each exclusion needs semantic accounting in the compiler crosswalk.
The ledger maps **1,316 legacy declarations**. Historical paired evidence records 1,521
native passes and the single approved OCC23497 skip; translated runtime expansion and
compatibility regression accounting are explained in [TKernel progress](tkernel-progress.md)
and [proof archive](../porting/api-inventory/README.md). A new refactor requires fresh
receipts; historical green results do not establish current-tree parity.

Counts were computed by summing `splitlines()` over the source glob, counting matching
`toolkit == "TKernel"` rows in `porting/parity.json`, and scanning Cargo manifest text.
Repeat at a new revision before using them as planning estimates.

A reproducible consumer screen: scan `crates/*/src/**/*.rs`, excluding `tkernel`, count literal
substring occurrences and files containing each substring. Comments and aliases are included;
this is a prioritization signal, not an exhaustive compiler call graph.

| Pattern | Occurrences | Files | Consumer crates |
| --- | ---: | ---: | ---: |
| `Handle<` | 7,056 | 1,291 | 14 |
| `SharedState<` | 286 | 228 | 12 |
| `Array1<` | 4,509 | 639 | 12 |
| `Sequence<` | 292 | 79 | 8 |
| `DataMap<` | 167 | 74 | 9 |
| `AsciiString` | 184 | 48 | 9 |
| `ExtendedString` | 0 | 0 | 0 |
| `BaseAllocator` | 58 | 19 | 4 |

Zero external text matches for `ExtendedString` do not justify removal: TKernel itself uses
it in persistence and messages, and future application/data-exchange ports are incomplete.
For example, [Storage header fields](../crates/tkernel/src/storage/header_data.rs) expose
extended strings and lists of comments.

## Classification and replacement inventory

**Retain** means preserve the compatibility behavior and mapped interface. **Replace internally**
means consider changing storage only after its observable constraints are proven. **Facade**
means add an opt-in idiomatic interface without silently changing the mapped one. **Deprecate
usage** means discourage a redundant pattern in new internal code; it is not API deletion.

| Surface | Decision | Rust resources and conditions |
| --- | --- | --- |
| Primitive Standard scalar spellings, C++ typedef/macro syntax | Deprecate usage | Use `bool`, `i32`, `f32`, `f64`, references and generics internally. Preserve signed widths, constants, numeric rounding and macro functionality; do not globally turn indices into `usize`. |
| `Array1`, `Array2`, HArray wrappers | Retain + facade | Existing Vec/slice storage is appropriate. Borrow `&[T]`/`&mut [T]` in private algorithms; retain lower bounds, row/column bounds, ownership queries, resize/copy semantics and shared HArray identity. |
| `LinearVector` | Deprecate usage + retain | Use `Vec<T>` for new private growable arrays. Keep wrapper operations and capacity/error behavior accessible. Audit each conversion for allocation and index changes. |
| `LocalArray` | Retain | Already uses SmallVec. Profile stack size and spill behavior before changing default inline capacity; stack allocation is not always cheaper. |
| `DynamicArray`, sparse arrays | Retain; conditional internal replacement | Block allocation and stability are distinct from Vec. Dense Vec/`Vec<Option<T>>` can serve new private workloads only where sparse-index and lifetime behavior are absent. |
| `List`, `Sequence`, detached cursors | Retain | Native node identity, splice/transfer and mutation across cursors prevent a blanket Vec/VecDeque replacement. Private append-only temporary sequences can use Vec after proving no escaped cursor/reference/allocator contract. |
| Map/DataMap/DoubleMap/FlatMap variants | Retain | Bucket growth, hash/equality policies, collision traversal, replacement and removal are observable. std HashMap does not reproduce the native ordering. A private membership-only set may use HashSet after proving iteration cannot affect geometry/output. |
| Indexed maps | Retain + facade | Preserve 1-based indices and reindexing/removal/substitution. `indexmap` is an experiment for private index-stable workloads, not a drop-in replacement. |
| Ordered maps | Retain; conditional internal replacement | Native insertion order is useful, but stable node views and allocator identity add constraints beyond indexmap. BTreeMap key order is a different contract. |
| Packed integer sets | Retain | Preserve signed key domain, ordered/bit traversal and set operations; consider native Rust bitset crates only for private finite nonnegative domains with measured benefit. |
| KDTree/UBTree/CellFilter/math vectors | Retain algorithms | Tree tie-breaking, selector callbacks and floating-point operation order need oracle evidence. `[T; N]` is suitable private fixed storage; generic math or spatial crates require separate numeric equivalence work. |
| `AsciiString` / `ExtendedString` | Retain + fallible facade | Keep byte and UTF-16 units respectively. Provide explicit validated UTF-8 conversions; never implicitly normalize or replace invalid sequences. |
| HAsciiString/HExtendedString | Retain | Shared mutable identity and dynamic types remain capabilities; new exclusively owned Rust text may simply use String/Vec. |
| Handle/Transient/RuntimeType | Retain + borrowed facade | Arc already owns lifetime. Borrow `&T`/HandleView for non-owning calls. Keep null handles, native counts, projection, ancestry and identity. |
| SharedState | Retain; refactor access patterns | Short std lock scopes, snapshot only the required fields, invoke foreign/virtual callbacks unlocked, and preserve incremental state visibility. |
| Standard allocation / NCollection allocators | Retain compatibility; isolate unsafe storage | RAII owners, checked Layout, existing allocator-api2 bridge. Private single-owner memory may use Vec/Box; arena reset/statistics/alignment are separate contracts. |
| Failure/StdFail/ErrorHandler | Retain + Result facade | Use typed `Result`, `?`, and RAII in new code. Preserve native exception class, stack/message and catch boundaries; do not use panic as ordinary native failure transport. |
| OSD path/files/timing/parallel | Retain + std-backed adapters | std I/O, Path/OsStr, Instant and threads for private operations; preserve native syntax/error/worker numbering and platform-dependent behavior. |
| Message/Resource/Plugin | Retain + injected adapters | Owned printer/resource/module interfaces over std streams and platform loader services. Keep printer order, filtering, formatting, identity and resource lookup semantics. |
| Quantity/Precision/Units/UnitsAPI/UnitsMethods | Retain | Domain rules, tolerances, formatting and legacy parsing are not obsolete C++ scaffolding. New unit-safe Rust types may be a separate facade. Preserve OCC23497 current behavior. |
| Storage/FSD/TColStd/TShort/FlexLexer | Retain obligations | Rust aliases/generics can replace template spelling; binary persistence, generated lexical behavior and typed collection contracts require explicit mapping. Do not substitute Serde or another lexer and assume format parity. |

Evidence for the storage distinctions is in
[Sequence](../crates/tkernel/src/ncollection/sequence.rs),
[List](../crates/tkernel/src/ncollection/list.rs),
[DynamicArray](../crates/tkernel/src/ncollection/dynamic_array.rs),
[DataMap](../crates/tkernel/src/ncollection/data_map.rs),
[IndexedMap](../crates/tkernel/src/ncollection/indexed_map.rs),
[OrderedMap](../crates/tkernel/src/ncollection/ordered_map.rs), and
[FlatMap](../crates/tkernel/src/ncollection/flat_map.rs).
Native anchors, all under pinned `src/FoundationClasses/TKernel/NCollection/`:
`NCollection_Array1.hxx:127` (Lower), `:133` (IsDeletable), `:416` (Resize), and
`NCollection_IndexedMap.hxx:110` (iterator), `:446` (Substitute).
Rust's collection documentation explicitly distinguishes sequence, key-sorted and arbitrary
hash iteration order. [Official collections documentation](https://doc.rust-lang.org/std/collections/).

## Ownership, identity and callbacks

`Handle<T>` stores `Option<Arc<dyn Transient>>`, increments the embedded StandardTransient
counter on adoption and decrements it on drop. `Transient: Any + Send + Sync`; RuntimeType
contains projectors for concrete embedded bases and trait interfaces. Native
`Standard_Transient.hxx:101,107,112,123` exposes `This`, `GetRefCount`, increment and decrement.
Consequently replacing the wrapper with bare Arc, or deriving visible counts from
`Arc::strong_count`, needs proof covering owner references, casts, weak upgrades and manual
counter operations. Redundant-looking counters have an observable purpose.
[Current runtime implementation](../crates/tkernel/src/standard/runtime_type.rs),
[counter implementation](../crates/tkernel/src/standard/transient.rs),
[weak handles](../crates/tkernel/src/standard/handle/weak.rs).

Rust `Any` identifies concrete types; it does not provide the OCCT named inheritance graph
or arbitrary embedded-base projection. Preserve that graph. A closed enum may simplify a
private algorithm with a closed set of cases, but cannot replace public subclass extension
points. Arc already provides shared ownership and Weak breaks non-owning cycles; converting
all handles to Rc would remove Send/Sync capability and disrupt parallel consumers.
[Official Any](https://doc.rust-lang.org/std/any/trait.Any.html),
[official Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html).

The first ownership experiment should replace temporary handle clones with borrowed views
where lifetimes permit, then measure. Do not replace a clone that is itself part of observable
native reference-count behavior. Do not turn shared native setters into `&mut self` methods
unless a separate facade explicitly owns the object exclusively.

`SharedState` documents the rule against calling other handles while a guard is held, but
its public guards and poison recovery do not enforce that rule. Its comment that a panic
cannot happen while holding a guard is an assumption requiring audit, particularly around
user-defined callbacks. std RwLock does not guarantee a locking priority and recursive
acquisition may deadlock. Switching to another lock crate does not fix callback reentrancy.
[SharedState](../crates/tkernel/src/standard/shared_state.rs),
[official RwLock behavior](https://doc.rust-lang.org/std/sync/struct.RwLock.html).

Use an explicit sequence: read required inputs; release guard; invoke callbacks; reacquire
and apply the native next mutation. A snapshot-and-commit transaction is only valid if
native observers cannot see intermediate changes. Preserve callback count/order, errors and
partial outputs. Messenger already snapshots printer handles before delivery, a useful
existing pattern, but its semantics cannot simply be imposed on every algorithm.
[Messenger](../crates/tkernel/src/message/messenger.rs).

A concrete consumer constraint is `SurfaceSplitValues = Handle<HSequence<f64>>` in
[SplitSurface](../crates/tkshhealing/src/shape_upgrade/split_surface/mod.rs): returned sequence
identity is shared and mutations are observable. Its build implementation already wraps
borrowed arrays with `Array1::from_shared`. Replacing every such value with a fresh Vec
would lose shared identity even when numeric contents initially match.

## Strings, errors and allocation

`AsciiString` keeps arbitrary bytes, a final NUL and potentially embedded NULs; explicit-span
and C-string constructors intentionally differ. `CString` disallows interior NUL; String
requires valid UTF-8. Neither is its general replacement.
[AsciiString](../crates/tkernel/src/tcollection/ascii_string.rs),
[official CString](https://doc.rust-lang.org/std/ffi/struct.CString.html),
[official String](https://doc.rust-lang.org/std/string/struct.String.html).
`ExtendedString` uses UTF-16 code-unit lengths and may split surrogate pairs in native edits;
retain Vec<u16> semantics. Native `TCollection_ExtendedString.hxx:47-53` explicitly documents
this. Use `TryFrom`-style conversions for valid Unicode and explicitly named lossy conversions
only when the caller chooses that behavior. OS filenames should use OsStr/Path at host I/O
seams, while OSD_Path retains its own parsed syntax and byte rules.
[ExtendedString](../crates/tkernel/src/tcollection/extended_string.rs),
[OSD path behavior](../crates/tkernel/src/osd/path/relative.rs).

Do not collapse all errors to Option or a string: Failure retains class and message, while
some native missing-value operations already map to Option. Keep each mapping explicit.
A new facade can return tuples instead of out-parameters, but must preserve partial writes
before an error when exposing the native operation. [Failure](../crates/tkernel/src/standard/failure.rs).

`BaseAllocator` already has owned RAII blocks and an allocator-api2 implementation, with
common/heap/incremental/accumulating/aligned policies. Allocation identity, alignment,
zero-fill, per-allocation release, reset, statistics, callbacks, failure and destruction
order must be profiled separately. Native `NCollection_IncAllocator.hxx:78` makes Free a
no-op; `:93` exposes Reset; `:136` warns about invalidated allocations.
[Allocator](../crates/tkernel/src/ncollection/allocator.rs),
[owned blocks](../crates/tkernel/src/ncollection/allocator/owned.rs),
[raw allocation](../crates/tkernel/src/ncollection/allocator/raw_heap.rs).

For new private buffers, prefer Vec/Box and fallible reservation where allocation failure
is part of the contract. Do not promise recoverable out-of-memory from every ordinary Vec
operation. Arena allocation is appropriate only when lifetimes share a proven region;
reset must not invalidate live safe references, and destructors must still run when required.
Runtime-stride `AliasedArray` also has alignment and POD restrictions; substituting `Vec<u8>`
would not supply its current 16-byte storage guarantee.
[AliasedArray](../crates/tkernel/src/ncollection/aliased_array.rs).

## Cargo candidates and selection gates

Prefer std first. These are alternatives for targeted experiments, not approved additions.
Package documentation was checked on the research date; recheck MSRV, features, licenses,
maintenance and versions against the locked workspace toolchain before execution.

| Candidate | Suitable experiment | Constraint / source |
| --- | --- | --- |
| std Vec/slices/Box/Option/Result/iterators | First choice for private ownership and traversal | No dependency; retain compatibility bounds and failure semantics. [std collections](https://doc.rust-lang.org/std/collections/) |
| Existing `smallvec` 1.16.2 | Profile LocalArray and small private scratch buffers | Already declared; validate spill and stack footprint. [workspace manifest](../crates/tkernel/Cargo.toml) |
| Existing `allocator-api2` =0.4.0 | Keep injected allocator interface with stable Rust | std Allocator remains marked nightly-only in the checked official docs; do not migrate production to nightly just to remove this dependency. [std Allocator](https://doc.rust-lang.org/std/alloc/trait.Allocator.html) |
| Existing `bytemuck` =1.25.2 | Retain checked POD adapter for aliased storage | Project-owned ArrayValue interface already wraps it; no new dependency needed. [ArrayValue](../crates/tkernel/src/ncollection/aliased_array.rs) |
| `indexmap` | Private insertion-ordered membership/index experiment | Ordered storage does not establish native cursor/address or removal parity. [crate documentation](https://docs.rs/indexmap/latest/indexmap/) |
| `slotmap` | New private graph with generational identifiers | Keys replace addresses only in a new internal contract; native address identity/serialization must not change. [crate documentation](https://docs.rs/slotmap/latest/slotmap/) |
| `bumpalo` | Algorithm-local temporary allocation region | Plain bump allocations do not run Drop; use a destruction strategy and compare peak retained memory. Not a BaseAllocator substitute. [crate documentation](https://docs.rs/bumpalo/latest/bumpalo/) |
| `rayon` | Independent new parallel loops behind a project adapter | Work scheduling/reductions can change order, worker IDs and failure handling. Keep OSD ThreadPool semantics unless proven equivalent. [crate documentation](https://docs.rs/rayon/latest/rayon/) |

All optional candidates above are Rust implementations, not native-library wrappers. No
version upgrade is recommended by this report. The platform/native dependency replacement
work remains separately planned in [external dependency migration](external-dependency-migration-plan.md).

## Ordered execution plan

1. **Baseline and inventory gate.** Freeze a clean implementation revision; rerun paired
   TKernel verification and all direct/transitive consumer tests. Attach compiler inventory
   and declaration-to-test receipts. Enumerate actual call sites for each proposed change,
   separating public/protected compatibility methods from private helpers. Review the 57
   partial claims and 381 exclusions; do not use them to reduce the denominator silently.
2. **First pilot: borrowed numeric slices.** Select one private Array1 consumer whose bounds
   are fixed and no view escapes. Move its loop into a slice-taking private function; keep
   the mapped wrapper translating bounds. Compare geometry results and measure removed
   allocations/clones. Reject a generic policy framework for this small change.
3. **Second pilot: temporary ownership.** Select one private append-only Sequence/LinearVector
   workload with no allocator/cursor observers. Use Vec inside that algorithm, retain its
   public outputs. Compare insertion/error order and memory usage. This validates the
   largest plausible collection simplification without changing every collection class.
4. **Callback and lock audit.** Enumerate guard lifetimes crossing trait calls, printers,
   allocators, Drop and nested handle operations. Add named reentrant observers; shorten
   lock scopes while preserving partial native state. Review poison handling explicitly.
5. **Allocation pilot.** Instrument allocation count, total bytes, peak live/retained bytes,
   alignment and drops for a bounded operation. Compare current allocator against ordinary
   Vec/Box first; evaluate an arena only if measurements support it. Keep public allocation
   operations and native callbacks on their compatible implementation.
6. **Idiomatic facade.** Add borrowed views, fallible text conversions and owned result
   values where callers benefit. Keep native mapping aliases available. Document index
   units (offset vs native bound; byte vs UTF-16 unit) in each interface.
7. **Targeted deprecation.** After migrating all intended internal users and proving API
   reachability, discourage redundant wrapper construction in new Rust-only code. Any
   formal `#[deprecated]` must have a supported replacement and preserve compilation of
   the compatibility path; blanket warnings over translated legacy tests are unhelpful.
8. **Optional deeper experiments.** Evaluate indexmap, slotmap, bumpalo or Rayon separately
   with their own behavior/performance report. Never combine ownership, ordering and
   numeric algorithm rewrites in one parity review.
9. **Rebuild proof.** Regenerate Rust/compiler crosswalk, exact test mapping and runtime
   evidence at the new revision; require no lost declarations or new unexplained skips.
   Record accepted implementation substitutions without recategorizing them as exclusions.

Each pilot is independently reversible. Proceed only with a demonstrated simplification,
measurable resource improvement, or fixed correctness problem; this research asserts no
speedup, memory saving, deletion count, or refactor completion percentage.

## Validation required before landing an implementation

- Run the existing single-command Rust suite (`cargo test --locked -p rustycad-tkernel`),
  the paired oracle (`python3 tools/verify_tkernel.py --source <pinned-OCCT> --jobs 2`),
  and affected consumer suites. Retain all 1,316 legacy declarations and explicit
  OCC23497 skip/current-behavior distinction. Supplemental tests do not replace legacy tests.
- Differential sequences: arbitrary/negative array bounds, empty ranges, borrowed ownership,
  resize aliases, map collisions/rehash/removal order, detached cursor survival, allocator
  identity, destruction order, weak expiry, cast counts and shared state seen through bases.
- Strings: all byte values, embedded NUL, invalid UTF-8, lone surrogates, surrogate pairs,
  one-based slicing/editing and native formatting/locale. No normalization in parity tests.
- Errors/callbacks: exact exception class and catch boundary, partial outputs, callbacks
  that reenter the same object, mutate a related object, fail, or panic. Mock external I/O
  using named fake types; do not fake OCCT numerical results.
- Memory: zero-sized and over-aligned types, reset with live owners, allocation failure
  injection, destructor counters, grow/shrink and raw-pointer provenance. Use Miri for
  supported pure Rust unsafe paths and native sanitizers where applicable; neither proves
  all platform behavior. Isolate potentially blocking reentrancy probes in a timed process.
- Performance: release builds with identical compiler/options and fixed hardware; report
  input scale, distributions, allocation counts, peak memory, code size and clone/lock
  counts. Include small, large, sparse, collision-heavy and shared workloads. Numerical
  result/tolerance and deterministic output gates take priority over throughput.
- Platform matrix: Linux, Windows, macOS host behavior and enabled feature configurations.
  std-backed I/O does not erase native path, timing, threading or error differences.

Remaining uncertainty: this is a source-level investigation with targeted consumer examples,
not a whole-program ownership analysis or benchmark run. Public API completeness is still
unproven, consumer ports are evolving, and no existing TKernel wrapper has been demonstrated
removable wholesale. The safest first work is the two bounded private-storage pilots plus
callback lock auditing while the main API/test port continues.

## Pinned native source references

The local source anchors above refer to the immutable OCCT revision, available directly at:

- [TKernel package list](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/FoundationClasses/TKernel/PACKAGES.cmake).
- [Standard_Transient reference-count and This contract](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/FoundationClasses/TKernel/Standard/Standard_Transient.hxx#L101).
- [NCollection_Array1 bounds and ownership](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/FoundationClasses/TKernel/NCollection/NCollection_Array1.hxx#L127).
- [NCollection_IndexedMap substitution](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/FoundationClasses/TKernel/NCollection/NCollection_IndexedMap.hxx#L446).
- [ExtendedString UTF-16 indexing](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/FoundationClasses/TKernel/TCollection/TCollection_ExtendedString.hxx#L47).
- [IncAllocator lifetime/reset contract](https://github.com/Open-Cascade-SAS/OCCT/blob/3d097a0328e71b826377d4814ab05ec3c3d23871/src/FoundationClasses/TKernel/NCollection/NCollection_IncAllocator.hxx#L78).

These native contents were read from the pinned sibling checkout. Relative Rust links refer
to the report's snapshot and may move as the port advances. Official web documentation was
consulted for language/library guarantees; latest documentation may describe a newer Rust
release than the workspace's pinned 1.98.1, so implementation must check availability on that
pinned toolchain. No compiler, benchmark, Miri, or implementation test was run for this
research-only change; relative document links and `git diff --check` were checked.
