# Private analytic region candidates

This Martensite prerequisite follows [ADR0014](adr/0014-owned-analytic-sketch-regions.md)
and consumes the already-owned [arrangement stage](sketch-region-arrangement.md).
It supplies diagnostic candidate extraction and seed classification, not a
public region API, validated face, named solid or complete issue #10 operation.

A fresh aggregate `BRepBuilderAPICopy(true,true)` owns topology, geometry and
meshes before plane pcurve construction. The existing mapped `BuilderFace`
generates candidates from forward/reversed arranged edges. Its sampled loop
and containment decisions remain unchanged and are **not certified complete
search**. The supplied plane is retained; seed coordinates are in that plane's
UV frame, not world XY. Cancellation is observed only at copy, generation and
capture boundaries; synchronous mapped stages are not interruptible here.

Every captured face/wire/edge/vertex occurrence uses cumulative location and
orientation with `IsEqual`. Copy composition keeps previous Generated parents
Generated, turns previous Preserved into Modified and keeps Modified parents.
New face/wire containers derive Generated parents only from represented
actual descendants. Original deleted-member identities are retained verbatim.
Unrepresented copied members are unknown coverage, never fabricated deletions.
All original parent sets retain full equality and stable source-ID ordering;
operation-local addresses are not persistent names. Actual mapped reports are
retained in an independent container with their original typed alert payloads.

Boundary capture compares the mapped `WireExplorer` walk against the complete
actual wire-child occurrence multiset. Omitted, unexpected or unsupported
occurrences yield typed Incomplete. Child acquisition uses explicit fallible
More/Value/Next traversal: a Value exception cannot masquerade as exhaustion.
The shared named changing-child-count fixture proves that for both Face and Wire.

The classifier uses the existing exact rational/nonrational Bézier wire policy
on captured coefficients, weights, bounds and directed occurrences. Supported
finite lines use their evaluated trim endpoints as linear controls; original
parameter bounds remain recorded. This proves signs for the supplied stored
polynomials, not zero construction error relative to original analytic geometry.
Unsupported representations—including conversion paths that might approximate—
remain typed Incomplete; there is no curve-specific retry or sampled replacement.

Positive directed winding, corrected for Forward/Reversed face orientation,
denotes material in the candidate's declared UV frame; negative loops describe
complements or holes. Internal/External face orientation remains explicit
Incomplete rather than silently defining material. All wires and candidates
are evaluated. Empty, Boundary, Unique and Ambiguous selections are distinct;
there is no first-match selection. A unique classification is still explicitly
Incomplete: construction error, search completeness, self-intersection and
tolerance certificates are missing, and achieved error remains unknown.

Shared-incidence reuse is also incomplete. The current whole-wire API internally
owns endpoint closure and root evaluation. Its public segment contribution does
not expose the endpoint/certificate/work bundle needed for correct reuse.
Counterpart candidate faces currently perform real repeated evaluations; their
actual subdivisions are charged to one shrinking request budget and reported.
No cache hit or performance saving is fabricated. The required narrow next seam
is an operation-local owned captured segment evaluation/certificate keyed by
coefficients, weights, trim, ray/seed and the complete numerical policy, with
separate directed occurrence assembly. This is not a global geometry cache or
another classification engine.

Rust tests exercise ordinary/rational cubic boundaries, exact lines, reversal,
nesting and hole candidates, touching, explicit ambiguity, geometry isolation
in both directions, failure/cancellation reuse, full member relations and actual
generated intersection vertices. The ambiguity test deliberately combines two
actual generated overlapping candidates to exercise the defensive selector;
it is not evidence that mapped search produces duplicate candidates.

The pinned OCCT oracle emits fourteen candidate/seed rows plus ten cut-history
and wire-coverage rows, repeated byte-exactly. Native finite/infinite candidate
classification is distinguished. The native sampled classifier reports the
cubic cusp endpoint as bounded IN1, while the exact supplied endpoint is
Boundary; that diagnostic remains recorded and does not override the exact
policy. The extended rectangular cut generates intersection vertices and two
internal open wires in the infinite complement; native WireExplorer visits
zero of each wire's one actual child. Full extraction therefore retains a typed
Incomplete outcome even though the separate actual candidate history proof
preserves every generated parent through copying. These are complementary
native observations, not full native parity or valid-body certificates.

```sh
cargo test --locked -p rustycad-tkbo -p rustycad-tkbrep --all-targets
```

Attributed fixtures ship inside Rust packages. OCCT is only the removable local
fixture generator. General incidence certificates, supported representations,
complete candidate history/search, ordered validation, prism/history composition,
persistent application naming and editable/save-reopen/undo acceptance remain
required before a successful public operation.
