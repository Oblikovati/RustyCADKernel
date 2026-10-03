# ShapeFix split tool

Pinned oracle: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`tkshhealing_split_tool.cxx` produces 18 rows; the Rust integration tests compare
flags, each representation's range, vertex position/tolerance, sequence order,
selected-piece index and context recording.

The cases cover both trimmed-line cut directions, a non-endpoint cut, early and
late rejection, independent p-curves on two surfaces, single splits, displaced
split vertices, reversed occurrences, gap removal, all four span/endpoint
combinations, reversed parameter order and rejected sequence output preservation.

Two source discrepancies were reproduced before correction: the old private cut
helper rejected ordinary trimmed-line cuts, and only changed the selected face's
range rather than all parametric representations. The new cases observe the 3D
curve directly and inspect a second surface representation too.

The older tiny-cut Rust fixture inserted its p-curve before its 3D curve, unlike
the native fixture. `BRep_Tool::Range` chooses the first qualifying representation,
so the Rust fixture now constructs the native 3D-first order. Expected native
output was retained. The bounded-curve cut path now invokes FixSameParameter even
when no p-curve exists, as the source does.

Null context/p-curve domains use explicit Rust failures where native can
crash. Curved, seam, placed-face and non-SameParameter splitting require broader
oracle coverage. These are additional differential cases, not new legacy
mappings or a full behavioral parity claim.
