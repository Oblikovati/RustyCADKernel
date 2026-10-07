//! The derivatives of the unit normal of a surface projected into the plane of a blend
//! section, as BlendFunc_ConstRad.cxx and BlendFunc_EvolRad.cxx compute them.
//!
//! With `n` the surface normal (`D1U ^ D1V`) and `np` the section plane normal, the
//! projected unit normal is `(np.n * np - n) / |np ^ n|`. Each function below is one block
//! of the native `ComputeValues`, with its operation order.
use rustycad_tkmath::gp::Vector3;

/// `Eps` of the native sources: below it a normal is degenerate.
pub(crate) const EPS: f64 = 1.0e-15;

/// The `1 / |np ^ n|` of the native code, or 1 for a singular surface ("unsatisfactory,
/// but it is not necessary to crash").
pub(crate) fn inverse_norm(norm: f64) -> f64 {
    if norm > EPS { 1. / norm } else { 1. }
}

/// The quantities of one surface shared by every derivative block: the plane normal, the
/// surface normal, `np ^ n`, `1 / |np ^ n|` and `np.n`.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Projection {
    pub nplan: Vector3,
    pub nsurf: Vector3,
    pub ncrossns: Vector3,
    pub invnorm: f64,
    pub ndotns: f64,
}

impl Projection {
    /// The projection of `nsurf` along `nplan`.
    pub(crate) fn new(nplan: Vector3, nsurf: Vector3) -> Self {
        let ncrossns = nplan.crossed(nsurf);
        Self {
            nplan,
            nsurf,
            ncrossns,
            invnorm: inverse_norm(ncrossns.magnitude()),
            ndotns: nplan.dot(nsurf),
        }
    }

    /// `temp.SetLinearForm(ndotns, nplan, -1., nsurf); temp.Multiply(invnorm)`: the
    /// projected unit normal.
    pub(crate) fn unit_normal(&self) -> Vector3 {
        let mut temp = Vector3::default();
        temp.set_linear_form(self.ndotns, self.nplan, -1., self.nsurf);
        temp *= self.invnorm;
        temp
    }

    /// `invnorm^3`, as `invnorm * invnorm * invnorm`.
    pub(crate) fn cube(&self) -> f64 {
        self.invnorm * self.invnorm * self.invnorm
    }

    /// The derivative of the projected normal along a surface parameter whose normal
    /// derivative is `dns` (the `dndu1`/`dndv1` blocks).
    pub(crate) fn first_derivative(&self, dns: Vector3) -> Vector3 {
        let grosterme = -self.ncrossns.dot(self.nplan.crossed(dns)) * self.cube();
        let mut out = Vector3::default();
        out.set_linear_form_three(
            [
                grosterme * self.ndotns + self.invnorm * self.nplan.dot(dns),
                -grosterme,
                -self.invnorm,
            ],
            [self.nplan, self.nsurf, dns],
        );
        out
    }

    /// The derivative of the projected normal along the guide (`dn1w`), for the plane
    /// normal derivative `dnplan`.
    pub(crate) fn guide_derivative(&self, dnplan: Vector3) -> Vector3 {
        let grosterme = self.ncrossns.dot(dnplan.crossed(self.nsurf)) * self.invnorm * self.invnorm;
        let mut out = Vector3::default();
        out.set_linear_form_three(
            [
                (dnplan.dot(self.nsurf) - grosterme * self.ndotns) * self.invnorm,
                self.ndotns * self.invnorm,
                grosterme * self.invnorm,
            ],
            [self.nplan, dnplan, self.nsurf],
        );
        out
    }

    /// The second derivative along one surface parameter, twice (`d2ndu1`, `d2ndv1`),
    /// from the first and second normal derivatives `dns`, `d2ns`.
    pub(crate) fn second_derivative(&self, dns: Vector3, d2ns: Vector3) -> Vector3 {
        let (carre, cube) = (
            self.invnorm * self.invnorm,
            self.invnorm * self.invnorm * self.invnorm,
        );
        let dprim = self.ncrossns.dot(self.nplan.crossed(dns));
        let smallterm = -2. * dprim * cube;
        let dsecn = self.ncrossns.dot(self.nplan.crossed(d2ns))
            + self.nplan.crossed(dns).square_magnitude();
        let grosterme = (3. * dprim * dprim * carre - dsecn) * cube;
        let mut temp = Vector3::default();
        temp.set_linear_form(grosterme * self.ndotns, self.nplan, -grosterme, self.nsurf);
        let (p1, p2) = (self.nplan.dot(dns), self.nplan.dot(d2ns));
        let mut out = Vector3::default();
        out.set_linear_form_three(
            [
                self.invnorm * p2 + smallterm * p1,
                -smallterm,
                -self.invnorm,
            ],
            [self.nplan, dns, d2ns],
        );
        out += temp;
        out
    }

