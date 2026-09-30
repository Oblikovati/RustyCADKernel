# TKHLR port status

**The crate and its numeric oracle are in place; no package is ported yet.**
`crates/tkhlr` (`rustycad-tkhlr`) is the port of the ModelingAlgorithms toolkit TKHLR.
It depends on the TKernel, TKMath, TKG2d, TKG3d, TKGeomBase, TKBRep, TKGeomAlgo and
TKTopAlgo ports, as the native toolkit does (its EXTERNLIB). Each of the 8 OCCT packages
is one module of the crate, declared up front in `src/lib.rs`; a module whose package
is not ported yet holds only its documentation. Ported: none.

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
