# TKernel port status

**FoundationClasses is in progress. TKernel is not accepted as complete.**
All 51 TKernel GoogleTest source suites are accounted for. Their geometry dependencies
now include initial TKMath, TKBRep, and TKTopAlgo implementations. These dependency
toolkits remain incomplete; production Rust code does not link OCCT.

## Legacy suites translated

| Upstream source | Rust integration target | Cases |
| --- | --- | ---: |
| `Quantity_Period_Test.cxx` | `quantity_period` | 20 |
| `Quantity_Date_Test.cxx` | `quantity_date` | 20 |
| `Standard_GUID_Test.cxx` | `standard_guid` | 4 |
| `NCollection_Array1_Test.cxx` | `ncollection_array1` | 44 |
| `NCollection_LocalArray_Test.cxx` | `ncollection_local_array` | 33 |
| `NCollection_Array2_Test.cxx` | `ncollection_array2` | 37 |
| `Standard_ArrayStreamBuffer_Test.cxx` | `standard_array_stream_buffer` | 20 |
| `NCollection_LinearVector_Test.cxx` | `ncollection_linear_vector` | 49 |
| `Standard_Failure_Test.cxx` | `standard_failure` | 1 |
| `NCollection_Vec4_Test.cxx` | `ncollection_vec4` | 5 |
| `Quantity_Color_Test.cxx` | `quantity_color` | 21 |
| `Quantity_ColorRGBA_Test.cxx` | `quantity_color_rgba` | 15 |
| `OSD_Parallel_Test.cxx` | `osd_parallel` | 2 |
| `Standard_Atomic_Test.cxx` | `standard_atomic` | 1 |
| `Standard_Strtod_Test.cxx` | `standard_strtod` | 4 |
| `NCollection_DynamicArray_Test.cxx` | `ncollection_dynamic_array` | 52 |
| `NCollection_BaseAllocator_Test.cxx` | `ncollection_base_allocator` | 7 |
| `NCollection_Buffer_Test.cxx` | `ncollection_buffer` | 1 |
| `NCollection_OccAllocator_Test.cxx` | `ncollection_occ_allocator` | 5 |
| `NCollection_List_Test.cxx` | `ncollection_list` | 34 |
| `Standard_Character_Test.cxx` | `standard_character` | 1 |
| `NCollection_SparseArray_Test.cxx` | `ncollection_sparse_array` | 10 |
| `NCollection_Sequence_Test.cxx` | `ncollection_sequence` | 20 |
| `TCollection_AsciiString_Test.cxx` | `tcollection_ascii_string` | 114 |
| `NCollection_DataMap_Test.cxx` | `ncollection_data_map` | 36 |
| `TCollection_ExtendedString_Test.cxx` | `tcollection_extended_string` | 105 |
| `NCollection_Map_Test.cxx` | `ncollection_map` | 20 |
| `NCollection_IndexedMap_Test.cxx` | `ncollection_indexed_map` | 33 |
| `NCollection_IndexedDataMap_Test.cxx` | `ncollection_indexed_data_map` | 51 |
| `NCollection_DoubleMap_Test.cxx` | `ncollection_double_map` | 23 |
| `NCollection_FlatMap_Test.cxx` | `ncollection_flat_map` | 25 |
| `NCollection_FlatDataMap_Test.cxx` | `ncollection_flat_data_map` | 41 |
| `NCollection_OrderedMap_Test.cxx` | `ncollection_ordered_map` | 38 |
| `NCollection_OrderedDataMap_Test.cxx` | `ncollection_ordered_data_map` | 47 |
| `NCollection_ForwardRange_Test.cxx` | `ncollection_forward_range` | 22 |
| `OSD_Path_Test.cxx` | `osd_path` | 19 |
| `FSD_BinaryFile_Test.cxx` | `fsd_binary_file` | 7 |
| `NCollection_CellFilter_Test.cxx` | `ncollection_cell_filter` | 9 |
| `NCollection_UBTree_Test.cxx` | `ncollection_ubtree` | 16 |
| `Message_Messenger_Test.cxx` | `message_messenger` | 2 |
| `NCollection_PackedMap_Test.cxx` | `ncollection_packed_map` | 58 declarations / 264 cases |
| `NCollection_AliasedArray_Test.cxx` | `ncollection_aliased_array` | 4 |
| `OSD_PerfMeter_Test.cxx` | `osd_perf_meter` | 12 |
| `Handle_Operations_Test.cxx` | `handle_operations` | 7 |
| `Handle_Advanced_Test.cxx` | `handle_advanced` | 7 |
| `Standard_Handle_Test.cxx` | `standard_handle` | 1 |
| `Resource_Manager_Test.cxx` | `resource_manager` | 3 |
| `NCollection_KDTree_Test.cxx` | `ncollection_kdtree` | 187 |
| `UnitsAPI_Test.cxx` | `units_api`, `units_api_current_behavior` | 9 translated + 1 upstream skip with compatibility regression |
| `Standard_Dump_Test.cxx` | `standard_dump` | 12 |
| `TopLoc_Location_Test.cxx` | `toploc_location` | 1 |
| **Total** | | **1316 declarations / 1522 mapped Rust cases** |

