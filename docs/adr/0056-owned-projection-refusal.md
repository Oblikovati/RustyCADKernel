# ADR0056: owned post-publication projection refusal V1

Status: private implementation; release capability remains incomplete.

## Decision and interface

`own_projection_refusal(Box<ScopedPrismFailure>)` consumes only the actual Projection
branch issued by ADR0053. It moves the original boxed refusal and exact outer
callback into `OwnedProjectionRefusal`. Every other branch is returned in the same unchanged box,
with its local lifetime and original payload. There is no detached context/cause
constructor, new geometry evaluator, recapture, or retry.

The owned refusal borrows no operation-local builder. It exposes immutable exact
request/policy/caller-revision operands, the actual naming dictionary, the original
typed child cause, and the accepted child publication **as failure context**.
Its retained coverage is that publication alone. Names, reservation, body or
volume projections computed before a later refusal were dropped by the producer;
this bridge neither reconstructs them nor reports them as retained.

## Class and identity provenance

The outer callback remains separate from the child. Wrapper classification V1
uses an actual callback first: native failure is Failed, explicit cancellation is
Cancelled, and ordinal exhaustion is Incomplete. Otherwise it uses a retained
Body or Volume class. Names, Reservations and callback-only causes have no stored
generic child class header: the bridge explicitly reports Unavailable and its
own Incomplete classification with `BridgePolicyV1` provenance. This is an
additive diagnostic rule, not a change to any producer's behavior or class.

Map failures retain complete authored member/use observations, including every
ancestry token. A failed map does not establish caller correspondence for those
observations. The actual full request and dictionary qualify the operation;
unknown observations remain unqualified rather than being guessed from rank,
coordinates, proximity or process addresses. Body errors retain their existing
semantic entities; Volume retains its exact authored original key and scalar
cause. All exact native message, stack and explicit/derived provenance bytes
remain unchanged.

## Version, persistence and migration

The new private replay domain `Martensite.OwnedProjectionRefusal/v1` contains full
request bytes, actual dictionary and accepted publication bytes, publication-only
retained coverage, wrapper class/provenance, retained child class availability,
outer checkpoint/callback and every supported child operand. Every cause has an
explicit tag. Binary64 values preserve their bits; there is no Debug/Display
fallback, hash-only binding or unsupported-payload omission.

All earlier preparation, publication and complete-operation interfaces and bytes
remain selectable unchanged. This slice does not expose a public complete owned
Result. Remaining preparation transactions, resolver nested diagnostics, and
publication join/validation/history/measures families are explicit scoped gaps.
The RustyCADKernel#10 completion owner must close their reachable corpus before
replacing the scoped finishing interface. The deletion task is to remove the
intermediate family-by-family public adaptation requirement once the complete
owned refusal gate passes, within the existing ADR0014 application R3 migration
limit; no independent milestone deadline or release version is invented here.

## Verification and limits

Pin existing local callback/header outcomes and full child bytes before exposing
shared encoders. Exercise actual callbacks at all six post-publication checkpoints,
including native outer failure with retained Volume cancellation, all three
nonprojection branches, distinct caller IDs using the same internal symbols,
source disposal/mutation, same-process and thread replay. Independent literal
codec cases cover all map/reservation/body/volume variants and scalar edge bits.
Synthetic typed payload fixtures test move identity and codec breadth; they are
not claims that each child naturally fails on admitted geometry.

Compile-time Send/Sync and ordinary host thread tests cover the owned values.
ThreadSanitizer remains incomplete due the separately retained unchanged-host
startup failure; this slice does not repeat or bypass it. No live OCCT, GPU,
performance, application acceptance or complete release gate is claimed.
