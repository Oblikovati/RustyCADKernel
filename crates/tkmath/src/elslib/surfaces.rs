// Rust translation of FoundationClasses/TKMath/ElSLib/ElSLib.lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The gp-object overloads of ElSLib as traits over planes and quadrics.
use super::first::{SurfaceD1, cone_d1, cylinder_d1, plane_d1, sphere_d1, torus_d1};
use super::second::{SurfaceD2, cone_d2, cylinder_d2, sphere_d2, torus_d2};
use super::third::{SurfaceD3, cone_d3, cylinder_d3, sphere_d3, torus_d3};
use super::{derivatives as dn, parameters as at, values as at_uv};
use crate::gp::{Cone3, Cylinder3, Plane3, Point3, Sphere3, Torus3, Vector3};

/// Surfaces with ElSLib Value/D0/D1/DN/Parameters overloads.
pub trait ElementarySurface {
    /// Point at `(u, v)`; `surface.value(0.5,0.2)`.
    fn value(&self, u: f64, v: f64) -> Point3;
    /// Point at `(u, v)` through the D0 entry point; `surface.d0(0.5,0.2)`.
    fn d0(&self, u: f64, v: f64) -> Point3;
    /// Point and first partial derivatives; `surface.d1(0.5,0.2)`.
    fn d1(&self, u: f64, v: f64) -> SurfaceD1;
    /// Mixed partial derivative of orders `(nu, nv)`; `surface.dn(0.5,0.2,1,1)`.
    fn dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Vector3;
    /// `(u, v)` of a point; `surface.parameters(point)`.
    fn parameters(&self, point: Point3) -> (f64, f64);
}
/// Curved surfaces with ElSLib D2/D3 overloads (the plane has none).
pub trait CurvedSurface: ElementarySurface {
    /// Point with derivatives up to the second; `surface.d2(0.5,0.2)`.
    fn d2(&self, u: f64, v: f64) -> SurfaceD2;
    /// Point with derivatives up to the third; `surface.d3(0.5,0.2)`.
    fn d3(&self, u: f64, v: f64) -> SurfaceD3;
}

/// ElSLib::Value overloads; `elslib::value(0.5,0.2,&sphere)`.
pub fn value<S: ElementarySurface>(u: f64, v: f64, surface: &S) -> Point3 {
    surface.value(u, v)
}
/// ElSLib::D0 overloads; `elslib::d0(0.5,0.2,&sphere)`.
pub fn d0<S: ElementarySurface>(u: f64, v: f64, surface: &S) -> Point3 {
    surface.d0(u, v)
}
/// ElSLib::D1 overloads; `elslib::d1(0.5,0.2,&sphere)`.
pub fn d1<S: ElementarySurface>(u: f64, v: f64, surface: &S) -> SurfaceD1 {
    surface.d1(u, v)
}
/// ElSLib::D2 overloads; `elslib::d2(0.5,0.2,&sphere)`.
pub fn d2<S: CurvedSurface>(u: f64, v: f64, surface: &S) -> SurfaceD2 {
    surface.d2(u, v)
}
/// ElSLib::D3 overloads; `elslib::d3(0.5,0.2,&sphere)`.
pub fn d3<S: CurvedSurface>(u: f64, v: f64, surface: &S) -> SurfaceD3 {
    surface.d3(u, v)
}
/// ElSLib::DN overloads; `elslib::dn(0.5,0.2,&sphere,1,1)`.
pub fn dn<S: ElementarySurface>(u: f64, v: f64, surface: &S, nu: i32, nv: i32) -> Vector3 {
    surface.dn(u, v, nu, nv)
}
/// ElSLib::Parameters overloads; `elslib::parameters(&sphere,point)`.
pub fn parameters<S: ElementarySurface>(surface: &S, point: Point3) -> (f64, f64) {
    surface.parameters(point)
}

macro_rules! elementary_surface {
    ($surface:ty, ($($len:ident),*), $value:path, $d0:path, $d1:path, $dn:path, $parameters:path) => {
        impl ElementarySurface for $surface {
            fn value(&self, u: f64, v: f64) -> Point3 {
                $value(u, v, self.position() $(, self.$len())*)
            }
            fn d0(&self, u: f64, v: f64) -> Point3 {
                $d0(u, v, self.position() $(, self.$len())*)
            }
            fn d1(&self, u: f64, v: f64) -> SurfaceD1 {
                $d1(u, v, self.position() $(, self.$len())*)
            }
            fn dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Vector3 {
                $dn(u, v, self.position() $(, self.$len())*, nu, nv)
            }
            fn parameters(&self, point: Point3) -> (f64, f64) {
                $parameters(self.position() $(, self.$len())*, point)
            }
        }
    };
}
macro_rules! curved_surface {
    ($surface:ty, ($($len:ident),*), $d2:path, $d3:path) => {
        impl CurvedSurface for $surface {
            fn d2(&self, u: f64, v: f64) -> SurfaceD2 {
                $d2(u, v, self.position() $(, self.$len())*)
            }
            fn d3(&self, u: f64, v: f64) -> SurfaceD3 {
                $d3(u, v, self.position() $(, self.$len())*)
            }
        }
    };
}

elementary_surface!(
    Plane3,
    (),
    at_uv::plane_value,
    at_uv::plane_d0,
    plane_d1,
    dn::plane_dn,
    at::plane_parameters
);
elementary_surface!(
    Cone3,
    (ref_radius, semi_angle),
    at_uv::cone_value,
    at_uv::cone_d0,
    cone_d1,
    dn::cone_dn,
    at::cone_parameters
);
elementary_surface!(
    Cylinder3,
    (radius),
    at_uv::cylinder_value,
    at_uv::cylinder_d0,
    cylinder_d1,
    dn::cylinder_dn,
    at::cylinder_parameters
);
elementary_surface!(
    Sphere3,
    (radius),
    at_uv::sphere_value,
    at_uv::sphere_d0,
    sphere_d1,
    dn::sphere_dn,
    at::sphere_parameters
);
elementary_surface!(
    Torus3,
    (major_radius, minor_radius),
    at_uv::torus_value,
    at_uv::torus_d0,
    torus_d1,
    dn::torus_dn,
    at::torus_parameters
);
curved_surface!(Cone3, (ref_radius, semi_angle), cone_d2, cone_d3);
curved_surface!(Cylinder3, (radius), cylinder_d2, cylinder_d3);
curved_surface!(Sphere3, (radius), sphere_d2, sphere_d3);
curved_surface!(Torus3, (major_radius, minor_radius), torus_d2, torus_d3);
