# ADR0023: privately owned frozen prism projection

Status: accepted bounded implementation; behavior version 1.
Basis: `2543ec6d0ccce52f6e9c6811eb61a003532afd04`.

## Decision

Keep the existing copy, prism builder, analytic capture, exact metric, and graph
incidence pipeline. Add one private capture Interface that owns its output and
returns borrowed immutable views. A captured projection is diagnostic geometry,
never a successful valid body. Ordered validation remains an independent gate.

Require explicit caller-authored operation, selected-region, member, and directed
use identities. Operation-local addresses/context paths may locate an explicitly
bound object, but are never published identity. Derived keys contain only authored
IDs, unambiguous history/sweep roles, and explicitly bound parent uses. Missing,
colliding, or ambiguous identity bindings leave publication incomplete. Ordinary
receipts currently lack this input contract. No automatic naming heuristic is added.

Freeze coefficients, placements, incidence, tolerances, history, coverage, limits,
and supported typed evidence. No Shape, geometry Handle, Report, graph registry,
cache, or lock may be reachable from a captured result. Reject unsupported
diagnostic/evidence payloads; do not turn typed failures into strings. Certificates
must be bound to precisely the captured values. Reusing stale observations is not
allowed. Rational/general surfaces and synthesized topology remain outside this
bounded polynomial-Bezier/plane/linear-extrusion domain.

## Migration and version evidence

This adds a private module to an unpublished diagnostic path. It neither replaces
a released Interface nor changes a numerical algorithm, so no package semantic
version change is warranted. Version 1 identifies new capture behavior. Existing
private receipts and their replay fixtures remain selectable unchanged. Structural
exposure needed for reuse is committed separately from capture behavior.

Deletion task: remove alias-bearing receipt transport from the eventual publication
seam once its replacement gate passes, within the region-to-solid milestone. Keep
operation-local builders and existing evaluation engines. Gate: a complete admitted
prism with exact value/identity binding, full ordered validation, complete history,
source-mutation isolation, deterministic process/thread comparisons, concurrency
stress and successful sanitizer acceptance. An unavailable sanitizer is an explicit
remaining limitation, not a waived gate.

## Owned baseline

`crates/tkbo/tests/frozen_prism_projection/closed_cubic_v1.json` pins independent
incidence/formula expectations and the separately attributed existing stored-chart
convention. It predates production changes. The negative stored extrusion direction
and negative second surface parameter are retained exactly. Euler and scalar volume
checks do not establish geometry consistency or solid validity.

## Admission and evidence binding

Version 1 admits at most 4096 explicitly bound directed occurrences and at most
17 polynomial poles (degree 16); excess is refused, never truncated. Explicit
weights, including unit weights, remain outside this capture domain. Forward and
Reversed are the admitted orientation domain. Internal and External are refused
before local coedge reversal is computed. A graph coedge carries its authored full
use key, face/wire/edge keys and reversal; indistinguishable repeated uses are
incomplete rather than named by insertion order.

Every capture creates a fresh BRepGraph with explicit CumulativeOccurrenceV2
pcurve extraction, disabled automatic product creation, and an unchanged Audit
gate. This importer behavior affects newly extracted definitions; its metadata
alone is no certification or repair of arbitrary existing graphs. Graph IDs are
used only during operation-local lookup. Captured topology and replay order use
explicit stable keys; directed child order is the actual bound containment order.

Freeze re-enumerates and rebinds the actual body before graph capture and again at
Commit. The final pass recaptures geometry, graph incidence, history, coverage and
limits and compares exact replay bytes. Retained source/chart proof operands must
match fresh scalar values and actual source/use/history contexts. Checkpoint
callbacks may mutate legacy aliases; mutations must be detected. Concurrent writes
to those aliases during capture are outside the operation-local Interface.

Owned nodes retain raw TShape tolerance and compatibility-accessor effective
tolerance independently. Raw metadata must be finite and nonnegative; valid zero
and sub-floor values retain their exact binary64 bits. The existing BRepTool
floor is unchanged. Endpoint proof reuses the existing transported residual engine
on frozen operands with exact spatial limit zero and constant parameter maps.
This bounded equality admission is independent of requested body tolerances;
Evidence and Commit checkpoints surround it. It neither computes an aggregate
body error nor discharges whole-body tolerance consistency.

Historical validation observations remain explicitly unbound. Present unsupported
region/affine/report/alert causes are refused as UnsupportedDiagnostic. The known
concrete dataless AlertIntersectionFailed observation is retained with its observed
Fail gravity. Unknown or owner-bearing alerts are not reduced to strings. Native
intermediate retirement retains its copied source key, placement, orientation and
all original relation classes; it is not credited as composed original deletion.
Rejected selection history retains all parents with explicitly unbound outputs.
Selection, affine and whole-body evidence remain unimported/incomplete.

## Verification scope

The owned fixture supplies semantic identities before inspecting generated output.
Its baseline bytes are unchanged. Tests exercise one complete diagnostic capture,
all cancellation/failure checkpoints and reuse, clone isolation, exact source and
body mutation rejection, raw/effective tolerance distinctions, missing/colliding
identity bindings, typed alert refusal and bounded-domain limits. Compiled
Send/Sync assertions, ordinary deterministic thread/process tests and sanitizer
acceptance are separate obligations. The final evidence receipt records actual
commands and limitations; no successful body, application acceptance, or
previous-release accepted-workload benchmark is claimed. Function-size audits use
executable body lines, excluding formatted signatures and documentation; files
remain below 500 lines.

Known refusal entities accompany the original typed cause through an owned
context wrapper. Node, face and history occurrence failures retain the authored
primary key and directed-use context. Endpoint numerical errors and mismatches
retain their edge/vertex pair. The outer failure exposes the primary key when
known; global or genuinely unbound failures keep it absent. Wrapping does not
change native failure classification or discard numerical error operands.
