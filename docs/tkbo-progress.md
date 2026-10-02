# TKBO port status

Toolkit acceptance is incomplete. The dependency-first port now implements six
IntTools foundation types: Range, Root, BaseRangeSample, CurveRangeSample,
SurfaceRangeSample, and MarkedRangeSet. The 80 API rows use the header audit
identities; the remainder of IntTools and the Boolean algorithms remain pending.

`tools/verify_tkbo_foundations.py --source ../OCCT --native-build BUILD` passes
19,109 exact native comparisons, including signed zero, all root transitions,
range subdivision and hashing, duplicate boundaries, pave-block splitting and
common-block ordering. Its companion radius-status replay still records 176 undefined-status
native calls and 32 differences; complete TKBO parity remains false.

BOPDS IndexRange, Pave, Pair, Point, PaveBlock, CommonBlock, CoupleOfPaveBlocks,
ShapeInfo, FaceInfo and DataStructure add 230 API rows.
Their comparison, hash, mutation, splitting, shared-handle and Dump behavior are checked against native
calls, including signed zero. PaveBlock's six focused Rust regressions cover default
sentinels, endpoint matching, deduplicated extras, strict parameter tolerance, ordered
interval updates and shrunk-data storage. These additions do not map an original TKBO
test declaration.

CommonBlock's four Rust regressions cover native first-block ordering, shared-handle
identity and mutation, duplicate face indices, empty-list exceptions, and exact Dump
text. Its twenty callable header entries have API rows; allocator storage and RTTI
metadata are recorded as excluded.

CoupleOfPaveBlocks adds fourteen callable rows for sentinel indices, nullable shared
handles, output-parameter copying and raw tolerance storage. Two Rust regressions and
native differential cases cover the constructor and setter paths; the default
destructor is recorded as Rust Drop glue.

ShapeInfo adds twenty-one callable rows for shape type, box, subshape list, reference,
flag and interference queries. Three Rust regressions and 72 native type/state cases
cover the BRep participation rules, signed flags, bound gaps and exact Dump text.

FaceInfo adds sixteen callable rows for its In, On and section maps. Its two Rust
regressions and native differential probe cover identity-keyed handle maps, unique
vertex sets, and `Clear()` retaining section state while clearing In and On.

DataStructure adds the complete `BOPDS_DS` header inventory. Its initialization pass
prepares vertex, edge and face bounds, flattens face and single-solid child indices,
builds shared-vertex edge links, and initializes endpoint pave blocks on demand.
Native differential probes cover depth-first ordering, shared-vertex links and paves,
face/edge/vertex flattening in a single-solid box, and the resulting shape bounds.
Two-sided and one-sided infinite line ranges also match the native synthetic endpoint
vertices and pave parameters; internal finite-edge paves use the topological endpoint
orientations. Forward and reversed one-vertex pcurve-only and 3D circle seam edges produce
the native duplicate endpoint index and full-period pave interval; their curve and pcurve
presence also match. Open/inverted shells and same-domain mapping remain incomplete.
No original GTest declaration is mapped by this storage slice.

IntTools Curve, CommonPrt and point-on-face records add 62 implemented APIs.
The cumulative oracle passes 3,027 cases. `IntTools_PntOnFace::IsValid()` is
declared but has no native definition: a direct link fails with an undefined
reference. Its ledger status is `upstream_unimplemented`, an explicit acceptance
gap. The separately implemented native `Valid()` method is ported.

CurveRangeLocalizeData adds eight APIs and preserves native map iteration order
through resize, cache replacement and exclusion behavior; cumulative numeric
coverage is 3,099 cases. The missing IsValid symbol now has a permanent compiling
probe and a verifier that requires precisely that linker failure.

SurfaceRangeLocalizeData adds 34 APIs; TopolTool adds 11, with all nine native
surface kinds, adaptive sampling and retained-array behavior exercised. The
integrated 4,043-case native oracle passes. FClass2d and the ten original FaceFace
test declarations remain pending; no Boolean algorithm acceptance is claimed.

FClass2d adds seven callable APIs and a destructor mapped to Rust Drop. Its own
5,051-case native gate passes, including repeated Init, infinite UV bounds,
periodic surfaces, open/missing-pcurve wires and native fallback tolerances.
Context and FaceFace remain in progress.

