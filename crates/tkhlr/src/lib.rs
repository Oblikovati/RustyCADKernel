// Rust translation of ModelingAlgorithms/TKHLR at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT TKHLR: the hidden line removal of ModelingAlgorithms (interval sets, the
//! polyhedral and exact hidden line algorithms, contour lines of surfaces seen from a
//! direction or a point, and the conversion of the results back to shapes).
//!
//! TKHLR depends on TKBRep, TKernel, TKMath, TKGeomBase, TKG2d, TKG3d, TKGeomAlgo and
//! TKTopAlgo natively (its EXTERNLIB); this crate depends on their ports. Each of the
//! 8 OCCT packages is one module named after it in snake case (`Intrv` is `intrv`,
//! `TopBas` is `top_bas`, `HLRAlgo` is `hlr_algo`, `HLRTopoBRep` is `hlr_topo_brep`),
//! declared here up front so that packages ported in parallel never edit this file; a
//! module whose package is not ported yet holds only its documentation.
//! docs/hlr-design.md fixes how the C++ constructs of the packages map to Rust and the
//! class order of the HLRTopoBRep, HLRBRep and HLRAppli work, and
//! docs/tkhlr-progress.md lists the ported packages.
pub mod contap;
pub mod hlr_algo;
pub mod hlr_appli;
pub mod hlr_brep;
pub mod hlr_topo_brep;
pub mod intrv;
pub mod top_bas;
pub mod top_cnx;
