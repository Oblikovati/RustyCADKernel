// Rust translation of ModelingAlgorithms/TKPrim/BRepPrimAPI at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package BRepPrimAPI: the application interface of the primitives and sweeps:
//! the makers of boxes, wedges, cylinders, cones, spheres, tori, revolved bodies,
//! half-spaces, prisms and revolutions (docs/prim-design.md section 4).
mod make_box;
mod make_cone;
mod make_cylinder;
mod make_half_space;
mod make_one_axis;
mod make_prism;
mod make_revol;
mod make_revol_build;
mod make_revol_validity;
mod make_revolution;
mod make_sphere;
mod make_sweep;
mod make_torus;
mod make_wedge;

pub use make_box::BRepPrimAPIMakeBox;
pub use make_cone::BRepPrimAPIMakeCone;
pub use make_cylinder::BRepPrimAPIMakeCylinder;
pub use make_half_space::BRepPrimAPIMakeHalfSpace;
pub use make_one_axis::{BRepPrimAPIMakeOneAxis, one_axis_build};
pub use make_prism::BRepPrimAPIMakePrism;
pub use make_revol::BRepPrimAPIMakeRevol;
pub use make_revolution::BRepPrimAPIMakeRevolution;
pub use make_sphere::BRepPrimAPIMakeSphere;
pub use make_sweep::BRepPrimAPIMakeSweep;
pub use make_torus::BRepPrimAPIMakeTorus;
pub use make_wedge::BRepPrimAPIMakeWedge;
