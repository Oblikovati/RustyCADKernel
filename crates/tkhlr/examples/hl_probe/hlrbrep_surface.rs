//! HLRBRep_Surface, HLRBRep_SurfaceTool, HLRBRep_SLProps and HLRBRep_InterCSurf, the
//! Rust twin of tools/oracle/tkhlr_probe_hlrbrep_surface.cxx.
use super::common::{Reader, attempt, dir, end, flag, int, pnt, real, show};
use super::hlrbrep_faces::{face, ics_point};
use super::hlrbrep_fixtures::{edge, projector};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::int_curve_surface::SamplePolyhedron;
use rustycad_tkhlr::hlr_algo::{Projector, init_min_max};
use rustycad_tkhlr::hlr_brep::{
    Curve, InterCSurf, SLProps, Surface, SurfaceTool, ThePolygonOfInterCSurf,
    ThePolyhedronOfInterCSurf,
};
use std::sync::Arc;

fn samples(s: &Surface) -> Result<(), Failure> {
    int(SurfaceTool::nb_samples_u(s)?);
    int(SurfaceTool::nb_samples_v(s)?);
    Ok(())
}

fn samples_in(s: &Surface) -> Result<(), Failure> {
    let (u1, u2) = (
        SurfaceTool::first_u_parameter(s)?,
        SurfaceTool::last_u_parameter(s)?,
    );
    let (v1, v2) = (
        SurfaceTool::first_v_parameter(s)?,
        SurfaceTool::last_v_parameter(s)?,
    );
    int(SurfaceTool::nb_samples_u_in(s, u1, u2)?);
    int(SurfaceTool::nb_samples_v_in(s, v1, v2)?);
    int(SurfaceTool::nb_samples_u_in(s, u1, (u1 + u2) / 3.)?);
    int(SurfaceTool::nb_samples_v_in(s, v1, (v1 + v2) / 3.)?);
    Ok(())
}

fn queries(s: &Surface, tf: f64, tr: f64) {
    int(s.get_type() as i32);
    show(s.is_side(tf, tr), flag);
    attempt(|| {
        let p = s.plane()?;
        pnt(p.location());
        dir(p.axis().direction());
        Ok(())
    });
    attempt(|| samples(s));
    attempt(|| samples_in(s));
}

fn slprops(s: &Surface, u: f64, v: f64) {
    attempt(|| {
        let mut props = SLProps::new(s, 2, 1e-9)?;
        props.set_parameters(u, v)?;
        pnt(props.value());
        let n = props.is_normal_defined()?;
        flag(n);
        if n {
            dir(props.normal()?);
        }
        let c = props.is_curvature_defined()?;
        flag(c);
        if c {
            real(props.max_curvature()?);
            real(props.min_curvature()?);
        }
        Ok(())
    });
}

fn print_inter(i: &InterCSurf<'_>) -> Result<(), Failure> {
    flag(i.is_done());
    if !i.is_done() {
        return Ok(());
    }
    int(i.nb_points()?);
    for k in 1..=i.nb_points()? {
        ics_point(&i.point(k)?);
    }
    int(i.nb_segments()?);
    Ok(())
}

fn polyhedron_perform<'a>(
    i: &mut InterCSurf<'a>,
    l: &rustycad_tkmath::gp::Line3,
    s: &'a Surface,
    (pmin, pmax): (f64, f64),
) -> Result<(), Failure> {
    let (u1, u2) = (
        SurfaceTool::first_u_parameter(s)?,
        SurfaceTool::last_u_parameter(s)?,
    );
    let (v1, v2) = (
        SurfaceTool::first_v_parameter(s)?,
        SurfaceTool::last_v_parameter(s)?,
    );
    let nu = SurfaceTool::nb_samples_u_in(s, u1, u2)?;
    let nv = SurfaceTool::nb_samples_v_in(s, v1, v2)?;
    let poly: ThePolyhedronOfInterCSurf =
        SamplePolyhedron::<SurfaceTool<'_>>::uniform(&s, nu, nv, [u1, v1, u2, v2])?;
    let polygon = ThePolygonOfInterCSurf::with_range(l, pmin, pmax, 3)?;
    real(poly.deflection_over_estimation());
    i.perform_polygon_polyhedron(l, &polygon, &s, &poly)
}

fn sight(projector: &Projector, s: &Surface, r: &mut Reader) {
    let (x, y, mode, pmin, pmax) = (r.real(), r.real(), r.integer(), r.real(), r.real());
    attempt(|| {
        let l = projector.shoot(x, y)?;
        let mut i = InterCSurf::new();
        if mode == 0 {
            i.perform(&l, &s)?;
        } else {
            polyhedron_perform(&mut i, &l, s, (pmin, pmax))?;
        }
        print_inter(&i)
    });
}

fn surface_body(r: &mut Reader, projector: &Arc<Projector>) -> Result<(), Failure> {
    let f = face(r)?;
    let (tf, tr) = (r.real(), r.real());
    let (pu1, pv1, pu2, pv2) = (r.real(), r.real(), r.real(), r.real());
    let e = edge(r)?;
    let (back, tc) = (r.flag(), r.real());
    let mut s = Surface::new();
    s.set_projector(Some(projector.clone()));
    s.set_surface(&f)?;
    queries(&s, tf, tr);
    slprops(&s, pu1, pv1);
    slprops(&s, pu2, pv2);
    let mut c = Curve::new();
    c.set_projector(Some(projector.clone()));
    c.set_curve(&e)?;
    let (mut mn, mut mx) = ([0.; 16], [0.; 16]);
    init_min_max(1e100, &mut mn, &mut mx);
    attempt(|| {
        c.update(&mut mn, &mut mx)?;
        flag(s.is_above(back, &c, tc)?);
        Ok(())
    });
    for _ in 0..r.integer() {
        sight(projector, &s, r);
    }
    Ok(())
}

/// `projector face tolf toler u1 v1 u2 v2 edge back tolc nsight ...`: see
/// `hb_surface_case`.
pub fn surface_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match projector(&mut r) {
        Ok(p) => {
            let p = Arc::new(p);
            attempt(|| surface_body(&mut r, &p));
        }
        Err(e) => print!("{} ", e.exception_type()),
    }
    end();
}
