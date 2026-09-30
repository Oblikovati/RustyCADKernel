//! deriv_and_norm surface branches preserve native evaluation order.
use super::*;
impl ContapSurfProps {
    pub(super) fn deriv_and_norm_plane(
        s: &Adaptor3dSurfaceHandle,
        u: f64,
        v: f64,
    ) -> Result<ContapDerivAndNorm, Failure> {
        let pl = hst::plane(s)?;
        let mut norm = vector_of_dir(pl.axis().direction());
        let (p, d1u, d1v) = elslib::d1(u, v, &pl);
        if !pl.direct()? {
            norm.reverse();
        }
        Ok((p, d1u, d1v, norm))
    }
    pub(super) fn deriv_and_norm_sphere(
        s: &Adaptor3dSurfaceHandle,
        u: f64,
        v: f64,
    ) -> Result<ContapDerivAndNorm, Failure> {
        let sp = hst::sphere(s)?;
        let (p, d1u, d1v) = elslib::d1(u, v, &sp);
        Ok((
            p,
            d1u,
            d1v,
            sphere_normal(sp.location(), p, sp.radius(), sp.direct()?),
        ))
    }
    pub(super) fn deriv_and_norm_cylinder(
        s: &Adaptor3dSurfaceHandle,
        u: f64,
        v: f64,
    ) -> Result<ContapDerivAndNorm, Failure> {
        let cy = hst::cylinder(s)?;
        let (p, d1u, d1v) = elslib::d1(u, v, &cy);
        let (x, y) = (
            vector_of_dir(cy.x_axis().direction()),
            vector_of_dir(cy.y_axis().direction()),
        );
        Ok((p, d1u, d1v, cylinder_normal(x, y, u, cy.direct()?)))
    }
    pub(super) fn deriv_and_norm_cone(
        s: &Adaptor3dSurfaceHandle,
        u: f64,
        v: f64,
    ) -> Result<ContapDerivAndNorm, Failure> {
        let co = hst::cone(s)?;
        let (p, d1u, d1v) = elslib::d1(u, v, &co);
        let vcalc = cone_v_off_apex(s, &co, v)?;
        Ok((p, d1u, d1v, cone_normal(&co, u, vcalc)?))
    }
    pub(super) fn deriv_and_norm_generic(
        s: &Adaptor3dSurfaceHandle,
        u: f64,
        v: f64,
    ) -> Result<ContapDerivAndNorm, Failure> {
        let d1 = hst::d1(s, u, v)?;
        Ok((d1.point, d1.d1u, d1.d1v, d1.d1u.crossed(d1.d1v)))
    }
}
