//! The equations of the fillet functions and their first derivatives (the "positioning
//! of order 0" and "of order 1" blocks of `ComputeValues`).
use super::super::guide::GuideScratch;
use super::super::linalg::{mset, point_from, vset};
use super::super::normal_terms::Projection;
use super::{Radii, TwoSurfaceCore};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point3, Vector3, Xyz};
use rustycad_tkmath::math::{Matrix, Vector};

/// Writes the components of `value` to rows 2..4 of the column `col` of `matrix`.
pub(super) fn set_column(matrix: &mut Matrix<'_>, col: i32, value: Vector3) -> Result<(), Failure> {
    mset(matrix, 2, col, value.x())?;
    mset(matrix, 3, col, value.y())?;
    mset(matrix, 4, col, value.z())
}

/// Writes the components of `value` to the entries 2..4 of `vector`.
pub(super) fn set_tail(vector: &mut Vector<'_>, value: Vector3) -> Result<(), Failure> {
    vset(vector, 2, value.x())?;
    vset(vector, 3, value.y())?;
    vset(vector, 4, value.z())
}

/// `(pts1.XYZ() + pts2.XYZ()) / 2 - ptgui.XYZ()`: the middle of the section seen from
/// the guide point.
pub(super) fn middle_from_guide(pts1: Point3, pts2: Point3, ptgui: Point3) -> Vector3 {
    let middle: Xyz = (pts1.xyz() + pts2.xyz()) / 2.;
    Vector3::from(middle - ptgui.xyz())
}

/// `pts1.XYZ() + ray1 * ns1.XYZ()`: the center of the section circle.
pub(super) fn center(pts1: Point3, ray1: f64, ns1: Vector3) -> Point3 {
    point_from(pts1.xyz() + ray1 * *ns1.xyz())
}

impl TwoSurfaceCore {
    pub(super) fn projections(&self) -> (Projection, Projection) {
        let nplan = self.plane.nplan;
        (
            Projection::new(nplan, self.s1.nsurf),
            Projection::new(nplan, self.s2.nsurf),
        )
    }

    /// The positioning of order 0: `E`.
    pub(super) fn equations_order0(
        &mut self,
        guide: &GuideScratch,
        radii: Radii,
    ) -> Result<(), Failure> {
        let (p1, p2) = self.projections();
        let (pts1, pts2, nplan) = (self.s1.pts, self.s2.pts, self.plane.nplan);
        let the_d = -nplan.xyz().dot(guide.ptgui.xyz());
        let e1 = (nplan.x() * (pts1.x() + pts2.x())
            + nplan.y() * (pts1.y() + pts2.y())
            + nplan.z() * (pts1.z() + pts2.z()))
            / 2.
            + the_d;
        let (ray1, ray2) = (radii.ray1(), radii.ray2());
        let mut resul = Vector3::default();
        match radii {
            Radii::Constant { .. } => {
                resul.set_scaled_sum(ray1, p1.unit_normal(), Vector3::between(pts2, pts1));
                resul -= ray2 * p2.unit_normal();
            }
            Radii::Evolutive { .. } => {
                resul.set_linear_form_offset(
                    ray1,
                    p1.unit_normal(),
                    -ray2,
                    p2.unit_normal(),
                    Vector3::between(pts2, pts1),
                );
            }
        }
        vset(&mut self.eq.e, 1, e1)?;
        set_tail(&mut self.eq.e, resul)
    }

    /// The positioning of order 1: `DEDX` and, by parameter, `DEDT`.
    pub(super) fn equations_order1(
        &mut self,
        guide: &GuideScratch,
        by_param: bool,
        radii: Radii,
    ) -> Result<(), Failure> {
        let (p1, p2) = self.projections();
        let nplan = self.plane.nplan;
        let jets = [self.s1.jet, self.s2.jet];
        for (col, d) in (1..).zip([jets[0].d1u, jets[0].d1v, jets[1].d1u, jets[1].d1v]) {
            mset(&mut self.eq.dedx, 1, col, nplan.dot(d) / 2.)?;
        }
        self.s1.normal.dndu = p1.first_derivative(self.s1.dnsu);
        self.s1.normal.dndv = p1.first_derivative(self.s1.dnsv);
        self.s2.normal.dndu = p2.first_derivative(self.s2.dnsu);
        self.s2.normal.dndv = p2.first_derivative(self.s2.dnsv);
        let (ray1, ray2) = (radii.ray1(), radii.ray2());
        let mut resul = Vector3::default();
        resul.set_scaled_sum(ray1, self.s1.normal.dndu, jets[0].d1u);
        set_column(&mut self.eq.dedx, 1, resul)?;
        resul.set_scaled_sum(ray1, self.s1.normal.dndv, jets[0].d1v);
        set_column(&mut self.eq.dedx, 2, resul)?;
        resul.set_linear_form(-ray2, self.s2.normal.dndu, -1., jets[1].d1u);
        set_column(&mut self.eq.dedx, 3, resul)?;
        resul.set_linear_form(-ray2, self.s2.normal.dndv, -1., jets[1].d1v);
        set_column(&mut self.eq.dedx, 4, resul)?;
        if by_param {
            self.guide_derivatives(guide, (p1, p2), radii)?;
        }
        Ok(())
    }

    /// `dn1w`, `dn2w` and `DEDT`.
    fn guide_derivatives(
        &mut self,
        guide: &GuideScratch,
        (p1, p2): (Projection, Projection),
        radii: Radii,
    ) -> Result<(), Failure> {
        let dnplan = self.plane.dnplan;
        let temp = middle_from_guide(self.s1.pts, self.s2.pts, guide.ptgui);
        self.s1.normal.dnw = p1.guide_derivative(dnplan);
        self.s2.normal.dnw = p2.guide_derivative(dnplan);
        let (dn1w, dn2w) = (self.s1.normal.dnw, self.s2.normal.dnw);
        let (ray1, ray2) = (radii.ray1(), radii.ray2());
        vset(
            &mut self.eq.dedt,
            1,
            dnplan.dot(temp) - 1. / guide.invnormtg,
        )?;
        let tail = match radii {
            Radii::Constant { .. } => Vector3::new(
                ray1 * dn1w.x() - ray2 * dn2w.x(),
                ray1 * dn1w.y() - ray2 * dn2w.y(),
                ray1 * dn1w.z() - ray2 * dn2w.z(),
            ),
            Radii::Evolutive { sg1, sg2, dray, .. } => {
                let mut temp = Vector3::default();
                temp.set_linear_form(ray2, dn2w, sg2 * dray, p2.unit_normal());
                let mut resul = Vector3::default();
                resul
                    .set_linear_form_three([ray1, sg1 * dray, -1.], [dn1w, p1.unit_normal(), temp]);
                resul
            }
        };
        set_tail(&mut self.eq.dedt, tail)
    }
}
