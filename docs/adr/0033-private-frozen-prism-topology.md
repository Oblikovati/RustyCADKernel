# Private frozen prism topology certificate v1

## Decision and scope

RustyCADKernel#10's analytic region-to-solid gate needs exact combinatorial
incidence evidence distinct from mapped BRepCheck observations. The private
`frozen::topology::certify_frozen_topology` returns either one immutable complete
certificate, a witnessed typed failure, typed incomplete admission/callback
failure, or cancellation at an observed checkpoint. No partial certificate
escapes. No body, topology owner, geometry evaluator, mesh engine, healing step,
or history mutation is introduced.

The certificate binds every byte of one immutable frozen projection, including
its original history, limits, geometry, containment, graph incidence and ordered
traversals. `report.binds(&projection)` is the exact read-only composition join.
It does not join independently captured receipts by coordinates or counts.
Operation and root authored identity remain explicit. The certificate's version
is 1; evidence is `ExactCombinatorial`, metric achieved error `NotApplicable`.
There is no requested/effective metric tolerance to widen. Existing frozen
coverage and `is_valid_body()` remain unchanged and incomplete/false.

## Required capture behavior: ordered traversal V2

An owned baseline exposed a distinction hidden by the initial design: source
ShapeIterator children preserve insertion order, not cyclic wire traversal. The
actual side insertion is `up, down, first-loop, last-loop`. The first two neighbors
connect, but the loop transition and wraparound do not. Existing graph population
already obtains wire order using the existing WireExplorer/canonicalization
pipeline. V1 discarded that order when sorting its coedges by authored identity.
The retained red run and minimized endpoint table are acceptance evidence for
this correction, not evidence that the original source wire was invalid.

`freeze_prism` continues selecting `PrismFreezeBehavior::ContainmentV1`. The
additive `freeze_prism_with_behavior(..., OrderedTraversalV2)` selects frozen
version 2 and retains owned rows `(wire occurrence, ordered coedge occurrences)`
from the **same** populated graph. Containment child order is never changed.
V1 has no rows and its exact replay bytes remain identical; V2 appends rows to
its replay and includes them in the final callback-free recapture comparison.
There is one population pass per initial or final capture, not a second order
engine. Stable identities still come from existing unique actual-context
bindings, never graph position, coordinate rank, or proximity.

Existing population admits only `AddStatus::Success`. Partial explorer order,
appended unmatched edges and tolerance/invalid canonicalization order propagate
`SuccessWithWarnings`, already refused by V1. That existing gate remains common;
new traversal extraction runs only for V2. Consequently no honest V1-positive,
partial-explorer fixture is asserted. Each ordered coedge must map uniquely to
an actual context; topology independently proves a bijection to every original
wire child, then checks every cyclic corner. An explorer success flag alone
never establishes topology. V1 or V2 without traversal evidence is incomplete.

## Exact obligations and orientation mapping

The operation runs these checks in order:

1. Unique authored definitions and full occurrence ancestry, a single solid root
   and shell, correct Solid/Shell/Face/Wire/Edge/Vertex containment, exactly one
   occurrence of each container, no orphan or duplicate child references. Each
   face has one nonempty wire in this bounded domain. Counts are recomputed.
   Every edge has one canonical start/end row and each edge occurrence has one
   start and one end vertex use, distinguished by vertex-versus-edge cumulative
   orientation. Coedge face/wire/edge names agree with actual ancestry, and its
   local reversal agrees with edge-versus-wire cumulative orientation.
2. Ordered wire closure at **all** corners, including last to first. The local
   coedge reversal chooses canonical start/end. A one-coedge cap is legitimate
   precisely when those vertex identities coincide.
3. Exactly two distinct directed uses of every edge, with opposite cumulative
   directions in the common shell frame. Two uses in the same face are allowed:
   the side seam is not rejected by counting distinct faces. No supplied face
   orientation is automatically flipped or repaired.
4. Each physical vertex has one connected degree-two link multigraph. Link
   vertices are `(edge identity, Start|End)` **germs**, even when a loop edge's
   physical start and end vertex coincide. Every cyclic face corner contributes
   one link edge. Parallel corners and self-loops retain multiplicity; a
   self-loop contributes degree two. Disconnected fans at a pinched vertex fail.
5. Face adjacency through paired edges has one connected component. A same-face
   seam contributes no artificial connection between different faces.
6. Exact integer `V - E + F = 2` for the declared single-Jordan prism topology
   domain, after the local manifold and connectivity checks. Counts alone are
   never sufficient.

Whole-shell reversal preserves these combinatorial obligations. Reversing one
face and its descendants can violate edge pairing. Consistently reversed wires
may still satisfy this certificate. **Geometric outwardness, chart-normal/trim
winding consistency, embedding, self-intersection, tolerance consistency, and
boundary-of-solid validity are separate gates.** The tests' authored tetrahedron,
two-disk sphere and torus words are finite combinatorial fixtures, not a mesh
used for modelling decisions or geometric acceptance.

## Bounded work, isolation and replay

The existing 4096 occurrence cap also bounds admitted definitions, coedges,
endpoint rows and total ordered entries. Excess is incomplete without clamping.
All indexes are operation-local ordered maps/sets borrowing the immutable source.
Integer counts are bounded before arithmetic. Checks poll in deterministic order
through containment, wire corners, edge pairs, vertex links, face traversal and
commit. Callback failure remains typed incomplete; cancellation never publishes
partial success. No shared mutable state or unsafe Send/Sync implementation is
introduced. Published report reads require no lock and own their source bytes.

Owned fixtures cover the actual prism's two loop caps and same-face seam;
missing/duplicate/foreign/misordered traversal; wraparound and single-edge open
wires; one/three/equal-direction edge uses; pinched and disconnected complexes;
a torus failing Euler2; whole-shell reversal, one-face reversal, and all 16
orientation masks of a tetrahedron. Source isolation, every observed cancellation
and callback-failure checkpoint, exact process/thread replay and scalar Send/Sync
assertions are separate gates. Ordinary host-thread checks do not substitute for
ThreadSanitizer; any host startup failure remains explicitly incomplete.

## Migration and deletion task

This is additive private evidence for next compatible patch target 0.2.8; no
workspace version or publication changes occur here. The RustyCADKernel#10
immutable publication coordinator owns selecting V2 and composing this exact
source-bound certificate with the geometry, embedding, tolerance and history
proofs. Within the same C1 milestone it must remove V1 selection from the new
publication path after the combined corpus gate. V1 remains explicitly
selectable for diagnostic replay until that recorded migration; historical
mapped coverage is not relabeled exact. Release/application acceptance and
previous-release performance acceptance remain separate.
