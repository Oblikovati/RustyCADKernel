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
MDTV local units. The full Units, UnitsAPI and UnitsMethods API is covered by the
"Units API completion" section below.

Standard_Dump has all 12 legacy tests translated. Point, direction, frame, and
bounding-box fragments use native formatting and one-based parser cursors.
Additional probes check malformed fragments, partial coordinate writes, exception
messages, cursor changes, and translation rounding. The geometry corpus has 461
cases. The combined differential corpus contains 81557 cases, 3144 of them NCollection requests.

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

## NCollection API completion

`python3 tools/audit_api_rows.py src/FoundationClasses/TKernel/NCollection --source ../OCCT`
reports 0 public members without api rows and 0 stale rows. The package has 1,961 rows:
1,691 implemented and 270 excluded. Each excluded row names the Rust construct that covers
it: destructors are Drop glue, move constructors and move assignment are Rust moves, STL
`begin`/`end` boilerplate is covered by `iter`/`IntoIterator`, and internal node classes are
covered by the owning container. The audit tool now reads hidden-friend operators,
header-local macros (followed through same-package includes), cv-qualified conversion
operators, multi-literal deprecation markers, specializations with call arguments and
deduction guides; `tools/tests/test_audit_api_rows.py` covers each rule.

The differential corpus of `tools/verify_tkernel.py` includes one NCollection probe family per
container family (`tools/oracle/tkernel_ncollection_<family>_probe.cxx`, Rust twin in
`crates/tkernel/examples/oracle_probe/ncollection/`, requests in
`tools/tkernel_ncollection_<family>_corpus.py`): 3,144 requests, all identical to OCCT.

### Vectors and matrices

