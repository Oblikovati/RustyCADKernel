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

Native finite rectangular prisms demonstrate that a base wire can report
`IsDeleted=true` while both cap wires and its base edges survive. The transaction
retains that intermediate native observation with its kinded parents. It does
not reinterpret it as deletion of all original source edges. Composed original
deletion coverage remains an explicit gap where the intermediate evidence does
not establish it. Native forward and reversed history rows are attributed
fixtures, not certificates of search or geometric validity.

Every terminal receipt carries phase, original typed failure, requested and
effective root/spatial/parameter limits, unknown achieved error, selection
proof/work prefix, input and sweep history, and any ordered validation report.
Copy, sweep and mapped validation are uninterruptible calls; cancellation and
injected typed failures are checked at their boundaries. The exact winding
solver also polls the same progress protocol and keeps its charged-work prefix
when callbacks stop or fail.

Constructed geometry is diagnostic only, including output retained after a later
history or validation cancellation. A unique seed with missing construction,
original plane/parameter transport, complete search/history, self-intersection
or tolerance certificates remains incomplete. Mapped validity, successful sweep
and volume do not establish the injective-extrusion/surface-chart theorem or
complete analytic solid validation. No public successful body, API completion,
application acceptance or document naming policy is introduced.

Run the private regressions with:

```sh
cargo test -p rustycad-tkbo --lib sketch_region_prism
```
