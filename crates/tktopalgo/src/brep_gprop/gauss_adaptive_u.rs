// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepGProp/BRepGProp_Gauss.cxx (the
// inner U loop of the adaptive Compute) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The inner integral of the adaptive `Compute`: along U at the V of one boundary
//! point, from the U lower bound to the boundary's U, bisecting the U intervals.
use super::face::BRepGPropFace;
use super::gauss::{GaussType, Inertia, add_volume_part};
use super::gauss_adaptive::{AdaptiveRun, SUBS_POWER, max_element, max_subs, sm};
use rustycad_tkernel::standard::Failure;

impl AdaptiveRun<'_, '_> {
    /// The do-while loop over the U intervals for the Gauss order `pass` of the L
    /// integral; the number of intervals computed (`JU`). Its error estimate is taken
    /// once, when the knot intervals are done, so the loop refines only once `EpsU` is
    /// not 0, as natively.
    pub(super) fn u_integral(
        &mut self,
        surface: &BRepGPropFace,
        pass: usize,
    ) -> Result<usize, Failure> {
        let (u1, u2) = (self.input.bounds.0, self.state.u2);
        let end = self.u.fill(u1, u2, &self.input.u_knots, None);
        let max = max_subs(end, SUBS_POWER).min(sm());
        self.u.reset();
        self.state.error_u = 0.0;
        let (mut k_end, mut j) = (1, 0);
        loop {
            j += 1;
            let range = self.u.next_range(j, end);
            if self.u.is_exhausted(j, max) {
                if k_end != 1 {
                    j -= 1;
                    self.state.eps_u = self.state.error_u;
                    let span = ((u2 - u1) * self.state.dul).abs();
                    self.state.eps = 10. * self.state.eps_u * span;
                    self.state.eps_l = 0.9 * self.state.eps;
                    break;
                }
                self.u.error[j - 1] = 0.0;
                self.u.inertia[j - 1] = Inertia::default();
            } else {
                for index in range.into_iter().take(k_end) {
                    self.u_span(surface, pass, index)?;
                }
            }
            if j == end {
                k_end = 2;
                self.state.error_u = self.u.error[max_element(&self.u.error)];
            }
            let refine = self.state.error_u - self.state.eps_u > 0.0 && self.state.eps_u != 0.0;
            if !(refine || k_end == 1) {
                break;
            }
        }
        Ok(j)
    }

    /// One U interval `index`: the Gauss points of the orders `pass..` (a surface error
    /// order sums the area only), the moments into `u.inertia[index]` and, on the first
    /// L order, the error into `u.error[index]`.
    fn u_span(
        &mut self,
        surface: &BRepGPropFace,
        pass: usize,
        index: usize,
    ) -> Result<(), Failure> {
        let mut local = [Inertia::default(); 2];
        let um = 0.5 * (self.u.second[index] + self.u.first[index]);
        let ur = 0.5 * (self.u.second[index] - self.u.first[index]);
        let orders = self.input.error_passes - pass;
        for (order, sums) in local.iter_mut().enumerate().take(orders) {
            let (points, weights) = &self.input.u_rules.rules[order];
            for (point, weight) in points.iter().zip(weights) {
                let (p, normal) = surface.normal(um + ur * point, self.state.v)?;
                self.add_u_point(order, (p, normal), *weight, sums);
            }
        }
        self.store_u_span(pass, index, ur, &local);
        Ok(())
    }

    /// One Gauss point of a U interval into the sums of Gauss order `order`.
    fn add_u_point(
        &self,
        order: usize,
        (point, normal): (rustycad_tkmath::gp::Point3, rustycad_tkmath::gp::Vector3),
        weight: f64,
        local: &mut Inertia,
    ) {
        let input = &self.input;
        if self.gauss.kind == GaussType::Vinert {
            let plane = (input.coeff, input.by_point);
            add_volume_part(point, normal, input.location, weight, plane, local);
        } else if order > 0 {
            local.mass += weight * normal.magnitude();
        } else {
            self.gauss
                .add_surface_part(point, normal, input.location, weight, local);
        }
    }

    /// The moments of a U interval (times its half range) and its error.
    fn store_u_span(&mut self, pass: usize, index: usize, ur: f64, local: &[Inertia; 2]) {
        let g = self.gauss;
        let ui = &mut self.u.inertia[index];
        ui.mass = g.mult(local[0].mass, ur);
        if g.kind == GaussType::Vinert {
            ui.ixx = g.mult(local[0].ixx, ur);
            ui.iyy = g.mult(local[0].iyy, ur);
            ui.izz = g.mult(local[0].izz, ur);
        }
        if pass > 0 {
            return;
        }
        let d_mass = (local[1].mass - local[0].mass).abs();
        ui.ix = g.mult(local[0].ix, ur);
        ui.iy = g.mult(local[0].iy, ur);
        ui.iz = g.mult(local[0].iz, ur);
        if g.kind == GaussType::Sinert {
            ui.ixx = g.mult(local[0].ixx, ur);
            ui.iyy = g.mult(local[0].iyy, ur);
            ui.izz = g.mult(local[0].izz, ur);
        }
        ui.ixy = g.mult(local[0].ixy, ur);
        ui.ixz = g.mult(local[0].ixz, ur);
        ui.iyz = g.mult(local[0].iyz, ur);
        self.u.error[index] = g.mult(d_mass, ur);
    }
}