**NCollection vectors and matrices (Vec2/Vec3/Vec4, Mat3/Mat4, Lerp).** Every public member
now has an exact api row; the audit reports 0 missing / 0 stale for these six headers. No Rust
code changed: the existing implementation already covered every member, and the TKMath numeric
probe (`ncollection_vec` 600 requests, `ncollection_mat` 400 requests) compares each operator,
swizzle, conversion and matrix member bit-for-bit. The work re-spelled 84 rows to the header's
own identities (`const NCollection_Vec3<Element_t>&` instead of the `Vec3` shorthand), gave the
hidden-friend operators (`operator+/-/*//(const NCollection_VecN&, const NCollection_VecN&)`)
their two-operand signatures, and corrected trait spellings (`<Vec3 as Sub>::sub`, scalar
division as `Div<T>`). Four rows were dropped because the header has no such member: the three
`operator[]` rows (C++ subscripting goes through the `operator Element_t*()` conversion, whose
rows map to `data`/`change_data`; Rust's `Index<usize>` impl is the idiomatic counterpart) and a
duplicate `SetValues(x,y)`. The audit tool now follows same-package includes transitively, since
NCollection_Vec4 reaches `NCOLLECTION_VEC_COMPONENTS_2D` only through NCollection_Vec3.hxx.

### Arrays

The public members of `NCollection_Array1`, `Array2`, `HArray1`, `HArray2`, `LocalArray`,
`LinearVector`, `DynamicArray`, `AliasedArray`, `SparseArray`, `SparseArrayBase` and the
deprecated `NCollection_Vector` alias have 378 api rows. The audit reports no gaps and no
stale rows for these headers. Destructors, STL `begin`/`end` boilerplate and the OCCT
`More`/`Next` iterator classes are excluded. Each exclusion note names the Rust iterator
that covers it.

Newly ported members:

- `HArray1<T>` and `HArray2<T>` are handle-shared transients. The array sits behind a
  lock. The `array1()`/`change_array1()` guards expose the full array API to every handle
  holder. Each instantiation registers one `NCollection_HArray1`/`2` descriptor whose parent
  is `Standard_Transient`.
- `Array1`: `update_upper_bound`, `length` and `range_size`.
- `Array2`: `array_size`, `range_size`, `upper_for_size`, `begin_position`,
  `last_position`, the `int`/`size_t` counts, the flat `at_offset`/`change_at_offset`
  accessors and `first`/`last`.
- `LinearVector`: `has_data`, `max_size` and the const `to_array1_view`.
- `LocalArray`: `to_array1_view`.
- `AliasedArray`: `lower`/`upper`/`is_allocated`, `first`/`last`, the untyped
  `byte_value` accessors and `move_from`.
- `DynamicArray`: `with_occ_allocator`.
- `SparseArray`: `capacity` and `entries_mut`.

Preserved native behaviour:

- Bound arithmetic uses wrapping 32-bit `int` math (`UpdateUpperBound`, `BeginPosition`,
  `LastPosition`, `UpperForSize`, `Length`).
- `RangeSize` sign-extends a negative count to `usize`, as `static_cast<size_t>(int)` does.
- An `Array2` constructed with `upper == lower - 1` is empty. This fixes an earlier Rust
  deviation that refused the empty range. Only `Resize` refuses it.
- The `SparseArray` block table starts at `increment` entries and doubles, so `Capacity`
  is `increment²` after the first `SetValue`. A block whose last value is unset is freed.
- `LinearVector::SetValue` returns the stored slot.

Deliberate deviations:

- Natively, `AliasedArray::Move` leaves the source as a dangling non-owning alias. In Rust
  the source is left empty instead.
- The const `ToArray1()` overloads return a shared view that copies itself on the first
  write. It never writes through a const buffer.
- `DynamicArray::set_increment` also drops the empty blocks that were allocated with the
  old size.
- Unchecked native accessors (`First`, `Last` and the `DynamicArray` `int` operators) are
  checked `Result`s.

A new differential probe family (`ncoll_arrays_*`, 54 cases) compares these results with
OCCT bit for bit: bounds and layout (including `int` wrap-around and empty dimensions),
`AliasedArray` shapes, the `LinearVector` capacity rule, `SparseArray` capacity, assign
and exchange, and `DynamicArray` inserts and `SetValue` with the native out-of-range
throws.

### Lists and sequences

All 188 public members of the seven headers have api rows (151 implemented, 37 justified
exclusions: 3 destructors, 12 STL `begin`/`end`/`cbegin`/`cend` members covered by `iter`,
`iter_mut` and `IntoIterator` for `&`/`&mut`, 19 internal node members of
`NCollection_ListNode`/`TListNode`/`SeqNode`/`Sequence::Node`, `Sequence::delNode`, and the raw
`CurrentNode`/`PreviousNode` pointers of the base sequence iterator).
Native exception conditions are now `Result` errors instead of silent no-ops or panics:
`List::remove_first` on an empty list and every cursor edit at an end or foreign cursor report
`Standard_NoSuchObject`; every sequence index, range, exchange, split and insertion check reports
`Standard_OutOfRange` naming the index and valid range. `First`/`Last` keep returning `None`
where OCCT raises; `Sequence`'s `Index`/`IndexMut` panic with the OutOfRange message.

Newly ported: `List::append_at` (`Append(item, Iterator&)`), `insert_list_after`, `last_mut`,
`contains`, `clear_with_allocator`, `length`, `ListCursor::init`; `Sequence::at`/`change_at`,
`first_mut`/`last_mut`, `insert_sequence_before`, `remove_cursor`, `insert_after_cursor`, and a
bidirectional `SequenceCursor` (`new(seq, is_start)`, `init`, `previous`, equality on the current
node only); `NCollection_HSequence` as `HSequence<T>`, a Transient whose sequence sits behind a
lock so all handle holders share it, with one RTTI descriptor per item type named
`NCollection_HSequence`.

Preserved quirks: `Sequence::split` clears and rebinds the destination to this allocator before
it checks the index; `InsertAfter(index, Sequence&)` returns for an empty source before checking
the index while `InsertBefore(0, Sequence&)` raises even for an empty source;
`List::insert_list_before` with an empty source bound to another allocator does not check the
cursor; an end cursor makes `InsertAfter(Iterator&)`/`EmplaceAfter(Iterator&)` prepend and
`List::InsertAfter(List&, Iterator&)` append; the list move constructor leaves the source empty
with its own allocator. Deliberate deviations: the `int` index overloads map to the `usize`
methods (negative indices are unrepresentable); list/sequence cursors name nodes by identity, so
a cursor naming no node of the container reports NoSuchObject where C++ is undefined; closure
emplacement constructs the value only after a position check; `HSequence::append_handle` reports
`Standard_NullObject` for a null handle where C++ dereferences it.

Differential evidence: 122 `ncoll_lists_*` probe scripts (fixed edge cases plus 100 seeded
scripts whose indices straddle the valid range, and three HSequence cases) produce identical
native and Rust output.

### Hash maps

Every public member of `NCollection_BaseMap`, `Map`, `DataMap`, `IndexedMap`,
`IndexedDataMap`, `DoubleMap`, `MapAlgo`, `DefaultHasher` (all pure specializations) and
`Primes` has an api row: 432 rows, and the audit reports 0 missing and 0 stale for these headers.
Newly ported: the OCCT `Iterator` cursors (`DataMapIterator`/`DataMapIteratorMut`,
`MapIterator`, `IndexedMapIterator`, `IndexedDataMapIterator`/`IndexedDataMapIteratorMut`,
`DoubleMapIterator`), the `BaseMap` trait (`NbBuckets`, `Extent`, `Length`, `Size`, `IsEmpty`,
`Allocator`), `DataMap::{bound, contained, contained_mut, seek, change_seek, find_into, exchange}`
and `Index<&K>`, `IndexedMap::{try_emplace, try_emplaced, contained}`,
`IndexedDataMap::{try_bind, contained, contained_mut, indexed_items_mut}`,
`DoubleMap::{find1_into, find2_into}`, `clear_with_allocator` on every map, `buckets_from_int`,
pointer-key hashing, and `map_algo::{unite, has_intersection, intersect, subtract, differ, is_equal}`.

Preserved native behaviour:
- `NextPrimeForMap` evaluates `theN + 1` in `size_t`, so `SIZE_MAX` yields 101.
- The constructors keep a zero bucket count until the first insertion (Rust no longer clamps to 1).
- `DoubleMap::AreBound` scans the key-2 bucket through the key-1 chain link (`Next()`, not
  `Next2()`), so a bound pair that is not first in its key-2 bucket can report false.
- `DoubleMap::Clear(allocator)` always releases its buckets; the other maps release only when
  the allocator changes. `DoubleMap`'s move assignment clears the destination and leaves the
  source empty; the other maps' move assignment swaps (mapped to `exchange`).
- `MapAlgo::Intersect`/`Differ` rebuild the map from a fresh one-bucket map with a default hasher
  (the native `Exchange` step), so the result keeps that hasher and the rebuilt bucket order.
- `IndexedMap::ReSize(int)` clamps a negative size to 0 while the other maps' int overloads raise
  `Standard_OutOfRange`.

Deliberate Rust deviations: int index/bucket overloads map to the `usize` functions (negative
values are unrepresentable; `buckets_from_int` performs the native check);
`IndexedDataMap::RemoveKey` returns `bool` where the native function returns `void`; the
DataMap `operator()` becomes `Index`/`IndexMut`, which panic with the Standard_NoSuchObject text.
Exclusions: destructors, move constructors, the bucket-chain node classes, STL
`begin`/`end`/`cbegin`/`cend`, the protected-constructible `BaseMap::Iterator`, the STL
`IndexedIterator` and raw `Buckets()`; each note names the covering Rust construct.

The `hash_maps` differential probe family compares 260 requests with OCCT bit for bit: prime-table
boundaries, `DefaultHasher` values for every integer specialization and `double`, fixed and
seeded scripts over all five maps (bucket-order traversals through collisions, resizes, clears,
removals, copies and exception classes) and every `MapAlgo` function.

### Flat, ordered and packed maps

All 312 public members of `NCollection_FlatMap`, `NCollection_FlatDataMap`, `NCollection_OrderedMap`,
`NCollection_OrderedDataMap`, `NCollection_PackedMap` and `NCollection_PackedMapAlgo` have api rows
(297 implemented, 15 excluded: five destructors, and the ten ordered-map node constructors and
`delNode`, which are allocator plumbing — the Rust map owns its nodes as allocator-backed slots).
The 269-script differential corpus `ncoll_flat_maps_*` compares OCCT and Rust output exactly for
Robin Hood displacement and growth, ordered bucket selection, insertion order, move semantics,
`Reset`, node traversal, packed signed resizing and the deprecated packed set algebra on 32- and
64-bit keys.

Two defects were fixed. An ordered map built with zero buckets keeps `NbBuckets() == 0` until the
first insertion, as in `NCollection_BaseMap`; the Rust table raised the count to one. Move
assignment of `NCollection_OrderedMap`/`OrderedDataMap` swaps the two maps, so the moved-from map
receives the target's former contents; the Rust `move_assign` emptied the source. It now swaps.
`FlatMap`/`FlatDataMap`/`PackedMap` move assignment still empties the source, as upstream does.

Newly ported: `FlatMap::contained` and `move_assign`, `FlatDataMap::contained`/`contained_mut`,
default (exhausted) iterators for the flat and packed maps, the ordered maps' signed-count
constructors (`with_int_buckets`) and `ReSize(int)` (`resize_int`; a negative count is a
`Standard_OutOfRange` error), `Clear(allocator)` (`clear_with_allocator`), `FirstNode()`/`OrderNext()`
as `OrderedNodeRef`/`OrderedNodeMut` views, the iterators' `Reset`/`Initialize`/`IsEqual`,
`PackedMap(int)` (`from_int`, clamps to one bucket) and `ReSize(int)` (`resize_int`, clamps to
zero), and the twelve deprecated `PackedMap` set members. The deprecated members keep
`#[deprecated]` and forward to `packed_map_algo`, as upstream forwards to `NCollection_PackedMapAlgo`.

Deliberate Rust spellings: the native `More()/Next()/Key()` protocol of the flat-map iterators is
`Iterator::next` returning `Option`. The legacy `OrderedDataMap::Iterator` is the detached
`OrderedDataCursor`, which takes the map on each access. Its `reset(&map)` restarts a bound cursor
at the map's current head, and a default cursor stays exhausted. Native self-aliased set
operations (`aMap.Unite(aMap)`) cannot be expressed under Rust borrowing. `FlatDataMap::Extent`
returns the `usize` count, not a truncated `int`.

### Allocators, buffer, handle and shared

All 105 public members of `NCollection_Allocator`, `_BaseAllocator`, `_IncAllocator`,
`_HeapAllocator`, `_AlignedAllocator`, `_AccAllocator`, `_OccAllocator`, `_Buffer`, `_Handle`
and `_Shared` have api rows (93 implemented, 12 excluded: 3 destructors, 2 OccAllocator move
members, 6 Buffer STL iterator accessors covered by `as_slice().iter()`, and
`IncAllocator::SetThreadSafe`, because Rust allocator handles are always Mutex-guarded).
`NCollection_DefineAlloc.hxx` declares only a macro. The virtual allocator interface is one shared
`BaseAllocator` handle whose policy is chosen at construction; `allocate_raw`,
`allocate_optimal_raw` and the unsafe `free_raw` mirror the native pointer API, and RAII owners
serve safe code.

The earlier incremental arena was not a port (49-byte threshold, first-fit, alignment padding, no
growth). It is replaced by the upstream block engine. Newly ported: `NCollection_AccAllocator`,
`NCollection_AlignedAllocator`, the separate `GlobalHeapAllocator` singleton, `NCollection_Allocator`
(`StandardAllocator`), `NCollection_Handle` (`ObjectHandle`), `NCollection_Shared` (`Shared`),
`Buffer::with_data`/`data_ptr`/`dump_json`, the transient descriptors of Buffer and every allocator
class, and the raw element members of `OccAllocator`.

Native quirks kept:
- `IncAllocator::Reset(true)` (`clean()`) restores the 12 KB default block size and forgets the
  size given to the constructor.
- Incremental raw allocations are not aligned; the bump offset is exactly the requested size.
- A buffer on an `NCollection_AccAllocator` cannot be built. `Allocate` and `Free` call
  `Free(nullptr)` first, and the accumulating allocator raises `Standard_ProgramError` for it.
  `Buffer::new/allocate/free/set_allocator` return that failure, and the buffer does not change.
- `NCollection_AccAllocator::Free` fails for a zero-size allocation at a fresh block's end,
  because `findBlock` cannot reach that address.
- The `"NCollection_AccAllocator::Free: …"` message keeps the whitespace from its
  backslash-continued literal.
- A default `OccAllocator` holds a null handle. It compares unequal to `CommonBaseAllocator`.
- `NCollection_HeapAllocator` rounds sizes up to 4 bytes.
- `_mm_malloc` alignment rules apply (Linux/GCC profile). An alignment of 1 uses malloc. An
  alignment of 2 or 4 becomes 8. An alignment of 0 or any other non-power of two gives NULL.
- `DumpJson` pointer text is `"0xx…"`. `GetPointerInfo`'s short form prepends `0x` again after the
  stream's own `0x`.

Deliberate Rust deviations:
- `Reset` returns `Standard_ProgramError` while RAII or collection allocations are alive,
  because rewinding would alias them.
- RAII byte allocations read as zero. Rust cannot expose uninitialized or reused bytes.
- Typed (over-aligned) allocations from arenas pad to the layout alignment. Upstream never aligns.
- A zero Acc block size is refused. Natively it divides by zero.
- Overflowing byte counts return `None`. Natively they wrap.
- A null-allocator `Buffer::allocate(n > 0)` returns false. Natively it dereferences null.
- Handle accessors return `Option`.
- Mixed-type `OccAllocator` `==` is ambiguous in native C++17. The probe calls the member
  template explicitly.

Differential evidence: 208 `ncoll_allocators_*` cases, all identical. They cover seeded
incremental and accumulating allocate/free/reset walks, reported as block order and offsets.
They also cover aligned-allocator alignments, Buffer on every allocator kind, and the
OccAllocator/Allocator equality and `max_size` tables.

### Trees and filters

All 81 public members of the five headers have `api` rows (76 implemented, 5 excluded: the
`Selector`, `TreeNode` and `UBTree` destructors, the abstract `Selector()` constructor that only
clears `myStop`, and the static `TreeNode::delNode` storage helper, whose sub-branch release
`UBTree::kill`/`clear` perform inside the node arena). Native `TreeNode*` links are arena node
ids: `UBTree::root_id`, `BoundingNode::child_id`/`parent_id`, resolved by `UBTree::child`,
`parent`, `change_child`, `change_parent`; `TreeNode::Gemmate` and `Kill` are `UBTree::gemmate`
and `UBTree::kill` (Kill does not refresh ancestor bounds, as natively; EBTree removal now runs
through it). `Clear(aNewAlloc)` is `clear_with_allocator` on both trees, and `EBTree::as_ubtree`
exposes the inherited members. Calls that dereference null natively (`Root` of an empty tree,
`Child` of a leaf) return `None`.

`NCollection_UBTreeFiller` is newly ported as `UBTreeFiller`: a bit-exact `std::mt19937` (seed
5489; the standard's 10000th-draw value is a unit test) drives both native orders
(`draw % i` and `i - draw % i - 1`), the drop implementation fills pending items like the native
destructor, and `check_tree` reproduces the `Sprintf` text, including glibc's `-nan` balance of a
one-leaf tree (an empty tree, a native null dereference, is a Standard_NoSuchObject error).

CellFilter gains the explicit-dimension constructor, the allocator arguments and both `Reset`
overloads. Preserved quirk: `Reset(double)` writes indices `0..myDim-1` of the size array through
the checked `operator()`, so after `Reset(Array1)` with other bounds (e.g. lower bound 1) it throws
Standard_OutOfRange part-way, keeping the earlier writes and the stored targets; `reset` returns
that error. `Reset(Array1)` copies bounds but leaves the iterated dimension unchanged; a cell
index keeps one entry per size, and range operations iterate only the first `dimension` axes.
KDTree gains the `Array1` build overloads (`build_array`, `build_radii_array`; a short radius
array is a Standard_OutOfRange error leaving an empty tree, where the native tree is left
half-built) and the move operations; `build_radii` now ignores surplus radii like the native
pointer overload.

Differential probe: 251 `ncoll_trees_*` cases (UBTree add/select-with-stop shapes, Gemmate, Kill,
EBTree add/remove/contains, filler orders for 1-700 items in both modes with CheckTree text,
CellFilter scripts with purges, ranges and both Reset quirks, KDTree Array1 builds and moves)
match OCCT byte for byte.

### UTF strings, cursors, items views and forward range

`NCollection_UtfIterator` and `NCollection_UtfString` (with the `NCollection_String` alias
`String8`) are now ported as `UtfIterator<'a, T>` and `UtfString<T>`, generic over the `UtfUnit`
code units `u8`, `u16` and `u32`. `wchar_t` is `WideChar`, which is `u32` (`u16` on Windows). A
NUL-terminated pointer becomes a slice. The end of the slice is the terminator, and storage past
the end reads as zeros. So the unprotected UTF-8 overrun of a truncated sequence (undefined
behaviour natively) decodes exactly as it does over zero-padded memory. Buffer pointers become
code-unit offsets. These native quirks are preserved:

- 5- and 6-byte and overlong UTF-8 forms are decoded without validation. An overlong NUL
  (`C0 80`) ends the text.
- Same-unit copies keep invalid sequences verbatim. For example, `FF 41` becomes a 6-byte
  string that swallows the padding zeros.
- Converted copies drop surrogates and values above U+10FFFF, but still count them in
  `Length()`.
- `operator+` always adds both cached lengths. `operator+=` skips an empty right side.
- `SubString` uses `end - start` as the symbol budget, so a negative start widens the copy.
- `FromLocale`/`ToLocale` implement the POSIX "C" locale that every program starts in. Rust has
  no process locale, which is a deliberate deviation from locale-dependent hosts. `FromLocale`
  converts at most one character, because `mblen` measures only the first one. `ToLocale`
  reports true even when `wcstombs` fails on a non-ASCII symbol. It reports false only for an
  empty conversion, and it leaves a full buffer without a terminator.

`GetUtf*` write into a destination slice and return the count of code units written. A slice
that is too short is a `UtfBufferError`; the native code overruns.

`NCollection_Iterator` is `CollectionCursor`/`CollectionCursorMut` over contiguous storage (its
only native owner is `Array1`). A dereference past the end is an `ArrayError::Index`.
`NCollection_ItemsView::View` is `ItemsView`/`ItemsViewMut` over the `ItemsSource` extractors
(`KeyValueItems`, `KeyValueIndexItems`, `KeyIndexItems`), which are implemented for `DataMap`,
`IndexedDataMap`, `FlatDataMap`, `OrderedDataMap` and `IndexedMap`.

`NCollection_ForwardRange` was already ported. Its rows record one deliberate deviation: Rust
guards advancing or dereferencing an exhausted host, where the native code calls the host
unconditionally.

The utilities probe family (`ncoll_utilities_utf`, `ncoll_utilities_locale`) compares 1,980
cases bit-for-bit with OCCT. The cases cover every UTF-8 lead byte alone and with continuation,
ASCII and truncation tails, and overlong, surrogate and beyond-U+10FFFF encodings. They also
cover UTF-16 lone, reversed and paired surrogates, UTF-32 boundary values and seeded random
sequences. All 1,980 cases are identical.

122 api rows: 103 implemented and 19 excluded. The excluded rows are destructors, move
operations, STL iterator boilerplate, the structured-binding `get<I>()` protocol and
`ChangeBufferHere`.
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

All 266 public members have rows (261 implemented, 5 excluded: move constructor, move
assignment, destructor, the RTTI macro, and the static `IsEqual(string, const char*)`,
which OCCT declares but never defines). The members follow the `.cxx`/`.lxx` line by line.
Positions stay `i32`. Every OCCT exception is a `Failure` of the same class:
`Standard_OutOfRange`, `Standard_NegativeValue`, `Standard_NumericError` or
`Standard_NullObject`. The message adds the offending value and the accepted range.

`TCollection_HAsciiString` is a `Standard_Transient`. Its string sits behind a lock, so
every member takes `&self`, and all handles to the object see an edit. A string argument
is read before the object is locked, so `h->AssignCat(h)` is legal. The three members that
check for a null handle (`IsDifferent`, both `IsSameString`) take `&Handle` and raise
`Standard_NullObject`. The other handle parameters are `&HAsciiString`. OCCT dereferences
those without a check, and a `&Handle` coerces to that type. `TCollection::NextPrimeForMap`
is `tcollection::next_prime_for_map`.

Overloads and operators map as follows:

- The overloads for a pointer/length pair, `const char*`, `std::string_view`,
  `const char(&)[N]` and `TCollection_AsciiString` merge into one `impl AsRef<[u8]>`
  item. A view, a pointer/length pair or a string passes its exact span. A C string
  passes the bytes before its NUL. Rust slices are never null, so OCCT's `nullptr`
  branches have no counterpart.
- `AssignCat` and `+=` take `impl Into<AsciiString>`. A `&str` converts as a C string,
  a byte slice as a span, `u8` as `char`, `i32` as `%d`, `f64` as `%g` and
  `&ExtendedString` as UTF-8.
- `Cat` and `+` take `impl CatOperand`. Its `u8` impl is `Cat(char)`, which appends a NUL
  byte. `AssignCat(char)` ignores a NUL byte.
- The `wchar_t` members use `&[u32]`, because `wchar_t` is 32 bits on Linux.
- The members that take `const char*` only (`Token`, the joining constructor and every
  HAsciiString C-string parameter) read the argument through its first NUL.

These native quirks are preserved and confirmed by the probe:

- `SubString`, `Split` and `Token` copy their result through the first NUL.
- `Location` restarts at the failing byte after a partial match. Thus
  `"aaab".Location("aab")` is 0.
- `Copy(ToCString(), n)` does nothing.
- `SetValue(int, char)` refuses NUL.
- `Insert(int, char)` checks "too big" before "too small".
- `UsefullLength` decodes with `NCollection_UtfIterator`'s permissive UTF-8 reader.
  `C0 80` decodes to 0 and ends the scan, and a lead byte can step over an embedded NUL.
- `IntegerValue` truncates the clamped 64-bit `strtol` result to `int`. Thus
  `-9223372036854775809` gives 0.
- `IsIntegerValue` looks for `'.'` after an embedded NUL too.
- `IsRealValue(true)` accepts a number followed by NUL, and it accepts the empty string.
- `Print` stops at an embedded NUL.
- `Read` takes at most 8189 bytes and does not read the delimiter.
- HAsciiString's `IsDifferent`, `IsSameString(h)` and `IsSameState` use `strncmp`, so
  `"a\0b"` and `"a\0c"` compare equal.
- `NextPrimeForMap(-1)` is 101, because `SIZE_MAX + 1` wraps to 0. Negative values and
  values past the last table prime give `value + 1`, which wraps at `INT_MAX`.

Deliberate deviations cover only inputs for which OCCT has undefined behavior:

- `Remove` with a position below 1 or a negative count raises `Standard_OutOfRange`.
- `SetValue(string)` past `Length() + 1` raises `Standard_OutOfRange`, where OCCT leaves
  uninitialized bytes.
- The fill constructors take `usize`, because a negative length writes before the buffer.
- `UsefullLength` stops when a sequence runs past the terminator, where OCCT reads
  allocation padding.
- `Read` of whitespace-only input gives an empty string and `false`, where OCCT reads an
  uninitialized stack buffer.

`Print`/`Read` use injected `std::io::Write`/`BufRead`. OCCT's default arguments (`Token`,
`ChangeAll`, `Remove`) are explicit.

`tools/tkernel_ascii_string_api_corpus.py` sends 64,231 requests, and all of them match the
native kernel: every member and operator over empty strings, embedded NULs, high-bit and
malformed UTF-8 bytes, positions 0/1/N/N+1 and `INT_MIN`/`INT_MAX`, special reals, invalid
`wchar_t` values, unpaired surrogates, self-aliasing handle edits and null handles.
Exceptions are compared by class. The corpus docstring lists the undefined-behavior inputs
that it never sends. The translated suites keep all their assertions. Only their call sites
changed, to the `i32`/`Result` signatures and to `insert_char`/`set_value_char` for the
`char` overloads.

### TCollection_ExtendedString and TCollection_HExtendedString

Every constructor and member of both headers is ported from the `.cxx` files (169 `api`
rows: 165 implemented, 4 excluded for the move constructor, move assignment, destructor and
`DEFINE_STANDARD_RTTIEXT`). Members that raise return `Result<_, Failure>` with OCCT's class
(`Standard_OutOfRange`, `Standard_NullObject`, `Standard_NumericError`); the message is OCCT's
text followed by the offending value and the expected range.

Overloads and operators map as follows. A `const char16_t*` argument becomes the slice of
units before its terminator; because `extStringLen(nullptr)` is 0, a null pointer behaves like
an empty string in every such member except the constructors and `Token`. The pointer,
`char16_t(&)[N]`, `std::u16string_view` and class overloads of one member share one generic
item (`impl AsRef<[u16]>` or `impl Into<ExtendedString>`, whose `From` impls give the `char`,
`char16_t`, `int` and `double` conversions). Each (pointer, length) core is a `*_span` method
taking `Option<&[u16]>` and the signed length, with OCCT's argument order and null/negative
answers; it reads only the units OCCT reads. `+=` and `+` are `AddAssign`/`Add`, `==`/`!=`
are `PartialEq`, `<`/`>` are `PartialOrd` (unsigned unit order), `operator<<` is `Display`,
the `std::hash` specialization is `HashKey`, and the pointer-taking operators map to the named
methods. Aliased `AssignCat`/`Prepend` pointers into the string itself are
`assign_cat_self`/`prepend_self` ranges, because borrowing rules forbid the alias.

`HExtendedString` is a `Standard_Transient` registered with `impl_transient!`. Its string sits
behind one reader-writer lock, so every member takes `&self` and edits are seen through every
`Handle`. A handle argument is taken as `&HExtendedString`, because OCCT dereferences it without
a check. Arguments are copied before the object's own lock is taken, so `AssignCat` of the same
object is well defined. `String()` returns the read guard, and `ToExtString()` returns a copy
because a pointer cannot outlive the guard. The `std::hash`/`std::equal_to` handle
specializations are the `HExtendedStringHasher` NCollection policy: 0 for a null handle, and
equality for a shared object or equal strings.

Native quirks kept:

- The UTF-8 constructors use `NCollection_UtfIterator`, which does not validate. It sums the
  bytes that the lead byte announces and subtracts the Unicode offset table in unsigned 32-bit
  arithmetic. Only a value above U+10FFFF makes the input "not UTF-8" (then every byte is
  widened). Therefore `C3 41` gives U+0081, a lone continuation byte gives U+0080 to U+00BF,
  and Latin-1 `E9 74 E9` gives U+8D69. Encoded surrogates are dropped, and the overlong
  `C0 80` decodes to zero and ends the string.
- The `AsciiString` constructor decodes only up to the first NUL, but without UTF-8 it widens
  every byte of the `AsciiString` length, embedded NULs included.
- UTF-8 output (`ToUTF8CString`, `LengthOfCString`, `Print`) stops at the first zero unit and
  drops unpaired surrogates. `wchar_t` input (UTF-32 on the Linux profile) drops surrogates,
  values above U+10FFFF and negative values.
- `Split` builds the tail with the `const char16_t*` constructor, so the tail ends at its first
  zero unit. `Token` with an empty separator set or `theWhichOne < 1` gives an empty token,
  and a token ends at a zero unit. `Insert(int, char16_t)` inserts a zero unit, but
  `AssignCat(char16_t)` and `Cat(char)` ignore zero.
- `FirstLocationInSet`/`NotInSet` do not reject `from > to` (the result is 0). With an empty
  set, `NotInSet` returns `from`.
- `HashCode` hashes two bytes per unit (FNV-1a with the signed-`char` quirk below two units,
  Murmur otherwise). Therefore ASCII text does not hash like `AsciiString`, although the
  header says it does.
- `HExtendedString(const char*)` widens bytes, because it uses the default
  `theIsMultiByte = false`.

Deliberate deviations, all at native undefined behavior or at places that Rust cannot express:

- Slices are never null, so the `Standard_NullObject` branches of the `const char*`,
  `const char16_t*` and `Token` separators arguments have no counterpart (`from_span(None, n)`
  keeps the one of the (pointer, length) constructor).
- A UTF-8 sequence cut by the terminator makes OCCT read past the string. The port treats such
  input as not UTF-8 and widens the bytes.
- `Cat(ptr, len)` with a negative length copies the whole string into a shorter buffer when the
  total stays non-negative. The port raises `Standard_OutOfRange`, as OCCT does for a negative
  total. `Remove` computes `theWhere + theHowMany` in i64, where OCCT's `int` sum overflows (the
  native call crashed in the probe). `ToUTF8CString` refuses a buffer that is too short instead
  of writing past it. A span shorter than the units a core must read panics.
