// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepGProp at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package BRepGProp: the global properties (mass, centre of mass, matrix of
//! inertia) of edges, faces and solids, by Gauss and Gauss-Kronrod integration of their
//! geometry or of their meshes (docs/topalgo-design.md section 9).
//!
//! The derived `GProp_GProps` frameworks ([`BRepGPropCinert`], [`BRepGPropSinert`],
//! [`BRepGPropVinert`], [`BRepGPropVinertGK`], [`BRepGPropMeshCinert`],
//! [`BRepGPropMeshProps`]) reach the TKGeomBase `GProps` members through `Deref`; the
//! package functions of [`BRepGProp`] bring the properties of a shape together.
mod cinert;
mod domain;
mod edge_tool;
mod face;
mod face_boundary;
mod face_orders;
mod gauss;
mod gauss_adaptive;
mod gauss_adaptive_l;
mod gauss_adaptive_u;
mod gauss_fixed;
mod gauss_table;
mod line_moments;
mod mesh_cinert;
mod mesh_props;
mod mesh_triangle;
mod package;
mod sinert;
mod tfunction;
mod ufunction;
mod vinert;
mod vinert_gk;
mod vinert_gk_perform;
mod vinert_new;
mod volume;
mod volume_gk;
mod volume_solids;

pub use cinert::BRepGPropCinert;
pub use domain::BRepGPropDomain;
pub use edge_tool::BRepGPropEdgeTool;
pub use face::BRepGPropFace;
pub use gauss::{BRepGPropGauss, GaussProps, GaussType};
pub use mesh_cinert::BRepGPropMeshCinert;
pub use mesh_props::{BRepGPropMeshProps, MeshObjType};
pub use mesh_triangle::GPropsSums;
pub use package::BRepGProp;
pub use sinert::BRepGPropSinert;
pub use tfunction::BRepGPropTFunction;
pub use ufunction::BRepGPropUFunction;
pub use vinert::BRepGPropVinert;
pub use vinert_gk::{BRepGPropVinertGK, VinertGKOptions};
pub use volume_gk::VolumeGKFlags;
