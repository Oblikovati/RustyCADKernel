# ADR0053: scoped composition of the owned polynomial prism operation

Status: private bounded implementation and candidate verification complete; main integration pending.

## Decision

Compose the existing owned request preparation, actual semantic correspondence,
publication, reservation inspection, portable body projection and physical-volume
interpretation. The admitted domain remains the single full-range polynomial
Jordan boundary of ADR0045. No new geometry, topology or numerical engine is added.

The new entry authors the complete V2 naming dictionary before construction.
Request, naming, publication and composition versions are distinct. The existing
V1 preparation entry, child callbacks and replay remain selectable and unchanged.
Publication uses the exact requested version and policies; composition never
upgrades a caller's numerical limits or diagnostic behavior implicitly.

One operation-local `Send` progress context assigns a monotonic outer ordinal to
all preparation, resolution, publication and projection checkpoints. Child phases
and local ordinals remain separate provenance. A refused checkpoint is terminal;
the exact outer callback cause does not replace or relabel any child failure.

## Ownership boundary

Success owns the original request, complete semantic map and reservation coverage,
portable scalar body, accepted private publication evidence and positive volume
interpretation. Its envelope binds all request bits, including caller revisions,
all policies and signed zeros, plus complete existing child replay bytes.
Private join provenance is not exposed as portable application naming: the body
read projection and semantic dictionary remain the portable view.

The entry invokes a higher-ranked finishing callback while all operation-local
owners remain alive. Exact preparation, resolver and publication failures remain
available inside that scope. A scoped failure cannot escape through the callback's
return type. This avoids a self-referential owner or unsafe lifetime extension.
It is **not** a complete owned refusal result: borrowed history diagnostics and
other unsupported failure projections remain a public-interface completion gate.

## Verification contract

First prove one real request-to-body operation through production constructors
and correspondence, including 13 members, 26 uses, six trims and all 101 V2
reservations classified. Preserve earlier V1 receipts byte for byte around the
structural preparation extraction. Then test every outer cancellation prefix,
native callback failures, reversal cases, input/revision isolation, thread and
process replay, and explicit empty/multiple-domain refusals. Separate executed
checks from pending checks and from ordinary-host ThreadSanitizer limitations.

This private composition does not close Kernel#10 general analytic profiles,
multi-boundary/hole coverage, application C1 acceptance, or public owned failures.

## Candidate evidence

[Final source-frozen evidence](../evidence/scoped-owned-prism-v1/FINAL.md) records
the actual complete operation, 1,957 affected tests, 35 doctests, every outer
cancellation/native-failure prefix and exact process/thread receipts. The public
owned-failure and ordinary-host ThreadSanitizer gates remain incomplete.
