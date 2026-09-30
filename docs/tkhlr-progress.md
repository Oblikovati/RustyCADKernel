# TKHLR port status

**Intrv, TopBas, TopCnx and HLRAlgo are ported; the other packages follow in the order below.**
`crates/tkhlr` (`rustycad-tkhlr`) is the port of the ModelingAlgorithms toolkit TKHLR.
It depends on the TKernel, TKMath, TKG2d, TKG3d, TKGeomBase, TKBRep, TKGeomAlgo and
TKTopAlgo ports, as the native toolkit does (its EXTERNLIB). Each of the 8 OCCT packages
is one module of the crate, declared up front in `src/lib.rs`; a module whose package
is not ported yet holds only its documentation. Ported: Intrv, TopBas, TopCnx, HLRAlgo.

[hlr-design.md](hlr-design.md) fixes how the C++ constructs of the packages map to Rust
and gives the class order of the HLRTopoBRep, HLRBRep and HLRAppli work.

A package may start when the packages it includes are ported (the lower toolkits are
ported):

| Level | Packages |
| --- | --- |
| 1 | Contap, Intrv, TopBas, TopCnx |
| 2 | HLRAlgo |
| 3 | HLRTopoBRep |
| 4 | HLRBRep |
| 5 | HLRAppli |

The toolkit test inventory has two suites. Both test HLRBRep classes, so they wait for
the HLRBRep port and are not mapped yet:

| Source | Rust target | Declarations | Native | Rust |
| --- | --- | ---: | --- | --- |
| HLRBRep_Data_Test.cxx | waits for HLRBRep | - | - | - |
| HLRBRep_InternalAlgo_Test.cxx | waits for HLRBRep | - | - | - |

## Numeric corpus

The numeric oracle (`tools/verify_tkhlr_numeric.py`) compares
`tools/oracle/tkhlr_probe.cxx`, linked with libTKHLR and the toolkits below it, with
`crates/tkhlr/examples/numeric_probe.rs` on the requests of `tools/tkhlr_corpus.py`, in
64 round-robin shards. The frame family (`hl_frame`, 23 requests) compares the
transformation to the viewing frame that `HLRAlgo_Projector(const gp_Ax2&)` stores and
its inverse; it proves the pipeline before the HLR classes are compared.

The Intrv families (`tools/tkhlr_corpus_intrv.py`, 1034 requests) compare the three
`Intrv_Interval` constructors on zero, signed-zero, subnormal, huge and RealFirst/RealLast
bounds with negative, zero, tiny and float-overflowing tolerances, the six bound edits,
`Position` and the 13 `Is*` tests on a grid around a reference interval and on seeded
random pairs, the eight boolean operations of `Intrv_Intervals` on fixed and random sets
(empty sets and tools, touching, nested, split, infinite and reversed intervals), and
`Value` inside and outside the sequence.

The TopBas and TopCnx families (`tools/tkhlr_corpus_topcnx.py`, 227 requests) compare
`TopBas_TestInterference` set by its setters, by its constructor and through
`ChangeIntersection`/`ChangeBoundary`, and `TopCnx_EdgeFaceTransition` before any reset,
after both resets and over seeded sequences of interferences (every orientation,
transition and boundary transition; tangent, normal and opposite directions;
curvatures and tolerances), with `Transition` and `BoundaryTransition` after each step.

The HLRAlgo families (`tools/tkhlr_corpus_hlralgo.py` and
`tools/tkhlr_corpus_hlralgo_poly.py`, 525 requests) compare the 16-direction boxes and
their packing (in and beyond the 15-bit fields), the six `HLRAlgo_BiPoint`
constructors and its flags, the edge and wire blocks with out-of-range ranks,
`HLRAlgo_EdgeStatus` sequences with the visible and hidden walks of
`HLRAlgo_EdgeIterator`, the interference records, and `HLRAlgo_Projector` on the four
standard views, skew, scaled and translated frames, in parallel and in perspective
(every projection, `Shoot`, `Transform`, `Set`, `Scaled`, and the null sight and zero
scale failures). The polyhedral part compares `HideByPolyData` on faces with integer
coordinates (segments through vertices, along edges and in the face plane) and random
ones, with and without connexity and with packed boxes that pass or fail,
`HLRAlgo_PolyAlgo` over one to three shells (Update with its packed boxes, hiding
planes and edge flags, then the hide and show walks), `HLRAlgo_PolyInternalData` on
grid triangulations (UpdateLinks, AddNode, the split UpdateLinks on interior and
boundary segments with array growth, a segment that is not there, the flags and
`Dump`), and the `Inc`/`Dec` members with shared and distinct array pointers.

