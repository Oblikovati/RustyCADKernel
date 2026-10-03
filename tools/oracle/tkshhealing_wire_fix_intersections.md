# Adjacent wire intersection repair

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The fixture has two line p-curves crossing at `(0, 0)` and distinct shared
vertices moved to `(1, 0)`. `FixSelfIntersection()` invokes the adjacent
intersection repair with topology mode disabled. The oracle records the
aggregate statuses and both updated shared-vertex tolerances.

A second run enables topology mode and records the split statuses and both
edge p-curve ranges for this line case.

Build with the toolkit's pinned OCCT library and compare executable output to
`tkshhealing_wire_fix_intersections.expected`.
