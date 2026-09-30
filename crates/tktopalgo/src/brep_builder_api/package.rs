// Rust translation of BRepBuilderAPI.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The package class `BRepBuilderAPI`: the default plane and precision of the makers,
//! which are the process-wide settings of BRepLib (docs/topalgo-design.md sections 7
//! and 10).
use crate::brep_lib;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::GeomPlane;

/// `BRepBuilderAPI::Plane(P)`: sets the plane of the 2d makers (`BRepLib::Plane(P)`);
/// `brep_builder_api::set_plane(&plane)`.
pub fn set_plane(plane: &Handle<GeomPlane>) {
    brep_lib::set_plane(plane);
}

/// `BRepBuilderAPI::Plane()`: the current plane (`BRepLib::Plane()`), created on the
/// XOY axes when none was set; `assert!(rustycad_tktopalgo::brep_builder_api::plane().is_some())`.
pub fn plane() -> Handle<GeomPlane> {
    brep_lib::plane()
}

/// `BRepBuilderAPI::Precision(P)`: sets the default precision of the makers
/// (`BRepLib::Precision(P)`); `brep_builder_api::set_precision(1e-6)`.
pub fn set_precision(value: f64) {
    brep_lib::set_precision(value);
}

/// `BRepBuilderAPI::Precision()`: the default precision (`BRepLib::Precision()`);
/// `assert_eq!(rustycad_tktopalgo::brep_builder_api::precision(), 1e-7)`.
pub fn precision() -> f64 {
    brep_lib::precision()
}
