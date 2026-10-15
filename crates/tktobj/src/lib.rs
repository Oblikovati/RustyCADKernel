//! Rust port of the OCCT TKTObj toolkit: the TObj object model over OCAF.
//!
//! - [`tobj`]: models (`TObj_Model`), objects (`TObj_Object`), partitions, the object
//!   iterators, the persistence registry and the OCAF attributes that store them.
//!
//! The design rules are in `docs/tobj-design.md`. The native file formats of TObj
//! (`BinTObjDrivers`, `XmlTObjDrivers`) are out of scope: a model saves and loads through
//! whatever storage and retrieval drivers the application defines.
pub mod tobj;
