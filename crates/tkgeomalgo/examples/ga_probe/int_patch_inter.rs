//! IntPatch_Intersection and IntPatch_PrmPrmIntersection probe operations, the Rust twin
//! of tools/oracle/tkgeomalgo_probe_intpatch_inter.cxx.
use super::common::{Reader, attempt, end, flag, int, real};
use super::ics_geometry::surface;
use super::int_patch::{Fixture, point};
use super::int_patch_wltool::{WtDomain, domain, report};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkgeomalgo::int_patch::{
    IntPatchIntersection, IntPatchIntersectionOptions, IntPatchPolyhedron,
    IntPatchPrmPrmIntersection, IntPatchPrmPrmTolerances,
};
use rustycad_tkgeomalgo::int_surf::IntSurfPntOn2S;

/// `ga_in_report`.
fn inter_report(fx: &Fixture, inter: &IntPatchIntersection) {
    flag(inter.is_done());
    attempt(|| inter.is_empty().map(flag));
    attempt(|| inter.tangent_faces().map(flag));
    attempt(|| inter.opposite_faces().map(flag));
    attempt(|| {
        let n = inter.nb_pnts()?;
        int(n);
        for i in 1..=n {
            point(fx, &inter.point(i)?);
        }
        Ok(())
    });
    attempt(|| {
        let n = inter.nb_lines()?;
        int(n);
        for i in 1..=n {
            report(fx, &inter.line(i)?);
        }
        attempt(|| inter.line(0).map(|_| ()));
        attempt(|| inter.line(n + 1).map(|_| ()));
        Ok(())
    });
    int(inter.sequence_of_line().len() as i32);
}

/// The surfaces and domains of a request.
struct Inputs {
    s1: Adaptor3dSurfaceHandle,
    d1: WtDomain,
    s2: Adaptor3dSurfaceHandle,
    d2: WtDomain,
}

fn inputs(r: &mut Reader, fx: &Fixture) -> Result<Inputs, Failure> {
    let s1 = surface(r);
    let d1 = domain(r, fx, &s1)?;
    let s2 = surface(r);
    let d2 = domain(r, fx, &s2)?;
    Ok(Inputs { s1, d1, s2, d2 })
}

/// `n (u1 v1 u2 v2)*`: points with their parameters only.
fn start_points(r: &mut Reader) -> Vec<IntSurfPntOn2S> {
    (0..r.integer())
        .map(|_| {
            let mut q = IntSurfPntOn2S::new();
            let (a, b, c, d) = (r.real(), r.real(), r.real(), r.real());
            q.set_parameters(a, b, c, d);
            q
        })
        .collect()
}

fn options(r: &mut Reader) -> IntPatchIntersectionOptions {
    IntPatchIntersectionOptions {
        is_geom_int: r.flag(),
        is_req_to_keep_rline: r.flag(),
        is_req_to_post_wl_proc: r.flag(),
    }
}

/// The `Perform` of `ga_ip_inter_case` of kind 0 to 3.
fn inter_perform(
    r: &mut Reader,
    i: &Inputs,
    inter: &mut IntPatchIntersection,
    (kind, tols): (i32, (f64, f64)),
) {
    match kind {
        0 => {
            let o = options(r);
            attempt(|| inter.perform(&i.s1, &i.d1, &i.s2, &i.d2, tols, o));
        }
        1 => {
            let mut pts = start_points(r);
            let o = options(r);
            attempt(|| inter.perform_with_points(&i.s1, &i.d1, &i.s2, &i.d2, tols, &mut pts, o));
            int(pts.len() as i32);
        }
        2 => {
            let start = [r.real(), r.real(), r.real(), r.real()];
            attempt(|| inter.perform_from_start((&i.s1, &i.d1), (&i.s2, &i.d2), start, tols));
        }
        _ => attempt(|| inter.perform_self(&i.s1, &i.d1, tols.0, tols.1)),
    }
}

/// `ga_ip_inter_case`.
pub fn inter_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let fx = Fixture::new();
    attempt(|| {
        let i = inputs(&mut r, &fx)?;
        let (ta, tt) = (r.real(), r.real());
        let set = r.flag();
        let t: [f64; 4] = if set {
            [r.real(), r.real(), r.real(), r.real()]
        } else {
            [0.0; 4]
        };
        let kind = r.integer();
        if kind == 4 {
            inter_report(
                &fx,
                &IntPatchIntersection::with_surfaces(&i.s1, &i.d1, &i.s2, &i.d2, ta, tt)?,
            );
            return Ok(());
        }
        if kind == 5 {
            inter_report(
                &fx,
                &IntPatchIntersection::with_surface(&i.s1, &i.d1, ta, tt)?,
            );
            return Ok(());
        }
        let mut inter = IntPatchIntersection::new();
        if set {
            inter.set_tolerances(t[0], t[1], t[2], t[3]);
        }
        inter_perform(&mut r, &i, &mut inter, (kind, (ta, tt)));
        inter_report(&fx, &inter);
        Ok(())
    });
    end();
}

