# ADR 0061: Owned annular prism preparation

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
traversal; owned inputs publish end-to-end through ADR0060 with identical replay
across independent preparations; refusals for names, pole counts and version;
cancellation at every observed preparation checkpoint with clean reuse.

Not provided: owned refusal projection of `AnnularPreparationCause`, a public
outcome type, general profiles beyond two nested closed polynomial loops.
