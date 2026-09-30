//! normale surface branches preserve native evaluation order.
use super::*;
impl ContapSurfProps {
    pub(super) fn normale_plane(
        s: &Adaptor3dSurfaceHandle,
        u: f64,
        v: f64,
    ) -> Result<(Point3, Vector3), Failure> {
        let pl = hst::plane(s)?;
        let mut norm = vector_of_dir(pl.axis().direction());
        let p = elslib::value(u, v, &pl);
        if !pl.direct()? {
            norm.reverse();
        }
        Ok((p, norm))
    }
    pub(super) fn normale_sphere(
        s: &Adaptor3dSurfaceHandle,
        u: f64,
        v: f64,
    ) -> Result<(Point3, Vector3), Failure> {
        let sp = hst::sphere(s)?;
        let p = elslib::value(u, v, &sp);
        Ok((
            p,
            sphere_normal(sp.location(), p, sp.radius(), sp.direct()?),
        ))
    }
    pub(super) fn normale_cylinder(
        s: &Adaptor3dSurfaceHandle,
        u: f64,
        v: f64,
    ) -> Result<(Point3, Vector3), Failure> {
        let cy = hst::cylinder(s)?;
        let p = elslib::value(u, v, &cy);
        let (x, y) = (
            vector_of_dir(cy.x_axis().direction()),
            vector_of_dir(cy.y_axis().direction()),
        );
        Ok((p, cylinder_normal(x, y, u, cy.direct()?)))
    }
    pub(super) fn normale_cone(
        s: &Adaptor3dSurfaceHandle,
        u: f64,
        v: f64,
    ) -> Result<(Point3, Vector3), Failure> {
        let co = hst::cone(s)?;
        let p = elslib::value(u, v, &co);
        let (sina, rad) = (co.semi_angle().sin(), co.ref_radius());
        if (v * sina + rad).abs() <= 1e-12 {
            // At the apex.
            return Ok((p, Vector3::new(0., 0., 0.)));
        }
        Ok((p, cone_normal(&co, u, v)?))
    }
    pub(super) fn normale_generic(
        s: &Adaptor3dSurfaceHandle,
        u: f64,
        v: f64,
    ) -> Result<(Point3, Vector3), Failure> {
        let d1 = hst::d1(s, u, v)?;
        Ok((d1.point, d1.d1u.crossed(d1.d1v)))
    }
}