    /// The cross second derivative along both surface parameters (`d2nduv1`).
    pub(crate) fn cross_derivative(
        &self,
        dnsu: Vector3,
        dnsv: Vector3,
        d2nsuv: Vector3,
    ) -> Vector3 {
        let (carre, cube) = (
            self.invnorm * self.invnorm,
            self.invnorm * self.invnorm * self.invnorm,
        );
        let mut uterm = self.ncrossns.dot(self.nplan.crossed(dnsu));
        let mut vterm = self.ncrossns.dot(self.nplan.crossed(dnsv));
        let dsecn = self.nplan.crossed(dnsv).dot(self.nplan.crossed(dnsu))
            + self.ncrossns.dot(self.nplan.crossed(d2nsuv));
        let grosterme = (3. * uterm * vterm * carre - dsecn) * cube;
        // and only now
        uterm *= -cube;
        vterm *= -cube;
        let (p1, p2) = (self.nplan.dot(dnsu), self.nplan.dot(dnsv));
        let mut temp = Vector3::default();
        temp.set_linear_form_three(
            [grosterme * self.ndotns, -grosterme, -self.invnorm],
            [self.nplan, self.nsurf, d2nsuv],
        );
        let mut out = Vector3::default();
        out.set_linear_form_three(
            [
                self.invnorm * self.nplan.dot(d2nsuv) + uterm * p2 + vterm * p1,
                -uterm,
                -vterm,
            ],
            [self.nplan, dnsv, dnsu],
        );
        out += temp;
        out
    }

    /// The cross second derivative along a surface parameter and the guide (`d2ndtu1`).
    /// `swapped` sums the first coefficient as the native `d2ndtv1` block does
    /// (`uterm * p1` before `smallterm * p2`), the other blocks sum `smallterm * p2` first.
    pub(crate) fn guide_cross_derivative(
        &self,
        dnplan: Vector3,
        dns: Vector3,
        swapped: bool,
    ) -> Vector3 {
        let (carre, cube) = (
            self.invnorm * self.invnorm,
            self.invnorm * self.invnorm * self.invnorm,
        );
        let tterm = self.ncrossns.dot(dnplan.crossed(self.nsurf));
        let smallterm = -tterm * cube;
        let mut uterm = self.ncrossns.dot(self.nplan.crossed(dns));
        let dsecn = self.nplan.crossed(dns).dot(dnplan.crossed(self.nsurf))
            + self.ncrossns.dot(dnplan.crossed(dns));
        let grosterme = (3. * uterm * tterm * carre - dsecn) * cube;
        uterm *= -cube;
        let (p1, p2, p12) = (dnplan.dot(self.nsurf), self.nplan.dot(dns), dnplan.dot(dns));
        let first = if swapped {
            self.invnorm * p12 + uterm * p1 + smallterm * p2 + grosterme * self.ndotns
        } else {
            self.invnorm * p12 + smallterm * p2 + uterm * p1 + grosterme * self.ndotns
        };
        let mut out = Vector3::default();
        out.set_linear_form_three(
            [first, self.invnorm * p2 + uterm * self.ndotns, -smallterm],
            [self.nplan, dnplan, dns],
        );
        out -= grosterme * self.nsurf;
        out
    }

    /// The second derivative along the guide (`d2n1w`), for the plane normal derivatives.
    pub(crate) fn guide_second_derivative(&self, dnplan: Vector3, d2nplan: Vector3) -> Vector3 {
        let (carre, cube) = (
            self.invnorm * self.invnorm,
            self.invnorm * self.invnorm * self.invnorm,
        );
        let dprim = self.ncrossns.dot(dnplan.crossed(self.nsurf));
        let smallterm = -2. * dprim * cube;
        let dsecn = dnplan.crossed(self.nsurf).square_magnitude()
            + self.ncrossns.dot(d2nplan.crossed(self.nsurf));
        let grosterme = (3. * dprim * dprim * carre - dsecn) * cube;
        let (p1, p2) = (dnplan.dot(self.nsurf), d2nplan.dot(self.nsurf));
        let mut temp = Vector3::default();
        temp.set_linear_form(grosterme * self.ndotns, self.nplan, -grosterme, self.nsurf);
        let mut out = Vector3::default();
        out.set_linear_form_three(
            [
                smallterm * p1 + self.invnorm * p2,
                smallterm * self.ndotns + 2. * self.invnorm * p1,
                self.ndotns * self.invnorm,
            ],
            [self.nplan, dnplan, d2nplan],
        );
        out += temp;
        out
    }
}

