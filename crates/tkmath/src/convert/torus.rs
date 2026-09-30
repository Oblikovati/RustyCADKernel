// Rust translation of FoundationClasses/TKMath/Convert/Convert_TorusToBSplineSurface.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Tori: a minor circle at distance R from the axis, swept around it.
use super::elementary::{ElementarySurfaceToBSplineSurface, SurfaceSizes};
use super::revolution::{arc_profile, periodic_knots, revolve, spans, trimmed_knots};
use crate::gp::{Point3, Torus3};
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::Failure;
use std::f64::consts::PI;

const TORUS_SIZES: SurfaceSizes = SurfaceSizes {
    u_poles: 7,
    v_poles: 7,
    u_knots: 4,
    v_knots: 4,
    u_degree: 2,
    v_degree: 2,
};

// ComputePoles: the minor circle arc on [v1, v2] swept over [u1, u2].
fn compute_poles(
    (major, minor): (f64, f64),
    (u1, u2): (f64, f64),
    (v1, v2): (f64, f64),
    poles: &mut Array2<'_, Point3>,
) -> Result<(), Failure> {
    let profile = arc_profile(
        v1,
        v2,
        TORUS_SIZES.v_poles as usize,
        |angle| (major + minor * angle.cos(), minor * angle.sin()),
        |angle, cosine| {
            (
                major + minor * angle.cos() / cosine,
                minor * angle.sin() / cosine,
            )
        },
    )?;
    let nb_v_poles: i32 = spans(v2 - v1).0.wrapping_mul(2).wrapping_add(1);
    revolve(&profile, nb_v_poles, u1, u2, poles)
}

fn domain(message: String) -> Failure {
    Failure::with_type("Standard_DomainError", &message, "")
}

fn radii(torus: &Torus3) -> (f64, f64) {
    (torus.major_radius(), torus.minor_radius())
}

impl ElementarySurfaceToBSplineSurface {
    /// Torus patch `[u1, u2] x [v1, v2]` (`Convert_TorusToBSplineSurface(T, U1, U2, V1, V2)`);
    /// both openings must lie in `[0, 2 PI]`;
    /// `ElementarySurfaceToBSplineSurface::from_torus_patch(&torus, 0.0, PI, 0.0, PI)?`.
    pub fn from_torus_patch(
        torus: &Torus3,
        u1: f64,
        u2: f64,
        v1: f64,
        v2: f64,
    ) -> Result<Self, Failure> {
        let (delta_u, delta_v) = (u2 - u1, v2 - v1);
        if (delta_u > 2. * PI) || (delta_u < 0.) || (delta_v > 2. * PI) || (delta_v < 0.) {
            return Err(domain(format!(
                "Convert_TorusToBSplineSurface: patch [{u1}, {u2}] x [{v1}, {v2}], expected openings in [0, 2 PI]"
            )));
        }
        let mut result = Self::with_sizes(TORUS_SIZES)?;
        let (nb_u_spans, alfa_u) = spans(delta_u);
        let (nb_v_spans, alfa_v) = spans(delta_v);
        result.set_counts(nb_u_spans, nb_v_spans);
        compute_poles(radii(torus), (u1, u2), (v1, v2), &mut result.poles)?;
        trimmed_knots(
            &mut result.u_knots,
            &mut result.u_mults,
            u1,
            result.nb_u_knots,
            alfa_u,
        )?;
        trimmed_knots(
            &mut result.v_knots,
            &mut result.v_mults,
            v1,
            result.nb_v_knots,
            alfa_v,
        )?;
        result.weigh_and_place(torus.position(), (alfa_u.cos(), alfa_v.cos()))?;
        result.finalize()?;
        Ok(result)
    }

