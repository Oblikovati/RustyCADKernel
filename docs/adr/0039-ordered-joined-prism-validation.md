# Ordered joined prism validation v1

## Decision and authority

RustyCADKernel#10 needs one ordered validation operation over one immutable
`JoinedPrismProof`. The private validation-only entry accepts that proof, explicit
versioned options and operation-local progress. It runs topology, geometry
consistency, self-intersection and tolerance consistency in that order. It does
not capture another body or select independently captured inspector results.

The admitted domain is the existing single-Jordan polynomial prism. Version 1
uses exact combinatorial, coefficient, conformance and embedding evidence in
that domain. Unsupported representations, missing current evidence, incomplete
search and unresolved current obligations cannot produce a complete validation
certificate. No geometry evaluator, graph owner, tessellation predicate or
production fallback is added.

The existing mapped validation entry and frozen replay remain selectable and
unchanged. The new entry is explicit opt-in authority for its declared domain.
Historical mapped coverage and typed findings remain in the joined bytes,
including `AlertIntersectionFailed` at fail gravity. They are historical checker
observations, never rewritten as successful exact checks. The new exact stage
evidence identifies its own authority and source. No diagnostic string selects
or suppresses an obligation.

## Stage obligations

1. **Topology:** execute the existing frozen topology certificate against the
   join's ordered V2 projection and require exact projection binding. Its closed
   containment, cyclic traversal, opposite directed edge uses, endpoint-germ
   links, connected shell and Euler obligations are unchanged.
2. **Geometry consistency:** consume the same join's actual coefficient/trim
   correspondence and pure joined outward/trim orientation certificate. Compute
   exact joined conformance once: six canonical edge endpoints and all twelve
   directed uses, six trims, and all twelve upstream semantic closure contexts
   against the captured original query. Raw local coordinates and actual owner
   frames are operands; a previous rounded point is not an equality oracle.
3. **Self-intersection:** consume the join's same-body nonzero-Jacobian,
   regularity and Jordan embedding theorem. Check the actual named domain and
   conclusion correspondence for all three self rows and all three unordered
   face pairs. Preserve rim/seam roles, multiplicities and authored occurrences;
   row count alone does not prove exhaustive coverage. The theorem supplies the
   abstract face-image premise required by topology. Outwardness remains the
   geometry stage's separate directional obligation.
4. **Tolerance consistency:** pass the already-computed conformance certificate
   to the pure joined metadata/cap operation. It verifies the complete join bytes
   and applies the explicit stored metadata policy. It never recomputes geometry
   or forces geometry to run before topology. Requested spatial/parameter/root
   settings remain distinct from exact zero-equality queries and stored caps.

Each constituent certificate must bind the full joined input appropriate to its
contract. Validation retains the full join replay bytes and original typed
constituent evidence. It does not infer identity from enumeration, coordinates
or counts and does not join different operations by equal scalar measurements.

## Completion, cancellation and diagnostic ownership

A complete result is named a **validation certificate**, not a completed prism
operation. Failed, incomplete and cancelled outcomes retain a typed operation,
stage, checkpoint, affected identities and original cause. A stage ledger keeps
the completed prefix; all later stages remain blocked. No partial successful
certificate escapes. Callback failures remain incomplete, independently from a
caller cancellation. Polling and row traversal are deterministic.

Failure means a witnessed violated obligation or explicit refusing policy.
Unavailable, unsupported or inconclusive proof is incomplete. Exact predicates
report their own exact evidence; the topology metric error is not applicable.
Stored metadata is not treated as achieved world-space error. A reporting cap
retains exceedance diagnostics; it is not silently changed into a refusing cap,
and no implicit unlimited cap is selected.

The input and published certificate are immutable. Operation-local builders
alone mutate the stage ledger. No mutable geometry alias becomes part of a
published result. Results bind the original joined options, limits, operands and
typed historical diagnostics. Success, failure, cancellation and subsequent
source mutation receive separate isolation checks.

## Operation publication remains separate

Validation completion does not supply complete original-member history,
requested/effective/achieved area and volume evidence, or final live source
rebinding. The Kernel#10 publication coordinator must compose those proofs with
this exact validation certificate, then perform the callback-free source rebind
before returning a successful complete operation result. This module does not
change `FrozenPrismProjection::is_valid_body`, existing mapped coverage, or
application acceptance. No standalone validator test authorizes a release.

## Baseline, corpus and migration

Owned baseline fixtures first retain mapped coverage, the typed historical
intersection alert, legacy V1 replay and missing publication obligations. The
new corpus exercises every stage's success/refusal/incomplete/cancel outcome,
strict early stopping, actual named rim/seam correspondence, unknown current
proof refusal, source isolation, exact process/thread replay and the complete
closed cubic fixture. Named test adapters exercise control flow; they never
serve as geometric acceptance evidence.

RustyCADKernel#10 owns replacing mapped checker authority only for this admitted
publication path after the combined full-operation corpus gate. The deletion
task and one-milestone limit remain ADR0014's application R3 ownership and exit
gate; this decision invents no additional milestone. Historical receipts and
legacy diagnostic replay remain available until a recorded migration. Broader
profile admission requires its own proof corpus. This private additive decision
targets the next compatible patch; it changes no workspace release version.

Ordinary thread replay and Send/Sync assertions do not replace ThreadSanitizer.
Any sanitizer startup failure remains incomplete. No application, performance or
release acceptance follows from this validation-only prerequisite.