## Intrv quirks kept

- The default interval is `[RealFirst, RealLast]` with tolerances 0:
  `Epsilon(RealFirst())` and `Epsilon(RealLast())` are 0 (no double lies beyond them).
- `FuseAtStart` and `CutAtStart` do nothing when the start is exactly RealFirst, and
  `FuseAtEnd` and `CutAtEnd` when the end is RealLast, whatever the other bound.
- The tolerances are `float`: two tolerances add in single precision before they are
  compared with a double difference, and a fused tolerance is `(float)(b - a) / 2`.
  `SetStart` and `SetEnd` keep a zero or negative tolerance; only the constructor with
  tolerances raises them to `(float)Epsilon(bound)`.
- `Position` and the `Is*` tests are separate formulas (double envelopes against float
  sums), so at ties they can disagree.
- `Subtract` and `Unite` leave their walk at the first interval that lies after the
  tool, so on intervals that are reversed or carry negative tolerances (possible through
  `SetStart`/`SetEnd`) the set can stay unsorted or overlapping, as natively. `Intersect(tool)` is `Unite(tool)` minus
  the exclusive union, and `XUnite(tool)` is `tool` minus the set united with the set
  minus `tool`, each with these walks.

Deviation: a native call whose tool is the object itself (`x.Unite(x)`) edits the
sequence while it walks it; Rust borrowing forbids that call (docs/hlr-design.md
section 3).

## TopBas and TopCnx quirks kept

- A new `TopCnx_EdgeFaceTransition` answers FORWARD before its first reset: its
  `TopTrans_CurveTransition` starts with `Init` false and FORWARD transitions, so the
  states read OUT then IN.
- `BoundaryTransition` is EXTERNAL when the FORWARD and REVERSED counts are equal, on
  both branches of the native parity test; INTERNAL and EXTERNAL boundary transitions
  are not counted.

Deviation: the default `TopBas_TestInterference` is `0.0`, `0` and FORWARD, where the
native default constructor leaves the fields uninitialized (docs/hlr-design.md
section 4); the probe never reads a default record.

## HLRAlgo quirks kept

- `HLRAlgo_Projector::Project(P, Pout)` computes but never stores the projection for the
  axonometric view of type 0, so `Pout` keeps its value; `Project(P, X, Y, Z)` uses
  `0.7071067811865475` for that view and `0.7071067811865476` for the others.
- A default projector keeps zero directions (its constructor does not call
  `SetDirection`). Without `Scaled(true)` the projection drops the scale, and a
  parallel projection drops the translation too: `Transformation` differs from
  `FullTransformation`, and only then are the four standard views recognized.
  `Focus` raises Standard_NoSuchObject for a parallel projection.
- `MinMaxIndices::Minimize` lowers the maximum array as well, and `Maximize` raises
  the minimum array; `EncodeMinMax` keeps 15 bits of each index; `SizeBox` multiplies
  14 extents (the 8 minimum fields and the first 6 maximum fields).
- `HLRAlgo_EdgeStatus::Initialize` keeps the visible intervals of the previous state
  (unused while all is visible); `Hide` ignores `OnBoundary` and hides nothing with
  `OnFace`; `VisiblePart` answers the whole range for any rank while all is visible.
- The hidden walk of `HLRAlgo_EdgeIterator` skips the gaps whose fuzzy bounds meet.
- `HLRAlgo_PolyAlgo::Update` keeps the plane normal of the previous hiding triangle of a
  face for a degenerate triangle (the native locals live across the triangle loop), and
  packs the boxes with `(int)` conversions that give `INT_MIN` for NaN or out-of-range
  values (an empty global box).
- `HideByPolyData` reads entry 1 of the hiding array even when the array is empty
  (Standard_OutOfRange), and `hideByOneTriangle` keeps at most two crossing points.
- `HLRAlgo_PolyInternalData(nbNod, nbTri)` reads node 1 before creating the nodes, so
  no node raises Standard_OutOfRange; the arrays double when full (from index 1), and
  a node array of size 0 cannot grow (`IncPINod` reads entry 1 first).
- `AddNode` gives a node the normal `(1, 0, 0)` when the interpolated normal is null.
- The split `UpdateLinks` grows the arrays before it looks for the segment; when the
  segment is not there only the counts are restored.

