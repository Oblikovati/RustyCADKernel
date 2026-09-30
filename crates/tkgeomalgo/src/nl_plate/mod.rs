// Rust translation of ModelingAlgorithms/TKGeomAlgo/NLPlate at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package NLPlate: empty at this commit, so there is nothing to port.
//!
//! `PACKAGES.cmake` still lists NLPlate, but its `FILES.cmake` declares no file: OCCT
//! commit caad597813 ("Clean up dead headers #1442") deleted every NLPlate source and
//! header (the non-linear plate `NLPlate_NLPlate` and its `NLPlate_HGPPConstraint`
//! family), and no toolkit uses them. The only remaining references are two
//! deprecated aliases in `src/Deprecated/NCollectionAliases` (outside TKGeomAlgo):
//! `NLPlate_StackOfPlate.hxx`, a typedef of `NCollection_List<Plate_Plate>`, and
//! `NLPlate_SequenceOfHGPPConstraint.hxx`, which includes the deleted
//! `NLPlate_HGPPConstraint.hxx` and so no longer compiles. `tools/audit_api_rows.py`
//! finds no public member, and this module stays empty until OCCT adds sources again.
