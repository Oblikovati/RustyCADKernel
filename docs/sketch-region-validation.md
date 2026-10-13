# Ordered region validation prerequisite

Martensite's private `tkbo::sketch_region_validation` stage implements the first
coverage prerequisite for [ADR0014](adr/0014-owned-analytic-sketch-regions.md).
It does not return a successful region or solid and adds no upstream API claim.
It accepts an already-owned arrangement candidate; the shape reference and
cloned diagnostic occurrences do not establish copy isolation. The arrangement
owner must supply that boundary before using mapped checkers, which retain their
native state transitions and may adjust representations.

`validate_owned_candidate` checks the declared body kind, then uses distinct
`BRepCheckAnalyzer` instances with geometry controls off and on. Every own status
and every required mapped subshape/context status is inspected explicitly.
An absent context, empty status list, `CheckFail` or typed native failure is
incomplete. Other native defects are failed. Later stages stay blocked until
the prerequisite passes. Native minimum checks can conservatively report
geometry defects even with geometry controls off; this first prerequisite is
mapped coverage, not a pure combinatorial topology certificate.

Required context kinds mirror the existing checker: edge/vertex,
face/wire-edge-vertex and solid/shell. Contexts deduplicate by `IsSame`, matching
native contextual loops. Own occurrences and source relations use `IsEqual`,
retaining orientation and location. Known source parent sets are sorted and
merged without dropping parents; unknown attribution remains `None`.

The first regression reinitializes a genuine face-edge checker through its
mapped public `Init`. That clears its edge/face context. The mapped analyzer
still returns valid, but strict inspection returns incomplete with the exact
missing edge and face. No mapped checker implementation, ordering or failure
timing was changed. The initial red log is retained in
[evidence/region-validation/first-red.log](evidence/region-validation/first-red.log).

Real box and planar-face candidates advance the two implemented coverage
stages. A deliberately displaced pcurve passes the first prerequisite but fails
geometry with the actual native statuses, offending edge/face and full supplied
source-parent sets. Wrong body kind and a spherical face cannot become a planar
region. Null input, missing analyzer result, checker failure, unknown coverage
and success/failure/success reuse have independent tests.

Run the focused regression command without OCCT:

```sh
cargo test -p rustycad-tkbo --lib sketch_region_validation --locked
```

## Required continuation

Mapped geometry passing is not a certified achieved-error bound. Analytic
self-intersection must expose actual completion and affected entities, including
unfinished face intersectors; tolerance consistency needs declared-policy
invariants and certified requested/effective/measured error. Neither sampled
edge tolerance nor mesh overlap is promoted to such a certificate. These later
stages are explicitly incomplete/blocked, so full completion remains false.

Additional body-kind invariants, general trims/holes/incidences, cancellation,
copy/cache isolation and full member history remain part of ADR0014. Light's
private arrangement contract has its own source-identity types; integrating a
shared adapter is a later coordinating change, not an invented persistence
identity. This stage adds no application acceptance or complete-class credit.
