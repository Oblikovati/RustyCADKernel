//! The second derivatives of the `BlendFunc_ConstRad` equations (the "positioning of
//! order 2" block of `ComputeValues`): `D2EDX2` and, by parameter, `D2EDXDT`, `D2EDT2`.
use super::super::linalg::{mset, vset};
use super::super::normal_terms::{Projection, SurfaceJet};
use super::super::tensor::Tensor;
use super::compute::ConstRadScratch;
use super::equations::{middle_from_guide, set_tail};
use super::{ConstRad, SurfacePoint};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Vector3;

/// Writes `value` to `D2EDX2(2..4, j, k)` and, when `j != k`, to `D2EDX2(2..4, k, j)`.
fn set_tensor(tensor: &mut Tensor, j: i32, k: i32, value: Vector3) -> Result<(), Failure> {
    for (row, coordinate) in [(2, value.x()), (3, value.y()), (4, value.z())] {
        *tensor.change_value(row, j, k)? = coordinate;
        *tensor.change_value(row, k, j)? = coordinate;
    }
    Ok(())
}

/// The surface jet with the third derivatives of the thread scratch.
fn jet_with(point: &SurfacePoint, d3: [Vector3; 4]) -> SurfaceJet {
    let mut jet = point.jet;
    [jet.d3u, jet.d3v, jet.d3uuv, jet.d3uvv] = d3;
    jet
}

/// The `-ray2 * x - y` combination of the second surface or the `ray1 * x + y` one of
/// the first.
fn combine(ray: f64, second: bool, normal_term: Vector3, surface_term: Vector3) -> Vector3 {
    let mut resul = Vector3::default();
    if second {
        resul.set_linear_form(-ray, normal_term, -1., surface_term);
    } else {
        resul.set_scaled_sum(ray, normal_term, surface_term);
    }
    resul
}

impl ConstRad {
    /// The positioning of order 2.
    pub(super) fn equations_order2(
        &mut self,
        scratch: &ConstRadScratch,
        by_param: bool,
    ) -> Result<(), Failure> {
        let nplan = self.plane.nplan;
        let tensor = &mut self.eq.d2edx2;
        tensor.init(0.);
        for (j, k, d) in [
            (1, 1, self.s1.jet.d2u),
            (2, 1, self.s1.jet.d2uv),
            (2, 2, self.s1.jet.d2v),
        ] {
            *tensor.change_value(1, j, k)? = nplan.dot(d) / 2.;
            *tensor.change_value(1, k, j)? = nplan.dot(d) / 2.;
        }
        for (j, k, d) in [
            (3, 3, self.s2.jet.d2u),
            (4, 3, self.s2.jet.d2uv),
            (4, 4, self.s2.jet.d2v),
        ] {
            *tensor.change_value(1, j, k)? = nplan.dot(d) / 2.;
            *tensor.change_value(1, k, j)? = nplan.dot(d) / 2.;
        }
        let (p1, p2) = self.projections();
        self.surface_second(&p1, scratch.d3[0], false)?;
        self.surface_second(&p2, scratch.d3[1], true)?;
        if by_param {
            self.parameter_second(scratch, (p1, p2))?;
        }
        Ok(())
    }

    /// `d2ndu`, `d2nduv`, `d2ndv` of one surface and their `D2EDX2` entries.
    fn surface_second(
        &mut self,
        projection: &Projection,
        d3: [Vector3; 4],
        second: bool,
    ) -> Result<(), Failure> {
        let (point, ray, base) = if second {
            (&mut self.s2, self.ray2, 3)
        } else {
            (&mut self.s1, self.ray1, 1)
        };
        let jet = jet_with(point, d3);
        point.normal.d2ndu = projection.second_derivative(point.dnsu, jet.d2ns_u());
        point.normal.d2nduv = projection.cross_derivative(point.dnsu, point.dnsv, jet.d2ns_uv());
        point.normal.d2ndv = projection.second_derivative(point.dnsv, jet.d2ns_v());
        let normal = point.normal;
        let tensor = &mut self.eq.d2edx2;
        set_tensor(
            tensor,
            base,
            base,
            combine(ray, second, normal.d2ndu, jet.d2u),
        )?;
        set_tensor(
            tensor,
            base + 1,
            base,
            combine(ray, second, normal.d2nduv, jet.d2uv),
        )?;
        set_tensor(
            tensor,
            base + 1,
            base + 1,
            combine(ray, second, normal.d2ndv, jet.d2v),
        )
    }

    /// The double derivation in `t` and `X` and in `t`.
    fn parameter_second(
        &mut self,
        scratch: &ConstRadScratch,
        (p1, p2): (Projection, Projection),
    ) -> Result<(), Failure> {
        let (dnplan, d2nplan) = (self.plane.dnplan, self.plane.d2nplan);
        let jets = [self.s1.jet, self.s2.jet];
        for (col, d) in (1..).zip([jets[0].d1u, jets[0].d1v, jets[1].d1u, jets[1].d1v]) {
            mset(&mut self.eq.d2edxdt, 1, col, dnplan.dot(d) / 2.)?;
        }
        self.s1.normal.d2ndtu = p1.guide_cross_derivative(dnplan, self.s1.dnsu, false);
        self.s1.normal.d2ndtv = p1.guide_cross_derivative(dnplan, self.s1.dnsv, true);
        self.s2.normal.d2ndtu = p2.guide_cross_derivative(dnplan, self.s2.dnsu, false);
        self.s2.normal.d2ndtv = p2.guide_cross_derivative(dnplan, self.s2.dnsv, false);
        let (n1, n2) = (self.s1.normal, self.s2.normal);
        let columns = [
            self.ray1 * n1.d2ndtu,
            self.ray1 * n1.d2ndtv,
            -self.ray2 * n2.d2ndtu,
            -self.ray2 * n2.d2ndtv,
        ];
        for (col, resul) in (1..).zip(columns) {
            super::equations::set_column(&mut self.eq.d2edxdt, col, resul)?;
        }
        self.s1.normal.d2nw = p1.guide_second_derivative(dnplan, d2nplan);
        self.s2.normal.d2nw = p2.guide_second_derivative(dnplan, d2nplan);
        let guide = &scratch.guide;
        let temp = middle_from_guide(self.s1.pts, self.s2.pts, guide.ptgui);
        let first =
            d2nplan.dot(temp) - 2. * dnplan.dot(guide.d1gui) - self.plane.nplan.dot(guide.d2gui);
        vset(&mut self.eq.d2edt2, 1, first)?;
        let (d2n1w, d2n2w) = (self.s1.normal.d2nw, self.s2.normal.d2nw);
        let (ray1, ray2) = (self.ray1, self.ray2);
        set_tail(
            &mut self.eq.d2edt2,
            Vector3::new(
                ray1 * d2n1w.x() - ray2 * d2n2w.x(),
                ray1 * d2n1w.y() - ray2 * d2n2w.y(),
                ray1 * d2n1w.z() - ray2 * d2n2w.z(),
            ),
        )
    }
}
