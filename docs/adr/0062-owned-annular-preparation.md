# ADR 0062: Owned annular prism preparation

Status: private preparation implemented on a worktree branch (2026-10-15).

`owned_request::annular::prepare_owned_annular_prism(request, progress)` takes
only owned coefficients: an explicit **outer** and **inner**
`OwnedPolynomialBoundary`, authored names for the annulus, inner domain, outer
complement and their four wires, a material seed, interior witness, declared
plane, extrusion and explicit child policies.

It reuses the existing pipelines unchanged: owned closed-polynomial
construction (now parameterized by the authored source identity), boundary
capture and source-bound census, partition arrangement, candidate copies,
partition binding, contact transport and the retained prism builder. Admission
reuses the single-boundary pole/plane/extrusion checks per role and refuses
zero or duplicate names.

Authored region names attach to actual candidates only through their native
source parents (both sources: annulus; inner only: inner domain; outer only:
outer complement); every wire must have exactly one source parent. Batch order
never names anything, and the independent two-loop proof certifies the result
downstream. `PreparedAnnularPrism::with_region` lends the two-loop request in
authored outer/inner order.

Evidence: certified by `inspect_two_loop_region` for forward and reversed
traversal; owned inputs publish end-to-end through ADR0061 with identical replay
across independent preparations; refusals for names, pole counts and version;
cancellation at every observed preparation checkpoint with clean reuse.

Not provided: owned refusal projection of `AnnularPreparationCause`, a public
outcome type, general profiles beyond two nested closed polynomial loops.

## Follow-up: owned annular preparation refusal (2026-10-15)

`project_annular_preparation_refusal(Box<AnnularPreparationFailure>)` copies the
actual typed cause into `OwnedAnnularPreparationDiagnostic`: admission
(`Owned` single-boundary operands, including exact open-boundary endpoints, and
`Names`), `Role` operands, and `Stage` causes through the unchanged ADR0048 owned
cause vocabulary. The full original request (both roles, seven authored names,
interior witness, seed, plane, extrusion and every child limit) is encoded by
`Martensite.OwnedAnnularPrismRequest/v1`; the diagnostic replay format is
`Martensite.OwnedAnnularPreparationDiagnostic/v1`, explicit tags only.

Retained partial transactions (constructed sources, census, partition, contact
transaction, the new `RegionIdentities` owner tag 6, prism receipt) and the
Census/Contact/RegionPrism/StageCompletion nested families are explicit gaps; on
any gap the exact original Box is returned beside the diagnostic. Admission
refusals therefore project completely; post-construction refusals do not yet.
