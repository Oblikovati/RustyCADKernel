# TKPrim port status

**In progress; toolkit acceptance is not complete.** `crates/tkprim` (`rustycad-tkprim`)
is the port of the ModelingAlgorithms toolkit TKPrim. It depends on the TKernel, TKMath,
TKG2d, TKG3d, TKGeomBase, TKBRep, TKGeomAlgo and TKTopAlgo ports, as the native toolkit
does (its EXTERNLIB lists TKBRep, TKernel, TKMath, TKG2d, TKGeomBase, TKG3d and
TKTopAlgo; TKTopAlgo brings TKGeomAlgo). Each of the 5 OCCT packages is one module of the
crate, declared up front in `src/lib.rs`; a module whose package is not ported yet holds
only its documentation. Ported so far: Sweep, BRepPrim and BRepSweep.

[prim-design.md](prim-design.md) fixes how the C++ constructs of the toolkit (the
abstract sweep with its virtual geometric members, the one-axis primitives with their
virtual meridian, the makers derived from `BRepBuilderAPI_MakeShape`) map to Rust.

A package may start when the packages it includes are ported (the lower toolkits are
ported):

| Level | Packages |
| --- | --- |
| 1 | BRepPrim, Sweep |
| 2 | BRepSweep |
| 3 | BRepPrimAPI |
| 4 | BRepPreviewAPI |

The levels come from the `#include` lines of each package: BRepSweep includes Sweep,
BRepPrimAPI includes BRepPrim and BRepSweep, BRepPreviewAPI includes BRepPrimAPI.
Outside TKPrim, the packages need BRepTools, BRepAdaptor and BRepLProp (TKBRep), GeomProjLib
and Extrema (TKGeomBase), and BRepLib, BRepBuilderAPI and BRepExtrema (TKTopAlgo).

The toolkit test inventory maps these suites (0 declarations):

| Source | Rust target | Declarations | Native | Rust |
| --- | --- | ---: | --- | --- |

## Verification

`python3 tools/verify_toolkit.py TKPrim --jobs 16 --source OCCT --native-build BUILD`
compiles the unchanged upstream suites against the native closure, runs them and the
Rust targets, checks exact runtime names, then runs `tools/verify_tkprim_numeric.py`.
While no suite is mapped it runs the numeric verifier alone and reports 0 mapped
declarations. Evidence is in `target/oracle-tkprim/`. CI runs it like the TKTopAlgo
verifier. The shared native build already builds and links `libTKPrim`.

## Numeric comparison policy

`tools/verify_tkprim_numeric.py` uses the shared driver `tools/toolkit_numeric.py`. It
builds `tools/oracle/tkprim_probe.cxx` (a table of operations over
`tkprim_probe_<package>.cxx`, linked with libTKPrim and the toolkits below it) and its
twin `crates/tkprim/examples/numeric_probe.rs` (one module per probe file under
`examples/pr_probe`), runs both on `tools/tkprim_corpus.py` (one
`tkprim_corpus_<package>.py` per package) and requires identical tokens: doubles as IEEE
bits (NaN by classification), exception types by name. Both probes reuse the TKGeomAlgo
reader and printers (`tkgeomalgo_probe_common.cxx` and `ga_probe/common.rs`) and the
TKTopAlgo shape digest `tp_shape` (`tktopalgo_probe_breplib_common.cxx` and
`tp_probe/brep_lib_common.rs`): every sub-shape of a result in `TopExp::MapShapes`
order, with its type, orientation, Closed flag, location, children and geometry (vertex
points and tolerances; edge tolerances, flags, 3D curves, curves on surfaces and vertex
parameters; face tolerances and surfaces). The corpus runs in 64 round-robin shards
(`tools/probe_shards.py`), so every request prints exactly one line and never relies on
an earlier request. `tools/coverage_report.py` replays the saved corpus under coverage.

The shared digest: `pr_digest_vertex` prints the digest of a vertex built with
BRep_Builder on 27 points and tolerances (signed zeros, tiny, huge and decimal values,
20 seeded random vertices). It proves that both probes print the shared digest
identically before any TKPrim class is compared.

