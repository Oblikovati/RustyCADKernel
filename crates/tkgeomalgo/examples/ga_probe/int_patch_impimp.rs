//! IntPatch analytic intersection probe operations, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_intpatch_impimp.cxx: IntPatch_ImpImpIntersection,
//! IntPatch_ALineToWLine and IntPatch_SpecialPoints.
use super::common::{Reader, attempt, end, flag, int, real, xyz};
use super::ics_geometry::surface;
use super::int_patch_report::{Domains, line, point};
use super::int_walk::pnt2s;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dTopolTool;
use rustycad_tkgeomalgo::int_patch::{
    IntPatchALine, IntPatchALineToWLine, IntPatchImpImpIntersection, IntPatchLine, IntPatchPoint,
    IntPatchPointLine, IntPatchSpecPntType, IntPatchSpecialPoints, IntPatchWLine,
};
use rustycad_tkgeomalgo::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S};
use rustycad_tkmath::gp::{Axis3, Cone3, Point3, Vector3, Xyz};
use rustycad_tkmath::math::Vector;

/// `ga_ipl_impimp`.
fn impimp(dm: &Domains, x: &IntPatchImpImpIntersection) {
    int(x.get_status() as i32);
    flag(x.is_done());
    attempt(|| x.is_empty().map(flag));
    attempt(|| x.tangent_faces().map(flag));
    attempt(|| x.opposite_faces().map(flag));
    attempt(|| {
        let n = x.nb_pnts()?;
        int(n);
        for i in 0..=n + 1 {
            attempt(|| x.point(i).map(|p| point(dm, p)));
        }
        Ok(())
    });
    attempt(|| {
        let n = x.nb_lines()?;
        int(n);
        for i in 0..=n + 1 {
            attempt(|| x.line(i).map(|l| line(dm, l)));
        }
        Ok(())
    });
}

/// `ga_ip_impimp_case`.
pub fn impimp_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let (s1, s2) = (surface(&mut r), surface(&mut r));
        let (tolarc, toltang, keep) = (r.real(), r.real(), r.flag());
        let (d1, d2) = (Adaptor3dTopolTool::with_surface(&s1)?, Adaptor3dTopolTool::with_surface(&s2)?);
        let dm = Domains::new(&d1, &d2);
        impimp(&dm, &IntPatchImpImpIntersection::new());
        let x = IntPatchImpImpIntersection::with_surfaces(&s1, &d1, &s2, &d2, tolarc, toltang, keep)?;
        impimp(&dm, &x);
        Ok(())
    });
    end();
}

/// `ga_ipl_convert`.
fn convert(dm: &Domains, c: &IntPatchALineToWLine, a: &Handle<IntPatchALine>, (ranged, fp, lp): (bool, f64, f64)) {
    let mut lines: Vec<Handle<dyn IntPatchLine>> = Vec::new();
    attempt(|| if ranged { c.make_wline_range(a, fp, lp, &mut lines) } else { c.make_wline(a, &mut lines) });
    int(lines.len() as i32);
    lines.iter().for_each(|l| line(dm, l));
    let al: &IntPatchALine = a.get().expect("analytic line");
    let (mut f, mut l) = (0.0, 0.0);
    attempt(|| {
        f = al.first_parameter()?.0;
        l = al.last_parameter()?.0;
        Ok(())
    });
    for k in 0..=4 {
        attempt(|| {
            let p: Point3 = al.value(f + (l - f) * f64::from(k) / 4.0)?;
            real(c.get_section_radius(p)?);
            int(c.check_deflection(p.xyz(), c.tol_3d()));
            int(c.check_deflection(p.xyz() + Xyz::new(1e-3, 0.0, 0.0), 1e-5));
            Ok(())
        });
    }
    attempt(|| {
        let (mut p, mut v) = (Point3::default(), Vector3::default());
        al.d1(0.5 * (f + l), &mut p, &mut v)?;
        let mut q = IntSurfPntOn2S::new();
        q.set_value_all(p, 0.0, 0.0, 0.0, 0.0);
        let mut step: f64 = 0.1;
        let ok = c.step_computing(al, &q, l, 0.5 * (f + l), v.magnitude(), [1e-3, 1.0], c.tol_3d(), &mut step)?;
        flag(ok);
        real(step);
        Ok(())
    });
    lines.iter().for_each(|l| correct_ends(c, l));
}

/// CorrectEndPoint at both ends of a copy of the points of a walking line.
fn correct_ends(c: &IntPatchALineToWLine, l: &Handle<dyn IntPatchLine>) {
    let w: Handle<IntPatchWLine> = l.cast();
    let Some(w) = w.get() else {
        return;
    };
    let n = w.nb_pnts().expect("walking line points");
    if n < 3 {
        return;
    }
    let copy = Handle::new(IntSurfLineOn2S::new(None));
    for k in 1..=n {
        copy.add(&w.point(k).expect("walking line point"));
    }
    attempt(|| {
        c.correct_end_point(&copy, 1)?;
        c.correct_end_point(&copy, copy.nb_points())?;
        pnt2s(&copy.value(1)?);
        pnt2s(&copy.value(copy.nb_points())?);
        Ok(())
    });
}

