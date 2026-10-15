// SPDX-License-Identifier: GPL-2.0-only
//! Display tessellation of any body (ADR 0077): a presentation mesh downstream of
//! modelling and naming. It never changes the body or its history and is never
//! modelling or acceptance evidence.
mod detached;
mod edge_polyline;
mod face_mesh;
mod mesh;
mod options;
mod tessellate;

pub use mesh::{EdgePolyline, FaceTriangles, PresentationMesh};
pub use options::{DisplayOptions, EffectiveMeshSettings};
pub use tessellate::tessellate;
