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
