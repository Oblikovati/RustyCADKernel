// Rust translation of ModelingAlgorithms/TKPrim/BRepPrim at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package BRepPrim: the primitive algorithms. [`BRepPrimGWedge`] (and
//! [`BRepPrimWedge`], its constructors without a builder) builds boxes and wedges;
//! [`BRepPrimOneAxis`] builds the bodies of revolution of a meridian, which
//! [`BRepPrimRevolution`] gives as curves and [`BRepPrimCylinder`], [`BRepPrimCone`],
//! [`BRepPrimSphere`] and [`BRepPrimTorus`] specialize with their elementary lateral
//! surfaces. [`BRepPrimBuilder`] holds the topological services of both
//! (docs/prim-design.md section 2).
mod builder;
mod cone;
mod cylinder;
mod direction;
mod gwedge;
mod gwedge_geometry;
mod gwedge_topology;
mod one_axis;
mod one_axis_edges;
mod one_axis_faces;
mod one_axis_vertices;
mod one_axis_wires;
mod revolution;
mod revolution_derived;
mod sphere;
mod torus;
mod wedge;

pub use builder::BRepPrimBuilder;
pub(crate) use cylinder::standard_frame;
pub use cone::BRepPrimCone;
pub use cylinder::BRepPrimCylinder;
pub use direction::BRepPrimDirection;
pub use gwedge::BRepPrimGWedge;
pub use one_axis::{BRepPrimOneAxis, BRepPrimOneAxisBase};
pub use revolution::BRepPrimRevolution;
pub use revolution_derived::BRepPrimRevolutionDerived;
pub use sphere::BRepPrimSphere;
pub use torus::BRepPrimTorus;
pub use wedge::BRepPrimWedge;
