//! The ChFiKPart helpers without the data structure (`fl_kpart_fcts`,
//! `fl_corner_spine`); the twin of tools/oracle/tkfillet_probe_chfikpart.cxx.
use super::blend_func::{geom_guide, geom_surface, surface};
use super::builder0::curve2d;
use super::common::{Reader, attempt, end, pnt, real, xyz};
use rustycad_tkfillet::chfikpart;
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};

/// `fl_kpart_fcts`: see tools/oracle/tkfillet_probe_chfikpart.cxx.
pub fn fcts_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let (u, f, l, eps) = (r.real(), r.real(), r.real(), r.real());
    let (uv1, uv2, pd, pf) = (r.pnt2d(), r.pnt2d(), r.real(), r.real());
    let (c, gf, gl) = (geom_guide(r), r.real(), r.real());
    let s = geom_surface(r);
    real(chfikpart::in_period(u, f, l, eps));
    attempt(|| {
        curve2d(&chfikpart::pcurve(uv1, uv2, pd, pf)?.cast());
        Ok(())
    });
    attempt(|| {
        let (cg, sg) = (
            GeomAdaptorCurve::with_range(&c, gf, gl)?,
            GeomAdaptorSurface::with_surface(&s)?,
        );
        curve2d(&chfikpart::proj_pc(&cg, &sg)?);
        Ok(())
    });
    end();
}

/// `fl_corner_spine`: see tools/oracle/tkfillet_probe_chfikpart.cxx.
pub fn corner_spine_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let (s1, s2) = (surface(r), surface(r));
    let (p1s1, p1s2, p2s2, rad) = (r.pnt2d(), r.pnt2d(), r.pnt2d(), r.real());
    attempt(|| {
        let (cyl, circ, first, last) =
            chfikpart::corner_spine((&s1, &s2), (p1s1, p1s2, p2s2), rad)?;
        let pos = cyl.position();
        pnt(pos.location());
        xyz(pos.direction().xyz());
        xyz(pos.x_direction().xyz());
        xyz(pos.y_direction().xyz());
        real(cyl.radius());
        let cpos = circ.position();
        pnt(cpos.location());
        xyz(cpos.direction().xyz());
        xyz(cpos.x_direction().xyz());
        real(circ.radius());
        real(first);
        real(last);
        Ok(())
    });
    end();
}
