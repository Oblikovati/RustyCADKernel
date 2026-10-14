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

## Explicit publication diagnostic retention V2

Baseline `75dab08f3` pins the V1 callback wrapper: its outer phase/class describe
the callback, and its retained downstream value contains only the child's cause.
The child's former phase/class cannot be reconstructed from that payload. The
owned adapter must distinguish the publication-operation class from a child
classification and record unavailable legacy child headers explicitly.

An additive `publish_named_prism_v2` entry requires publication options version 2.
The original `publish_named_prism` entry continues to require version 1, including
its exact refusal of any other requested version. Both entries share one admitted
geometry, history, measurement, callback and final-rebind implementation. V2
selects `CallbackWithChild`, which retains the complete actual child stage failure
(phase, class and cause) before callback precedence replaces the outer header.
Absent children remain absent; nested children retain each distinct header. No
legacy receipt is retroactively enriched. Successful V2 results identify
`CompleteNamedPrismV2` and encode requested version 2 in the existing V1 wire
format; all child proof bytes remain identical for identical accepted inputs.
Wire-format version and requested publication behavior are separate.

The Kernel#10 operation-contract owner must migrate actual invocation consumers
to V2 and delete the lossy V1 producer after the retained-header corpus and full
operation migration gate, within the existing ADR0014 application R3 limit. V1
remains explicitly selectable for historical replay until that recorded migration.
This private additive behavior neither expands the geometry domain nor changes
the workspace release version. The future invocation adapter must select V2
explicitly and retain that choice in the actual request context.

Complete caller-qualified refusal publication still requires the sealed actual
invocation packet and bounded typed adapters/codecs. Validation, measure, history
and unsupported join numerical/frozen families remain explicit completion tasks;
unsupported local owners cannot become lossless diagnostics through Debug text
or a known outer class. Preserving producer headers does not close those gaps.

## Sealed publication adapter V1

The private `project_invoked_publication_refusal` consumes only
`InvokedPrismPublicationFailure`, issued by the actual resolver invocation. The
issuer owns the source vector and bindings and captures the exact request and
actual V2 naming plan at the call. Neither the adapter nor its tests add a packet
constructor accepting detached context and failure values. The invocation
explicitly dispatches requested publication behavior 2 to the V2 producer; all
other requests retain the original V1 entry and its admission behavior.

The adapter verifies the observed operation token, exact encoded publication
policy bits, and caller operation/source/endpoint dictionary against the sealed
request. Equal binary64 NaN payloads compare equal as requested bits; signed zero
and different NaN payloads remain distinct. These checks supplement actual issuer
provenance; reusable internal operation token 257 is not proof of that provenance.
The owned context includes the complete request and actual plan version. Tests
execute otherwise identical callers with different IDs, rather than treating
those internal tokens as persistent caller identities.

Outer publication class and phase are copied unchanged. V2 retains each actual
stage header. The reused join projector receives a class with an explicit origin:
`RetainedStage(phase)` for a retained stage, or `PublicationOperation` for a V1
callback descendant whose child header was discarded. A V1 descendant always
records `UnretainedChildHeader`; no cause-based inference reconstructs its former
class. Actual nested binding cancellation currently has child class Incomplete
under the existing publication classifier, while a native callback makes the
outer operation Failed. The adapter pins that distinction without changing it.

Supported lossless cases are outer version refusal, callback cancellation/native
failure/exhaustion and exact checkpoint, binding proof label, history cancellation
header, and join missing-query/version/cancellation/stale-label cases. A join
binding cancellation or mapped native failure is supported. Embedding versions
and those binding causes are supported only without an original identity payload.
The adapter reuses ADR0043 projection and the existing native provenance encoder;
it does not duplicate identity resolution. A nonempty join entity is conservatively
recorded as a full unqualified authored key and a projection gap until the actual
semantic-map projection is supplied. A foreign observed history operation also
records a gap rather than attributing it through the outer operation token.

The distinct full format `Martensite.OwnedPublicationDiagnostic/v1` plus NUL
contains length-delimited actual context, observed internal operation, explicit
outer phase/class, diagnostic coverage, typed supported cause tree, class-origin
tags, checkpoint ordinals, and ordered gaps. Supported join bytes use a separate
versioned block with exact embedding options, phases and native provenance.
Primitive literal fixtures pin tags and operands; actual invocation process and
thread fixtures use this complete format. No Debug or Display text is a substitute
for unsupported payload encoding. Encoding the available envelope completely does
not claim complete projection of local owners.

On any gap, the consuming result retains the same original boxed publication
failure, including all borrowed history diagnostics. The immutable owned report
has Send/Sync assertions and lock-free read tests; the retention carrier has no
published ownership or Send/Sync claim. Successful diagnostic projection conveys
no successful body. It does not claim the final one-call failure gate is complete.

### Remaining completion work

The Kernel#10 operation-contract owner must close the actual reachable refusal
corpus before final owned one-call outcome acceptance. This includes:

- Resolver failures before a publication packet exists: preserve the actual
  request/plan availability at invocation, scalar/plan/native operands, and exact
  Binding/Prism/History causes with explicit unknown entity context until bound.
- Ordered validation, complete-history and complete-measure typed payloads,
  including precise numerical limits/work/error and actual identity observations.
- Remaining join closure-query/region, embedding regularity, frozen capture,
  binding and link families, including qualified entity/original correspondence.
- Borrowed `PrismHistoryLocalDiagnostic` partition, arrangement and candidate
  payloads. An owning prepared object containing a failure that borrows itself is
  not an accepted immutable publication representation. Retaining the local
  original is an intermediate inspection path, not a completed public solution.

The existing migration owner and ADR0014 application R3 limit apply. The lossy V1
producer remains selectable until its recorded migration. No public completeness,
release benchmark, application acceptance, or successful TSan suite is claimed.