Sweep: 149 requests. `sw_numshape` prints every accessor of a directing shape, its tool
(the indexation over 0..NbShapes+1, past both ends, and the first and last vertices,
which raise Standard_ConstructionError on an infinite end) and its iteration, for the
edges of 0, 1 and 2 vertices with every closed and infinite flag, the vertices of index
1 and 2, the dummy shape, every other shape type and out-of-range indices (-1, 0, 3, 7);
`sw_reinit` restarts an iteration on another shape after 0 to 3 steps; `sw_init`
reinitializes a shape and reads the default constructors.

BRepPrim: 758 requests, through the digest `pr_shape` (`tp_shape` followed by the first
derivatives of the located surface of each face at (0.3, 0.7)). `bp_wedge` (282) builds the
default wedge, boxes, right angular wedges (ltx 0, inside, equal to and beyond dx, negative)
and general wedges (YMax faces shifted, flat in X2 or Z2, reduced to a point, inverted)
through BRepPrim_Wedge and BRepPrim_GWedge on four frames, then runs every existence test
(same-axis pairs and triples raise Standard_DomainError), every plane, line and point, builds
sub-shapes in eleven orders (faces backward, the YMax edges and vertices first, open and
closed directions, Open and Close after building) and prints the accessors and the shell.
`bp_builder` (12) runs every BRepPrim_Builder member. `bp_oneaxis` (464) builds cylinders,
cones, spheres, tori and revolutions (line and circle meridians) through every constructor,
with the parameters each one rejects (null heights, angles outside 0..PI/2, equal or negative
radii, negative radii of the meridian circles), sets angles (half, quarter, 3PI, just below
2PI), meridian ranges (partial spheres, tori over symmetric, inverted and equal-height ranges)
and axes, then runs every query and the topology members forward, backward and in a seeded
shuffle, and the setters after building.

BRepSweep: 598 requests. The generating shapes are shape programs of the TKTopAlgo probe
(`tp_shapes`, built with BRep_Builder on both sides; tools/tkprim_corpus_sweep_shapes.py):
vertices, segments, arcs, closed circles and B-spline edges, open and closed wires, planar
faces (with a hole), the lateral face of a cylinder (with its seam), shells, compounds, a
located face and a solid (refused with Standard_NoSuchObject), each program ending with a
foreign vertex. `bs_prism` (126) sweeps them along vectors (axis-aligned, oblique,
reversed, of the length Precision::Confusion() and below it) and directions (infinite and
semi-infinite) with every copy and canonize flag, then prints the result and the history of
every program shape: IsUsed, GenIsUsed and the shape generated by it, at the first and at
the last directing vertex (a generated shape prints as its index among the sub-shapes of
the result, or as its digest). `bs_revol` (378) revolves vertices on and off the axis,
segments parallel, inclined, radial and on the axis, arcs centered on and off the axis,
B-splines, a profile wire and faces touching the axis or not, around three axes by
quarter, half, complete, negative, reduced (3PI, 7PI + 0.1) and too small (1e-13, 0)
angles, and through the complete constructor. `bs_members` (80) builds BRepSweep_Translation
and BRepSweep_Rotation directly on five directing edges (two vertices, a semi-infinite and
an infinite edge, a closed edge) and calls HasShape and IsInvariant of every program shape
at every directing shape, the empty shapes of every sub-shape (the copy checks raise
Standard_ConstructionError), the add tests of the side faces and SplitShell. `bs_tools`
(14) runs the Tool (Standard_OutOfRange past both ends), the Iterator and the Builder
(Standard_TypeMismatch for a slot of another type).

## Preserved native behaviour

### Sweep

- `Sweep_NumShapeIterator()` leaves the current orientation uninitialized; the port
  starts it FORWARD. The native value is read only after a `Next()` on an iterator whose
  `Init` did not start an edge iteration (a non-edge shape with a positive index), so
  the corpus never advances such an iterator.
- `Sweep_NumShapeIterator::Init` on a shape that is not an edge only stores the shape:
  `More()`, `Value()` and `Orientation()` keep the state of the previous iteration, and
  `Next()` then compares the next range with the index of the new shape.