- `ToWideString` exists only on Windows (`#[cfg(windows)]`), where `wchar_t` is UTF-16.

`tools/verify_tkernel_api.py --only extended_string` compares 17,382 requests with the native
kernel: every constructor on a UTF-8 corpus of fixed and seeded byte strings, UTF-8 output and
hashes, numeric parsing, every comparison and search overload and span core, `Token`, the set
locators, every edit at the indices `INT_MIN`, -1, 0, 1, 2, N, N+1, N+2 and `INT_MAX`, and
the `HExtendedString` members and handle policy. Exceptions are compared by class. The corpus
docstring lists the undefined-behavior inputs that it never sends. The `HAsciiString`
handle constructor is probed with every UTF-8 corpus string, embedded NULs included.

### Standard scalar, character, hashing, time, GUID and atomic helpers

Headers: Standard_Real (+ .cxx), Standard_ShortReal, Standard_Integer, Standard_Character,
Standard_ExtCharacter, Standard_CString (+ .cxx), Standard_CStringHasher, Standard_HashUtils
(+ .lxx), Standard_Time, Standard_GUID (+ .cxx), Standard_Atomic and Standard_VersionInfo.
138 `api` rows (131 implemented, 7 excluded) replace the former hand-written GUID/Epsilon/IsEven/IsOdd
rows. `tools/verify_tkernel_api.py --only standard_values_` compares 10,983 requests exactly.

