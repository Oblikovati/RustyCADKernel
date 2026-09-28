//! The IntAna package probe; the counterpart of tools/oracle/tkgeombase_probe_intana.cxx,
//! split like its conic, geometric and analytic parts.
mod analytic;
mod conics;
mod geometric;
pub use analytic::*;
pub use conics::*;
pub use geometric::*;

use super::common::Reader;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana::Quadric;
use rustycad_tkmath::gp::{Cone3, Cylinder3, Plane3, Sphere3, Torus3};

/// One surface of the corpus (GbSurface).
pub enum Surface {
    Plane(Plane3),
    Cylinder(Cylinder3),
    Sphere(Sphere3),
    Cone(Cone3),
    Torus(Torus3),
}

/// gb_surface: the kind, the Ax3, then the radii (and angle).
pub fn surface(r: &mut Reader) -> Result<Surface, Failure> {
    let kind = r.integer();
    let a = r.ax3();
    Ok(match kind {
        0 => Surface::Plane(Plane3::from_position(a)),
        1 => Surface::Cylinder(Cylinder3::new(a, r.real())?),
        2 => Surface::Sphere(Sphere3::new(a, r.real())?),
        3 => {
            let angle = r.real();
            Surface::Cone(Cone3::new(a, angle, r.real())?)
        }
        _ => {
            let major = r.real();
            Surface::Torus(Torus3::new(a, major, r.real())?)
        }
    })
}

/// gb_quadric: the quadric of a plane, cylinder, sphere or cone.
pub fn quadric(s: &Surface) -> Result<Quadric, Failure> {
    Ok(match *s {
        Surface::Plane(x) => Quadric::from_plane(x)?,
        Surface::Cylinder(x) => Quadric::from_cylinder(x),
        Surface::Sphere(x) => Quadric::from_sphere(x),
        Surface::Cone(x) => Quadric::from_cone(x),
        Surface::Torus(_) => unreachable!("the corpus never makes a quadric of a torus"),
    })
}
