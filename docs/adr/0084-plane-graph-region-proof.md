# ADR 0084: Plane graph region proof (touching loops and shared edges)

Status: implemented on a worktree branch (2026-10-15); integration and release are the
coordinator's.

## Context

The ADR 0066 part C loop proof certifies authored loops that are pairwise disjoint: every
source is used exactly once and every certified contact must be a join of consecutive uses.
Sketches routinely have adjacent regions that share an edge (a rectangle split by a line)
and loops that touch at a vertex (a figure eight). ADR 0068 declined both
(`BranchingProfile`, or an unexpected contact). The application needs every bounded face of
the curve graph, with the shared edge mapped to both faces.

## Decision

A second proof, `region_certificate::loop_region::graph::inspect_plane_graph`, reached
through `certify_profile_graph` and a version-2 `OwnedProfileRequest` (edges only, no
authored loops, regions or seeds). It reuses the ADR 0066 stages unchanged: preparation,
capture, the part B carrier census, partition and contact transport, the census binding,
the winding classifier and the part A certified integrals. Its steps:

1. **Embedding.** The expected contacts are exactly the vertex incidences: for every pair
   of sources and every pair of their ends with one caller vertex id, a contact at those
   exact native endpoints (and the closure of a closed single source, which must be alone at
   its vertex). The shared `certify_contacts` requires the complete current pair matrix, no
   other contact (`UnexpectedContact` with its world point) and every expected contact.
   Injective sources meeting only at shared vertices form an embedded plane graph.
2. **Rotation.** At every vertex the darts leaving it are ordered counterclockwise. A dart's
   direction is its first control distinct from its start (the exact tangent direction of a
   positive-weight Bézier). Comparison is exact: half-plane by coordinate comparison, then
   `orient2d` within a half-plane. All darts at a vertex must start at one chart point
   (`VertexChart`). Equal directions at a vertex of three or more darts are refused by name
   (`TangentBranches`): their order would need curvature. At degree two the order is unique.
3. **Faces.** The face orbits of the rotation system (leave by the clockwise neighbour of
   the reversed arrival dart, so the face is on the left). Each orbit's Green integral is
   certified (`certify_rational_boundary_integral`; an orbit that is the exact reverse of an
   earlier one takes the negated sign) and must exclude zero. Per connected component:
   Euler's relation `F = E - V + 2` must hold and exactly one orbit is negative, the outer
   walk; the positive orbits are the bounded faces. A bounded face whose walk revisits a
   vertex is refused (`PinchedFace`): its boundary is not a Jordan curve (for example a loop
   touching its enclosing loop from inside).
4. **Nesting.** Components are disjoint point sets, so the exact start point of one
   component's first source decides, by the certified winding, which bounded faces of other
   components contain it. Containers must come from distinct components and form a strict
   chain (`NestingContradiction`); the innermost is the component's container. The outer walk
   of a contained component is a hole of its container and must be simple (`PinchedHole`).
5. **Area.** Each bounded face's certified area: `certify_rational_face_area` of its walk
   minus its holes' walks (ADR 0066 part A).

The certificate lists the bounded faces (counterclockwise walk, component, holes, area) and
the components (clockwise outer walk, container), with contact and pair-domain counts.

### Public seam (ADR 0068)

`build_planar_regions` now certifies every profile through this proof; there is no
separate loop assembly. A vertex must end at least two curves (`OpenProfile`); branches are
admitted (`BranchingProfile` is no longer produced). Every bounded face is one region:

- outer loops are counterclockwise and holes clockwise in their `ProfileLoop` uses (the
  traversal of the face), each loop starting at its earliest caller curve; regions are
  ordered by the caller index of their outer loop's first curve, forward use first;
- `curve_edges()` maps a curve shared by two regions to both (the edge is used forward by
  one face and reversed by the other);
- new named declines, class `Unsupported`: `TangentBranches { vertex }` and
  `PinchedRegion { vertex, hole }` with the caller vertex, and `BridgeCurve { curve }` at
  the `Loops` stage for a curve whose removal disconnects its ends (it bounds no face on
  either side, and the shared native partition stage refuses such arrangements).

`RegionCertificate` gains `components`; `loops` counts bounded faces. For the previous
corpus (disjoint and nested loops) the regions, loops, areas and evidence counts are
unchanged.

## Evidence

Proof unit tests (`owned_request::profile::tests::graph_tests`, 8): a 2 x 1 rectangle split
by a chord has two unit faces using the chord in opposite directions, ten incidence
contacts, one component with a six-use outer walk; a figure eight of two triangles touching
at a vertex has two unit faces (its pinched outer walk bounds nothing and is admitted); a
circle in the left half of the split rectangle is a hole of that face only (`1 - pi/16`) and
its own disc (`pi/16`); a triangle touching its enclosing square from inside is
`PinchedFace` at the touching vertex; the figure eight inside a square is `PinchedHole`; a
line and a tangent quarter arc leaving one vertex vertically are `TangentBranches`; replay
is identical; cancellation is observed at every stage. The 30 previous seam tests and the
17 ADR 0066 part C tests pass unchanged (the old branching decline test now expects the
dangling vertex's `OpenProfile`).

Public seam (`profile_extrusion::graph`, 8 tests): a 3 x 2 rectangle split by the line
`x = 1` gives regions of area 2 and 4 (closed form and adaptive native face integration),
the chord's single native edge bounds both (opposite uses) and every other curve one; each
region extrudes to a valid six-face box with the chord's side face in its history; a figure
eight of two triangles touching at a vertex is two unit regions that extrude to valid
solids; a triangle touching its enclosing square from inside is
`PinchedRegion { vertex: 0, hole: false }`; a line and a tangent half circle leaving one
vertex upward is `TangentBranches`; a bridge between two squares is `BridgeCurve`; a circle
in the right half of the split rectangle is a clockwise hole of that half only (area
`4 - pi/4`, valid prism); results are equal across threads.

## Limits

Tangent branches at a vertex of degree three or more, pinched faces and pinched holes, and
bridge curves are refused. Orbits and faces are bounded by the integral limits (64 curves, degree 16 per
piece). The rotation uses chart controls: a vertex whose darts start at different chart
points is refused rather than merged.