Mapping:

- Free functions of a header are attributed to its primary class in the ledger
  (`Standard_Real::ACos(double)`); each is a free function of `standard::real`,
  `short_real`, `integer`, `character`, `ext_character`, `c_string`, `time`, `atomic` or
  `version_info`. The deprecated `<cmath>` one-liners (ACos, ATanh, Cosh, NextAfter, Round, ...)
  are ported too. The inverse hyperbolic functions call the platform libm through
  `rustycad_cmath`, because Rust's own formulas differ in the last bit. `Max`/`Min` keep the
  `std::max`/`std::min` comparison `a < b ? b : a`, not IEEE `fmax`: NaN order and signed zeros
  are kept.
- `standard::hashing` is public: `murmur_hash::{murmur_hash_64a, murmur_hash_2a, hash_combine,
  optimal_seed}`, `fnv_hash::{fnv_hash_64a, fnv_hash_1a, hash_combine, optimal_seed}`, and the
  top-level `hash`, `hash_bytes` and `hash_combine`. The C++ templates hash the object
  representation of a value. The port takes `&[T]` with `T: bytemuck::NoUninit` (no padding, so
  the byte view equals the C++ one). The result type `T` (default `size_t`) is the `HashWord`
  parameter (`u32`, `u64`, `usize`): an 8-byte word selects the 64-bit engine and any other width
  the 32-bit one, as the two `enable_if` overloads do. The crate-internal `murmur64`/`murmur32`
  keep their signatures and delegate to the engines.
