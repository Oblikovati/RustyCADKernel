// Rust translation of ModelingAlgorithms/TKHLR/HLRTopoBRep at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package HLRTopoBRep: the outlines of the faces of a shape seen from a
//! direction or a point (Contap contours turned into edges), their isolines, and the
//! shape rebuilt with them ([`OutLiner`]), with the data structure that records them
//! ([`Data`]).
mod data;
mod ds_filler;
mod face_iso_liner;
mod out_liner;
mod records;

pub use data::Data;
pub use ds_filler::{DSFiller, MapOfShapeTool};
pub use face_iso_liner::FaceIsoLiner;
pub use out_liner::OutLiner;
pub use records::{FaceData, VData};