- `Sweep_NumShapeTool::Shape(anIndex)` has no range check: 1 is the directing shape and
  any other index (0, negative, past `NbShapes()`) is the vertex of index `anIndex - 1`.
  `LastVertex()` of a closed edge with fewer than two vertices is the vertex of index 0
  or -1, as natively.
- `Sweep_NumShape::Orientation()` is REVERSED for every non-edge shape whose index is not
  2, the dummy shape of the default constructor included.

### BRepPrim

- `BRepPrim_OneAxis::LateralStartWire()` and `LateralWire()` share one wire slot
  (`WLATERALSTART == WLATERAL`): whichever runs first fills it and the other returns it.
  When the start wire comes first on a primitive with a finite end, the lateral face then
  adds that one-edge wire and sets a p-curve on the top or bottom edge, which was never
  built: natively a null dereference (a crash), in the port a Standard_NullObject error.
  The corpus calls LateralStartWire only alone or after the shell, each time followed by a
  stop before the final shell.
- `BRepPrim_GWedge::Vertex` on a YMax face flat in Z2 (or X2) copies both vertex pairs of
  the flat side from the built pair, even when the other vertex of the pair is not built
  yet: the copy is a null vertex, marked built. The shell of the native build order never
  meets it, but other orders (the vertex XMax ZMin YMax before XMin ZMax YMax, or the edge
  ZMin XMin first) return a null vertex and the shell then crashes natively (an error in the
  port). The corpus builds these orders and stops before the shell.
- `BRepPrim_Cone(Angle, Axes)` leaves the radius uninitialized natively; the port uses 0.
  The corpus does not build this cone.
- `BRepPrim_Wedge_Check` and `BRepPrim_OneAxis_Check` do not look at the shell flag: Open,
  Close and the one-axis setters raise Standard_DomainError once any vertex, edge, wire or
  face is built, but not after a shell whose faces all failed.
- The frames `gp_Ax2(Center, gp_Dir(gp_Dir::D::Z), gp_Dir(gp_Dir::D::X))` of the constructors
  at a point call the general gp_Ax2 constructor on two gp_Dir values, not the constexpr
  one on gp_Dir::D values: its cross products give the X direction (1, 0, -0) and the Y
  direction (-0, 1, 0), which the digests print; `gp::XOY()` uses the constexpr one.
- `BRepPrim_Revolution::MeridianValue` and the members that build on the meridian
  dereference the meridian handles without a check (a crash natively for a null
  meridian); the port raises Standard_NullObject.

### BRepSweep

- `BRepSweep_Revol` reduces the angle by subtracting 2PI while it exceeds 2PI +
  Precision::Angular(): for a huge angle the subtraction no longer changes it and the
  constructor never returns, natively and in the port. The corpus uses angles up to 7PI.
- `BRepSweep_Translation` refuses a vector shorter than Precision::Confusion() before
  building, `BRepSweep_Prism` refuses a vector of that length or shorter after its
  translation is built (Init has run): a vector of exactly Precision::Confusion() builds
  the translation, then raises.
- `FirstShape(aGenS)` and `LastShape(aGenS)` of a shape that is not a generating sub-shape
  raise Standard_OutOfRange (the range check of the shape grid on index 0), where
  `Shape(aGenS)` returns a null shape and `IsUsed` and `GenIsUsed` return false.
- `BRepSweep_Translation::MakeEmptyGeneratingEdge` raises its copy-mode error with the
  message of MakeEmptyVertex (the exception type is compared, not the message).
- `BRepSweep_Rotation::MakeEmptyVertex` and `MakeEmptyGeneratingEdge` of an invariant shape
  at the first vertex of an open revolution also store the new shape as the shape of the
  last vertex (the two are the same shape on the axis), so calling them directly after
  building replaces that cell.
- The faces of the shell swept by a wire of a generating face are added to the solid
  without being flagged used, unlike the faces swept by an edge.
- The members that dereference a missing 3D curve or surface (`MakeEmptyFace` of an edge
  without curve, the rotation's copy of such an edge, `SetGeneratingPCurve` in a planar
  face whose new edge is not a line) crash natively; the port raises Standard_NullObject.
  The corpus never builds them.
