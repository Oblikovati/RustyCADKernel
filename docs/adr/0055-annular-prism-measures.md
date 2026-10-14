# Bound annular prism measurements

Status: implementation in progress; private scalar evidence only.

## Scope and shared pipeline

Kernel #10 and Martensite C1 require per-face areas and bounded volume evidence
for the actual sketch-with-hole prism. Consume the sealed `AnnularFrozenDomain`
from ADR0052; preserve all named faces, directed uses, sources and the whole
same-operation replay. No caller-built coefficient bundle can issue this report.
Do not query mutable geometry or construct a second region classifier.

Reuse the existing exact polynomial plane/sweep integrators, the certified
polynomial side integrator and nonnegative enclosure sum. Reuse ADR0054's
source-bound winding recovery, including its explicit traversal correction.
Normalize the outer boundary to positive chart orientation and the inner boundary
to negative chart orientation. This normalization defines the signed material
sweep in this new version: reflection or extrusion reversal changes its sign
through the exact placed Jacobian. It is not a claim of physical body volume or
outward orientation; those require the complete ordered result.

Both cap reports integrate the complete two-boundary material domain in the
actual cap chart and its placed metric. The cap report uses the exact Gram
factor, not native area alone. Side reports use the actual generating curve,
placement and extrusion for each full source-bound side chart. Unique full
identities select operands; the shared Generated role does not select a side.
The total adds exactly two cap and two side enclosures, retaining child errors.

## Options and result contract

The explicit V2 request contains separate cap, signed-sweep, side and total
arithmetic policies, plus an aggregate side-subdivision cap. Preserve the
unchanged original per-side policy alongside each child effective policy. Apply
at most the remaining aggregate work to a child; report this clamp explicitly.
Do not widen an error cap, silently double the prior budget, or retry an exhausted
integral with different settings. Validate the original side policy through the
same scalar admission function before applying any shared-budget clamp; an invalid
original limit must not become valid because the remaining budget is smaller.
Evaluate outer then inner by certified semantic roles. This order is observable
when work is shared and is preserved in replay. Retain work spent by a failing
child in the aggregate count. A remaining budget of zero still reaches the scalar
integrator, which may complete without subdivision. Aggregate work is a count of completed scalar
subdivisions, not a wall-time or allocation-byte bound.

A completed immutable report owns all four named face results, the signed
material-sweep result, total area, requested/effective policies, measured work,
method tags and full bound input bytes. A terminal typed failure retains the
operation, phase, relevant named entity, exact arithmetic cause, completed partial
reports and work; partial reports cannot become a successful body. Cancellation
is polled during arithmetic and immediately before issuance. No callback can
mutate the sealed joined input. No new unsafe or global mutable state is allowed.

## Verification and migration

Pin unchanged V1 scalar codecs and full joined-measure bytes before any shared
visibility extraction. New behavior has its own versioned replay and independent
expected results. For the nested cubic fixture, the inner curve is a half-scale
translation of the outer, so its native area is one quarter and its swept side
area is one half. Derive exact cap/sweep fractions and side bounds from the owned
independent integration fixture, recording the derivation and directed rounding.
Do not use tessellation or the implementation under test as the expected oracle.

Check every face name, loop role, chart/trim binding and report error. Cover
stored-pole reversal separately from directed traversal, reflection, negative
extrusion, source reorder, policy/version boundaries, insufficient aggregate
work, every cancellation prefix and reuse. Compare full bytes across processes
and concurrent readers/operations; preserve all previous owned baselines.
ThreadSanitizer acceptance remains separate and incomplete on the recorded host.

This module is a prerequisite for annular ordered validation, history and
publication, not a public successful operation. Keep V1 selectable until the
recorded migration; remove superseded temporary adapters within this milestone
only after the complete operation and consumer corpus passes.