- `Standard_CStringHasher` is the unit struct `c_string::CStringHasher` (`hash_code`,
  `is_equal`, and `ncollection::HashPolicy` for any `AsRef<[u8]>` key). C-string arguments are
  byte slices read up to the first NUL. `Atof` wraps the existing `standard::strtod`.
- `Standard_GUID`: `from_ascii`/`FromStr`, `from_utf16`, `from_components`,
  `from_uuid`/`From<Uuid>`, `Copy` for the copy constructor and `operator=`, `PartialEq` for
  `==`/`!=` (`is_same`/`is_not_same` for const contexts), `assign`/`assign_uuid`,
  `to_c_string`/`Display`, `to_ext_string`, `to_uuid`, `shallow_dump` (any `fmt::Write`),
  `check_format` (`None` is the null pointer) and `hash_code` for `std::hash<Standard_GUID>`
  (also fed to `Hash`). The string constructors now return `Failure` of class
  `Standard_RangeError` instead of the legacy `crate::Error`.
- `Standard_Atomic_*` take `&AtomicI32` with `SeqCst` ordering and return the new value, as the
  full-barrier `__sync` builtins do. The header's per-platform definitions collapse into one
  function.

Native behavior kept:

- `Epsilon(x)` is transcribed from its `nextafter` expression. The former `|x|` formulation
  cleared the sign bit of a negative NaN.
