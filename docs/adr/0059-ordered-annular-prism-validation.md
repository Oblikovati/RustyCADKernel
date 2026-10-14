# ADR0059: ordered validation for a bound annular prism

Status: development candidate; integration and full verification remain separate.
Scope: private Kernel #10 prerequisite, using the shared exact validators. This
neither publishes a successful solid operation nor completes C1 or release scope.

## Decision

`validate_annular_prism` takes the already sealed `AnnularFrozenDomain`, an explicit
V2 behavior selection and stored-tolerance cap. It reuses the existing stage ledger
and composes the annular incidence checker, named orientation, exact geometry,
sealed embedding correspondence and stored metadata in this order:

1. Topology over the actual annular cap parameter domains.
2. Geometry: actual face/trim direction, then exact endpoint/closure/trim checks.
3. Self-intersection: correspondence of four complete domains and all ten face
   pairs with the already certified affine annular embedding theorem.
4. Tolerance: the same geometry certificate plus all fourteen physical metadata
   observations under the caller's explicit policy.

The table check reconstructs domains and claims using the existing bound domain
capture and theorem-table functions. Full source/output contexts identify rows;
returned order has no identity authority. Duplicate, omitted, foreign or changed
claims are incomplete. It introduces no geometric search, sampled classification,
new intersection theorem or approximation policy.

## Completion and ownership

Only a completely passed ordered ledger constructs the sealed validation result.
It owns every child proof, the complete joined input bytes, requested policy and
historical checker findings. Historical findings remain separate from current
exact coverage. Body publication, complete source history, area/volume and final
live source rebinding belong to the later operation composer.

Failures own the operation ID, current checkpoint, typed cause, complete source
binding, stage trace and already completed children. Required later stages remain
blocked. A callback failure retains its original native cause independently of
its downstream child stop; cancellation is distinct. Checked ordinal addition
reports exhaustion as incomplete. Once a callback stops, subsequent child polls
cannot replace its checkpoint or call the application again.

There is no global state or lock in published evidence. Progress is `Send`;
result, partial evidence and failures have compile-time `Send`/`Sync` assertions.
Ordinary threaded tests do not replace the outstanding ThreadSanitizer gate.

## Versions and reuse

Existing V1 validators, receipts and replay bytes stay selectable and unchanged.
V2 is a domain-specific composition of those existing geometry and topology
engines, with explicit annular proof types. The eventual owned general-profile
operation must select admitted domains internally, without exposing this stack of
private certificates as an application responsibility. No public breaking API or
version bump is introduced by this private addition.

The final general-operation migration owns removal of superseded private
orchestration after its full corpus and application gate in C1; this addition does
not extend that migration limit or replace the original domain's proof with a cast.

## Verification

Development tests exercise the actual native-built nested-cubic prism, four named
faces, twelve directed trims, twenty-four directed endpoint contexts, twenty
upstream closure rows and fourteen metadata observations. Independent expected
stage order and finite pair counts accompany exact child-binding checks.

The corpus includes changed/duplicate/omitted domain and incidence rows, explicit
cap refusal, wrong behavior version, every cancellation and native callback stop,
first-stop precedence, ordinal exhaustion, input-owner destruction, independent
threads and full process replay. Complete root/candidate matrices and immutable
source snapshots are required before integration acceptance. Source-provenance
corrections from the conformance owner must be included in the final candidate.