    /// Torus band (`Convert_TorusToBSplineSurface(T, Param1, Param2, UTrim)`): trimmed
    /// in U on `[param1, param2]` and periodic in V when `u_trim`, otherwise periodic
    /// in U and trimmed in V;
    /// `ElementarySurfaceToBSplineSurface::from_torus_band(&torus, 0.0, PI, true)?`.
    pub fn from_torus_band(
        torus: &Torus3,
        param1: f64,
        param2: f64,
        u_trim: bool,
    ) -> Result<Self, Failure> {
        let delta: f64 = param2 - param1;
        if (delta > 2. * PI) || (delta < 0.) {
            return Err(domain(format!(
                "Convert_TorusToBSplineSurface: band [{param1}, {param2}], expected 0 <= Param2 - Param1 <= 2 PI"
            )));
        }
        let mut result = Self::with_sizes(TORUS_SIZES)?;
        (result.u_periodic, result.v_periodic) = (!u_trim, u_trim);
        let cosines: (f64, f64) = if u_trim {
            result.torus_u_band(radii(torus), param1, param2)?
        } else {
            result.torus_v_band(radii(torus), param1, param2)?
        };
        result.weigh_and_place(torus.position(), cosines)?;
        result.finalize()?;
        Ok(result)
    }

    /// Whole torus, periodic in U and V (`Convert_TorusToBSplineSurface(T)`);
    /// `ElementarySurfaceToBSplineSurface::from_torus(&torus)?.nb_v_poles() == 6`.
    pub fn from_torus(torus: &Torus3) -> Result<Self, Failure> {
        let mut result = Self::with_sizes(TORUS_SIZES)?;
        (result.u_periodic, result.v_periodic) = (true, true);
        (result.nb_u_poles, result.nb_v_poles) = (6, 6);
        (result.nb_u_knots, result.nb_v_knots) = (4, 4);
        compute_poles(
            radii(torus),
            (0., 2. * PI),
            (0., 2. * PI),
            &mut result.poles,
        )?;
        let knots = [0., 2. * PI / 3., 4. * PI / 3., 2. * PI];
        for (i, knot) in (1..=4).zip(knots) {
            (result.u_knots[i], result.v_knots[i]) = (knot, knot);
            (result.u_mults[i], result.v_mults[i]) = (2, 2);
        }
        result.weigh_and_place(torus.position(), (0.5, 0.5))?;
        result.finalize()?;
        Ok(result)
    }

    // Periodic in U, trimmed in V on [param1, param2].
    fn torus_v_band(
        &mut self,
        radii: (f64, f64),
        param1: f64,
        param2: f64,
    ) -> Result<(f64, f64), Failure> {
        compute_poles(radii, (0., 2. * PI), (param1, param2), &mut self.poles)?;
        (self.nb_u_poles, self.nb_u_knots) = (6, 4);
        let (nb_v_spans, alfa_v) = spans(param2 - param1);
        self.nb_v_poles = nb_v_spans.wrapping_mul(2).wrapping_add(1);
        self.nb_v_knots = nb_v_spans.wrapping_add(1);
        periodic_knots(&mut self.u_knots, &mut self.u_mults)?;
        trimmed_knots(
            &mut self.v_knots,
            &mut self.v_mults,
            param1,
            self.nb_v_knots,
            alfa_v,
        )?;
        Ok((0.5, alfa_v.cos()))
    }

    // Trimmed in U on [param1, param2], periodic in V.
    fn torus_u_band(
        &mut self,
        radii: (f64, f64),
        param1: f64,
        param2: f64,
    ) -> Result<(f64, f64), Failure> {
        compute_poles(radii, (param1, param2), (0., 2. * PI), &mut self.poles)?;
        (self.nb_v_poles, self.nb_v_knots) = (6, 4);
        let (nb_u_spans, alfa_u) = spans(param2 - param1);
        self.nb_u_poles = nb_u_spans.wrapping_mul(2).wrapping_add(1);
        self.nb_u_knots = nb_u_spans.wrapping_add(1);
        periodic_knots(&mut self.v_knots, &mut self.v_mults)?;
        trimmed_knots(
            &mut self.u_knots,
            &mut self.u_mults,
            param1,
            self.nb_u_knots,
            alfa_u,
        )?;
        Ok((alfa_u.cos(), 0.5))
    }
}
