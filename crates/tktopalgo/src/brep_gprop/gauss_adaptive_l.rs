// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepGProp/BRepGProp_Gauss.cxx (the
// outer loops of the adaptive Compute: the boundary curves and the L intervals) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The outer integral of the adaptive `Compute`: along each boundary curve (or the V
//! range of a naturally bounded face), bisecting the L intervals.
use super::domain::BRepGPropDomain;
use super::face::BRepGPropFace;
use super::gauss::Inertia;
use super::gauss_adaptive::{
    AdaptiveRun, EPS_PARAM, GaussRules, SUBS_POWER, knots_of, max_element, max_subs, sm,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkmath::math;

/// The sums of one L interval (`CDim`, `CIxx`, `CIyy`, `CIzz` per Gauss order, and the
/// static and product moments of the first order).
#[derive(Default)]
struct LineSums {
    dim: [f64; 2],
    ixx: [f64; 2],
    iyy: [f64; 2],
    izz: [f64; 2],
    ix: f64,
    iy: f64,
    iz: f64,
    ixy: f64,
    ixz: f64,
    iyz: f64,
}

/// The rules and knots of the boundary curve under integration.
struct Boundary {
    rules: GaussRules,
    knots: Vec<f64>,
    range: (f64, f64),
}

impl AdaptiveRun<'_, '_> {
    /// The `while (isNaturalRestriction || theDomain.More())` loop: the sum of the
    /// integrals along the boundary curves, `None` for an edge without pcurve (the
    /// native `return Precision::Infinite()`).
    pub(super) fn boundaries(
        &mut self,
        surface: &mut BRepGPropFace,
        domain: &mut BRepGPropDomain,
    ) -> Result<Option<Inertia>, Failure> {
        let mut total = Inertia::default();
        while self.input.natural || domain.more() {
            let Some(boundary) = self.boundary(surface, domain)? else {
                return Ok(None);
            };
            self.curve_integral(surface, &boundary, &mut total)?;
            if self.input.natural {
                break;
            }
            domain.next()?;
        }
        Ok(Some(total))
    }

    /// The Gauss rules, knots and range of the next boundary: the V range of a natural
    /// face, else the pcurve of the domain's edge (`None` without pcurve).
    fn boundary(
        &self,
        surface: &mut BRepGPropFace,
        domain: &BRepGPropDomain,
    ) -> Result<Option<Boundary>, Failure> {
        if self.input.natural {
            let order = (2 * self.input.u_rules.orders[0]).min(math::gauss_points_max());
            let (_, _, v1, v2) = self.input.bounds;
            return Ok(Some(Boundary {
                rules: GaussRules::new(order)?,
                knots: knots_of(surface.sv_int_subs()?, |k| surface.v_knots(k))?,
                range: (v1, v2),
            }));
        }
        if !surface.load_edge(&domain.value()?)? {
            return Ok(None);
        }
        let rules = GaussRules::new(surface.l_int_order(self.input.epsilon)?)?;
        let knots = knots_of(surface.l_int_subs()?, |k| surface.l_knots(k))?;
        let range = (surface.first_parameter()?, surface.last_parameter()?);
        Ok(Some(Boundary {
            rules,
            knots,
            range,
        }))
    }

    /// The do-while loop over the L intervals of one boundary (bisecting the interval of
    /// largest error while the verification asks for it), added to `total`.
    fn curve_integral(
        &mut self,
        surface: &BRepGPropFace,
        boundary: &Boundary,
        total: &mut Inertia,
    ) -> Result<(), Failure> {
        let (l1, l2) = boundary.range;
        if (l2 - l1).abs() <= EPS_PARAM {
            return Ok(());
        }
        let end = self.l.fill(l1, l2, &boundary.knots, Some(&mut self.err_ul));
        let max = max_subs(end, SUBS_POWER).min(sm());
        self.l.reset();
        self.err_ul.iter_mut().for_each(|error| *error = 0.0);
        let (mut error_l, mut k_end, mut j) = (0.0, 1, 0);
        loop {
            j += 1;
            let range = self.l.next_range(j, end);
            if self.l.is_exhausted(j, max) {
                if k_end != 1 {
                    j -= 1;
                    self.state.eps_l = error_l;
                    self.state.eps = self.state.eps_l / 0.9;
                    break;
                }
                self.l.inertia[j - 1] = Inertia::default();
                self.l.error[j - 1] = 0.0;
            } else {
                for index in range.into_iter().take(k_end) {
                    self.l_span(surface, &boundary.rules, index)?;
                }
            }
            if j == end {
                k_end = 2;
                self.correct_epsilon(j);
            }
            if k_end == 2 {
                error_l = self.l.error[max_element(&self.l.error)];
            }
            if !((error_l - self.state.eps_l > 0.0 && self.is_verify) || k_end == 1) {
                break;
            }
        }
        for inertia in &self.l.inertia[..j] {
            self.gauss.add_inertia(inertia, total);
        }
        self.state.error_l_max = std_max(self.state.error_l_max, error_l);
        Ok(())
    }

    /// The epsilon corrected by the current mass (`DDim = |sum of masses * Eps|`).
    fn correct_epsilon(&mut self, j: usize) {
        let mut ddim = 0.0;
        for inertia in &self.l.inertia[..j] {
            ddim += inertia.mass;
        }
        ddim = (ddim * self.input.epsilon).abs();
        if ddim > self.state.eps {
            self.state.eps = ddim;
            self.state.eps_l = 0.9 * self.state.eps;
        }
    }

    /// One L interval `index`: the inner U integral at each Gauss point of both orders,
    /// its moments into `l.inertia[index]` and its error into `l.error[index]`.
    fn l_span(
        &mut self,
        surface: &BRepGPropFace,
        rules: &GaussRules,
        index: usize,
    ) -> Result<(), Failure> {
        let lm = 0.5 * (self.l.second[index] + self.l.first[index]);
        let lr = 0.5 * (self.l.second[index] - self.l.first[index]);
        let mut sums = LineSums::default();
        for pass in 0..self.input.error_passes {
            (
                sums.dim[pass],
                sums.ixx[pass],
                sums.iyy[pass],
                sums.izz[pass],
            ) = (0.0, 0.0, 0.0, 0.0);
            let (points, weights) = &rules.rules[pass];
            for (point, weight) in points.iter().zip(weights) {
                self.l_point(surface, (pass, index), lm + lr * point, *weight, &mut sums)?;
            }
        }
        self.store_l_span(index, lr, &sums);
        Ok(())
    }

    /// One Gauss point `l` of an L interval: the boundary point, then the U integral at
    /// its V, summed with the weight `Dul`.
    fn l_point(
        &mut self,
        surface: &BRepGPropFace,
        (pass, index): (usize, usize),
        l: f64,
        weight: f64,
        sums: &mut LineSums,
    ) -> Result<(), Failure> {
        if !self.boundary_point(surface, l, weight)? {
            return Ok(());
        }
        self.err_ul[index] = 0.0;
        let u1 = self.input.bounds.0;
        if (self.state.u2 - u1).abs() < EPS_PARAM {
            return Ok(());
        }
        let count = self.u_integral(surface, pass)?;
        let dul = self.state.dul;
        let g = self.gauss;
        for ui in &self.u.inertia[..count] {
            sums.dim[pass] = g.add(sums.dim[pass], g.mult(ui.mass, dul));
            sums.ixx[pass] = g.add(sums.ixx[pass], g.mult(ui.ixx, dul));
            sums.iyy[pass] = g.add(sums.iyy[pass], g.mult(ui.iyy, dul));
            sums.izz[pass] = g.add(sums.izz[pass], g.mult(ui.izz, dul));
        }
        if pass > 0 {
            return Ok(());
        }
        self.err_ul[index] = self.state.error_u * ((self.state.u2 - u1) * dul).abs();
        for ui in &self.u.inertia[..count] {
            sums.ix = g.add(sums.ix, g.mult(ui.ix, dul));
            sums.iy = g.add(sums.iy, g.mult(ui.iy, dul));
            sums.iz = g.add(sums.iz, g.mult(ui.iz, dul));
            sums.ixy = g.add(sums.ixy, g.mult(ui.ixy, dul));
            sums.ixz = g.add(sums.ixz, g.mult(ui.ixz, dul));
            sums.iyz = g.add(sums.iyz, g.mult(ui.iyz, dul));
        }
        Ok(())
    }

    /// The V, the U end and the weight `Dul` at the boundary parameter `l`: the V itself
    /// and the U upper bound on a natural face, else the pcurve point clamped to the
    /// bounds; false (the point is skipped) when `|Dul|` is below EPS_PARAM.
    fn boundary_point(
        &mut self,
        surface: &BRepGPropFace,
        l: f64,
        weight: f64,
    ) -> Result<bool, Failure> {
        let (bu1, bu2, bv1, bv2) = self.input.bounds;
        if self.input.natural {
            (self.state.v, self.state.u2, self.state.dul) = (l, bu2, weight);
            return Ok(true);
        }
        let (puv, vuv) = surface.d12d(l)?;
        self.state.dul = vuv.y() * weight;
        if self.state.dul.abs() < EPS_PARAM {
            return Ok(false);
        }
        self.state.v = clamp(puv.y(), bv1, bv2);
        self.state.u2 = clamp(puv.x(), bu1, bu2);
        Ok(true)
    }

    /// The moments and error of an L interval from its sums.
    fn store_l_span(&mut self, index: usize, lr: f64, sums: &LineSums) {
        let g = self.gauss;
        let li = &mut self.l.inertia[index];
        li.mass = g.mult(sums.dim[0], lr);
        li.ixx = g.mult(sums.ixx[0], lr);
        li.iyy = g.mult(sums.iyy[0], lr);
        li.izz = g.mult(sums.izz[0], lr);
        if self.input.error_passes == 2 {
            let sub_dim = (sums.dim[1] - sums.dim[0]).abs();
            if g.kind == super::gauss::GaussType::Vinert {
                // Natively the Vinert branch copies ErrUL into the loop's ErrorU first.
                self.state.error_u = self.err_ul[index];
            }
            self.l.error[index] = g.add(g.mult(sub_dim, lr), self.err_ul[index]);
        }
        li.ix = g.mult(sums.ix, lr);
        li.iy = g.mult(sums.iy, lr);
        li.iz = g.mult(sums.iz, lr);
        li.ixy = g.mult(sums.ixy, lr);
        li.ixz = g.mult(sums.ixz, lr);
        li.iyz = g.mult(sums.iyz, lr);
    }
}

/// `if (x < low) x = low; else if (x > high) x = high;`
fn clamp(x: f64, low: f64, high: f64) -> f64 {
    if x < low {
        low
    } else if x > high {
        high
    } else {
        x
    }
}
