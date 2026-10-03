# TKernel/TKMath selector audit at `52bcd058`

This audit corrects 114 `rust_item` selectors in `porting/parity.json`. It
does not change declaration status, test status, or the global completeness
flags. A corrected selector identifies a Rust API item; it is not evidence
that the implementation matches OCCT behavior.

## Corrections

| Declarations | Rows | Correct Rust API item |
| --- | ---: | --- |
| `ElCLib::{Value,D1,D2,D3,DN,Parameter}` concrete curve overloads | 54 | The corresponding generic `rustycad_tkmath::elclib::{value,d1,d2,d3,dn,parameter}` function |
| `ElSLib::{Value,DN,D0,D1,D2,D3,Parameters}` concrete surface overloads | 33 | The corresponding generic `rustycad_tkmath::elslib::{value,dn,d0,d1,d2,d3,parameters}` function |
| `CSLib_NormalPolyDef::{Value,Derivative,Values}` | 3 | The implementation under the public `Function` or `FunctionWithDerivative` trait |
| `BVH_Triangulation`, `BVH_BoxSet`, `BVH_IndexedBoxSet`, and `BVH_ObjectSet` set methods | 14 | The concrete type's `BvhSet` implementation (`Size`, `Box`, `Center`, `Swap`) |
| BVH point traversal `Accept` methods | 2 | The concrete type's `BvhTraverse::accept` implementation |
| `BVH_QuickSorter` and `BVH_RadixSorter` `Perform` overloads | 4 | Whole-set calls resolve to the `BvhSorter::perform` default; range calls resolve to the concrete `BvhSorter::perform_range` implementation |
| `BVH_BinnedBuilder::buildNode`, `BVH_SweepPlaneBuilder::buildNode` | 2 | The concrete `BvhQueueBuilder::build_node` implementation |
| `BVH_LinearBuilder::Build` | 1 | The concrete `BvhBuilder::build` implementation |
| `Bnd_Range::operator==` | 1 | `Range1::<core::cmp::PartialEq>::eq` |

The Rust functions for the `ElCLib` and `ElSLib` groups are generic over
public curve/surface traits. The selector therefore points to one shared Rust
function for several C++ overload declarations; the mapping does not assert
that those overloads have separately generated Rust symbols.

## Evidence and limits

The OCCT source pin is `3d097a0328e71b826377d4814ab05ec3c3d23871`, matching
`porting/baseline.json`. The corresponding declarations were checked in the
pinned `ElCLib.hxx`, `ElSLib.hxx`, `CSLib_NormalPolyDef.hxx`, BVH headers, and
`Bnd_Range.hxx`.

The available normalized Rust inventory and coverage report were captured at
an older Rust workspace revision (`432e74bc...`). They are not current global
inventory proof. For these 114 changed rows, each corrected path resolves to
exactly one declaration in that capture, and the source-file SHA-256 recorded
for each declaration matches the current source file. This validates the
selector targets only; it does not validate the complete inventory, test
coverage, or behavioral equivalence. A fresh capture is required for those
claims.