Context now maps all 30 callable APIs, plus the destructor to Drop; the native
IntermediatePoint helper is also ported. Shared cache identity, allocator ownership,
projection, block validity and vertex-on-curve behavior pass the integrated
7,919-case native gate and its required missing-IsValid-symbol check. FaceFace
legacy declarations are still pending the remaining Tools/WLineTool chain.

### Complete IntTools_FaceFace

The Tools/WLineTool/Context chain now supports the full FaceFace Perform and curve
construction paths. All 15 FaceFace header rows are mapped, including the analytic,
restriction and walking branches, approximation retries and tolerance recomputation.
All ten original declarations pass in both native OCCT and Rust without skips.
The integrated source gate passes 14,447 exact numeric requests; the separate private
helper gate passes 7,434. Native wrappers include the unchanged source and record its
hash and compiler command. The generic toolkit verifier now supports TKBO.

TKBO as a whole remains incomplete: 10 of its 299 discovered GoogleTest declarations
are mapped. The missing native IsValid symbol remains an explicit upstream gap.

### Edge working ranges

IntTools_ShrunkRange adds twelve callable APIs with an ownership mapping for its
destructor. The integrated Boolean foundation oracle passes 15,095 exact requests,
including 648 new range cases. Six direct regressions cover tolerance spheres,
unsplittable short ranges, repeated failures, retained length and box accumulation.
The ten original FaceFace declarations remain the only mapped TKBO originals.

### Edge/edge private foundations

Scalar subdivision, box distance, curve priority, deflection, closure and resolution
helpers, projection distances, and golden-section extrema searches and curve/box parameter bounds, coincidence checks and intersection parameter selection and analytic line intersections and recursive solution ranges pass 10,028 exact comparisons against the unchanged IntTools_EdgeEdge source.
Conic, spline and offset families are covered. Negative subdivision counts retain
native behavior; their corpus uses bounded resolutions to avoid enormous native
allocations. These private helpers add no complete public API or legacy-test claim.

### Public edge/edge intersection

IntTools_EdgeEdge now has all 18 callable public APIs and an ownership mapping for
its destructor. Its five Rust state regressions and 1,362 exact public native
histories cover construction, located edges, repeated execution and sticky errors.
The existing 10,028 helper cases cover the intermediate searches. Default IsDone
is true, later validation errors remain set, repeated successes can accumulate
results, and whole-range matches clear earlier results, following the pinned code.
No standalone original EdgeEdge declaration exists; TKBO remains 10/299 mapped.
The regular TKBO gate passes 16,457 exact numeric comparisons and all ten mapped
original declarations in both native and Rust executions. The complete header
audit has no missing or stale EdgeEdge rows.

### IntTools namespace algorithms

Seven utility APIs are mapped, with GetRadius explicitly partial. The defined-domain
corpus adds 2,560 exact requests, bringing the routine gate to 19,017. A mandatory
companion replay retains 176 coincident-sample calls that read an uninitialized
native gce_MakeCirc status. Thirty-two differ in the current run; none of the 176
counts as exact parity, including incidental matches. Rust retains the existing
zero-radius GCE behavior. Seven direct Rust regressions cover each callable utility.
No original legacy declaration is added by this dependency.

### BeanFace private range and box dependencies

Private range flags, stable sample grouping, subdivision limits, cached surface
boxes, trimmed B-spline knot grids, analytic line/plane intersections, and the
FastComputeAnalytic predicate path pass 5,661 exact native comparisons. The 1,728
new analytic cases cover plane/conic, cylinder/line-or-circle, and sphere/line
classifications, unsupported Bezier/spline/offset curves, and coincident result
ranges. The grid cases cover knot clipping, degenerate and
reversed bounds, derivative-based deflection, and reuse of prior cache values. The
line/plane cases cover parallel and coincident lines, UV clipping and tolerance
expansion. Box cases retain preexisting output, negative deflection, reversed frames
and deliberately distinct stored grid points.
The unchanged native translation unit is included before the observation probe;
its source hash and compiler command are recorded. The shared wrapper also passes
all 11,390 existing EdgeEdge comparisons. These dependencies add no public
BeanFace API or original legacy declaration claim.