The 1315 enabled declarations in these 51 source files are translated, preserving inputs, assertion
values, sequence, and upstream test identity. Most GoogleTest fixtures have empty
setup/teardown; the meter suite resets its injected registry before and after each case. Rust names use `Suite__Case` for upstream `Suite.Case`. Unused
out-parameters and original local names are retained to aid review. GUID's C++
const comparisons use Rust const methods because derived trait equality is not
const; runtime operator equality is also checked.

The unchanged C++ sources produce 1521 passes and one explicit upstream skip.
All 1522 mapped Rust runtime cases pass, including the separate compatibility regression.
Supplemental Rust tests cover state preservation, errors, signed storage overflow, operators and
GUID encodings. A deterministic 1,824-case corpus compares C++ and Rust output
exactly, including date arithmetic, carry/borrow, absolute differences, constructor
errors, all GUID fields and text formatting. These results apply to the recorded
Linux/GCC/C-locale profile. The GitHub workflow also runs the Rust suites on Windows
and macOS; those hosted runs have not been observed locally.

## Reproduce the evidence

```sh
cargo test --locked -p rustycad-tkernel
python3 tools/verify_tkernel.py --source ../OCCT --jobs 2
python3 tools/porting.py status
```

The verification script checks the pinned clean OCCT revision, test-source hashes,
complete mappings for each selected source file, actual GoogleTest enumeration,
C++/Rust test results and exact differential output. Missing tests, unexpected skips, and mismatches fail. The sole approved skip is
recorded separately and must still match the original reason and behavior decision. It compiles the unchanged C++ test sources against a separately
built OCCT TKernel and its legacy tests’ geometry dependencies. Production Rust crates do not link OCCT.

Evidence is written to `target/oracle-tkernel/`: upstream XML, Rust test logs,
compiler versions, input corpus, both output streams and a JSON report with input,
output and implementation hashes. A failed verification removes the previous
success report. CI uploads these artifacts even when verification fails. Re-run
after code or mappings change; a previous report is not proof for a new tree.

For the whole-toolkit acceptance check:

```sh
python3 tools/verify_tkernel.py --require-complete
```

**This must currently fail.** TKernel has 51 GoogleTest source files and 1,316
source-discovered declarations. All 1316 are mapped; 1315 are translated and one has an explicit upstream-skip mapping. The full native runtime expands to 1,522 cases: 1,521 pass and one is explicitly skipped by upstream (`UnitsAPI_Test.FClassesBug_23497_CompositeUnit`). The skip is accounted for by the user-approved current-behavior regression, but its intended-behavior assertions are never counted as passed. Every declaration is mapped to its native runtime expansions in `porting/tkernel-runtime.json`. DRAW inventories and authoritative API enumeration remain incomplete. The mapped
sources cover all their current legacy cases, not the entire Quantity or Standard
package. There is no claim of 100% API coverage for these types or TKernel.

## Upstream build dependency discovered

The full `OpenCascadeGTest` target does not build with only FoundationClasses at the
pinned revision: `TKernel/GTests/TopLoc_Location_Test.cxx` includes `TopoDS_Shape.hxx`
and `BRepBuilderAPI_MakeVertex.hxx` from modeling toolkits. The focused runner builds
`TKernel` and `gtest_main`, then compiles only the mapped test source files. It does
not edit OCCT or remove tests from TKernel's inventory. The full native baseline command now builds that dependency closure and compiles
all 51 unchanged TKernel test sources without selecting unrelated toolkit tests:

```sh
python3 tools/tkernel_native_baseline.py --jobs 4
python3 tools/map_tkernel_runtime.py
```

It records every expanded runtime identity and status in
`target/oracle-tkernel/full/runtime-inventory.json`. This is native baseline evidence,
not Rust parity evidence. OCC23497 remains an upstream skip with a separate verified Rust compatibility regression.

## Implemented interfaces and translation decisions

The facade exposes `rustycad_kernel::foundation`; direct consumers can use
`rustycad_tkernel::{quantity::{Period, Date}, standard::{Guid, Uuid}}`.
[The mapping ledger](../porting/parity.json) records individual API and test mappings.
These API mappings are manually inventoried and remain `implemented`, not globally
`verified`; test mappings remain `translated` with per-run evidence in the report.

- `Period` retains nonnegative time, microsecond normalization and absolute
  subtraction. OCCT's component constructor validates the **combined totals**;
  its six-component `IsValid` checks each input. This distinction is preserved.
