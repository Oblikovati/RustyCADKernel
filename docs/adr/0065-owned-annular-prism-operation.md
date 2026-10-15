# ADR 0065: One-call owned annular region-to-prism operation

Status: private implementation on a worktree branch (2026-10-15).

## Decision

`owned_request::annular::operation::run_owned_annular_prism(request, progress)`
takes one `OwnedAnnularPrismRequest` by value and returns an
`OwnedAnnularPrismOutcome`: `Published(OwnedAnnularPrism)` or
`Refused(OwnedAnnularPrismRefusal)`. It composes the existing stages unchanged
and in order: owned preparation (ADR0062), the independent two-loop region proof,
annular binding, production names (ADR0064), embedding, publication (ADR0061:
join, ordered validation, complete history, four-face measures, final rebind) and
physical volume (ADR0063). Nothing is retried, widened or replaced.

The request policy now carries every child policy explicitly: embedding,
publication and volume version join the census/source/contact/classification/
region policies. `Martensite.OwnedAnnularPrismRequest/v1` encodes them.

The published branch owns: the request; four caller-named faces (caps named by
the caller annulus and sweep end, sides by the caller boundary and annulus
wire), each with its certified area enclosure and error; the certified total
area; the physical volume (enclosure, representative, error, exact Jacobian
sign); ordered validation coverage; complete-history counts; the name
dictionary; and the full publication and volume receipts. Faces are paired with
measures by full occurrence key, never by position. No live shape, local failure
or process address is reachable; both branches are `Send + Sync`.

The refused branch owns the request, the refusing stage, the class, a typed
diagnostic and explicit gaps. Preparation and publication refusals carry their
owned diagnostics (ADR0062/0061 follow-ups); volume refusals carry the owned
`AnnularVolumeFailure`; body refusals carry a typed cause. Region, binding,
naming and embedding refusals borrow call-local state, so they are `Unprojected`
with an explicit `StageFailure` gap: their classes are kept, their payloads are
not yet owned. One caller progress observes every child checkpoint; the first
stop is recorded with its exact stage and ordinal and, following the ADR0056
wrapper rule, classifies the refusal first (a cancel-only child API cannot carry
a native failure itself).

## Evidence

Closed forms (cap area 9/5 per cap, volume 18/5) hold through the one call for
forward and reversed traversal; validation is Passed at all four ordered stages;
history continues six originals and retires two wires. Replay is identical
across calls and threads. Admission, volume-version and publication-child
refusals are complete; a borrowed embedding refusal is an explicit gap; the first
checkpoint of every stage and the last checkpoint refuse with the exact recorded
stop for both cancellation and native failure.

## Limits

The domain is two nested closed polynomial loops. There is no completed-empty
branch because a selected material seed always names one bounded region here; an
empty selection is a typed region refusal. The types are crate-private; a public
facade, owned projection of region/binding/naming/embedding refusals, a portable
annular body view, general profiles and application workflows remain open.
