# ADR0047: Portable named-prism body read projection V1

- Status: proposed; private implementation prerequisite, not public API acceptance.
- Scope: the accepted single polynomial Jordan prism operation from ADR0041.
- Source baseline: `892d9f99e8198731776568302a925d81846cf8ad`.

## Decision

Project an accepted `PublishedNamedPrism` into an owned scalar body read model using
one resolver-owned certified semantic correspondence. The projection does not
capture live shapes, populate another graph, evaluate geometry, assign names, or
change acceptance. The complete operation outcome remains responsible for
completion, diagnostics, ordered validation, history, and achieved error together.

The resolver owns the semantic member/use types and immutable correspondence under
`owned_request/names`. A member key qualifies the decoded source-member slot with
caller operation/boundary, sweep role and kind. This expands the existing authored
`FrozenPrismKey` injectively; that internal key itself contains no pointer.
Certified output-role associations are separate metadata: V1 has no dedicated
wire-member output token, so `LoopUse` cannot be repurposed as a member identity.
A use identity retains its complete ancestry and directed role. Repeated
vocabulary tokens are not occurrence identities. Caller namespaces
admit zero and the full unsigned 64-bit range; internal token restrictions do not
change caller identity. The body view refuses missing, ambiguous, foreign, or
colliding mappings and retains the exact dictionary/correspondence binding.

The owned rows retain named members, directed uses, orientation, scalar placement,
edge endpoint roles, coedge incidence, ordered wire traversal, surfaces, pcurves,
trim/native ranges, and raw/effective tolerance observations with explicit cap
policy and decision. Geometry coefficients and all binary64 operands remain
unchanged. Vertex-use rows retain exact local coordinates and affine placement;
captured rounded world points are separately labeled accessor observations and
are not substituted for exact affine transport. Containment order is not wire
traversal. Seam multiplicity and closed-edge endpoint roles remain explicit.

No mutable `Shape`, topology address, `Location` handle, or operation-local lookup
identity is reachable through this read model. Existing owned scalar capture
representations are reused behind the read interface. New portable encoding is
versioned independently; all historical frozen/publication V1 bytes remain
unchanged. The view does not overwrite diagnostic `is_valid_body` or historical
stage coverage. No standalone body view may create publication authority.

## Dependency and implementation boundary

The actual semantic resolver is a prerequisite for end-to-end acceptance.
Explicit test-only semantic maps can exercise row projection and strict coverage;
they do not establish production naming correspondence. Production construction
requires the resolver's immutable certified map, never a callback or a fixture
name allocator. The body-view owner does not implement a second resolver.

Use owned rows first and record copied coefficient/key counts, payload bytes,
allocation observations, and projection timing. Any later sharing/storage change
requires a separate measured proposal. Existing mesh factories and builders stay
the consumer integration seam; this decision adds no evaluator or tessellator.

## Independent expectations and gates

- For the retained single closed-edge prism, the authored seam quotient has two
  vertex definitions, three edges (two rims and one seam), six coedges, three
  wires, three faces, one shell and one solid. These give 13 definitions and 26
  containment uses, including 12 vertex uses. There are six face trims and eight
  tolerance-bearing definitions (two vertices, three edges, three faces). Endpoint
  roles remain two distinct uses even when a cap rim starts and ends at one
  physical vertex; the seam has two different side-face coedge uses.
- Compare every projected geometry/tolerance operand bit for bit with the already
  accepted captures; retain exact endpoint/coedge/traversal/trim correspondence.
- Refuse missing/duplicate/foreign map entries and semantic collisions; exercise
  repeated ancestry slots, start/end roles, seam uses, caller zero/full-range IDs.
- Preserve complete operation and frozen V1 byte fixtures unchanged.
- Recompute fresh getters and encodings after input mutation/disposal; assert
  `Send`/`Sync`, compare serial/thread/two-process bytes, and record concurrency
  evidence and host ThreadSanitizer limitations without claiming an unrun gate.
- Require actual resolver-to-publication-to-body-view correspondence before any
  public export or application-facing success claim.

## Migration and exclusions

This additive private V1 is not a public API break. Any public admission requires
its own reviewed semantic-version/changelog decision and owned outcome boundary.
No prior path is replaced, so no production deletion is activated by this slice.
Resolve the pending actual-correspondence gate within the Kernel #10 milestone;
do not extend this fixture-only prerequisite into an alternative production path.
Multiple boundaries, holes, rational/general profiles, persistent edit naming,
mesh publication, and application acceptance remain separate unfinished gates.

## Historical private row-projection checkpoint (`41b0250ed`)

`publication::body_view::project_named_prism_body` now requires an accepted
`PublishedNamedPrism` and an exact `ResolvedPrismNames` binding before projecting.
It retains one full naming-binding byte vector (which already contains the
publication binding), the caller plan, an explicit unique solid root occurrence,
and owned scalar/member/use/face/incidence rows. Complete-domain checks compare
sets and multiplicity, not merely counts. Raw vertex contexts, tolerance contexts,
face trims, edge endpoints, coedges, and ordered wire rows must all be covered.
Read methods expose no mutable topology or geometry handles. Replay is recomputed
from current owned rows using shared key, geometry and tolerance encoders.

The current tests exercise symbolic row mechanics, exact copying, source mutation
and disposal, threaded row replay, and refusal of an unbound symbolic map against
a real accepted publication. **They do not construct an accepted complete body
view.** The actual semantic resolver has no production issuer at this checkpoint;
its V2 dictionary/actual correspondence work is a pending dependency. Full body
replay, complete getter isolation, cost measurements on an actual accepted body,
and the final full toolkit gate remain pending that positive operation fixture.
The symbolic fixture is not a fallback or production name allocator.

## Actual resolver integration

The committed resolver source `a3b345b7e` (evidence `6ed91a0be`) supplies the first
actual sealed preparation→correspondence→publication→map issuance fixture. The
body uses that production-issued V2 dictionary/map unchanged; it does not decode
new roles or construct authority from symbolic rows. Map schema and body behavior
remain V1, independently of dictionary V2. The historical symbolic tests remain.

The actual corpus checks all 13 members, 26 uses, six trims and eight stored
tolerance observations, including both seam uses, three cyclic wire rows, native
closed-rim endpoint roles, and zero/full-range caller namespaces. Original
accepted geometry and metadata bytes are compared with projected operands.
Independent read-view snapshots rebuild every row from getters; request mutation,
disposal of the publication and map, and a test-only owned-row mutation distinguish
fresh values from cached binding bytes. Curve-buffer addresses are compared only
to demonstrate separate allocation, never to name a member. Construction owners
have already been dropped by the actual resolver fixture before body projection;
the earlier publication mutation corpus remains the live-owner isolation gate.

Retained allocation observations count accessible Vec blocks and capacities,
copied curve scalars, and the complete naming binding. The dictionary's private
capacity is not observable, so its entry payload is reported separately. Repeated
projection latency and process peak RSS are observations of this workload; they
are neither total transient allocator-call counts nor a previous-release whole
operation benchmark. No storage redesign or lock instrumentation is claimed.

Final toolkit/replay results and source hashes are recorded in the accompanying
actual-body evidence receipt. This private result read model does not expose the
whole join or add an application-facing outcome API; broader profiles, annular
publication, persistent edits and release benchmark/sanitizer gates remain open.
