# ADR 0061: Complete named annular prism publication

Status: private assembler implemented on a worktree branch (2026-10-15);
main integration, owned request/outcome integration and release remain open.

`publication::annular::publish_annular_prism(request, embedding, progress)`
composes one annulus from **one source binding**: the same two-loop request,
sweep receipt, authored publication/copy bindings and explicit history scopes.

1. The sealed embedding evidence is joined (`join_annular_frozen_domain`) under
   `Join` callbacks.
2. Ordered exact validation (ADR0059), complete V2 history (ADR0058, bound
   inspection without its own rebind) and conditional four-face measures
   (ADR0055) run against that one join; each child must bind its bytes or the
   operation is incomplete.
3. The terminal `Commit` callback precedes the callback-free current source
   rebind (`rebind_annular_frozen_domain`) and history-only full-byte
   reprojection. A prospective result never escapes before both pass.

Refusals keep the actual child carrier (`Join`, `Validation`, `History`,
`Measures`) and a callback stop retains the child it interrupted. Version 1 is
the only admitted publication version; V1 single-boundary publication is
unchanged and selected by its own entry points.

Evidence: actual nested pair published with all children bound to one join,
closed-form cap areas (9/5 each) and signed sweep magnitude (18/5) inside the
certified enclosures, deterministic replay across independent operations,
cancellation and callback failure at every phase without a body and with
clean reuse, unsupported version and history refusal phases.

Not provided: physical-volume certification for the annulus, semantic naming
of the annular body, owned public request/outcome, owned refusal projection of
these causes, application acceptance.

## Follow-up: owned annular publication refusal (2026-10-15)

`project_annular_publication_refusal(Box<AnnularPublicationFailure>)` first checks
the refusal's single leaf (beneath any callback chain) against an explicit codec,
then either moves the whole refusal into `OwnedAnnularPublicationDiagnostic` with
no retained original, or returns a summary with one explicit
`PublicationDiagnosticGap::Payload` and the exact original Box. Phase and class
are copied unchanged; projection never reclassifies.

Owned leaves: unsupported version; binding; history cancellation; every
callback stop (exact checkpoint and native failure bytes) with the child it
interrupted; the actual moved `AnnularValidationFailure` for version, binding,
cancellation and callback causes, with every partial certificate
(`Martensite.AnnularValidationRefusal/v1`); the actual moved
`AnnularMeasureFailure` for version, count, operand, work-overflow and
cancellation causes, with every partial face/sweep/total measure
(`Martensite.AnnularMeasureRefusal/v1`); history findings without a local owner
or nested join refusal (`Martensite.PublicationHistoryDiagnostic/v1`); and join
identity, version, name-budget, native-callback and cancellation causes
(`Martensite.AnnularJoinRefusal/v1`). The outer format is
`Martensite.OwnedAnnularPublicationDiagnostic/v1` and encodes every requested
child policy through `Martensite.AnnularPublicationOptions/v1`.

Still pending payloads (original retained): join embedding, current binding
context, original two-loop region, frozen capture and link families; nested
validation topology/orientation/conformance/intersection/tolerance failures;
nested measure direction/side-policy/cap/sweep/side/total numerical errors;
history findings that own a local diagnostic or a nested join refusal.
Tests drive actual refusals: version, all five phase callback stops (cancel and
native failure), child version refusals, a history scope refusal, and a foreign
embedding that keeps its original beside an explicit join gap.
