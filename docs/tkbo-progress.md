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
