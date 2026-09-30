//! GeomInt and GeomAPI_IntSS probe operations, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_geomint.cxx.
use super::common::{Reader, attempt, end, flag, int, pnt, pnt2d, real};
use super::geom_api_fit::{bspline_out, curve_out, curve2d_out};
use super::geom2d_api::bspline_out as bspline2d_out;
use super::ics_geometry::{geom_surface, surface};
use super::int_patch::{Fixture, point};
use super::int_patch_wltool::{domain, line, report};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::adaptor3d::Adaptor3dTopolTool;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve, GeomSurface, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::geom_api::IntSS;
use rustycad_tkgeomalgo::geom_int::{
    GeomIntIntSS, GeomIntIntSSApprox, GeomIntLineConstructor, GeomIntLineTool,
};
use rustycad_tkgeomalgo::int_patch::{
    IntPatchIntersection, IntPatchIntersectionOptions, IntPatchLine, IntPatchWLine,
};
use rustycad_tkgeomalgo::int_start::TheTopolTool;
use std::rc::Rc;

/// `ga_gi_curve`.
pub fn gi_curve(c: &Handle<dyn GeomCurve>) -> Result<(), Failure> {
    curve_out(c)?;
    if c.is_null() {
        return Ok(());
    }
    let t: Handle<GeomTrimmedCurve> = c.cast();
    flag(t.is_null());
    let b: Handle<dyn GeomCurve> = match t.get() {
        Some(t) => t.basis_curve(),
        None => c.clone(),
    };
    let s: Handle<GeomBSplineCurve> = b.cast();
    flag(s.is_null());
    if !s.is_null() {
        bspline_out(&s)?;
    }
    Ok(())
}

/// `ga_gi_curve2d`.
pub fn gi_curve2d(c: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
    curve2d_out(c)?;
    if c.is_null() {
        return Ok(());
    }
    let t: Handle<Geom2dTrimmedCurve> = c.cast();
    flag(t.is_null());
    let b: Handle<dyn Geom2dCurve> = match t.get() {
        Some(t) => t.basis_curve(),
        None => c.clone(),
    };
    let s: Handle<Geom2dBSplineCurve> = b.cast();
    flag(s.is_null());
    if !s.is_null() {
        bspline2d_out(&s)?;
    }
    Ok(())
}

/// `ga_gi_intss_report`.
fn intss_report(s: &GeomIntIntSS) {
    flag(s.is_done());
    real(s.tol_reached3d());
    real(s.tol_reached2d());
    let n = s.nb_lines();
    int(n);
    for i in 0..=n + 1 {
        attempt(|| gi_curve(&s.line(i)?));
        attempt(|| s.has_line_on_s1(i).map(flag));
        attempt(|| gi_curve2d(&s.line_on_s1(i)?));
        attempt(|| s.has_line_on_s2(i).map(flag));
        attempt(|| gi_curve2d(&s.line_on_s2(i)?));
    }
    attempt(|| {
        let nb = s.nb_boundaries()?;
        int(nb);
        for i in 0..=nb + 1 {
            attempt(|| gi_curve(&s.boundary(i)?));
        }
        Ok(())
    });
    attempt(|| {
        let np = s.nb_points()?;
        int(np);
        for i in 0..=np + 1 {
            attempt(|| s.point(i).map(pnt));
            attempt(|| s.pnt2d(i, true).map(pnt2d));
            attempt(|| s.pnt2d(i, false).map(pnt2d));
        }
        Ok(())
    });
}

/// `ga_gi_adaptor`.
fn gi_adaptor(
    r: &mut Reader,
    s: &Handle<dyn GeomSurface>,
) -> Result<Handle<GeomAdaptorSurface>, Failure> {
    if !r.flag() {
        return Ok(Handle::new(GeomAdaptorSurface::with_surface(s)?));
    }
    let bounds = (r.real(), r.real(), r.real(), r.real());
    Ok(Handle::new(GeomAdaptorSurface::with_bounds(s, bounds)?))
}

fn start(r: &mut Reader) -> [f64; 4] {
    [r.real(), r.real(), r.real(), r.real()]
}

/// The Perform of `ga_gi_intss_case` of mode 1 to 6.
fn intss_perform(
    r: &mut Reader,
    s: &mut GeomIntIntSS,
    (s1, s2): (&Handle<dyn GeomSurface>, &Handle<dyn GeomSurface>),
    (tol, flags, mode): (f64, GeomIntIntSSApprox, i32),
) -> Result<(), Failure> {
    match mode {
        1 => attempt(|| s.perform(s1, s2, tol, flags)),
        2 => {
            let st = start(r);
            attempt(|| s.perform_from_start(s1, s2, tol, st, flags));
        }
        3 | 4 => {
            let (h1, h2) = (gi_adaptor(r, s1)?, gi_adaptor(r, s2)?);
            if mode == 3 {
                attempt(|| s.perform_adaptors(&h1, &h2, tol, flags));
            } else {
                let st = start(r);
                attempt(|| s.perform_adaptors_from_start(&h1, &h2, tol, st, flags));
            }
        }
        5 => {
            attempt(|| s.perform(s1, s2, tol, flags));
            attempt(|| s.perform(s2, s1, tol, flags));
        }
        _ => attempt(|| s.perform(s1, s1, tol, flags)),
    }
    Ok(())
}

/// `ga_gi_intss_case`.
pub fn intss_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s1 = geom_surface(&mut r);
        let s2 = geom_surface(&mut r);
        let tol = r.real();
        let flags = GeomIntIntSSApprox {
            approx: r.flag(),
            approx_s1: r.flag(),
            approx_s2: r.flag(),
        };
        let mode = r.integer();
        if mode == 0 {
            intss_report(&GeomIntIntSS::with_surfaces(&s1, &s2, tol, flags)?);
            return Ok(());
        }
        let mut s = GeomIntIntSS::new();
        intss_perform(&mut r, &mut s, (&s1, &s2), (tol, flags, mode))?;
        intss_report(&s);
        Ok(())
    });
    end();
}