/// `ga_ip_aline2wline_case`.
pub fn aline2wline_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let (s1, s2) = (surface(&mut r), surface(&mut r));
        let (tolarc, toltang, nb) = (r.real(), r.real(), r.integer());
        let (topen, ttrans, t3d) = (r.real(), r.real(), r.real());
        let range = (r.flag(), r.real(), r.real());
        let (d1, d2) = (Adaptor3dTopolTool::with_surface(&s1)?, Adaptor3dTopolTool::with_surface(&s2)?);
        let dm = Domains::new(&d1, &d2);
        let x = IntPatchImpImpIntersection::with_surfaces(&s1, &d1, &s2, &d2, tolarc, toltang, false)?;
        let mut c = IntPatchALineToWLine::new(&s1, &s2, nb)?;
        real(c.tol_open_domain());
        real(c.tol_transition());
        real(c.tol_3d());
        c.set_tol_open_domain(topen);
        c.set_tol_transition(ttrans);
        c.set_tol_3d(t3d);
        real(c.tol_open_domain());
        real(c.tol_transition());
        real(c.tol_3d());
        let n = if x.is_done() { x.nb_lines()? } else { 0 };
        int(n);
        for i in 1..=n {
            let a: Handle<IntPatchALine> = x.line(i)?.cast();
            flag(a.is_null());
            if !a.is_null() {
                convert(&dm, &c, &a, range);
            }
        }
        Ok(())
    });
    end();
}

/// `ga_ipl_pnt2s`.
pub fn read_pnt2s(r: &mut Reader) -> IntSurfPntOn2S {
    let x = r.pnt();
    let (a, b, c, d) = (r.real(), r.real(), r.real(), r.real());
    let mut p = IntSurfPntOn2S::new();
    p.set_value_all(x, a, b, c, d);
    p
}

/// `ga_ipl_vec3`.
fn read_vec3(r: &mut Reader) -> Vector<'static> {
    let mut v = Vector::with_bounds(1, 3).expect("probe vector");
    for i in 1..=3 {
        v[i] = r.real();
    }
    v
}

type S = IntPatchSpecialPoints;

fn spec_type(t: i32) -> IntPatchSpecPntType {
    use IntPatchSpecPntType::*;
    [SPntNone, SPntSeamU, SPntSeamV, SPntSeamUV, SPntPoleSeamU, SPntPole][t as usize]
}

/// The body of `ga_ip_special_case` after its surfaces.
fn special(op: i32, r: &mut Reader, (q, p, rev): (&rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle, &rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle, bool)) -> Result<(), Failure> {
    let mut out = IntSurfPntOn2S::new();
    match op {
        0 => {
            let rf = read_pnt2s(r);
            let tol = r.real();
            attempt(|| {
                flag(S::add_cross_uv_iso_point(q, p, &rf, tol, &mut out, rev)?);
                pnt2s(&out);
                Ok(())
            });
        }
        1 => {
            let rf = read_pnt2s(r);
            let (isu, iso) = (r.flag(), r.real());
            let (t, i0, lo, hi) = (read_vec3(r), read_vec3(r), read_vec3(r), read_vec3(r));
            attempt(|| {
                flag(S::add_point_on_u_or_v_iso(q, p, &rf, (isu, iso), [&t, &i0, &lo, &hi], &mut out, rev)?);
                pnt2s(&out);
                Ok(())
            });
        }
        2 => {
            let iso = read_pnt2s(r);
            let vp = read_pnt2s(r);
            let (vt, chk) = (r.real(), r.flag());
            let mut v = IntPatchPoint::new();
            v.set_pnt_on_2s(&vp);
            v.set_tolerance(vt);
            attempt(|| {
                flag(S::add_singular_pole(q, p, &iso, &mut v, &mut out, rev, chk)?);
                pnt2s(&out);
                Ok(())
            });
            pnt2s(&v.pnt_on_2s());
        }
        3 => {
            let rf = read_pnt2s(r);
            let (t, tol2d) = (r.integer(), r.real());
            out = read_pnt2s(r);
            attempt(|| {
                flag(S::continue_after_special_point(q, p, &rf, spec_type(t), tol2d, &mut out, rev)?);
                pnt2s(&out);
                Ok(())
            });
        }
        _ => {
            let rf = read_pnt2s(r);
            let per = [r.real(), r.real(), r.real(), r.real()];
            out = read_pnt2s(r);
            let wv = r.flag();
            let mut v = IntPatchPoint::new();
            v.set_pnt_on_2s(&out);
            S::adjust_point_and_vertex(&rf, &per, &mut out, if wv { Some(&mut v) } else { None });
            pnt2s(&out);
            pnt2s(&v.pnt_on_2s());
        }
    }
    Ok(())
}

/// `ga_ip_special_case`.
pub fn special_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let op = r.integer();
        let (q, p) = (surface(&mut r), surface(&mut r));
        let rev = r.flag();
        special(op, &mut r, (&q, &p, rev))
    });
    end();
}

/// `ga_ip_special_pole_case`.
pub fn special_pole_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let kind = r.integer();
        let iso = read_pnt2s(&mut r);
        let (du, dv) = (Vector3::from(r.xyz()), Vector3::from(r.xyz()));
        let (ang, rev, u) = (r.real(), r.flag(), r.real());
        match kind {
            0 => attempt(|| {
                let (u, isoc) = S::process_sphere(&iso, du, dv, rev, ang)?;
                flag(true);
                real(u);
                flag(isoc);
                Ok(())
            }),
            1 => attempt(|| {
                let co = Cone3::new(Axis3::default(), ang, 1.0)?;
                let (u, isoc) = S::process_cone(&iso, du, dv, &co, rev, u)?;
                flag(true);
                real(u);
                flag(isoc);
                Ok(())
            }),
            _ => {
                let mut res = [Xyz::default(); 2];
                let n = S::get_tangent_to_int_line_for_cone(ang, *du.xyz(), &mut res);
                int(n);
                res[..n as usize].iter().for_each(|x| xyz(*x));
            }
        }
        Ok(())
    });
    end();
}
