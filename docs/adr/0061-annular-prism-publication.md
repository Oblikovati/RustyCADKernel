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