Not compared (the native code reads uninitialized memory there): a default
`MinMaxIndices`, the edge and flag entries of a new `HLRAlgo_EdgesBlock` before they
are set, the enumerations of a default `HLRAlgo_Intersection` and
`HLRAlgo_Interference`, the POD entries of the poly arrays before they are written, the
distance tolerance of `HLRAlgo_PolyAlgo::Hide` before `Update`, a hidden walk over a
status with no visible interval that is not all hidden, `VisiblePart` outside the
intervals, and a split `UpdateLinks` whose neighbour triangle does not hold the
segment (the opposite node `itpk` stays uninitialized; the port reads 0). Negative
block sizes make the native allocation fail; the port returns Standard_RangeError.

Deviations: the transients return copies or write guards where the native accessors
return references, and take `&mut` handles where the native members take array
pointers (docs/hlr-design.md section 5); a null handle or an out-of-range read that
crashes or is undefined natively is a Standard_NullObject or Standard_OutOfRange
failure.

## Recovered Contap records

`Contap_Point`, `Contap_Line`, `Contap_IType`, and `Contap_TFunction` are ported
with 42 audited API rows. The 1,120 new native comparisons exercise every point
flag combination and transition pair, null and non-null handles, all four line
kinds, duplicate parameter ordering, shared vertex edits, reset detachment, and
shared walking-point clearing. The complete numeric corpus passes 2,929 cases.
Native uninitialized line type/radius are documented deterministic Rust defaults;
native null walking dereferences become `Standard_NullObject`. Neither has a
defined native value to compare. Contour algorithms and HLRBRep tests remain pending.

`Contap_ContAna` now implements all nine sphere/cylinder/cone contour overloads
and its five result accessors. An additional 4,086 native comparisons cover direct
and indirect frames, draft angles, near-axis directions, zero/tiny/invalid radii,
perspective eyes, fresh/reused state, and all accessor error branches. The full
TKHLR numeric corpus passes 7,015 cases. Six focused analytic tests supplement it.

`Contap_SurfProps` adds all three normal/derivative APIs, with 1,184 new native
comparisons over elementary surfaces, indirect frames and generic Bezier patches.
The complete corpus passes 8,199 cases. The distinct cone-apex thresholds and
normal choices are retained. Six focused Rust tests also cover these contracts.

## Contap_HCurve2dTool

All 24 callable APIs are mapped with zero header audit gaps or stale rows. The
adaptor forwarding and Contap-specific sample-count formula pass 600 additional
native comparisons (8,799 total), including rational curves and zero/reversed
sample ranges. Three focused Rust regressions pass. The native 2D curve reader
is shared with TKGeomAlgo to preserve input construction. Null handles return
explicit Rust errors instead of native null dereferences.

### Contap_HContTool recovery

All 18 public members now use native contour sampling and arc behavior. The surface
range remains shared across instances, including partial writes when virtual adaptor
calls fail. The B-spline grid preserves the original asymmetric index/divisor rules.
Arc operations reuse the identical IntPatch implementations. The numeric corpus now
passes 11,787 exact requests, including 2,988 new requests for this class. Three direct
regressions cover shared range, null surfaces, and empty restriction results; crate
unit/integration tests and doctests pass. No standalone original declaration exists
for this helper; TKHLR's four original legacy declarations remain pending.

### Contap_SurfFunction recovery

All 24 public members now reproduce the original surface contour function. The
port preserves untouched DraftPrs output, tangent cache reuse across mode setters,
normal-based scaling, singular-point failures and first-failure capture in solver
callbacks. All 12,219 numeric requests pass exactly (432 new requests), the crate
suite and three direct regressions pass, and strict Clippy passes. These helper APIs
have no standalone original test declarations; overall toolkit completion is pending.

### Contap_ArcFunction recovery

All 16 public members now preserve native arc evaluation, saved-point numbering and
callback outputs. The arc and surface functions share their identical normal scale
and equation calculations. The exact corpus passes 13,083 requests (864 new), all
crate tests/doctests and strict Clippy pass, and three direct regressions cover draft
output retention, saved-state resets and failure capture. The default quadric member
is retained without asserting meaningful geometry for native uninitialized fields.

### Contap walking-line recovery

All 28 callable walking-line members now forward shared IntPatch behavior while
retaining the Contap runtime type. OCCT 3d097a0's Contap header writes the end-tangent
flag in SetTangencyAtBegining; both direct and trait calls preserve that bug. The
numeric corpus passes 13,139 exact requests (56 new scripts); three direct regressions,
all crate tests and strict Clippy pass. Header audit has no missing or stale rows.

### Contap search records

The 11 path-point and eight segment APIs reuse the matching native IntStart
specializations. The corpus passes 13,427 exact requests, including 288 new record
requests. Direct regressions and strict Clippy pass; both header audits have no gaps.

