# Retained analytic region prism transaction

Martensite's ADR0014 private transaction consumes an owned candidate batch and
classifies every captured boundary using the shared operation-owned winding
store. Empty, boundary and ambiguous seeds retain explicit no-body dispositions.
An unrepresented candidate member remains unknown; rejection during selection
is not native deletion. Input deletion keys and actual kinded parent relations
remain separate from selection dispositions.

The selected face is copied using a fresh `BRepBuilderAPI_Copy(true,true)` before
mapped sweep operations. Each actual sweep source occurrence is queried for
`Generated`, `FirstShape`, `LastShape` and `IsDeleted`. The reusable generated list
is captured before the next query. Original generated parents remain generated
through copying; preserved and modified parents become modified. Sweep-generated
entities retain the union of all actual source parents. New container provenance
comes only from its represented descendants, with unknown occurrences recorded
as explicit gaps.

A native cached image is distinct from its cumulative occurrence in the body.
`BRepSweep_NumLinearRegularSweep.cxx` at OCCT `3d097a0` applies containment
orientation through `myBuilder.Add` (lines 159 and 329–354), while line 459 and
`FirstShape`/`LastShape` return the cached image. The receipt retains both full
keys, the exact source query and role, and the actual root-to-child containment
path. Physical `IsSame` only locates candidate contexts; observed orientation
transport must produce the actual `IsEqual` context before final-row matching.
Opposite seam occurrences remain separate keys. Missing or contradictory
context paths are typed gaps, never guessed parent relations. Four native
forward/reversed cases at two locations replay these distinctions exactly.

Native finite rectangular prisms demonstrate that a base wire can report
`IsDeleted=true` while both cap wires and its base edges survive. The transaction
retains that intermediate native observation with its kinded parents. It does
not reinterpret it as deletion of all original source edges. Composed original
deletion coverage remains an explicit gap where the intermediate evidence does
not establish it. Native forward and reversed history rows are attributed
fixtures, not certificates of search or geometric validity.

Every terminal receipt carries phase, original typed failure, requested and
effective settings, unknown achieved error, selection
proof/work prefix, input and sweep history, and any ordered validation report.
Copy, sweep and mapped validation are uninterruptible calls; cancellation and
injected typed failures are checked at their boundaries. The exact winding
solver also polls the same progress protocol and keeps its charged-work prefix
when callbacks stop or fail. Solver cancellation remains cancelled rather
than becoming numerical failure. Selected metadata aliases and the completed
isolated-copy base/history survive subsequent copy/sweep/history/validation
stops or failures; metadata cloning itself makes no isolation claim.

The receipt records seed/root policy and extrusion only when actually invoked.
Requested spatial/parameter construction limits remain present, but their
effective settings are `None` until a corresponding certificate applies them.
The retained plane is a declaration, not original-source plane correspondence.

Constructed geometry is diagnostic only, including output retained after a later
history or validation cancellation. A unique seed with missing construction,
original plane/parameter transport, complete search/history, self-intersection
or tolerance certificates remains incomplete. Mapped validity, successful sweep
and volume do not establish the injective-extrusion/surface-chart theorem or
complete analytic solid validation. No public successful body, API completion,
application acceptance or document naming policy is introduced.

## Exact affine support-map evidence

Evidence version 1 now reads the selected face's actual planar support and full
placement after the existing deep copy. For
`F(u,v,w) = placement(frame(u,v)) + w * extrusion`, it retains the exact constant
signed Jacobian `det(dF/du, dF/dv, extrusion)`. A nonzero value proves this affine
map is injective on its complete parameter space; zero identifies a singular
map. This is a topological nondegeneracy sign, with no metric tolerance threshold.
It is not a volume measurement or an achieved modelling-error estimate.

The computation reuses exact homogeneous line embedding/placement and the exact
scalar triple-product determinant. It never subtracts rounded world endpoints
or multiplies transformed normals in binary64. Support origin and placement
coefficients must all be finite even though translations do not affect the
derivative. The unit-weight affine denominators remain positive powers of two,
so the signed determinant is retained as an exact `DyadicBound`. No root search,
sampling, new solver, tolerance adjustment or approximation is involved.

The observation owns its support scalars, requested extrusion and copied face
occurrence attribution. Unsupported supports and numerical input errors retain
typed causes. The copy/sweep cancellation boundaries bracket this fixed-size
calculation; completed evidence survives later failure and cancellation. A later
mutation of either input or copied geometry cannot change retained scalars, but
those scalars do not authorize the changed geometry. Receipt-local shape keys
are neither persistent identities nor evidence of immutable published geometry.

This is additive diagnostic evidence: existing operation behavior, requested
limits, success policy and replay remain unchanged. The combined
`PrismInjectivityAndSurfaceCharts` acceptance gap stays open until source
correspondence, region validity and actual generated cap/side surface charts are
proved together. No body can succeed on this scalar theorem alone. The owning
issue #10 deletion and migration gate from ADR0014 still applies.

Independent checks cover an oblique composite determinant of exactly 501,
rounded matrix cancellation, every finite binary64 power-of-two scale,
singular/tangent and near-tangent sweeps, actual face placement and handedness,
unsupported support, scalar input errors, operation reuse, cancellation and
source/copy mutation. The pure exact computation is also compared across eight
threads. The [exact determinant receipt](evidence/prism-transversality/exact-oblique-determinant.json)
records the independent FLINT rational calculation. Native fixtures remain
historical replay only; no native captures were added.

Run the private regressions with:

```sh
cargo test -p rustycad-tkbo --lib sketch_region_prism
```
