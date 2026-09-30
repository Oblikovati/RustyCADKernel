# TKHelix port status

**In progress; toolkit acceptance is not complete.** `crates/tkhelix`
(`rustycad-tkhelix`) is the port of the ModelingAlgorithms toolkit TKHelix. It depends
on the TKernel, TKMath, TKG2d, TKG3d, TKGeomBase, TKBRep, TKGeomAlgo and TKTopAlgo
ports, as the native toolkit does (its EXTERNLIB). Each of the 2 OCCT packages is one
module of the crate, declared up front in `src/lib.rs`; a module whose package is not
ported yet holds only its documentation. Ported so far: none.

[helix-design.md](helix-design.md) fixes how the adaptor, the abstract builder
hierarchy, the native exceptions and the parameter arrays map to Rust.

A package may start when the packages it includes are ported (the lower toolkits are
ported):

| Level | Packages |
| --- | --- |
| 1 | HelixGeom |
| 2 | HelixBRep |

The toolkit test inventory maps these suites (0 declarations):

| Source | Rust target | Declarations | Native | Rust |
| --- | --- | ---: | --- | --- |

## Numeric corpus

The numeric oracle (`tools/verify_tkhelix_numeric.py`) compares
`tools/oracle/tkhelix_probe.cxx`, linked with libTKHelix and the toolkits below it, with
`crates/tkhelix/examples/numeric_probe.rs` on the requests of `tools/tkhelix_corpus.py`,
in 64 round-robin shards. The placement family (`hx_placement`, 23 requests) compares
the displacement that `HelixGeom_BuilderHelix::Perform` applies to its coils; it proves
the pipeline before the HelixGeom classes are compared.