- `Date` retains the 1979 epoch, Gregorian validation, normalized arithmetic,
  absolute differences and failure before the epoch. Failed valid-range setters
  leave existing state unchanged.
- Storage retains OCCT's signed 32-bit seconds domain. Rust returns
  `ErrorKind::NumericOverflow` when calculations exceed it. C++ signed overflow is
  undefined; such inputs are documented Rust safety behavior and excluded from
  oracle equivalence claims. `Date::is_valid` still accepts years that overflow
  construction, matching OCCT's component-validation contract.
- Errors preserve upstream exception type and message through `Result`; this is
  an initial transport for these APIs, not a port of the complete Standard exception
  hierarchy. Arithmetic operators return `Result` when an operation can fail.
- Overloads receive distinct Rust names, output references become tuples, and C++
  default arguments become explicit Rust arguments. For example, a seconds-only
  period uses `Period::new(seconds, 0)`.
- GUID constructors, UUID fields, const comparisons, assignments, formatting and
  buffer encodings are implemented. ASCII input preserves C-string termination;
  UTF-16 input preserves OCCT's low-byte conversion of the first 36 units. Short
  UTF-16 slices are rejected safely instead of reproducing an out-of-bounds read.
  OCCT's `std::hash<Standard_GUID>` behavior remains unported; the Rust type does
  not claim a compatible hash implementation.

- `Array1` and `Array2` support borrowed/owned storage, deep copying, storage reuse,
  moves, arbitrary bounds and checked access. Rust initializes new primitive slots
  to defaults where C++ leaves them uninitialized. `Array2` retains the different
  packed and rectangular copy rules of `Resize` and `ResizeWithTrim`.
- `LocalArray` uses `smallvec` 1.16.2 behind a project-owned wrapper for real inline
  storage. Its tests retain allocation transitions and real construction/destruction
  counts. Rust moves relocate values without a user-defined move constructor, so
  C++ constructor-plus-move counts map to Rust construction counts. Shared views
  are immutable slices; mutable views require exclusive borrowing.
- Random collection inputs are captured from the native GCC `mt19937(1)` generator
  and distribution, preserving the 100 upstream values in a checked-in fixture.
- `ArrayStreamBuffer` implements Rust `Read`, `BufRead`, and `Seek`, borrows its
  input, and clamps forward seeks. `seek_occt` preserves the unusual C++ subtraction
  of positive end-relative offsets; the standard `Seek` trait retains Rust's signed
  end-relative convention. Invalid negative positions are rejected safely.
  Standard-library stream EOF state maps to Rust read results in translated tests.

- `LinearVector` preserves contiguous storage, capacity reuse, growth, erasure and
  mutable array views. Its heap-object regressions use the owned
  `AsciiString` implementation, whose full legacy suite is now translated.
- `Failure` preserves typed messages, explicit stack strings and copy behavior.
  Automatic stack capture and the full exception hierarchy remain pending.
- NCollection_Vec2/Vec3/Vec4, NCollection_Mat3 and NCollection_Mat4 now map every
  public member (see the NCollection vectors and matrices section below).
- `Color`/`ColorRgba` preserve float channel storage, all 509 canonical named colors
  and 521 enum spellings, Lab/Lch conversions, gamut reduction and CIEDE2000.
  The 999 deterministic round trips and upstream reference tables are retained.
  Epsilon access uses an atomic bit representation; upstream's source uses a
  shared double despite naming its getter/setter test `EpsilonThreadSafety`.
