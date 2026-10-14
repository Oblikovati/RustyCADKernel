# ADR 0048: owned prism diagnostic envelope V1

Status: approved bounded design; implementation in progress, private only.

## Decision

RustyCADKernel#10 needs an actionable immutable refusal even when an operation's
local diagnostic containers cannot yet be projected losslessly. Operation class
and diagnostic projection coverage are independent. Failed, incomplete, cancelled
and declined operations retain their original class; missing diagnostic coverage
never promotes a body or erases a failure.

This slice starts with actual owned-request admission and construction native
refusals, then the publication outer wrapper. Reuse the ADR0043 bounded join
projector and the original typed numerical/native causes. Do not mirror working
transactions, flatten supported causes to strings, or replace numerical evidence
with counts. Unsupported payloads receive owned typed incomplete-diagnostic
coverage with their original local failure retained for inspection.

## Context and identity

Construct diagnostic context only from the actual sealed preparation state. Own
the complete original request, including caller IDs, revisions, source
multiplicity and pole bits, declared plane, selection, extrusion and every policy
and work limit. The validated semantic plan is optional: early admission can
fail before a plan exists. Never manufacture a dictionary for that case.

One read-only preparation view supplies request, outer entry phase, optional plan
and outer retained-stage presence for both preparation success and failure. It borrows local state;
it is not a published result and does not expose mutable owners. Child causes and
callbacks retain their separate actual phases. The outer owner inventory excludes
nested bridge/cause payloads and cannot establish complete diagnostic coverage. A single shared
full-request encoder retains floating-point bits, including signed zero and NaN
payloads on refused input. The final outcome owner reuses that codec unchanged.

Internal authored tokens are reused between operations. Matching an internal
operation token and options alone cannot distinguish two callers with identical
geometry. Publication must therefore attach a sealed resolver context during the
actual publication invocation. A later arbitrary `(failure, preparation,
bindings)` tuple cannot qualify authored IDs. Verify the request policy and
operation/source dictionary binding before attribution; absent or foreign context
is typed projection-incomplete. Reuse resolver correspondence rather than adding
a second role/ancestry resolver.

## Scope and migration

The envelope is private and versioned independently of existing preparation and
publication behavior. Native message/stack bytes, exception type/parent, phase,
callback checkpoint/ordinal and original operation class stay intact. Later
census/contact/history/measure containers remain explicit incomplete projections
until their concrete reachable payloads have typed adapters. Body projection
errors are supplied by the separate body-view owner.

Kernel#10's operation-contract owner owns the migration/deletion task: finish the
bounded actual-operation refusal corpus and remove any superseded lossy public
adapter before the existing ADR0014 application R3 migration limit. This does not
introduce a release, workspace version change, new geometry engine, or additional
milestone. Earlier local failure paths remain selectable and unchanged.

## Verification obligations

Pin current local failures before structural access/encoder reuse. Exercise
actual admission and early callbacks, native byte payloads, all request fields,
foreign context, absent dictionaries, original class preservation and retained
unsupported payloads. Compare independent expected request bytes, signed zero,
NaN payloads and revision/source multiplicity. Verify process/thread replay,
Send/Sync of owned envelopes, and input isolation. Local retained containers have
no published lock-free ownership claim. Record source-frozen tests, docs, lint,
formatting, sanitizer attempt and any incomplete gate separately. No native OCCT,
GPU, release benchmark or application acceptance belongs to this slice.

## Early preparation implementation boundary

The first private envelope consumes an actual boxed preparation refusal. Its
owned context is cloned only from that refusal's retained request and optional
validated naming plan. Admission causes and direct native failures preserve their
existing typed operands, including invalid floating-point bits. Callback evidence
retains the exact checkpoint, ordinal, cancellation/exhaustion variant or native
failure. The operation class is copied without reinterpretation.

Projection coverage is complete only when that supported cause/callback evidence
is present and there are no retained outer owners. Census, contact transport,
unavailable-region and unavailable-stage causes always add explicit typed gaps,
even when the outer owner inventory is empty. They are not claimed to have
lossless projected child phases or numerical payloads: the same original boxed
failure remains available for operation-local inspection. An absent callback for
a callback cause also prevents complete coverage. The immutable envelope and the
local retention carrier are separate types; the carrier has no published
Send/Sync claim.

The context receipt encodes full request operands and the actual optional
semantic envelope. It is not a diagnostic-cause persistence format. Development
process/thread fixtures append typed cause observations solely for byte-for-byte
test comparisons. A publication wrapper is still pending the resolver-issued
actual-invocation packet; this prefix provides no arbitrary header/failure pairing
entrypoint and no publication-completeness claim.

## Native failure provenance seam

A separate additive TKernel API exposes `Failure::explicit_exception_parent()`
and `Failure::derived_message_bytes()`. The first returns only the stored parent
override; it does not substitute a base inferred by `is_kind()` from the existing
hierarchy. The second returns the optional derived message buffer, preserving
`None` versus `Some(empty)` and the distinction between base `message_bytes()` and
virtual `what()`. This permits an exact typed codec without Debug/Display parsing.
No constructor, mutation, hierarchy, buffer ownership or native behavior changes.
Baseline commit `8d9b35390` pins these distinctions before the accessors are added.
The final coordinated release accounts for this additive API; no mid-workspace
version change or replacement migration is introduced by these read-only methods.

## Typed owned-envelope replay V1

`OwnedPrismDiagnostic::replay_bytes()` is separate from
`OwnedPrismDiagnosticContext::replay_bytes()`. The envelope format starts with
`Martensite.OwnedPrismDiagnostic/v1` and a NUL, followed by a length-delimited
context, explicit outer phase, original class, projection coverage, supported
cause, optional callback and ordered coverage gaps. Integer fields use explicit
little-endian widths; refused floating-point fields preserve their binary64 bits.
Every current admission case, native field, phase and gap has an explicit tag.
There is no Debug/Display serialization or message parsing.

A native payload records exception type, optional explicit parent, base message
bytes, stack bytes and optional derived-message bytes. `None` and `Some(empty)`
remain different. Unsupported local transaction payloads are represented by
explicit coverage gaps and retained outside the owned envelope. Complete encoding
of that owned envelope never upgrades incomplete diagnostic coverage or claims to
encode its retained local owners.

The earlier context-only bytes and test-only context-plus-Debug observation stay
unchanged for historical replay. New full typed process/thread evidence uses the
actual envelope codec and independent literal/field fixtures. The publication
phase vocabulary is encoded, but publication attribution/construction still
requires the pending sealed actual-invocation packet; this codec adds no such
constructor or body-completion claim.
