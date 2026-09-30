# TKHelix port status

**Both packages are ported and every TKHelix GoogleTest suite is mapped; the toolkit
acceptance is pending.** `crates/tkhelix`
(`rustycad-tkhelix`) is the port of the ModelingAlgorithms toolkit TKHelix. It depends
on the TKernel, TKMath, TKG2d, TKG3d, TKGeomBase, TKBRep, TKGeomAlgo and TKTopAlgo
ports, as the native toolkit does (its EXTERNLIB). Each of the 2 OCCT packages is one
module of the crate, declared up front in `src/lib.rs`; a module whose package is not
ported yet holds only its documentation. Ported: HelixGeom and HelixBRep.

[helix-design.md](helix-design.md) fixes how the adaptor, the abstract builder
hierarchy, the native exceptions and the parameter arrays map to Rust.

A package may start when the packages it includes are ported (the lower toolkits are
ported):

| Level | Packages |
| --- | --- |
| 1 | HelixGeom |
| 2 | HelixBRep |

The toolkit test inventory maps these suites (44 declarations, all of TKHelix):

| Source | Rust target | Declarations | Native | Rust |
| --- | --- | ---: | --- | --- |
| HelixGeom_HelixCurve_Test.cxx | helix_geom_helix_curve | 9 | 9 of 9 | 9 of 9 |
| HelixGeom_Tools_Test.cxx | helix_geom_tools | 3 | 3 of 3 | 3 of 3 |
| HelixGeom_BuilderHelix_Test.cxx | helix_geom_builder_helix | 4 | 4 of 4 | 4 of 4 |
| HelixGeom_BuilderHelixCoil_Test.cxx | helix_geom_builder_helix_coil | 4 | 4 of 4 | 4 of 4 |
| HelixBRep_BuilderHelix_Test.cxx | helix_brep_builder_helix | 14 | 14 of 14 | 14 of 14 |
| HelixBRep_BuilderHelix_Integration_Test.cxx | helix_brep_builder_helix_integration | 10 | 10 of 10 | 10 of 10 |

## Numeric corpus

The numeric oracle (`tools/verify_tkhelix_numeric.py`) compares
`tools/oracle/tkhelix_probe.cxx`, linked with libTKHelix and the toolkits below it, with
`crates/tkhelix/examples/numeric_probe.rs` on the requests of `tools/tkhelix_corpus.py`,
in 64 round-robin shards. The placement family (`hx_placement`, 23 requests) compares
the displacement that `HelixGeom_BuilderHelix::Perform` applies to its coils; it proves
the pipeline before the HelixGeom classes are compared.

The HelixGeom families (`tools/tkhelix_corpus_helixgeom.py`, 241 requests) compare
`HelixGeom_HelixCurve` (loads, failed loads and reloads, the adaptor queries with
interval arrays of several bounds, and D0, D1, D2, D3 and DN), `HelixGeom_Tools`
(`ApprCurve3D` on helices, on a circle adaptor and on the `Adaptor3d_Curve` base class,
over tolerances, continuities, segment counts and degrees, and `ApprHelix`), and the
builders `HelixGeom_BuilderHelixCoil` and `HelixGeom_BuilderHelix` run several times on
one object (full and partial turns, reversed ranges, tapers, frames, refused settings).
Every B-spline is compared pole by pole and knot by knot.

The HelixBRep family (`hxb_build`, `tools/tkhelix_corpus_helixbrep.py`, 83 requests)
runs SetParameters (all six overloads), SetApproxParameters and Perform steps on one
`HelixBRep_BuilderHelix` and compares the state after each step: the statuses, the
reached tolerance and the wire edge by edge (orientation, tolerance, range, B-spline
and vertices). It covers the parameters of the upstream suites, several parts with
smoothed joints (including joints where MovePointAndTangent fails until the degree is
raised, once or up to the maximum 25), every error status the builder reaches (1 and
10 to 13), wrong array dimensions, arrays with lower bounds other than 1, no part,
numbers of turns of 0, approximation settings AdvApprox refuses, direct and indirect
frames, repeated runs and 24 seeded composite helices.

