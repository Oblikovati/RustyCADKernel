# TKPrim port status

**In progress; toolkit acceptance is not complete.** `crates/tkprim` (`rustycad-tkprim`)
is the port of the ModelingAlgorithms toolkit TKPrim. It depends on the TKernel, TKMath,
TKG2d, TKG3d, TKGeomBase, TKBRep, TKGeomAlgo and TKTopAlgo ports, as the native toolkit
does (its EXTERNLIB lists TKBRep, TKernel, TKMath, TKG2d, TKGeomBase, TKG3d and
TKTopAlgo; TKTopAlgo brings TKGeomAlgo). Each of the 5 OCCT packages is one module of the
crate, declared up front in `src/lib.rs`; a module whose package is not ported yet holds
only its documentation. No package is ported yet.

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

## Preserved native behaviour

None recorded yet; each ported package adds its own subsection.
