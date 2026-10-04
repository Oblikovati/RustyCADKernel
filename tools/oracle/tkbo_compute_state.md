# Boolean classification oracle

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Source: `BOPTools_AlgoTools.cxx`, ComputeState overloads and ComputeStateByOnePoint.

Build `tkbo_compute_state.cxx` against TKBO, TKPrim and their dependencies.
No argument produces 48 point/vertex/dispatch rows for boxes, cylinders and
spheres with four orientations and inside/boundary/outside/near-boundary points.
Any argument produces 40 edge/face/container rows. Compare with
`cargo test -p rustycad-tkbo --test solid_orientation`.

Edge cases cover finite, left-unbounded, right-unbounded and fully unbounded
ranges; curve-less edges with and without a vertex; face boundary selection
with empty and full bounds maps; and empty/nonempty compounds. Native container
dispatch recursively evaluates its first child but discards the return value,
therefore returning Unknown; this behavior and its side effects are preserved.

All five mappings remain partial. These supplemental cases do not establish
complete classifier behavior for arbitrary geometry, failure fallback paths,
or full legacy-test parity.