- `NextAfter` with a NaN argument returns `x + y`. glibc's build evaluates it with `y` as the
  SSE destination, so when both arguments are NaN the direction's payload wins. The port
  evaluates `direction + value`.
- `ACosApprox` keeps the Standard_Real.cxx literals and operation order, and returns 0 (not
  PI) at -1.
- `IsOdd` is `value % 2 == 1`: negative odd values are neither odd nor even.
- FNV-1a reads plain `char`. Bytes >= 0x80 are sign-extended where `char` is signed (the oracle
  profile). The port widens through `c_char`, so it follows the platform like the C++ code.
- The `std::hash<Standard_GUID>` image packs `Data4[0..2]` into one `char16_t` (high byte
  first) in native byte order.
- `Standard_GUID(const char16_t*)` keeps the low byte of each of the first 36 units.
- The `Standard_VersionInfo` values are those of the generated `Standard_Version.hxx` of the
  pinned build (`8.1.0`, `dev1-3d097a0328`).

Deliberate deviations (the C++ behavior is undefined or not expressible; none is probed):

- `RealToInt(NaN)` returns `INT_MIN`, the x86-64 `cvttsd2si` result.
- `Abs(INT_MIN)` and `Square(int)` overflow wrap in two's complement. `Modulus(x, 0)` and
  `Modulus(INT_MIN, -1)` trap natively (SIGFPE) and panic in Rust.
