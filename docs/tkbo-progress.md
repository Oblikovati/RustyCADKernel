# TKBO port status

Toolkit acceptance is incomplete. The dependency-first port now implements six
IntTools foundation types: Range, Root, BaseRangeSample, CurveRangeSample,
SurfaceRangeSample, and MarkedRangeSet. The 80 API rows use the header audit
identities; the remainder of IntTools and the Boolean algorithms remain pending.

`tools/verify_tkbo_foundations.py --source ../OCCT --native-build BUILD` passes
2,865 exact native comparisons, including signed zero, all root transitions,
range subdivision and hashing, duplicate boundaries, and insertion state.
These are API differential checks; no original TKBO test declarations are claimed
as translated by this foundation slice.

BOPDS IndexRange, Pave, Pair, and Point add 42 API rows. Their comparison, hash,
mutation and Dump behavior are checked against native calls, including signed zero.

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
helpers, projection distances, and golden-section extrema searches and curve/box parameter bounds, coincidence checks and intersection parameter selection pass 7,796 exact comparisons against the unchanged IntTools_EdgeEdge source.
Conic, spline and offset families are covered. Negative subdivision counts retain
native behavior; their corpus uses bounded resolutions to avoid enormous native
allocations. These private helpers add no complete public API or legacy-test claim.
