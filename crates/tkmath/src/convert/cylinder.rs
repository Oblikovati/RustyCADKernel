// Rust translation of Convert_CylinderToBSplineSurface.cxx and Convert_ConeToBSplineSurface.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Cylinders and cones: a straight generatrix (degree 1 in V) swept around the axis.
use super::elementary::{ElementarySurfaceToBSplineSurface, SurfaceSizes};
use super::revolution::{periodic_knots, revolve, spans, trimmed_knots};
use crate::gp::{Axis3, Cone3, Cylinder3};
use crate::math::real_epsilon;
use rustycad_tkernel::standard::Failure;
use std::f64::consts::PI;

const RULED_SIZES: SurfaceSizes = SurfaceSizes {
    u_poles: 9,
    v_poles: 2,
    u_knots: 5,
    v_knots: 2,
    u_degree: 2,
    v_degree: 1,
};

// The two generatrix ends (x, z) in the reference frame, the surface frame and
// the native class name.
struct Generatrix {
    ends: [(f64, f64); 2],
    position: Axis3,
    class: &'static str,
}

fn cylinder_generatrix(cylinder: &Cylinder3, v1: f64, v2: f64) -> Generatrix {
    let radius: f64 = cylinder.radius();
    Generatrix {
        ends: [(radius, v1), (radius, v2)],
        position: cylinder.position(),
        class: "Convert_CylinderToBSplineSurface",
    }
}

fn cone_generatrix(cone: &Cone3, v1: f64, v2: f64) -> Generatrix {
    let (radius, angle) = (cone.ref_radius(), cone.semi_angle());
    Generatrix {
        ends: [
            (radius + v1 * angle.sin(), v1 * angle.cos()),
            (radius + v2 * angle.sin(), v2 * angle.cos()),
        ],
        position: cone.position(),
        class: "Convert_ConeToBSplineSurface",
    }
}

impl ElementarySurfaceToBSplineSurface {
    /// Cylinder patch `[u1, u2] x [v1, v2]` (`Convert_CylinderToBSplineSurface(Cyl, U1, U2, V1, V2)`);
    /// `ElementarySurfaceToBSplineSurface::from_cylinder_patch(&cylinder, 0.0, PI, -5.0, 5.0)?`.
    pub fn from_cylinder_patch(
        cylinder: &Cylinder3,
        u1: f64,
        u2: f64,
        v1: f64,
        v2: f64,
    ) -> Result<Self, Failure> {
        ruled_patch(&cylinder_generatrix(cylinder, v1, v2), (u1, u2), (v1, v2))
    }

    /// Cylinder periodic in U between heights `v1` and `v2`
    /// (`Convert_CylinderToBSplineSurface(Cyl, V1, V2)`);
    /// `ElementarySurfaceToBSplineSurface::from_cylinder(&cylinder, 0.0, 10.0)?`.
    pub fn from_cylinder(cylinder: &Cylinder3, v1: f64, v2: f64) -> Result<Self, Failure> {
        ruled_periodic(&cylinder_generatrix(cylinder, v1, v2), (v1, v2))
    }

    /// Cone patch `[u1, u2] x [v1, v2]` (`Convert_ConeToBSplineSurface(C, U1, U2, V1, V2)`);
    /// `ElementarySurfaceToBSplineSurface::from_cone_patch(&cone, 0.0, PI, 0.0, 5.0)?`.
    pub fn from_cone_patch(
        cone: &Cone3,
        u1: f64,
        u2: f64,
        v1: f64,
        v2: f64,
    ) -> Result<Self, Failure> {
        ruled_patch(&cone_generatrix(cone, v1, v2), (u1, u2), (v1, v2))
    }

    /// Cone periodic in U between `v1` and `v2` along the generatrix
    /// (`Convert_ConeToBSplineSurface(C, V1, V2)`);
    /// `ElementarySurfaceToBSplineSurface::from_cone(&cone, 0.0, 10.0)?`.
    pub fn from_cone(cone: &Cone3, v1: f64, v2: f64) -> Result<Self, Failure> {
        ruled_periodic(&cone_generatrix(cone, v1, v2), (v1, v2))
    }

    // V knots v1, v2 of multiplicity 2.
    fn set_generatrix_knots(&mut self, v1: f64, v2: f64) {
        (self.v_knots[1], self.v_mults[1]) = (v1, 2);
        (self.v_knots[2], self.v_mults[2]) = (v2, 2);
    }
}

fn check_heights(class: &str, v1: f64, v2: f64) -> Result<(), Failure> {
    if (v2 - v1).abs() <= real_epsilon(v1).abs() {
        return Err(domain(format!("{class}: V range [{v1}, {v2}] is empty")));
    }
    Ok(())
}

fn domain(message: String) -> Failure {
    Failure::with_type("Standard_DomainError", &message, "")
}

fn ruled_patch(
    generatrix: &Generatrix,
    (u1, u2): (f64, f64),
    (v1, v2): (f64, f64),
) -> Result<ElementarySurfaceToBSplineSurface, Failure> {
    let delta_u: f64 = u2 - u1;
    if (v2 - v1).abs() <= real_epsilon(v1).abs() || (delta_u > 2. * PI) || (delta_u < 0.) {
        return Err(domain(format!(
            "{}: patch [{u1}, {u2}] x [{v1}, {v2}], expected 0 <= U2 - U1 <= 2 PI and V1 != V2",
            generatrix.class
        )));
    }
    let mut result = ElementarySurfaceToBSplineSurface::with_sizes(RULED_SIZES)?;
    let (nb_u_spans, alfa_u) = spans(delta_u);
    result.set_counts(nb_u_spans, 0);
    (result.nb_v_poles, result.nb_v_knots) = (2, 2);
    revolve(&generatrix.ends, 2, u1, u2, &mut result.poles)?;
    trimmed_knots(
        &mut result.u_knots,
        &mut result.u_mults,
        u1,
        result.nb_u_knots,
        alfa_u,
    )?;
    result.set_generatrix_knots(v1, v2);
    result.weigh_and_place(generatrix.position, (alfa_u.cos(), 1.))?;
    result.finalize()?;
    Ok(result)
}

fn ruled_periodic(
    generatrix: &Generatrix,
    (v1, v2): (f64, f64),
) -> Result<ElementarySurfaceToBSplineSurface, Failure> {
    check_heights(generatrix.class, v1, v2)?;
    let mut result = ElementarySurfaceToBSplineSurface::with_sizes(RULED_SIZES)?;
    result.u_periodic = true;
    revolve(&generatrix.ends, 2, 0., 2. * PI, &mut result.poles)?;
    (result.nb_u_poles, result.nb_u_knots) = (6, 4);
    (result.nb_v_poles, result.nb_v_knots) = (2, 2);
    periodic_knots(&mut result.u_knots, &mut result.u_mults)?;
    result.set_generatrix_knots(v1, v2);
    // = std::cos(pi / 3)
    result.weigh_and_place(generatrix.position, (0.5, 1.))?;
    result.finalize()?;
    Ok(result)
}
