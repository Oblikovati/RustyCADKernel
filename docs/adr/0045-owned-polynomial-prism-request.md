# ADR0045: private owned polynomial prism request

Status: admission/construction/orchestration checkpoint; semantic correspondence
and complete publication integration remain pending. Behavior: owned request V1.
Kernel #10 bounded prerequisite, not public API or release completion.

## Small caller Interface

The caller supplies an operation ID, a collection of polynomial sources, declared
plane, bounded-region-containing seed, extrusion and explicit policy bundle.
Each source owns world-space poles, source/revision/closure vertex/native start
and end use IDs and an independent traversal flag. There are no Shape/Handle
inputs and no caller-authored intermediate stage IDs. The operation keeps exact
input values; a revision number alone never establishes reusable evidence.

The policy bundle preserves separate census, original/construction conformance,
contact parameter/world, classification, prism spatial/parameter, and complete
publication/validation/history/measurement settings. The preparation stage does
not claim that deferred publication settings have been applied. Existing child
operations perform their own policy validation without silent redistribution.

Empty input is a typed decline. Multiple sources and unsupported request versions
are explicit incomplete domain refusals. Open boundaries are incomplete in this
closed-boundary domain. Invalid scalar/name inputs fail admission; numerical and
mapped operation failures retain their actual typed child causes. Outside and
boundary seed outcomes stay explicit private refusals, not completed empty solids.

## Constructor authority and shared pipeline

ADR0042 creates the semantic dictionary before construction. Its fixed Original
Boundary token is the operation-local RegionSourceId; the full caller source ID
remains in the dictionary and owned request. No source enumeration supplies names.

The constructor reuses GeomBezierCurve, BRepBuilderAPIMakeVertex and
BRepBuilderAPIMakeEdge::from_curve_vertices_params with the same explicitly held
closure vertex at native parameters 0 and 1. It retains the actual forward edge,
native endpoint uses and the independently oriented source edge. Caller start/end
IDs are never swapped when traversal reverses. The existing BRepTool parameter
query on a reversed closed edge swaps its endpoint interpretation: tests pin
native forward-edge 0/1 separately from oriented-source 1/0.

The production orchestration calls existing source capture/conformance census,
owned partition observation, actual candidate copy observation, partition binding,
paired contact transport and retain_source_bound_prism. It does not call any
cfg(test) fixture, add another classifier or change existing entry points.

A prepared stack retains only private diagnostics and the existing incomplete
PrismReceipt. It is not a successful operation result. Partial failures retain
request/source and actual available stage receipts; errors consumed by a child
retain that child's original payload. These private failures may contain mutable
local topology and are deliberately outside the future owned refusal contract.

## Context and ownership

The injected progress trait is Send and has ordered, typed phase/ordinal events.
The first callback cancellation/error is terminal; a callback error remains an
outer Failed outcome while preserving any child cancellation diagnostic. Native
constructor/copy/interference calls remain synchronous; cancellation is observed
at their existing checkpoints and added stage boundaries, not inside those calls.

Inputs are owned scalar buffers. Constructor-created geometry is operation-local;
retained native and oriented handles intentionally alias that private definition.
The context and actual prepared/failed state have compile-time Send assertions.
They are not published immutable bodies. Full threaded replay/TSan and immutable
publication proof remain whole-operation gates, not consequences of Clone or Send.

## Next correspondence/publication gate

Resolve predeclared names through actual constructor handles, native parameters,
copy/history relations, complete selected/complement domains and exact sweep/trim
roles with full parents. Missing, duplicated, extra or split correspondence must
refuse. Generic FaceUse/LoopUse tokens require full role-bearing keys and ancestry.
No caller sees those private binding structures. Consume the reviewed first
complete publication operation over the resolved same-body inputs, with its final
commit/rebind protocol, before exposing any completed result.

The initial version neither accepts multiple/hole profiles nor closes persistent
split identity, application edit/undo/save-reopen/display, canonical Martensite#13
or Kernel #10 release acceptance. The older stages and fixtures remain selectable.
