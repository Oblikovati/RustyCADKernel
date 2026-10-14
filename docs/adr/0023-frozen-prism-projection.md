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