- A hash length beyond the object panics; C++ reads out of bounds.
- `Standard_GUID(const char16_t*)` with fewer than 36 units returns a `Standard_RangeError`
  naming the length; C++ reads past the buffer.
- `Printf`, `Fprintf`, `Sprintf` and `Vsprintf` are excluded. Stable Rust cannot define C
  variadic functions, and Rust formatting never depends on a locale. The MSVC
  `_Interlocked*` declarations in Standard_Atomic.hxx are excluded for the same reason: they are
  not OCCT API.

Audit tool: `tools/audit_api_rows.py` now reads `extern "C" { ... }` blocks as members of the
enclosing scope, with a regression test. This makes Atof/Strtod/Printf and Standard_VersionInfo
visible. The TKMath and NCollection output is unchanged. OSD_OpenFile, OSD_WNT and
Resource_ConvertUnicode gain their C-linkage functions.

### Standard handles, transients, type descriptors, failures and TColStd handles

Standard_Handle, Standard_Transient, Standard_Type, Standard_Persistent, Standard_Failure,
Standard_OutOfMemory, Standard_ErrorHandler, Standard_Assert, `Standard::StackTrace` and
TColStd_HPackedMapOfInteger are ported (109 rows: 101 implemented, 8 excluded destructors,
moves and RTTI macros). The `standard_runtime` probe compares 488 requests with OCCT.

**Type registry.** `RuntimeType` is `Standard_Type`: `RuntimeType::register` keeps one
descriptor per class in a process-wide registry, returns the first descriptor unchanged
on a second registration, and deep-copies the name up to its NUL. `impl_transient!`
registers each class with `size_of` and its parent, and the descriptors are themselves
transients of type `Standard_Type`. `is_subtype` keeps the native early exit: the walk
stops at the first ancestor smaller than the base. `Print`/`operator<<` write
`0x<address> : <name>`. Deviations: the registry is keyed by the Rust `TypeId`, and
`system_name()` is `std::any::type_name` (C++ keys by and reports the mangled `typeid`
name); `size()` is the Rust size. The probe masks the address and does not compare the
size or the system name. Descriptors are never unregistered.

**Transients and handles.** `Transient` gains `is_instance`/`is_kind` by descriptor and by
name (a C string, so a name ends at its NUL); `this()` raises `Standard_ProgramError` with
OCCT's message for an object with count 0. The `handle` members map as follows:
`Default` (null), `Clone` (copy), `clone_from` (assignment, a no-op when both share the
object, as `Assign`), `Drop` (release; the last release drops the object, the Rust form of
`Delete()`), `Deref` (`->`/`*`; a null dereference panics where C++ is undefined),
`PartialEq` (`==`/`!=` against handles and raw pointers), `PartialOrd`/`Ord` (`<`, by
address, null first), `Hash` and `hash_code` (`std::hash`, the address). All `DownCast`
overloads, including the deprecated same- or unrelated-type ones, are one runtime check
(`Handle::down_cast`, `occ::down_cast`); the compile-time upcast constructors and
assignments are `cast` (one more reference) and `into_cast` (a move, no count change).
A raw pointer to an owned object is a `HandleView`: `Handle::from_view` is `handle(const
T*)`, `operator=(const T*)` and `DownCast(const T2*)`; Rust cannot adopt an unowned
pointer. `reset(new U)` is `reset(value)`. The C++ move assignment swaps the two handles;
Rust moves, and `std::mem::swap` is the counterpart when the source is kept (the probe
compares that swap). The `occ::` helpers take `Option<&dyn Transient>` for a raw pointer.
`StandardPersistent::type_num` returns an `&AtomicI32` for the C++ `int&`, so a shared
handle can write it. TColStd_HPackedMapOfInteger wraps the NCollection `PackedMap<i32>` in
a lock (`map()` / `change_map()` guards) so edits through one handle reach every holder; a
bucket count below 1 means 1.

**Failures.** `Failure::what()` returns bytes: the message, or the own buffer of
Standard_OutOfMemory. Copies share the message and stack strings (`Arc`), as the
reference-counted `StringRef`. The message constructors (`Failure::new`,
`Failure::with_message` and every exception constructor) capture a stack trace when the
process-wide `default_stack_trace_length()` is positive, into a buffer of
`clamp(length*200, 2048, 65536)` bytes (saturating where C++ `int` would overflow); the
`(message, stack)` constructor `with_type` never captures. `exceptions::out_of_memory`
keeps its message in a 1024-byte buffer cut to 1023 bytes (possibly inside a UTF-8
sequence) and at a NUL; the base message stays empty, so `Display` (`operator<<`) prints
only `Standard_OutOfMemory` plus any stack trace. `set_message_string` replaces the buffer
and refuses, with Standard_TypeMismatch, a failure of another class.

**Stack traces.** `standard::stack_trace` ports the glibc branch of `Standard::StackTrace`
with the same `backtrace()`/`backtrace_symbols()` calls and the same buffer arithmetic:
`"\n==Backtrace=="`, one `binary(symbol+0xoffset) [0xaddress]` line per frame that fits,
and `"\n============="` when room remains. The frames are Rust frames, so the probe compares
only the argument checks and traces where no frame or every frame fits; the `theContext`
argument, ignored on Linux, has no parameter. Deviations: a buffer already longer than its
size (a C++ overflow) and a skip count beyond the stack depth (a native SIGSEGV, since C++
passes a negative count to `backtrace_symbols`) return false; platforms without glibc's
`backtrace` return false (C++ implements Windows with DbgHelp).

**Error handler.** The prebuilt kernel is built with `OCC_CONVERT_SIGNALS`, so the Linux
behavior is ported (the probe defines the macro too). Handlers form a thread-local stack of
frames: `ErrorHandler::new` links, `destroy`/`Drop` unlink every handler above, pop, and run
the registered callbacks newest first. Destroying a handler that is no longer linked
unlinks the whole stack, and a callback left registered stays bound to its destroyed
handler, so `register_callback` is then a no-op: both are native quirks the probe
compares. There is no `longjmp`: `ErrorHandler::abort` stores the failure in the innermost
handler and returns it as `Err`, and `?` carries it to `ErrorHandler::catch_signals`, the
`OCC_CATCH_SIGNALS` block (the `Label()` row), which raises the stored failure whatever
the body returned, as the jump always lands. `abort` accepts only the 23 classes of the C++
`SignalException` variant (others are Standard_TypeMismatch instead of a compile error).
`abort` without a handler and `raise` without a failure print OCCT's `*** Abort ***` text
to stderr and exit with status 1, as C++. A callback's `DestroyCallback()` override is the
closure given to `ErrorHandlerCallback::new`.