### Contap interior search

All seven public APIs now instantiate the original IntStart search with the contour
surface function. The exact corpus passes 13,571 requests (144 new), including sampled
search, explicit starts, repeated performs and unset-result exceptions. Crate tests,
two direct regressions and strict Clippy pass; the header audit has no gaps.

### Contap boundary search

All eight public APIs now use the native IntStart restriction search with Contap
functions and records. The exact corpus passes 13,787 requests (216 new), with
complete point/segment output, repeated performs and all-arc cases. Two direct tests,
all crate tests and strict Clippy pass. The probe registration parser now tolerates
Rustfmt line breaks; all 253 tooling tests pass.

### Contap walking algorithm

All nine public walking APIs now instantiate IntWalk with the Contap function and
walking-line behavior. All 14,003 exact requests pass (216 new walks), including
boundary/interior seeds, both Perform overloads, reversal and hole filling. Complete
polyline and single-point results are compared. Two direct regressions, all crate
tests and strict Clippy pass. Contap_Contour orchestration remains unported.

### Contour orchestration: private math helpers

The first Contap_Contour helper translations pass 576 exact comparisons against an
isolated translation unit that includes the unchanged native source. The gate records
its source hash and compiler command. It covers periodic recadrage, signed transitions,
analytic projection, zero-radius circles and observable function state. Strict Clippy
passes. These helpers have no public API or original test completion claims; the main
Contour algorithm remains incomplete.

The private LineConstructor translation now passes another 576 exact requests
(1,152 combined), comparing every output vertex and walking point. Cases cover
empty and repeated vertices, tiny circle spans, closing-circle intervals, duplicate
walking points, restriction passthrough and domain rejection. These remain private
helper results, not completion of Contap_Contour.

Boundary attachment and interior-point exclusion now pass 72 additional exact
requests using real Contap searches (1,224 combined). Every emitted point, transition
and vertex flag is compared; singular normals and coincident restrictions are included.
Strict all-target Clippy passes. Main contour orchestration remains incomplete.

Restriction-curve internal-point search now passes 288 additional exact requests
(1,512 combined), including bisection and marking or inserting internal vertices.
All resulting vertices are compared on six real surface types with four initial
vertex arrangements. Strict all-target Clippy passes.

Restriction-segment processing passes 432 more exact requests (1,944 combined),
including repeated endpoint matches, shared vertex mutation, negative tolerance and
internal-vertex insertion. Strict all-target Clippy passes. Contap_Contour itself is
still unregistered while its orchestration is being completed.

### Elementary contour orchestration

The analytic branch now passes 624 public-Perform comparisons on planes, spheres,
cylinders and cones (2,568 combined private/orchestration requests). Fixtures cover
translated frames, trimmed domains, degenerate radii, perspective and draft modes,
and repeated calls. Complete analytic geometry, vertex flags and restriction identity
are compared using the existing record probes. Strict all-target Clippy and 11 focused
tooling tests pass.

A translated-sphere case caught the native left-associative `parameter + PI + PI`
closing-circle calculation; replacing it with `parameter + (PI + PI)` changes one bit.
The source operation order is now retained. The probe also preserves Init's implicit
vector-to-direction normalization. General surface orchestration remains incomplete,
so no Contap_Contour API or original TKHLR legacy declaration is claimed complete.

### Walking-line internal points

The native shifted root search and walking-point insertion now pass 1,296 differential
requests (3,864 combined contour cases). The corpus exercises 34 insertion results,
40 results with internal vertices, 48 native failure diagnostics and 324 empty-line
exceptions. Diagnostic text is compared exactly through an injected output writer.
Strict all-target Clippy passes; general Contap_Contour remains incomplete.

### Complete Contap contour orchestration

All 19 Contap_Contour public members are translated, including the general search,
walking endpoints, multiple vertices, restriction joins and mutable SurfaceFunction.
The isolated contour oracle passes 4,392 exact requests: 504 complete analytic/general
searches, 24 overload/state cases, and 3,864 helper cases. The separate unchanged-source
tangency oracle passes 8,100 exact requests, including rejected destination groups.
Five direct regressions cover initialization, all overloads, result access, failed
normalization and retained surface-function mutation. The four original HLRBRep
legacy declarations remain pending their exact hidden-line dependencies.

Integration checkpoint: `cargo test --workspace` passed 11,443 tests with zero
failures and 69 existing ignored doctests. Strict TKHLR all-target Clippy passed;
the toolkit's 14,003-case public numeric oracle and 8,100-case tangency oracle also
passed. Counts of Rust tests are supplementary evidence, not legacy declarations.
