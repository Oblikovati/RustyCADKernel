# Source-bound boundary census

Martensite issue #10 and [ADR0014](adr/0014-owned-analytic-sketch-regions.md)
require complete incidence, construction and search certificates before a
region or solid succeeds. `sketch_region_certification` is a private prerequisite;
it produces no healthy body and receives no mapped API completion credit.

`capture_boundary_inputs` reuses the arrangement admission and fresh aggregate
`BRepBuilderAPICopy(true,true)` path. It captures every copied original edge
member before arrangement, builds plane pcurves only on that copy and retains
all original occurrence keys in the copy relations. The caller plane is a
declaration, not a planarity proof. Exact coefficients describe the captured
pcurve; floating line endpoint construction and source-to-plane residuals remain
uncertified. Source/result geometry mutations and failure/cancellation reuse are
tested independently. The existing seven-test copy isolation corpus provides
additional geometry, pcurve, mesh and cache evidence.

`enumerate_boundary_census` retains each unordered occurrence pair and each
canonical self domain in stable input traversal order. It admits the existing
source-bound line–Bézier certificate for finite stored line domains. Each report
owns its coefficients, weights, trims, domain policy, complete denominator/root
isolations and outward world-point enclosure. Physical aliases reuse numerical
reports and typed failures only when the captured bits and full operation policy
match. Native source IDs, directions and complete member parents remain separate
from the operation-local cache role. Successful fresh subdivision work is counted;
failed work is not supplied by the existing incidence provider, so aggregate
work-budget acceptance remains incomplete. Cancellation retains its owned prefix,
original numerical cause and before/during-domain distinction.

Bézier–Bézier, self off-diagonal and same-physical-curve overlap domains remain
explicitly unsupported. Coincident supporting curves retain the provider's typed
Declined outcome; they never become an empty exclusion certificate. Unsupported
curve capture and missing history preserve their original typed causes.

Contact inspection first requires generated history from both original edges,
then actual shared vertex incidence on modified edges. It reads parameters on
those modified edges; the original source edge need not contain a generated
vertex parameter representation. Bracket/enclosure observations do not invent a
topological association. Candidate parent coverage retains represented and
unrepresented source occurrences, without inferring exhaustive parameter
partitions, loop search or healthy region coverage from parent membership alone.

The pinned native quadratic/line fixture and Rust reproduce the same actual split:
curve parameter `0.49999999999996131`, line parameter `1.4999999999999485`, and world
point `(0.49999999999994849, 0.24999999999997419, 0)`. Four modified edges share that
actual vertex. The exact stored incidence is `(0.5, 0.25)` at parameter `0.5`.
Both raw values are preserved; the bracket and enclosure are unchanged. An
outward mapped-construction residual and explicit parameter construction policy
are required to decide conformance. Proximity alone is never identity.

The operation remains incomplete for source conversion/placement, exhaustive
pair/self/overlap coverage, failed-work accounting, split correspondence,
candidate search/history coverage, self-intersection and tolerance. Camera's
captured-winding implementation and the general pair provider are independent
pending prerequisites; this branch consumes no unchecked peer work. A complete
planar certificate may later compose with nondegenerate affine extrusion and
actual per-face prism chart/history correspondence for the declared prism body.
It cannot certify arbitrary solids or substitute native CheckerSI completion.

Run the Rust-only regression command:

```sh
cargo test -p rustycad-tkbo -p rustycad-tkbrep --all-targets --all-features
```

Native OCCT remains a local removable oracle. The committed two-row asset is
raw stdout from pinned `3d097a0328e71b826377d4814ab05ec3c3d23871`, independently
repeated; Rust tests read the asset without loading native libraries. No renderer,
golden, tolerance, publication or application naming policy changes accompany
this stage.
