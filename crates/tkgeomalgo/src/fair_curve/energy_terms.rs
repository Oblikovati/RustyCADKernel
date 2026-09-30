// Rust translation of ModelingAlgorithms/TKGeomAlgo/FairCurve/FairCurve_Energy.cxx (Gradient1,
// Hessian1) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `FairCurve_Energy::Gradient1` and `Hessian1`: the chain rule from the
//! intermediate format (value, the gradient in every pole coordinate and the
//! sliding, then the packed lower Hessian of those, see `FairCurveEnergy::indice`)
//! to the free variables.
use super::FairCurveEnergy;
use rustycad_tkmath::gp::Xy;
use rustycad_tkmath::math::{Matrix, Vector};

/// The index ranges and end factors of `Hessian1`.
struct HessianFrame {
    vdeb: i32,
    vfin: i32,
    vup: i32,
    deb_h: i32,
    fin_h: i32,
    lambda: [f64; 2],
    /// `(cos^2, sin^2, 2 cos sin)` of each end form.
    trig: [(f64, f64, f64); 2],
}

impl FairCurveEnergy {
    fn poles_length(&self) -> i32 {
        self.poles.get().map_or(0, |p| p.array1().len() as i32)
    }
    /// The distance between an end pole and its neighbour (side 0: start).
    fn end_length(&self, side: usize) -> f64 {
        let poles = self.poles.get().expect("FairCurve poles").array1();
        let (end, next) = match side {
            0 => (poles.lower(), poles.lower() + 1),
            _ => (poles.upper(), poles.upper() - 1),
        };
        poles[end].distance(poles[next])
    }
    /// `LinearForm(side) + 2 * Lambda * QuadForm(side)`.
    fn laux(&self, side: usize, lambda: f64) -> Xy {
        self.linear_form[side] + 2.0 * lambda * self.quad_form[side]
    }
    /// `LinearForm(side).X() * Vect(a) + LinearForm(side).Y() * Vect(b)`.
    fn linear(&self, side: usize, v: &Vector<'_>, a: i32, b: i32) -> f64 {
        self.linear_form[side].x() * v[a] + self.linear_form[side].y() * v[b]
    }
    /// `Gradient1(Vect, Grad)`: the gradient in the free variables.
    pub(super) fn gradient1(&self, v: &Vector<'_>, grad: &mut Vector<'_>) {
        let (mut deb_g, mut fin_g) = (grad.lower(), grad.upper());
        let length: i32 = self.poles_length();
        let (mut vdeb, vfin) = (3, 2 * length - 2);
        if self.contr_order1 >= 1 {
            grad[deb_g] = self.linear_form[0] * Xy::new(v[vdeb], v[vdeb + 1]);
            (vdeb, deb_g) = (vdeb + 2, deb_g + 1);
        }
        if self.contr_order1 == 2 {
            let dpole = Xy::new(v[vdeb], v[vdeb + 1]);
            grad[deb_g - 1] += self.laux(0, self.end_length(0)) * dpole;
            grad[deb_g] = self.linear_form[0] * dpole;
            (vdeb, deb_g) = (vdeb + 2, deb_g + 1);
        }
        if self.with_aux_value {
            grad[fin_g] = v[2 * length + 1];
            fin_g -= 1;
        }
        if self.contr_order2 >= 1 {
            grad[fin_g] = self.linear_form[1] * Xy::new(v[vfin - 1], v[vfin]);
            fin_g -= 1;
        }
        if self.contr_order2 == 2 {
            let dpole = Xy::new(v[vfin - 3], v[vfin - 2]);
            grad[fin_g] = grad[fin_g + 1] + self.laux(1, self.end_length(1)) * dpole;
            grad[fin_g + 1] = self.linear_form[1] * dpole;
            fin_g -= 1;
        }
        for ii in deb_g..=fin_g {
            grad[ii] = v[vdeb];
            vdeb += 1;
        }
    }
    /// `Hessian1(Vect, H)`: the Hessian in the free variables.
    pub(super) fn hessian1(&self, v: &Vector<'_>, h: &mut Matrix<'_>) {
        let frame = self.hessian_frame();
        if self.contr_order1 >= 1 {
            self.left_lambda(&frame, v, h);
        }
        if self.contr_order1 >= 2 {
            self.left_mu(&frame, v, h);
        }
        if self.contr_order2 >= 1 {
            self.right_lambda(&frame, v, h);
        }
        if self.contr_order2 >= 2 {
            self.right_mu(&frame, v, h);
        }
        if self.with_aux_value {
            self.sliding_line(&frame, v, h);
        }
        self.inner_block(&frame, v, h);
        symmetrize(h);
    }
    fn hessian_frame(&self) -> HessianFrame {
        let length: i32 = self.poles_length();
        let aux: i32 = i32::from(self.with_aux_value);
        let trig = |form: Xy| {
            (
                form.x() * form.x(),
                form.y() * form.y(),
                2.0 * form.x() * form.y(),
            )
        };
        let lambda = [
            if self.contr_order1 >= 1 {
                self.end_length(0)
            } else {
                0.0
            },
            if self.contr_order2 >= 1 {
                self.end_length(1)
            } else {
                0.0
            },
        ];
        HessianFrame {
            vdeb: 3 + 2 * self.contr_order1,
            vfin: 2 * length - 2 * (self.contr_order2 + 1),
            vup: 2 * length + aux,
            deb_h: 1 + self.contr_order1,
            fin_h: self.nb_var - aux - self.contr_order2,
            lambda,
            trig: [trig(self.linear_form[0]), trig(self.linear_form[1])],
        }
    }
    /// `cos^2 * Vect(kk) + 2 cos sin * Vect(ii) + sin^2 * Vect(ii + 1)` of an end.
    fn quadratic(f: &HessianFrame, side: usize, v: &Vector<'_>, kk: i32, ii: i32) -> f64 {
        let (cos2, sin2, cos_sin) = f.trig[side];
        cos2 * v[kk] + cos_sin * v[ii] + sin2 * v[ii + 1]
    }
    /// The first column: the derivatives in the length of the start tangent.
    fn left_lambda(&self, f: &HessianFrame, v: &Vector<'_>, h: &mut Matrix<'_>) {
        let jj: i32 = f.vdeb - 2 * self.contr_order1;
        let (kk, ii) = (self.indice(jj, jj), self.indice(jj + 1, jj));
        h[(1, 1)] = Self::quadratic(f, 0, v, kk, ii);
        if self.contr_order1 >= 2 {
            self.left_lambda_g2(f, v, h);
        }
        let aux: i32 = i32::from(self.with_aux_value);
        if self.with_aux_value {
            let kk: i32 = self.indice(f.vup, f.vdeb - 2 * self.contr_order1);
            h[(self.nb_var, 1)] = self.linear(0, v, kk, kk + 1);
        }
        self.zero_right_ends(h, 1, aux);
        let mut kk: i32 = self.indice(f.vdeb, f.vdeb - 2 * self.contr_order1);
        for (vk, ii) in (f.vdeb..).zip(f.deb_h..=f.fin_h) {
            h[(ii, 1)] = self.linear(0, v, kk, kk + 1);
            kk += vk;
        }
    }
    /// The G2 part of the first two rows of the start.
    fn left_lambda_g2(&self, f: &HessianFrame, v: &Vector<'_>, h: &mut Matrix<'_>) {
        let (cos2, sin2, cos_sin) = f.trig[0];
        let (l, q) = (self.linear_form[0], self.quad_form[0]);
        let laux: Xy = self.laux(0, f.lambda[0]);
        let jj: i32 = f.vdeb - 2 * (self.contr_order1 - 1);
        let (kk, ii) = (self.indice(jj, jj - 2), self.indice(jj + 1, jj - 2));
        let aux = Xy::new(v[kk + 2], v[ii + 3]);
        h[(1, 1)] += 2.0
            * ((q.x() * v[5] + q.y() * v[6])
                + (laux.x() * (l.x() * v[kk] + l.y() * v[kk + 1])
                    + laux.y() * (l.x() * v[ii] + l.y() * v[ii + 1]))
                + laux.x() * laux.y() * v[ii + 2])
            + (laux.x() * laux.x() * v[kk + 2] + laux.y() * laux.y() * v[ii + 3]);
        h[(2, 1)] = (cos2 * v[kk] + cos_sin * (v[ii] + v[kk + 1]) / 2.0 + sin2 * v[ii + 1])
            + laux * l.multiplied_components(aux)
            + (laux.x() * l.y() + laux.y() * l.x()) * v[ii + 2];
    }
    /// `H(NbVar - Aux, column) = 0` (and the row before under a G2 end constraint):
    /// "correct if there are less than 3 nodes".
    fn zero_right_ends(&self, h: &mut Matrix<'_>, column: i32, aux: i32) {
        if self.contr_order2 >= 1 {
            h[(self.nb_var - aux, column)] = 0.0;
            if self.contr_order2 == 2 {
                h[(self.nb_var - aux - 1, column)] = 0.0;
            }
        }
    }
    /// The second row: the derivatives in the second-derivative term of the start.
    fn left_mu(&self, f: &HessianFrame, v: &Vector<'_>, h: &mut Matrix<'_>) {
        let jj: i32 = f.vdeb - 2 * (self.contr_order1 - 1);
        let (kk, ii) = (self.indice(jj, jj), self.indice(jj + 1, jj));
        h[(2, 2)] = Self::quadratic(f, 0, v, kk, ii);
        let laux: Xy = self.laux(0, f.lambda[0]);
        if self.with_aux_value {
            let kk: i32 = self.indice(f.vup, f.vdeb - 2 * (self.contr_order1 - 1));
            h[(self.nb_var, 1)] += laux * Xy::new(v[kk], v[kk + 1]);
            h[(self.nb_var, 2)] = self.linear(0, v, kk, kk + 1);
        }
        self.zero_right_ends(h, 2, i32::from(self.with_aux_value));
        let mut kk: i32 = self.indice(f.vdeb, f.vdeb - 2 * self.contr_order1 + 2);
        for (vk, ii) in (f.vdeb..).zip(f.deb_h..=f.fin_h) {
            h[(ii, 2)] = self.linear(0, v, kk, kk + 1);
            h[(ii, 1)] += laux.x() * v[kk] + laux.y() * v[kk + 1];
            kk += vk;
        }
    }
    /// The row of the length of the end tangent.
    fn right_lambda(&self, f: &HessianFrame, v: &Vector<'_>, h: &mut Matrix<'_>) {
        let jj: i32 = f.fin_h + 1;
        let vk: i32 = f.vfin + 2 * self.contr_order2 - 1;
        for (kk, ii) in (self.indice(vk, f.vdeb)..).zip(f.deb_h..=f.fin_h) {
            h[(jj, ii)] = self.linear(1, v, kk, kk + vk);
        }
        let kk: i32 = self.indice(vk, vk);
        let (cos2, sin2, cos_sin) = f.trig[1];
        h[(jj, jj)] = cos2 * v[kk] + cos_sin * v[kk + vk] + sin2 * v[kk + vk + 1];
        if self.contr_order2 >= 2 {
            self.right_lambda_g2(f, v, h);
        }
    }
    /// The G2 part of the last two rows of the end.
    fn right_lambda_g2(&self, f: &HessianFrame, v: &Vector<'_>, h: &mut Matrix<'_>) {
        let (cos2, sin2, cos_sin) = f.trig[1];
        let (l, q) = (self.linear_form[1], self.quad_form[1]);
        let laux: Xy = self.laux(1, f.lambda[1]);
        let jj: i32 = f.vfin + 2 * self.contr_order2 - 3;
        let (kk, ii) = (self.indice(jj + 2, jj), self.indice(jj + 3, jj));
        let ll: i32 = self.indice(jj, jj);
        let row: i32 = f.fin_h + 1;
        h[(row, row)] += 2.0
            * ((q.x() * v[jj] + q.y() * v[jj + 1])
                + (laux.x() * (l.x() * v[kk] + l.y() * v[ii])
                    + laux.y() * (l.x() * v[kk + 1] + l.y() * v[ii + 1]))
                + laux.x() * laux.y() * v[ll + jj])
            + (laux.x() * laux.x() * v[ll] + laux.y() * laux.y() * v[ll + jj + 1]);
        h[(row + 1, row)] = cos2 * v[kk] + cos_sin * (v[ii] + v[kk + 1]) / 2.0 + sin2 * v[ii + 1];
        let aux = Xy::new(v[ll], v[ll + jj + 1]);
        h[(row + 1, row)] += laux * l.multiplied_components(aux)
            + (laux.x() * l.y() + laux.y() * l.x()) * v[ll + jj];
    }
    /// The row of the second-derivative term of the end.
    fn right_mu(&self, f: &HessianFrame, v: &Vector<'_>, h: &mut Matrix<'_>) {
        let jj: i32 = f.fin_h + 2;
        let vk: i32 = f.vfin + 2 * self.contr_order2 - 3;
        let laux: Xy = self.laux(1, f.lambda[1]);
        for (kk, ii) in (self.indice(vk, f.vdeb)..).zip(f.deb_h..=f.fin_h) {
            h[(jj, ii)] = self.linear(1, v, kk, kk + vk);
            h[(jj - 1, ii)] += laux.x() * v[kk] + laux.y() * v[kk + vk];
        }
        let (kk, ii) = (self.indice(vk, vk), self.indice(vk + 1, vk));
        h[(jj, jj)] = Self::quadratic(f, 1, v, kk, ii);
    }
    /// The row of the free sliding.
    fn sliding_line(&self, f: &HessianFrame, v: &Vector<'_>, h: &mut Matrix<'_>) {
        for (kk, ii) in (self.indice(f.vup, f.vdeb)..).zip(f.deb_h..=f.fin_h) {
            h[(self.nb_var, ii)] = v[kk];
        }
        if self.contr_order2 >= 1 {
            let kk: i32 = self.indice(f.vup, f.vfin + 2 * self.contr_order2 - 1);
            h[(self.nb_var, f.fin_h + 1)] = self.linear(1, v, kk, kk + 1);
        }
        if self.contr_order2 >= 2 {
            let kk: i32 = self.indice(f.vup, f.vfin + 2 * self.contr_order2 - 3);
            let pole = Xy::new(v[kk], v[kk + 1]);
            h[(self.nb_var, f.fin_h + 1)] += self.laux(1, f.lambda[1]) * pole;
            h[(self.nb_var, f.fin_h + 2)] = self.linear_form[1] * pole;
        }
        let kk: i32 = self.indice(f.vup, f.vup);
        let upper: i32 = h.upper_row();
        h[(upper, upper)] = v[kk];
    }
    /// The block of the free poles, copied from the packed values.
    fn inner_block(&self, f: &HessianFrame, v: &Vector<'_>, h: &mut Matrix<'_>) {
        let mut kk: i32 = self.indice(f.vdeb, f.vdeb);
        for ii in f.deb_h..=f.fin_h {
            for jj in f.deb_h..=ii {
                h[(ii, jj)] = v[kk];
                kk += 1;
            }
            kk += f.vdeb - 1;
        }
    }
}

/// Copies the lower triangle to the upper one.
fn symmetrize(h: &mut Matrix<'_>) {
    let (lower, upper) = (h.lower_row(), h.upper_row());
    for ii in lower..=upper {
        for jj in ii + 1..=upper {
            h[(ii, jj)] = h[(jj, ii)];
        }
    }
}