/// The second derivatives of the surface normal `D1U ^ D1V` (`d2ns1u1`, `d2ns1v1`,
/// `d2ns1uv1`) from the surface derivatives up to order 3.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SurfaceJet {
    pub d1u: Vector3,
    pub d1v: Vector3,
    pub d2u: Vector3,
    pub d2v: Vector3,
    pub d2uv: Vector3,
    pub d3u: Vector3,
    pub d3v: Vector3,
    pub d3uuv: Vector3,
    pub d3uvv: Vector3,
}

impl SurfaceJet {
    /// `dns1u1 = D2U ^ D1V + D1U ^ D2UV`.
    pub(crate) fn dns_u(&self) -> Vector3 {
        self.d2u.crossed(self.d1v) + self.d1u.crossed(self.d2uv)
    }

    /// `dns1v1 = D2UV ^ D1V + D1U ^ D2V`.
    pub(crate) fn dns_v(&self) -> Vector3 {
        self.d2uv.crossed(self.d1v) + self.d1u.crossed(self.d2v)
    }

    /// `d2ns1u1 = D3U ^ D1V + 2 D2U ^ D2UV + D1U ^ D3UUV`.
    pub(crate) fn d2ns_u(&self) -> Vector3 {
        let mut out = Vector3::default();
        out.set_linear_form_three(
            [1., 2., 1.],
            [
                self.d3u.crossed(self.d1v),
                self.d2u.crossed(self.d2uv),
                self.d1u.crossed(self.d3uuv),
            ],
        );
        out
    }

    /// `d2ns1v1 = D1U ^ D3V + 2 D2UV ^ D2V + D3UVV ^ D1V`.
    pub(crate) fn d2ns_v(&self) -> Vector3 {
        let mut out = Vector3::default();
        out.set_linear_form_three(
            [1., 2., 1.],
            [
                self.d1u.crossed(self.d3v),
                self.d2uv.crossed(self.d2v),
                self.d3uvv.crossed(self.d1v),
            ],
        );
        out
    }

    /// `d2ns1uv1 = D3UUV ^ D1V + D2U ^ D2V + D1U ^ D3UVV`.
    pub(crate) fn d2ns_uv(&self) -> Vector3 {
        self.d3uuv.crossed(self.d1v) + self.d2u.crossed(self.d2v) + self.d1u.crossed(self.d3uvv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projected_normal_of_a_plane_normal_to_the_section() {
        let projection = Projection::new(Vector3::new(1., 0., 0.), Vector3::new(0., 0., 2.));
        assert_eq!(projection.invnorm, 0.5);
        assert_eq!(projection.unit_normal(), Vector3::new(0., 0., -1.));
    }

    #[test]
    fn singular_normal_keeps_a_unit_inverse_norm() {
        assert_eq!(inverse_norm(0.0), 1.0);
        assert_eq!(inverse_norm(4.0), 0.25);
    }

    #[test]
    fn constant_normal_has_zero_derivatives() {
        let projection = Projection::new(Vector3::new(1., 0., 0.), Vector3::new(0., 0., 1.));
        let zero = Vector3::default();
        assert_eq!(projection.first_derivative(zero).magnitude(), 0.0);
        assert_eq!(projection.second_derivative(zero, zero).magnitude(), 0.0);
        assert_eq!(
            projection.cross_derivative(zero, zero, zero).magnitude(),
            0.0
        );
        assert_eq!(
            projection
                .guide_cross_derivative(zero, zero, true)
                .magnitude(),
            0.0
        );
        assert_eq!(
            projection.guide_second_derivative(zero, zero).magnitude(),
            0.0
        );
        assert_eq!(projection.guide_derivative(zero).magnitude(), 0.0);
    }
}