**Asserts.** The reference kernel is a release build, where `Standard_ASSERT` only runs its
action. `standard_assert_report` writes the POSIX debug text of `Standard_ASSERT_REPORT_` to
a given stream and returns true; `standard_assert_raise` builds the Standard_ASSERT_RAISE
`Standard_ProgramError`; `StandardStaticAssert::<true>::assert_ok` exists only for a true
constant.

The probe excludes C++ undefined behavior and run-dependent values: descriptor sizes,
system names and addresses, handle ordering and hashes, stack traces whose frames only
partly fit, prefixes that overflow the trace buffer, large stack lengths, and the
process-exit paths of `Raise` and `Abort` (covered by Rust tests in a child process).

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

## Units API completion

`python3 tools/audit_api_rows.py src/FoundationClasses/TKernel/<Package> --source ../OCCT`
reports 0 public members without api rows and 0 stale rows for `Units` (217 rows: 207
implemented, 10 excluded RTTI macros), `UnitsAPI` (34 implemented) and `UnitsMethods`
(10 implemented).

The condensed evaluator was replaced by a class-by-class port: `Dimensions`, `Token` and
`ShiftedToken`, `Unit` and `ShiftedUnit`, `Quantity`, `Lexicon` and `UnitsLexicon`,
`UnitsDictionary`, `Sentence`, `UnitSentence` and `MathSentence`, `Measurement`,
`Explorer`, `UnitsSystem`, the `Units_Operators` free functions (`units::operators`) and the
`Units_NoSuchType`/`Units_NoSuchUnit` exception classes (`UNITS_HIERARCHY`, both
`Standard_NoSuchObject`). Transient classes are `Handle<T>` with lock-protected fields,
because the sentence analysers edit shared tokens in place. The virtual members
(`Creates`, `Multiplied`, `Divided`, `Dump`, `Token`) dispatch on the handle's dynamic
type through `TokenHandle`, `UnitHandle` and `LexiconHandle`. `Units_Token::Divide`
returns `this` for a near-zero divisor, so it is a `TokenHandle` member too.

Process-wide statics become injected contexts:

- `units::Units` holds the dictionary, both lexicons, the null dimensions, the
  `ToSI`/`FromSI` last-unit cache, the `FirstQuantity` cache and the
  `Units_Dimensions::Quantity` sequence cache. Every class that reads them natively takes
  the context (`UnitSentence::new(&units, ...)`, `Dimensions::quantity(&units)`).
  `std::cout` text and `Message::SendWarning`/`SendInfo` payloads go to an injected
  `UnitsDiagnostics`; `Dump` members write into a `&mut dyn Write`.
- `units_api::UnitsApi` holds the current/SI/MDTV resource catalogs, the local units
  systems and the selected systems, and reads resources through the injected
  `ResourceIo`/`ResourceEnvironment`.
- `units_methods::UnitsMethods` holds the cascade length unit in an atomic, as the native
  static does.

Preserved native behaviour:

- The caches start as the zero-initialized statics: `ToSI(x, "")` before any other unit
  returns 0 (and `FromSI` returns infinity), and `FirstQuantity("")` answers `""`.
- `Units_Dimensions::Quantity` searches the quantity sequence captured on its first call.
- `UnitsDictionary::Creates` keeps the native `sscanf` field widths, the `coeff`/`move`
  locals that survive from one line to the next, and the matrix propagation that counts
  every assignment. Recreating the dictionary re-adds every symbol to the units lexicon,
  so each meaning becomes `UU` and the duplicate warning prints for every symbol.
- `Units_Sentence::Evaluate` is destructive. An operator-free remainder evaluates to a null
  token. Index errors are `Standard_OutOfRange` with the native messages
  (`NCollection_Sequence::Value`, the `removeSeq` messages, and the empty `Exchange`
  message that `sq.km` raises, OCC23497).
- `Units_UnitsSystem::Remove` of the last unit of a quantity removes the quantity and then
  raises `Units_NoSuchUnit`. `Activate` bounds its unit search by the number of
  quantities, so it can raise `Standard_OutOfRange` or miss an existing unit.
  `ConvertUserSystemValueToSI` of a shifted unit computes `value * (factor + move)`.
- `UnitsAPI::Check` always answers false. `CurrentUnit` of a quantity without a current
  unit raises `Resource_NoSuchResource`.
- `Units_Measurement::Integer` truncates with the x86-64 `cvttsd2si` result.
  `GetCasCadeLengthUnit(Inch)` after `SetCasCadeLengthUnit(1)` is `25.4 * (1 / 25.4)`, not 1.

Deliberate Rust deviations:

- A null handle that the native code dereferences unchecked is a `Standard_NullObject`
  failure. This covers `AnyFromLS` and the dimensions overload of `AnyToLS` for an
  invalid unit, and tokens left null by empty brackets.
- A dictionary section whose conversion graph never connects is an error. Natively the
  propagation loops forever, which is what recreating the dictionary triggers.
- Unit symbols are `AsciiString` values in the shared symbol sequence. The native
  `TCollection_HAsciiString` handles are never edited after creation.
- The named `Units_UnitsSystem` constructor loads and drops the resource catalog.
  `ResourceCatalog` has no verbose mode, so the `verbose` flag is accepted and ignored.
- The default `ConsoleDiagnostics` prints messages without the native console colours.

Differential evidence: the `units_*` probe family (`tools/oracle/tkernel_units_probe.cxx`,
`crates/tkernel/examples/oracle_probe/units/`, `tools/tkernel_units_corpus.py`) runs 1,464
requests in one long-lived process on both sides, so the package, UnitsAPI and
UnitsMethods state evolves identically. Captured console text (with token addresses
masked) and messenger payloads are compared with the results. The requests cover every
dictionary symbol as a sentence and as a `ToSI`/`FromSI` conversion, composite, prefixed,
shifted, numeric and invalid expressions, formulas, raw lexicon sentences, every quantity
look-up, dimension arithmetic, token operators and copies, measurements, units and
quantities, explorer walks, a units-system session with the quirks above, 552 UnitsAPI
calls across the SI/MDTV/DEFAULT systems with current-unit overrides, every UnitsMethods
table entry, and the dictionary and lexicon dumps. All 1,464 are identical; with them the
combined `verify_tkernel.py` run checks 180,189 differential cases.
`DictionaryOfUnits(true)` is not in the corpus: natively it never returns (see above).
Behaviour tests are in `units_classes_api_behavior`; resource I/O uses the named fake
`InMemoryUnitsResources`.
