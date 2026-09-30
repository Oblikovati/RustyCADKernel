# TKHLR port status

**Intrv, TopBas and TopCnx are ported; the other packages follow in the order below.**
`crates/tkhlr` (`rustycad-tkhlr`) is the port of the ModelingAlgorithms toolkit TKHLR.
It depends on the TKernel, TKMath, TKG2d, TKG3d, TKGeomBase, TKBRep, TKGeomAlgo and
TKTopAlgo ports, as the native toolkit does (its EXTERNLIB). Each of the 8 OCCT packages
is one module of the crate, declared up front in `src/lib.rs`; a module whose package
is not ported yet holds only its documentation. Ported: Intrv, TopBas, TopCnx.

[hlr-design.md](hlr-design.md) fixes how the C++ constructs of the packages map to Rust
and gives the class order of the HLRTopoBRep, HLRBRep and HLRAppli work.

A package may start when the packages it includes are ported (the lower toolkits are
ported):

| Level | Packages |
| --- | --- |
| 1 | Contap, Intrv, TopBas, TopCnx |
| 2 | HLRAlgo |
| 3 | HLRTopoBRep |
| 4 | HLRBRep |
| 5 | HLRAppli |

The toolkit test inventory has two suites. Both test HLRBRep classes, so they wait for
the HLRBRep port and are not mapped yet:

| Source | Rust target | Declarations | Native | Rust |
| --- | --- | ---: | --- | --- |
| HLRBRep_Data_Test.cxx | waits for HLRBRep | - | - | - |
| HLRBRep_InternalAlgo_Test.cxx | waits for HLRBRep | - | - | - |

## Numeric corpus

The numeric oracle (`tools/verify_tkhlr_numeric.py`) compares
`tools/oracle/tkhlr_probe.cxx`, linked with libTKHLR and the toolkits below it, with
`crates/tkhlr/examples/numeric_probe.rs` on the requests of `tools/tkhlr_corpus.py`, in
64 round-robin shards. The frame family (`hl_frame`, 23 requests) compares the
transformation to the viewing frame that `HLRAlgo_Projector(const gp_Ax2&)` stores and
its inverse; it proves the pipeline before the HLR classes are compared.

The Intrv families (`tools/tkhlr_corpus_intrv.py`, 1034 requests) compare the three
`Intrv_Interval` constructors on zero, signed-zero, subnormal, huge and RealFirst/RealLast
bounds with negative, zero, tiny and float-overflowing tolerances, the six bound edits,
`Position` and the 13 `Is*` tests on a grid around a reference interval and on seeded
random pairs, the eight boolean operations of `Intrv_Intervals` on fixed and random sets
(empty sets and tools, touching, nested, split, infinite and reversed intervals), and
`Value` inside and outside the sequence.

The TopBas and TopCnx families (`tools/tkhlr_corpus_topcnx.py`, 227 requests) compare
`TopBas_TestInterference` set by its setters, by its constructor and through
`ChangeIntersection`/`ChangeBoundary`, and `TopCnx_EdgeFaceTransition` before any reset,
after both resets and over seeded sequences of interferences (every orientation,
transition and boundary transition; tangent, normal and opposite directions;
curvatures and tolerances), with `Transition` and `BoundaryTransition` after each step.

## Intrv quirks kept

- The default interval is `[RealFirst, RealLast]` with tolerances 0:
  `Epsilon(RealFirst())` and `Epsilon(RealLast())` are 0 (no double lies beyond them).
- `FuseAtStart` and `CutAtStart` do nothing when the start is exactly RealFirst, and
  `FuseAtEnd` and `CutAtEnd` when the end is RealLast, whatever the other bound.
- The tolerances are `float`: two tolerances add in single precision before they are
  compared with a double difference, and a fused tolerance is `(float)(b - a) / 2`.
  `SetStart` and `SetEnd` keep a zero or negative tolerance; only the constructor with
  tolerances raises them to `(float)Epsilon(bound)`.
- `Position` and the `Is*` tests are separate formulas (double envelopes against float
  sums), so at ties they can disagree.
- `Subtract` and `Unite` leave their walk at the first interval that lies after the
  tool, so on intervals that are reversed or carry negative tolerances (possible through
  `SetStart`/`SetEnd`) the set can stay unsorted or overlapping, as natively. `Intersect(tool)` is `Unite(tool)` minus
  the exclusive union, and `XUnite(tool)` is `tool` minus the set united with the set
  minus `tool`, each with these walks.

Deviation: a native call whose tool is the object itself (`x.Unite(x)`) edits the
sequence while it walks it; Rust borrowing forbids that call (docs/hlr-design.md
section 3).

## TopBas and TopCnx quirks kept

- A new `TopCnx_EdgeFaceTransition` answers FORWARD before its first reset: its
  `TopTrans_CurveTransition` starts with `Init` false and FORWARD transitions, so the
  states read OUT then IN.
- `BoundaryTransition` is EXTERNAL when the FORWARD and REVERSED counts are equal, on
  both branches of the native parity test; INTERNAL and EXTERNAL boundary transitions
  are not counted.

Deviation: the default `TopBas_TestInterference` is `0.0`, `0` and FORWARD, where the
native default constructor leaves the fields uninitialized (docs/hlr-design.md
section 4); the probe never reads a default record.
