# Owned arrangement prerequisite

This internal TKBO stage implements the first construction prerequisite of
[ADR0014](adr/0014-owned-analytic-sketch-regions.md). It is not a complete region
API or a validated body result. Mapped General Fuse still requires at least two
arguments. The stage's nonempty `BuilderHooks` admission allows one authored
closed curve and preserves actual filler errors.

The caller supplies a typed source identity and a read-only borrowed shape.
Caller state may still be mutable. A newly owned scratch compound borrows all
source occurrences in their original order without freezing their TShapes.
It is neither immutable nor isolated. A fresh `BRepBuilderAPICopy` then copies
that aggregate with geometry and mesh enabled before interference. Cyclic caller topology is rejected before recursive copy,
without a shape-specific fallback. Copying
roots separately would lose shared topology between source occurrences.
The new batch factory is an intentional nonmutation extension; mapped
`TopoDS_Builder::Add` keeps its upstream freezing behavior.

Copy `Modified` relations compose with actual arrangement history. An unchanged
copied entity is **modified from the original**, not preserved original topology.
Original and output occurrence collection uses cumulative location/orientation
and `IsEqual`, retaining actual forward and reversed occurrences within one
authored source. Physical mapped history still uses `IsSame`; merged outputs
retain all actual original-member parents without inventing output directions.
Every result occurrence records the discovered original-member parent sets;
new root-container parents come only from represented descendants. Deletions
retain original member identities, rather than treating deleted roots as complete
entity history. Missing attribution/history or unsupported history kinds produce
an explicit incomplete outcome. Native history supports vertices, edges, faces
and solids; wire/shell/compound coverage remains a dependency.

Member identity retains caller source ID, original TShape address, location and
orientation. Full-key equality deduplicates parents; stable source-ID order
preserves the first actual occurrence traversal within a source. Neither
location hashes nor memory-address ordering establish uniqueness or names.
These are operation-local provenance, not persistent names or a
pointer-based recovery mechanism. They must not become application naming keys.
Typed failures remain typed, and mapped alerts retain their actual payloads in
an independently owned report container. Unknown affected source stays unknown;
alert payload immutability is not assumed. Clearing a reused report does not
remove a previously captured failure.

Cancellation is checked before copy, before mapped interference plus General
Fuse, and before composition of history. Those mapped calls are synchronous and
uninterruptible through this seam. Declined empty input, failed computation,
cancellation, missing coverage and completed-empty stage results are distinct.
A completed-empty result keeps its deleted-member coverage. Stage completion
still reports unknown achieved error and uncertified incidences. Ordered body
validation, exact constructed incidences, persistent naming and full operation
acceptance remain required.

The first accepted test workload is the existing closed cubic, its bounded face
and prism vector `(0,0,2)`. The owned arrangement feeds existing face construction
and the exact diagnostic classifier; the prism matches the established native
per-face receipt, volume `4.8` and three faces. This test does not install a cubic
classifier or curve-specific production route. The shared authored fixture's
numeric construction expressions are unchanged.

Rust regressions cover single and multiple input, empty/null/duplicate identities,
shared reversed occurrences, two placements of one topology, source curve and
polygon-node mutation, output/source topology flags, typed reports, missing
history, deleted entities, boundary cancellation and prior result reuse after
failure. The borrowed-container regression retains both initial Free states.
The native ten-row fixture independently records mapped Add freezing, copied
alias/orientation identity, all 3x4 transformation values and world points. For
its translated vertex, copy moves translation into vertex geometry: original
translation `(3,4,5)` becomes identity while the world point stays `(4,6,8)`.
That measured case is not a claim about every representation's placement policy.

```sh
cargo test --locked -p rustycad-tkbo -p rustycad-tkbrep --all-targets
```

OCCT is only the local fixture generator. Rust replay needs the attributed
assets, not OCCT. No mapped API completion, final healthy-body validation,
application acceptance, release authorization or performance baseline follows
from this prerequisite.