## HelixGeom quirks kept

- `HelixGeom_HelixCurve::Load` stores the six parameters before it checks them, so a
  load that raises Standard_ConstructionError leaves the invalid parameters in the curve
  (with the `pitch / 2*PI` and the tangent of the taper of the last valid load), and a
  later `Load()` raises again.
- `Load` sets the tangent of the taper only when `|taper| > 1e-4`: a reload with a
  smaller taper keeps the tangent of the previous load instead of 0, so the reloaded
  "cylindrical" helix is still tapered.
- `Intervals` writes the absolute indices 1 and 2 of the array: an array with other
  bounds gets other elements written, or raises Standard_OutOfRange.
- The members the class does not override (`D3`, `Trim`, `GetType`, ...) raise
  Standard_NotImplemented from the `Adaptor3d_Curve` base; `Period` raises
  Standard_DomainError.
- AdvApprox never reports a missed tolerance (the `ErrorCode = -1` of
  `AdvApprox_ApproxAFunction` is commented out), so `ApprCurve3D` returns 0 with a
  maximum error above the tolerance, and its codes 1 and 2 need an evaluator error,
  which natively unwinds as an exception first. The builders' error status 2 is
  therefore not reached with the helix adaptor, and `ToleranceReached` can exceed
  `Tolerance` with status 0.
- `HelixGeom_BuilderHelix` computes the number of full turns with `(int)` truncation (a
  reversed range of more than one turn gives a negative count, so only the rest coil is
  built and its `Load` raises; NaN gives `INT_MIN` on the pinned x86-64 profile, which
  the port reproduces). For a cylindrical helix the turns after the first are
  translated copies of the first coil, so the reached tolerance only covers the first
  coil and the rest. It is -1 before a coil sets it.

## HelixBRep quirks kept

- `SetParameters` stores the axis, clears the diameter, height and pitch arrays (not the
  pitch flags) and the shape, and sets both statuses to 1 before it checks the array
  lengths; after its Standard_ConstructionError, `Perform` does nothing.
- The arrays are copied with the bounds of the caller's arrays
  (`NCollection_Array1::Assign`), but `Perform` reads them at the absolute indices
  1..n: an array with another lower bound gives other elements, or Standard_OutOfRange
  from `Perform`. The spiral overloads restart the partial heights at the absolute index
  1 of the heights array likewise; the turn overloads use each array's lower bound.
- With one diameter and empty arrays there is no part: `Perform` raises
  Standard_NoSuchObject (`NCollection_List::First` in `Smoothing`).
- A number of turns of 0 makes the pitch infinite, which passes the pitch check; the
  coil range is then `[0, 0]` and the helix `Load` raises Standard_ConstructionError out
  of `Perform`. A negative number of turns gives a negative pitch (status 11).
- The builder tolerance is also the smallest height, pitch and distance of the start
  point to the axis a part accepts (statuses 12, 11 and 10) and the tolerance of
  MovePointAndTangent at the joints.
- The reached tolerance restarts at 0 in `Perform` and grows part by part; each vertex
  and edge gets the value reached when it is built, so earlier parts carry smaller
  tolerances. The warning status is recomputed for each part.
- The joint smoothing moves the curves the edges hold (BRep_Tool::Curve returns the
  edge's own curve at the identity location): a degree raised by attempts that all fail
  stays in the edge although `UpdateEdge` is not called.
- The error statuses 2 (see the HelixGeom quirks) and 3 (BRepBuilderAPI_MakeEdge on a
  coil and vertices made at its own ends) are not reached.

Deviation: `BuildPart` has branches for a start angle other than 0 (a trimmed first
coil, coils before the start point); the angle is the local constant 0, so they are
dead natively and are not ported.