- Parallel loops and explicit launchers use [Rayon 1.12.0](https://docs.rs/rayon/1.12.0/rayon/struct.ThreadPool.html)
  behind project-owned types. Mutable callbacks receive exclusive slices or slots.
  The atomic test retains both 2^24-operation passes. Thread-pool configuration,
  exception and scheduling APIs are not yet fully ported.
- `strtod` uses Rust's correctly rounded decimal parser with C-prefix scanning,
  plus hexadecimal and NaN-payload handling. An additional 2,091-case oracle
  compares exact binary64 bits and byte end offsets, giving 3,915 differential
  cases overall. OCCT's hexadecimal subnormal-to-normal rounding mask returns
  zero at one boundary; that behavior is deliberately preserved and tested.
  C `errno`/floating-environment side effects have not been exposed as Rust APIs.
  Adapted hexadecimal/NaN routines retain the original Lucent permission notice.

## Next dependency-ready work

Continue inside TKernel: Standard errors/handles, bounded NCollection storage and
strings, then the remaining independent Quantity, resource, storage and OS APIs.
Translate each complete source suite as its API slice becomes available, retain
DRAW regressions in the backlog, and add the cross-toolkit test dependencies to the
oracle. Finish TKernel acceptance before advancing to TKMath.

Translated sources/tests retain OCCT notices and use
`LGPL-2.1-only WITH OCCT-exception-1.0` in the TKernel crate. The accompanying license
texts are under `licenses/`; the existing project root license is unchanged.

## Allocator-backed collections

DynamicArray uses power-of-two block increments with stable element storage. List
uses individually allocated nodes, allocator-preserving assignment (OCC25348),
splicing, detached cursors, and iterative destruction. SparseArray allocates
blocks on demand and visits occupied indices in ascending order. BaseAllocator
and OccAllocator keep allocator identity and allocation lifetimes together;
Buffer distinguishes an allocated zero-byte buffer from freed storage.

The `allocator-api2` 0.4.0 adapter contains a scoped unsafe implementation:
its allocator-trait implementation delegates global allocations or dispenses
disjoint aligned arena regions. Arena reset rejects live allocations. This safe
ownership restriction replaces upstream's caller-enforced lifetime requirement.
Rust allocation failure handling, allocator statistics, native raw-pointer
adoption, and runtime type information are not yet fully mapped APIs.

Character classification currently implements the recorded C locale; changing
the process locale is not yet supported. Test completion for these source files
does not imply API completion for their packages.

Sequence preserves one-based access and stable per-element allocations, with
bidirectional borrowed iterators and detached cursors. Its ordering table uses
Rust storage, so operation complexity does not yet match OCCT's linked sequence
in every case; performance parity remains part of acceptance work.

## OCC23497 behavior decision

The user selected **preserve current OCCT behavior** for the skipped
`UnitsAPI_Test.FClassesBug_23497_CompositeUnit` regression. The mapping retains its original skip and reason. The separate
`occ23497_current_composite_prefix_failure` regression verifies the native
`Standard_OutOfRange` / `NCollection_Sequence::Value` failure for `sq.km`.
The Units differential corpus includes this failure. The disabled regression’s
intended assertions have not run and are never counted as passed.

## String compatibility evidence

All 114 AsciiString legacy cases are translated. Safe self-range edit methods
represent overlapping source pointers. UTF-16 and wide-character conversions
now also pass all 105 ExtendedString legacy cases. The reference-counted object
hierarchy still requires its own complete test ports.

An additional 2,608 exact-output comparisons cover `%g` real formatting, native
string hashes (including OCCT's signed-character FNV behavior), numeric prefixes,
UTF-8 useful length, and embedded NUL boundaries. Together with the numeric and
time/GUID corpora, the combined verifier checks 6,523 differential cases.
`from_view` reproduces the NUL-stopping view constructor; copying bounded byte
slices retains embedded NULs. The supplemental regressions preserve that observed
distinction. Invalid UTF-8 input construction and non-C locales remain API parity work.

DataMap now passes all 36 declarations. It uses OCCT's exact bucket-growth table,
head insertion and rehash traversal, allocator-owned entries, and an injected
hash/equality policy. Assignment retains destination policy and allocator; copying
retains source policy. Wider map-family APIs remain pending.

ExtendedString additionally passes 66,565 exact native comparisons: all 65,536
single UTF-16 units, empty input, boundary sequences, and 1,024 seeded multi-unit
strings. OCCT drops unpaired surrogates during UTF-8 conversion; the port does the
same. The combined verifier now checks 73,088 differential cases.

- `Map` reuses the allocator-owned hash bucket engine with stable key addresses.
  Duplicate Add retains the old key; Emplace reconstructs it at the same address.
  Assignment retains destination policy/allocator; Exchange swaps both. The five
  set operations exercised by OCC24271 preserve operand traversal choices.

- `IndexedMap` shares OCCT's bucket growth table, stores stable allocator-owned
  keys and maintains dense one-based indices. Removal swaps the final entry into
  the removed index. Substitute preserves storage and rejects duplicate keys.
  The custom TestKey hasher uses captured libstdc++ string hashes on the oracle
  profile; `tools/oracle/indexed_key_hashes.cxx` reproduces those fixture values.

- `IndexedDataMap` composes the indexed key engine with stable allocator-owned
  value nodes. Add/TryBound retain old values; Bind/Bound/Emplace replace them
  in place. Mutable iteration preserves immutable keys and index order. The
  current storage uses separate key and value allocations; allocator allocation
  counts and the complete container API are not yet accepted as equivalent.

- `DoubleMap` maintains two hash indices over the same allocator-owned pair.
  Duplicate keys raise Standard_MultiplyDefined; TryBind leaves bindings intact.
  Floating-point hashes match the pinned libstdc++ oracle, including signed-zero
  normalization. Native platform-specific hash implementations remain profile work.

- `FlatMap` uses contiguous Robin Hood slots, the native 13/16 load limit,
  power-of-two capacity and backward-shift deletion. Copies preserve slot layout;
  copying an empty reserved map drops capacity as OCCT does. A confirmed native
  quirk is preserved: after Add(0), Add(1), Emplaced(8) returns key 1 whereas
  Added(8) returns key 8. `tools/oracle/flat_map_displacement.cxx` reproduces it.

- `FlatDataMap` shares the contiguous Robin Hood engine with FlatMap, preserving
  paired values across cluster shifts, slot order, capacity and lazy construction.
  Unlike FlatMap, native Emplaced returns the newly inserted binding after a
  displacement; the Rust value-returning methods preserve that distinction.

- `OrderedMap` threads insertion-order links through stable allocator-owned
  hash nodes. Removal unlinks a node; reused storage appends at the tail. Copies,
  rehashing, replacement, moves and cursors preserve the tested order contracts.

- `OrderedDataMap` shares ordered hash nodes, with stable key/value pairs, edge
  access and detached cursors that explicitly reborrow the map. Exclusive range
  traversal builds a temporary borrow table to yield disjoint mutable values in
  linked order safely; its setup allocation is a performance difference pending
  the complete API/performance acceptance work.

- `ForwardRange` uses explicit, compile-checked accessor descriptors in place of
  C++ method-name SFINAE. Value wins over Current, which wins over CurrentId.
  Borrowed/value outputs retain their types; postfix snapshots own their old value,
  and non-copyable hosts move into the range. Rust iteration borrows its host
  exclusively; accessors yielding references must borrow stable external storage.

- `CadPath` retains native Unix/Windows path predicates, name splitting, portable
  trek encoding and parent traversal. Current-directory and file existence access
  use injectable sources. The legacy current-directory regression also exercises
  the real process directory; the remaining OSD path/file APIs are pending.

- `BinaryFile` exposes byte inversion for integer, UTF-16, pointer-sized integer,
  binary32 and binary64 values. The seven OCC24537 tests retain literal expected
  bytes; supplemental checks preserve NaN payloads. Binary storage-driver I/O is
  not yet implemented.

- `CellFilter` uses native truncation/wrapping of cell coordinates, axis-zero-first
  range traversal, independent target copies and newest-first linked lists. Purge
  and removal destroy matching targets and clean up empty cells. Zero/nonfinite
  cell sizes remain outside the defined native-input parity domain.

- `UBTree` and `EBTree` preserve native branch selection, child-zero-first traversal,
  selector stopping and sibling absorption after removal. Iterative traversal and
  flat node ownership avoid recursive destruction. The current Rust representation
  allocates sibling nodes separately and retains vacant index slots after removal;
  native allocation-layout and full node-mutation APIs remain acceptance work.
  `tools/oracle/bounding_tree_order.cxx` reproduces the supplemental order fixture.

- `Messenger` supports injected shared printers, identity-based registration,
  gravity filtering, scoped buffered delivery and native empty formal stream copies.
  The legacy tests retain printer replacement/restoration using an isolated
  messenger instead of the process-global singleton. The complete handle/RTTI,
  console-default and binary stream APIs remain pending.

- `PackedMap` uses 32-bit or 64-bit words, native bucket growth/order, restartable
  cursors and set operations. Each of its 264 native typed cases has a distinct
  Rust test identity in the mapping ledger. The native cardinality-only `Differ`
  result and lowest-block maximum quirks are retained, with reproducible evidence
  in `tools/oracle/packed_map_edges.cxx`. Mutable result/operand aliasing is exposed
  through separate in-place operations; additional self-alias interfaces remain
  part of full API accounting.

- `AliasedArray` supports runtime stride, default aligned ownership, exclusive
  borrowing, reallocation/copying and checked whole-element/prefix typed views.
  [bytemuck 1.25.2](https://docs.rs/bytemuck/1.25.2/bytemuck/) performs the checked
  casts behind this interface. Rust initializes fresh bytes and rejects invalid
  alignment/overflow. Arbitrary native alignment parameters and moved-from raw
  aliases still require API mapping.

- `PerfMeter` retains name sharing, reset-on-start timing, inactive/running output,
  killed-name lookup and handle-independent watch lifetime. A per-fixture registry
  replaces global mutable test state. Its narrow, documented unsafe clock adapter
  follows OCCT's actual OS calls: Linux CLOCK_THREAD_CPUTIME_ID, Windows user-only
  GetThreadTimes, and macOS TASK_THREAD_TIMES_INFO user time. OS bindings are
  libc 0.2.189, windows-sys 0.61.2 and mach2 0.7.0. Windows GNU and macOS ARM test
  targets compile locally; native runtime equivalence remains Linux-only evidence.

## Recent foundation behavior

Handles now share real owned transient objects, preserve atomic observable reference
counts, check runtime types through explicit base projections, and support a
51-level inheritance hierarchy. Non-owning views borrow the owner without increasing
the reference count. Raw-pointer adoption, custom deletion, and the complete native
runtime type registry remain API work; translated handle tests do not establish those contracts.

Resource files use injected filesystem/environment interfaces. Both default files
populate the reference map; only explicit user overrides are saved. Save rereads the
user destination, creates parent directories, sorts keys, and preserves native
leading-space/backslash escaping. Verbose diagnostics and Unicode resource conversion
remain unported APIs. Legacy filesystem regressions use a named fake in Rust and
unchanged native filesystem operations in OCCT.

KDTree owns points and optional radii, builds a balanced median permutation, and
uses 32-point leaf buckets, spatial pruning, and a distance-only max heap. Its 187
legacy declarations preserve all original cloud sizes, queries, assertions, and
brute-force comparisons. C arrays use Rust heap storage to avoid test-thread stack
limits. Random constructor argument evaluation follows the GCC oracle's right-to-left
order; braced point initialization keeps left-to-right evaluation. Query collections
currently return Vec indices; native DynamicArray return-type API mapping remains
explicitly pending. Linux/GCC median tie ordering is reproduced and compared by a
964-case corpus covering exact indices, ordered results, callbacks, floating-point
bits, leaf boundaries, negative radii/tolerances, and overflowing squared distances.
Native standard-library tie ordering on other platforms still requires oracle profiles.

## Units and geometry dependencies

The compiled OCCT unit dictionary is vendored as byte-preserving data with source
and payload hashes. All nine enabled UnitsAPI legacy tests pass, plus 3900 exact
comparisons of token spelling, dimensions, conversion bits, shifted units, and
MDTV local units. Configurable resource units and process-global cache edge cases
still need API coverage; these results do not establish full Units API parity.

Standard_Dump has all 12 legacy tests translated. Point, direction, frame, and
bounding-box fragments use native formatting and one-based parser cursors.
Additional probes check malformed fragments, partial coordinate writes, exception
messages, cursor changes, and translation rounding. The geometry corpus has 461
cases. The combined differential corpus contains 78413 cases.

OCC25545 runs the original 1000-vertex concurrent-read scenario against shared
Rust vertex geometry and immutable location chains. Location transformations are
cached during construction as in current OCCT. Identity, translation, rotation,
uniform scale and composition gp_Trsf forms are implemented. Mirror forms and
the remaining topology APIs are still pending. Location operations propagate
native transform-inversion failures through `Result`. See [TKMath progress](tkmath-progress.md). The crate dependencies reflect the genuine test dependency closure;
this is not acceptance of TKMath, TKBRep, or TKTopAlgo.

### NCollection vectors and matrices

`Vec2`, `Vec3`, `Vec4`, `Mat3` and `Mat4` port every public member of
NCollection_Vec2/Vec3/Vec4/Mat3/Mat4 for `float`, `double` and (vectors) `int` elements:
all `NCOLLECTION_VEC_COMPONENTS_2D/3D` swizzles (`xy()` ... `wzy()`, `rgb()` ... `brg()`),
the colour aliases, the converting constructors (`convert_from`, `from_other`), the
Vec2/Vec3 construction overloads, `ChangeData`, `IsEqual`, every operator, `DumpJson`,
and the matrix sums, products, scalar operations, negation, transposition, inversion,
`Adjoint`, `Determinant`/`DeterminantMat3`, `GetMat3` and `Translate`. The two matrices
share their common members through one macro. `porting/parity.json` maps each member.
1,000 `ncollection_vec` and `ncollection_mat` differential requests in the TKMath
numeric corpus match OCCT exactly; supplemental tests are in `ncollection_vec_api_behavior`
and `ncollection_mat_api_behavior`.

The port keeps these native results:

- `NCollection_Mat4::Adjoint` builds its rows with `crossVec4`, whose sign convention
  gives the negated cofactor matrix: `Adjoint(I) = -I` and the inverse is
  `-Adjoint^T / det`. `NCollection_Mat3::Adjoint` has the usual sign.
- `Mat4::Multiply(A, B)` starts every element at `(Element_t)0`, so a `-0` product sum
  becomes `+0`; `Mat * Vec` starts from the first product and keeps `-0`.
- `Mat3::Inverted(out, det)` leaves `out` untouched for a zero determinant and divides
  the transposed adjugate by the determinant. `Mat4` leaves the unscaled cofactors in
  `out` and multiplies by the reciprocal.
- Integer `Modulus` computes `std::sqrt` in double and truncates the result. Out-of-range
  and NaN float-to-int conversions use the x86-64 `cvttsd2si` result, `INT_MIN`.
- NCollection_Mat3 has no `Rows`/`Cols` and no `Convert`; the Rust type has none either.

Deliberate Rust deviations: components are public fields, so the non-const `x()`
reference overloads are field access and `ChangeData` returns an array of component
references (a matrix returns its column array). `Map` copies instead of reinterpreting
memory. `DumpJson` exists only for float and double elements, because the native macro
reads each argument as a `double`, which is undefined for `int` elements.

## Standard, TCollection, TColStd, StdFail, Plugin and TShort API completion

Every public member of these packages' headers has an `api` row in `porting/parity.json`:
`python3 tools/audit_api_rows.py src/FoundationClasses/TKernel/<Package> --source ../OCCT`
reports 0 missing and 0 stale rows for each. TShort has no headers at the pinned revision
(the package only lists an empty `FILES.cmake`), so it has nothing to map.
`tools/verify_tkernel_api.py` (run by `verify_tkernel.py`) compiles
`tools/oracle/tkernel_api_probe.cxx` against the prebuilt TKernel and compares every
response with the Rust twin `crates/tkernel/examples/api_probe`, byte for byte. Each area
owns one `tools/oracle/tkernel_<area>_probe.cxx`, one `api_probe/<area>.rs` and one
`tools/tkernel_<area>_corpus.py`.

### Exception classes (Standard, StdFail, Plugin_Failure)

Each `DEFINE_STANDARD_EXCEPTION(C1, C2)` class is a constructor function returning a
`Failure` with the class name: `standard::exceptions::out_of_range`, `std_fail::not_done`,
`plugin::failure`. C++ catches an exception by any base class; Rust has one `Failure`
type, so the class tree is data (`standard::exceptions::exception_parent`) and
`Failure::is_kind("Standard_DomainError")` answers what `catch (Standard_DomainError&)`
would catch. Classes declared by other toolkits (`math_NotSquare`) are not in the TKernel
table; `is_kind` still reports them as `Standard_Failure`, the root of every OCCT
exception. The probe throws every class with empty, plain, NUL-truncated and UTF-8
messages and compares the full catch-by-base matrix, `ExceptionType()`, `what()` and
`operator<<` output (which omits `": "` for an empty message).

### TCollection_AsciiString, TCollection_HAsciiString and TCollection

Pending.

### TCollection_ExtendedString and TCollection_HExtendedString

Pending.

### Standard scalar, character, hashing, time, GUID and atomic helpers

Pending.

### Standard handles, transients, type descriptors, failures and TColStd handles

Pending.

### Standard memory managers, synchronization, locale, buffered readers, dump and Plugin

Standard.hxx (except `StackTrace`, ported with Standard_Failure), Standard_MMgrOpt,
Standard_MMgrRoot, Standard_MemoryUtils, Standard_DefineAlloc, Standard_Mutex,
Standard_Condition, Standard_CLocaleSentry, Standard_ReadBuffer, Standard_ReadLineBuffer,
Standard_ArrayStreamBuffer, Standard_Dump and Plugin.hxx have 89 rows (80 implemented,
9 excluded: destructors and the SunPro-only placement `operator new`). The
`services_` operations compare 3,931 requests with the native kernel.

- **Standard memory.** The oracle is built with `USE_MMGR_TYPE=NATIVE`, so
  `standard::memory` follows the C runtime calls it compiles to: `allocate` is zeroed
  (calloc), `allocate_optimal` returns `None` for null, `purge` returns 1 and
  `get_allocator_type` is `Native`. Blocks are owned `StandardMemory`/`AlignedMemory`
  values, a `None` slot is the null pointer, and the nulling template overloads
  `Free(T*&)` / `FreeAligned(T*&)` are `free_and_null` / `free_aligned_and_null`.
  Quirks kept: glibc `realloc(p, 0)` frees the block and returns NULL, so `reallocate`
  empties the slot and raises Standard_OutOfMemory; alignment follows GCC's
  `_mm_malloc` on x86 (1 is malloc, 2 and 4 widen to a pointer, other values need a
  power-of-two multiple of a pointer). Aligned blocks over-allocate a `Vec` instead of
  calling an unsafe allocator. Standard_OutOfMemory is built with
  `Failure::with_message` until the runtime area's `exceptions::out_of_memory` lands.
- **Standard_MMgrOpt.** `MMgrOpt` keeps OCCT's 8-byte headers and intrusive
  newest-first free lists inside real heap regions addressed by their start
  (`StorageAddress`), so rounded sizes, pool offsets, reuse order and Purge counts
  match. `bytes`/`bytes_mut` stand for dereferencing a pointer. Mapped memory is a
  page-aligned zeroed region; the page size comes from a narrow FFI adapter
  (`sysconf`, Mach `vm_page_size`, `GetSystemInfo`). The callback is process-wide like
  the native file-static pointer. Quirks kept: `ROUNDUP8` and the header size wrap, so
  sizes within 7 of `usize::MAX` become zero-size pool blocks and larger ones map
  length 0 (`strerror` "Invalid argument"); a failed allocation purges, then raises; a
  fully free pool is never released, because the Purge tally omits the pool's link
  word (P - 8 is never P). Deviations: native undefined behavior (freeing a foreign or
  freed address, a header past a too-small block) raises Standard_ProgramError; the
  manager takes `&mut self` instead of locking; the POSIX mapping path is modeled on
  every platform; the page the native `munmap` leaks is released.
- **Standard_MemoryUtils.** `make_shared`/`make_unique` are `Arc`/`Box` on the global
  allocator; `make_oshared` stores the value in a `BaseAllocator` (`Arc<AllocatedBox<T>>`)
  and both `NCollection_OccAllocator<T>` overloads are `make_oshared_in` (`OcctShared<T>`).
  Rust moves a constructed value in where C++ forwards constructor arguments.
- **Standard_Mutex and Standard_Condition.** The implementation creates a recursive
  pthread mutex (the header says otherwise), so `StandardMutex` keeps an owner and a
  depth: the owner re-locks, `try_lock` fails only for other threads, and an unlock by a
  non-owner does nothing (pthread EPERM). `Sentry` unlocks on drop, also during a panic,
  which replaces the error-handler callback. `StandardCondition` is the manual-reset
  event; `Wait(int)` is `wait_timeout`, and a zero or negative limit only tests the flag.
  Thread scripts are compared token by token.
- **Standard_CLocaleSentry.** Rust formatting never reads a process locale, so the
  sentry has nothing to switch or restore; it stays a scope marker, and `get_c_locale`
  returns one static `CLocale` (radix '.', empty group separator).
- **Standard_ReadBuffer / Standard_ReadLineBuffer.** Both take any `io::Read` for the
  `std::istream` and `FILE*` overloads, with the native block-read semantics. Quirks
  kept: lengths truncate with C++ `%`; a partial last chunk keeps stale cache bytes;
  a short read ends reading for good; a line's terminator is overwritten with NUL in
  place; `ReadLine`'s read count is the last block of the call; a backslash that ends a
  read block in multiline mode makes later newlines part of the line; an empty carry-over
  that never held a byte ends the stream (its `Data()` is NULL); `Clear` also resets the
  modes. `ReadChunk<T>` copies the chunk out (`T: AnyBitPattern`). Deviations: a zero
  chunk (native division by zero) and a `T` larger than a chunk raise
  Standard_ProgramError, and a failed `Init` hands out no chunk.
- **Standard_ArrayStreamBuffer.** Adds `xsgetn` and `uflow`. `put_back` compares the
  request with the previous `char` promoted to `int`, so where `char` is signed a byte of
  0x80 or more never matches (native quirk). Seeks that native code would leave outside
  the buffer (before the start, or past the end through a negative end offset) are
  refused with `InvalidInput` instead of being clamped silently.
- **Standard_Dump.** Every member is ported on one-based TCollection primitives, so
  `SubString` and `Location` exceptions (Location's without a message) match. Text
  arguments are `impl AsRef<[u8]>`; the variadic writers and `InitRealValues` take
  slices; `GetPointerInfo` is `get_pointer_info(Option<usize>)` and
  `get_handle_pointer_info(&Handle<T>)`; `HierarchicalValueIndices` returns a `Vec`. The
  `OCCT_DUMP_*`/`OCCT_INIT_*` macros are not members: `dump_vector`, `dump_real`,
  `dump_class_begin`, `dump_integer`, `init_vector`, `init_real_field` and
  `init_integer_field` write or read what their expansions do (the stale
  `OCCT_DUMP_CLASS_BEGIN` row was dropped). Quirks kept: `SplitJson` raises for any `{` at
  the level it splits, accepts any key as the pair separator and a one-byte text
  unparsed; object values keep their closing brace; `FormatJson` never closes the added
  outer brace when the text ends with a nested `}`; the short pointer text of `0x1234` is
  `0xx1234`; `DumpFieldToName` strips `an` from "angle" and raises for "()"; strings are
  streamed as C strings. Existing dump functions now accept byte text; their callers
  compile unchanged.
- **Plugin.** Loading native shared libraries has no meaning for the port.
  `PluginLoader` injects the resource io/environment, a `PluginLibraries` seam
  (`StaticPluginLibraries` registers `PLUGINFACTORY` factories by library file name)
  and a console writer, and follows `Plugin.cxx`: `<GUID>.Location` from a fresh
  "Plugin" catalog, `lib<name>.so`/`.dylib`/`<name>.dll`, Plugin_Failure messages and
  verbose output as natively, glibc `dlerror` wording, success-only caching (per loader
  instead of process-wide). The probe exercises the native failure paths with a
  temporary resource file.