/// `ga_gapi_intss_case`.
pub fn gapi_intss_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s1 = geom_surface(&mut r);
        let s2 = geom_surface(&mut r);
        let tol = r.real();
        let mode = r.integer();
        let mut s = if mode == 0 {
            IntSS::with_surfaces(&s1, &s2, tol)?
        } else {
            IntSS::new()
        };
        if mode == 1 {
            attempt(|| s.perform(&s1, &s2, tol));
        }
        flag(s.is_done());
        let n = s.nb_lines();
        int(n);
        for i in 0..=n + 1 {
            attempt(|| gi_curve(&s.line(i)?));
        }
        Ok(())
    });
    end();
}

/// `ga_gi_lc_report`.
fn lc_report(lc: &GeomIntLineConstructor) {
    flag(lc.is_done());
    attempt(|| {
        let n = lc.nb_parts()?;
        int(n);
        for i in 0..=n + 1 {
            attempt(|| {
                let (f, l) = lc.part(i)?;
                real(f);
                real(l);
                Ok(())
            });
        }
        Ok(())
    });
}

/// `ga_gi_lconst_case`.
pub fn lconst_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let fx = Fixture::new();
    attempt(|| {
        let (s1, s2) = (surface(&mut r), surface(&mut r));
        let d1: Rc<dyn TheTopolTool> = Rc::new(domain(&mut r, &fx, &s1)?);
        let d2: Rc<dyn TheTopolTool> = Rc::new(domain(&mut r, &fx, &s2)?);
        let l = line(&mut r, &fx)?;
        let mut lc = GeomIntLineConstructor::new();
        lc_report(&lc);
        lc.load(d1, d2, &s1.cast(), &s2.cast());
        attempt(|| lc.perform(&l));
        lc_report(&lc);
        attempt(|| lc.perform(&l));
        lc_report(&lc);
        Ok(())
    });
    end();
}

/// `GaGiInter`: the lines of the intersection of two adaptors with their own domains.
pub struct GiInter {
    pub s1: Adaptor3dSurfaceHandle,
    pub s2: Adaptor3dSurfaceHandle,
    pub d1: Rc<Adaptor3dTopolTool>,
    pub d2: Rc<Adaptor3dTopolTool>,
    pub inter: IntPatchIntersection,
}

/// `ga_gi_inter`.
pub fn gi_inter(r: &mut Reader) -> Result<GiInter, Failure> {
    let s1 = surface(r);
    let s2 = surface(r);
    let tol = r.real();
    let d1 = Rc::new(Adaptor3dTopolTool::with_surface(&s1)?);
    let d2 = Rc::new(Adaptor3dTopolTool::with_surface(&s2)?);
    let mut inter = IntPatchIntersection::new();
    let options = IntPatchIntersectionOptions::default();
    inter.perform(&s1, &*d1, &s2, &*d2, (tol, tol), options)?;
    Ok(GiInter {
        s1,
        s2,
        d1,
        d2,
        inter,
    })
}

/// The line tool on a line: its vertices and parameter range.
fn ltool_report(fx: &Fixture, l: &Handle<dyn IntPatchLine>) {
    attempt(|| {
        let nv = GeomIntLineTool::nb_vertex(l)?;
        int(nv);
        for k in 0..=nv + 1 {
            attempt(|| GeomIntLineTool::vertex(l, k).map(|v| point(fx, &v)));
        }
        Ok(())
    });
    attempt(|| GeomIntLineTool::first_parameter(l).map(real));
    attempt(|| GeomIntLineTool::last_parameter(l).map(real));
}

/// The decomposition of one walking line of `ga_gi_lines_case`.
fn decomposition(
    fx: &Fixture,
    w: &Handle<IntPatchWLine>,
    (h1, h2): (&Handle<GeomAdaptorSurface>, &Handle<GeomAdaptorSurface>),
    lc: &GeomIntLineConstructor,
) {
    let mut nl: Vec<Handle<dyn IntPatchLine>> = Vec::new();
    attempt(|| GeomIntLineTool::decomposition_of_wline(w, h1, h2, 2.0e-7, lc, &mut nl).map(flag));
    int(nl.len() as i32);
    for l in &nl {
        report(fx, l);
    }
}

/// `ga_gi_lines_case`.
pub fn lines_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let fx = Fixture::new();
    attempt(|| {
        let g = gi_inter(&mut r)?;
        flag(g.inter.is_done());
        let n = g.inter.nb_lines()?;
        int(n);
        let (h1, h2): (Handle<GeomAdaptorSurface>, Handle<GeomAdaptorSurface>) =
            (g.s1.cast(), g.s2.cast());
        for i in 1..=n {
            let l = g.inter.line(i)?;
            ltool_report(&fx, &l);
            let mut lc = GeomIntLineConstructor::new();
            lc.load(g.d1.clone(), g.d2.clone(), &h1, &h2);
            attempt(|| lc.perform(&l));
            lc_report(&lc);
            let w: Handle<IntPatchWLine> = l.cast();
            if !w.is_null() {
                decomposition(&fx, &w, (&h1, &h2), &lc);
            }
        }
        Ok(())
    });
    end();
}

/// `ga_gi_ltool_case`.
pub fn ltool_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let fx = Fixture::new();
    attempt(|| {
        let l = line(&mut r, &fx)?;
        ltool_report(&fx, &l);
        Ok(())
    });
    end();
}
