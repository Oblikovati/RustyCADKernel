# ADR0042: owned single-boundary semantic names

Status: private dictionary prefix; correspondence adapter and complete operation
integration pending. Behavior: semantic-name envelope V1. Kernel #10 prerequisite,
not general-profile or release completion.

## Decision

The caller supplies typed operation, boundary, shared closure vertex and distinct
native-start/native-end directed-use identities, plus bounded-region-containing
seed intent. All `u64` caller values are available, including zero and values equal
to private tokens. No copy-stage, container, cap or seam names are caller inputs.

`owned_request::names::SingleBoundaryNamePlan::v1` declares its entire symbol
vocabulary before any geometry lookup. V1 uses explicit stage and role
discriminants: `token = (stage << 8) | role`. These are lexical constants,
never result ranks, source enumeration, coordinates, addresses or hashes.
The dictionary retains the exact operation, source and typed caller anchor.
Native endpoint roles remain separate from reversed traversal and pole order.

The 97 entries are **reserved symbols**, including slots not used by an actual
operation. They are not 97 constructed entities, expected incidence counts or a
coverage certificate. Parent paths in existing compound keys distinguish uses
of shared definitions. Generic FaceUse/LoopUse tokens are never occurrence keys
on their own: full definition keys retain First/Last/Generated roles and full
parent ancestry distinguishes cap versus side uses in semantic projection. A later adapter must bind only applicable slots and prove
complete unique actual correspondence. An unused reservation is not evidence
that an entity exists or is missing.

## Additive compatibility

Existing `AuthoredPrismId`, `FrozenPrismKey`, `FrozenUseKey`, copy/history keys,
serializers and fixture bytes are unchanged. Internal token values will be passed
to those existing private bindings only through a reviewed correspondence adapter.
The token and caller-ID namespaces remain separate; external values are never
interpreted as tokens.

`SemanticReplayEnvelope` is an owned envelope for the name plan and opaque child
bytes. It serializes its format version, all caller IDs, exact seed bits, every
dictionary entry and unmodified length-delimited child bytes. Its constructor
accepts arbitrary child bytes and therefore grants **no certificate or body
completion authority**. Full plan equality includes signed-zero seed bits.

The envelope is not a public projection of `JoinedPrismProof`. That private join
still contains process-local scalar identities and immutable `Location` handles
in provenance. Portable application views must explicitly project through the
semantic dictionary; they must not expose the join wholesale or mistake private
numeric keys for globally persistent identity. A digest may index full bytes but
cannot replace full binding equality.

## Required next adapter

Reuse constructor-retained source handles, exact closure endpoint roles, existing
copy/history queries, fresh selected/complement classification and sweep
First/Last/Generated/chart/trim correspondence. Author the plan first, resolve
uniquely second, and refuse missing, duplicate or ambiguous mappings. Neither
shape kind/orientation alone nor common original parents authors an identity.
Unsupported splits and multi-boundary profiles remain incomplete. No production
resolver, source constructor or publication entry is supplied by this prefix.

The private complete operation's accepted history/publication gate must precede
consumer exposure. Application naming across split/edit/undo/save/reopen and the
canonical Martensite#13 with-hole workflow remain separate obligations.

## Evidence and limits

The fixture generator in `docs/evidence/prism-semantic-names-v1` independently
encodes literal stage/role/anchor expectations. Tests cover full binary equality,
all caller fields, token-value collisions, full caller range, endpoint-name
permutation, invalid selection, unknown reverse lookup, input-byte isolation,
legacy payload preservation and owned Send/Sync/thread equality. The prefix has
no mutable shared state, geometry evaluator or callbacks. Full operation
cancellation/source-isolation/actual correspondence tests belong to the later
adapter; passing this dictionary corpus does not substitute for them.

ThreadSanitizer and the full TKBO operation suite are not implied by the focused
prefix checks. Source-bookended executed commands and exclusions are retained in
the accompanying evidence receipt before handoff.

## V2 aggregate vocabulary extension

Actual Partition and CandidateCopy contexts contain an intermediate Compound
wrapper. Historical member naming requires one physical definition per member
symbol; assigning its Boundary symbol to both wrapper and edge would violate
that contract. A bounded region or loop symbol is also not the meaning of this
unclassified container. V2 therefore adds exactly four internal reservations:
Partition Aggregate/ AggregateUse (1058/1059), and CandidateCopy Aggregate/
AggregateUse (1314/1315). Roles 34 and 35 are explicit lexical constants. Their
caller anchor is the source boundary; actual membership still requires complete
native partition/copy provenance and ancestry correspondence.

`SingleBoundaryNamePlan::v2` reserves 101 symbols. `v1` retains exactly 97 and
rejects the four new tokens. Every earlier entry and V1 byte remains unchanged;
V2 envelopes record version 2. The legacy request preparation still authors V1.
A new whole-operation path must author V2 before construction; the private
resolver may consume an explicit V2 plan authored before lookup and must compare
its complete caller identities and exact selection bits to the retained request.
No stage or aggregate names become caller inputs. This is no automatic migration
of V1, and dictionary reservations remain distinct from observed coverage.
