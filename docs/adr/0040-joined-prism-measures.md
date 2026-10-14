# ADR0040: named measurements over an immutable prism proof

- Status: private behavior implemented; owner integration and release gates pending.
- Owner: Kernel #10.
- Date: 2026-10-14.
- Behavior: `JoinedPrismMeasureOptions::v1`.

## Decision

Use the existing complete polynomial prism integration pipeline on the captured
operands of `JoinedPrismProof`. Keep its exact requested extrusion, coefficients,
trim intervals, placement operands and separate cap, side, volume and total-area
policies. No live receipt, geometry handle, evaluator, tessellation or recapture
is needed during this calculation.

The certificate owns the full join replay bytes and the authored face/trim names
with every parent multiplicity. Named face lookup uses the authored occurrence;
role lookup only chooses the scalar report after unique chart admission. Names
are never derived from returned position, coordinates or process addresses.
All three face areas, signed volume and total area retain their original typed
certified bounds, requested/effective policies and work. Total area is the
existing sum of both cap enclosures and the side enclosure; no component budget
is reassigned to make the total pass.

A completed result certifies these scalar quantities conditional on the joined
source hypotheses. It does **not** make `FrozenPrismProjection::is_valid_body`
true, discharge ordered validation, supply complete source history or authorize
release. The complete operation must combine those checks on this same join and
perform the shared callback-free source rebind after its final callback.

## Ownership and errors

The pure scalar kernel now has a narrow owned error vocabulary. Exhaustive
one-way adapters preserve all old live-inspection error variants and payloads.
The joined API cannot return a legacy mutable shape/report inside an unreachable
error variant. Result, named report views, failure and nested causes are checked
as `Send + Sync` at compile time. Source geometry can be mutated or destroyed
without changing retained results; publication freshness remains a separate
source-rebind obligation.

Every refusal retains the operation identity, affected authored face contexts,
original options, classification and the typed arithmetic cause. Cancellation
inside cap, sweep, side or total arithmetic stays cancellation. Work exhaustion,
unsupported policy and unmet bounds stay incomplete; no partial certificate is
returned. Initial and final wrapper checkpoints enclose the original numerical
checkpoints without changing the older live API.

## Refactor and replay

The older complete scalar process fixture was pinned before extraction at
`f33490bc8`; structural changes `1e96d18ac` and `fddab07cb` retained the exact
cap/subset and complete scalar bytes and old failures. The latter narrows errors
without changing arithmetic. The shared region hypothesis helper replaces the temporary duplicate operand
checks in structural reconciliation `20703816f`; all earlier scalar and joined
replays and all 718 cancellation cutoffs remain unchanged.

New replay stores the complete joined source bytes, behavior version, exact
binary64 result/option/error bits, both requested and effective child policies,
side work and all total components. The named source and trim operands are
already part of the full join. The v1 schema fixes the scalar quantity/evidence
kinds; it does not serialize a process-dependent `Debug` representation of live
occurrence identities.

## Acceptance and scope

Owned tests check closed-form cap/volume results and the independently generated
rational/square-root side and total bounds in the existing owned cubic fixture.
They check all observed cancellation cutoffs, separate policy failures, exact
source binding, authored binding permutation, mutable source isolation,
independent workers, shared immutable readers and complete process replay.
Eight generated power-of-two scale, traversal and positive/negative extrusion
cases also check independent cap, signed-volume, side and total bounds. Owner
full-toolkit integration remains required before the final accepted receipt.

This first admitted domain remains one complete polynomial Jordan boundary and
three corresponding swept faces. General profiles, rational/spline extensions,
multiple or empty region results and application acceptance remain Kernel #10
work. No domain is silently faceted or treated as a valid body after an
inconclusive check.

## Migration and deletion

Keep the earlier live measurement entry selectable for replay during the Kernel
#10/application R3 migration defined in ADR0014. Once the full published operation
passes the owned corpus and application adopts its released package, delete the
superseded live measurement wrapper and duplicate capture path within that same
milestone. Keep the pure integration engine and historical fixture as regression
resources; no second integration implementation is introduced.
