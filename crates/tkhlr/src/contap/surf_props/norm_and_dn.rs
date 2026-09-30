//! norm_and_dn surface branches preserve native evaluation order.
use super::*;
impl ContapSurfProps {
    pub(super) fn norm_and_dn_plane(
        s: &Adaptor3dSurfaceHandle,
        u: f64,
        v: f64,
    ) -> Result<ContapNormAndDn, Failure> {
        let zero = Vector3::default();
        let pl = hst::plane(s)?;
        let p = elslib::value(u, v, &pl);
        let mut norm = vector_of_dir(pl.axis().direction());
        if !pl.direct()? {
            norm.reverse();
        }
        Ok((p, norm, zero, zero))
    }
    pub(super) fn norm_and_dn_sphere(
        s: &Adaptor3dSurfaceHandle,
        u: f64,
        v: f64,
    ) -> Result<ContapNormAndDn, Failure> {
        let sp = hst::sphere(s)?;
        let (p, dnu, dnv) = elslib::d1(u, v, &sp);
        let rad = if sp.direct()? {
            sp.radius()
        } else {
            -sp.radius()
        };
        let norm = Vector3::between(sp.location(), p);
        Ok((p, norm / rad, dnu / rad, dnv / rad))
    }
    pub(super) fn norm_and_dn_cylinder(
        s: &Adaptor3dSurfaceHandle,
        u: f64,
        v: f64,
    ) -> Result<ContapNormAndDn, Failure> {
        let zero = Vector3::default();
        let cy = hst::cylinder(s)?;
        let p = elslib::value(u, v, &cy);
        let (x, y) = (
            vector_of_dir(cy.x_axis().direction()),
            vector_of_dir(cy.y_axis().direction()),
        );
        let mut norm = Vector3::default();
        norm.set_linear_form(u.cos(), x, u.sin(), y);
        let mut dnu = Vector3::default();
        dnu.set_linear_form(-u.sin(), x, u.cos(), y);
        if !cy.direct()? {
            norm.reverse();
            dnu.reverse();
        }
        Ok((p, norm, dnu, zero))
    }
    pub(super) fn norm_and_dn_generic(
        s: &Adaptor3dSurfaceHandle,
        u: f64,
        v: f64,
    ) -> Result<ContapNormAndDn, Failure> {
        let d2 = hst::d2(s, u, v)?;
        let norm = d2.d1u.crossed(d2.d1v);
        let dnu = d2.d2u.crossed(d2.d1v) + d2.d1u.crossed(d2.d2uv);
        let dnv = d2.d2uv.crossed(d2.d1v) + d2.d1u.crossed(d2.d2v);
        Ok((d2.point, norm, dnu, dnv))
    }
}
