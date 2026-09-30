# TKHelix port status

**In progress; toolkit acceptance is not complete.** `crates/tkhelix`
(`rustycad-tkhelix`) is the port of the ModelingAlgorithms toolkit TKHelix. It depends
on the TKernel, TKMath, TKG2d, TKG3d, TKGeomBase, TKBRep, TKGeomAlgo and TKTopAlgo
ports, as the native toolkit does (its EXTERNLIB). Each of the 2 OCCT packages is one
module of the crate, declared up front in `src/lib.rs`; a module whose package is not
ported yet holds only its documentation. Ported so far: HelixGeom.

[helix-design.md](helix-design.md) fixes how the adaptor, the abstract builder
hierarchy, the native exceptions and the parameter arrays map to Rust.

A package may start when the packages it includes are ported (the lower toolkits are
ported):

| Level | Packages |
| --- | --- |
| 1 | HelixGeom |
| 2 | HelixBRep |

The toolkit test inventory maps these suites (20 declarations):

| Source | Rust target | Declarations | Native | Rust |
| --- | --- | ---: | --- | --- |
| HelixGeom_HelixCurve_Test.cxx | helix_geom_helix_curve | 9 | 9 of 9 | 9 of 9 |
| HelixGeom_Tools_Test.cxx | helix_geom_tools | 3 | 3 of 3 | 3 of 3 |
| HelixGeom_BuilderHelix_Test.cxx | helix_geom_builder_helix | 4 | 4 of 4 | 4 of 4 |
| HelixGeom_BuilderHelixCoil_Test.cxx | helix_geom_builder_helix_coil | 4 | 4 of 4 | 4 of 4 |

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