/// `ga_ip_isingular_case`.
pub fn isingular_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let fx = Fixture::new();
    attempt(|| {
        let i = inputs(&mut r, &fx)?;
        let tol = r.real();
        let show = |res: Result<(bool, f64), Failure>| {
            attempt(|| {
                let (f, d) = res?;
                flag(f);
                real(d);
                Ok(())
            })
        };
        show(IntPatchIntersection::check_singular_points(
            &i.s1, &i.d1, &i.s2,
        ));
        show(IntPatchIntersection::check_singular_points(
            &i.s2, &i.d2, &i.s1,
        ));
        show(IntPatchIntersection::check_singular_points(
            &i.s1, &i.d1, &i.s1,
        ));
        attempt(|| IntPatchIntersection::define_uv_max_step(&i.s1, &i.d1, &i.s2, &i.d2).map(real));
        attempt(|| IntPatchIntersection::define_uv_max_step(&i.s1, &i.d1, &i.s1, &i.d1).map(real));
        let (mut v1, mut v2) = (Vec::new(), Vec::new());
        attempt(|| {
            IntPatchIntersection::prepare_surfaces(
                &i.s1, &i.d1, &i.s2, &i.d2, tol, &mut v1, &mut v2,
            )
        });
        for v in [&v1, &v2] {
            int(v.len() as i32);
            for s in v.iter() {
                real(s.first_u_parameter()?);
                real(s.last_u_parameter()?);
                real(s.first_v_parameter()?);
                real(s.last_v_parameter()?);
            }
        }
        Ok(())
    });
    end();
}

/// The polyhedron of a surface: the automatic one for 0 subdivisions.
fn polyhedron(s: &Adaptor3dSurfaceHandle, nu: i32, nv: i32) -> Result<IntPatchPolyhedron, Failure> {
    if nu > 0 {
        IntPatchPolyhedron::with_subdivisions(s, nu, nv)
    } else {
        IntPatchPolyhedron::new(s)
    }
}

/// The `Perform` of `ga_ip_prmprm_case`.
fn prmprm_perform(
    r: &mut Reader,
    i: &Inputs,
    pp: &mut IntPatchPrmPrmIntersection,
    t: IntPatchPrmPrmTolerances,
) {
    let (s1, s2) = ((&i.s1, &i.d1 as _), (&i.s2, &i.d2 as _));
    match r.integer() {
        0 => {
            let c = r.flag();
            attempt(|| {
                pp.perform(
                    &i.s1,
                    &i.d1,
                    &i.s2,
                    &i.d2,
                    t.tol_tangency,
                    t.epsilon,
                    t.deflection,
                    t.increment,
                    c,
                )
            });
        }
        1 => {
            let mut pts = start_points(r);
            attempt(|| pp.perform_with_points(s1, s2, t, &mut pts));
            int(pts.len() as i32);
        }
        2 => {
            let start = [r.real(), r.real(), r.real(), r.real()];
            attempt(|| pp.perform_from_start(s1, s2, start, t));
        }
        3 => attempt(|| pp.perform_self(&i.s1, &i.d1, t)),
        4 => {
            let (a, b, c, d) = (r.integer(), r.integer(), r.integer(), r.integer());
            attempt(|| {
                if a > 0 && c > 0 {
                    let (p1, p2) = (polyhedron(&i.s1, a, b)?, polyhedron(&i.s2, c, d)?);
                    return pp.perform_polyhedra((&i.s1, &p1, &i.d1), (&i.s2, &p2, &i.d2), t);
                }
                if a > 0 {
                    return pp.perform_polyhedron1(
                        (&i.s1, &polyhedron(&i.s1, a, b)?, &i.d1),
                        s2,
                        t,
                    );
                }
                if c > 0 {
                    return pp.perform_polyhedron2(
                        s1,
                        (&i.s2, &polyhedron(&i.s2, c, d)?, &i.d2),
                        t,
                    );
                }
                let (p1, p2) = (polyhedron(&i.s1, 0, 0)?, polyhedron(&i.s2, 0, 0)?);
                pp.perform_polyhedra((&i.s1, &p1, &i.d1), (&i.s2, &p2, &i.d2), t)
            });
        }
        _ => {
            let (a, b) = (r.integer(), r.integer());
            attempt(|| {
                pp.perform_self_polyhedron(
                    (
                        &i.s1,
                        &IntPatchPolyhedron::with_subdivisions(&i.s1, a, b)?,
                        &i.d1,
                    ),
                    t,
                )
            });
        }
    }
}

/// `ga_ip_prmprm_case`.
pub fn prmprm_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let fx = Fixture::new();
    attempt(|| {
        let i = inputs(&mut r, &fx)?;
        let t = IntPatchPrmPrmTolerances {
            tol_tangency: r.real(),
            epsilon: r.real(),
            deflection: r.real(),
            increment: r.real(),
        };
        let mut pp = IntPatchPrmPrmIntersection::new();
        prmprm_perform(&mut r, &i, &mut pp, t);
        flag(pp.is_done());
        attempt(|| pp.is_empty().map(flag));
        attempt(|| {
            let n = pp.nb_lines()?;
            int(n);
            for k in 1..=n {
                report(&fx, &pp.line(k)?);
            }
            attempt(|| pp.line(0).map(|_| ()));
            attempt(|| pp.line(n + 1).map(|_| ()));
            Ok(())
        });
        for _ in 0..r.integer() {
            let (l, lo, hi, nb) = (r.integer(), r.integer(), r.integer(), r.integer());
            attempt(|| {
                pp.new_line(&i.s1, &i.s2, l, (lo, hi), nb)
                    .map(|line| report(&fx, &line))
            });
        }
        Ok(())
    });
    end();
}
