//! GeomAPI probe operations over the projections and the extrema, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_geomapi.cxx.
use super::common::{Reader, attempt, end, flag, int, pnt, real, show};
use super::ics_geometry::{geom_curve, geom_surface};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::geom_api::{
    ExtremaCurveCurve, ExtremaCurveSurface, ExtremaSurfaceSurface, ProjectPointOnCurve,
    ProjectPointOnSurf,
};
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag};
use rustycad_tkmath::gp::Point3;

fn pair(p: (Point3, Point3)) {
    pnt(p.0);
    pnt(p.1);
}

/// `ga_gapi_ppc_print`: every accessor, indices 0..NbPoints+1.
fn ppc_print(e: &ProjectPointOnCurve) -> Result<(), Failure> {
    let n = e.nb_points();
    int(n);
    int(e.as_int());
    for i in 0..=n + 1 {
        show(e.point(i), pnt);
        show(e.parameter(i), real);
        show(e.parameter(i), real);
        show(e.distance(i), real);
    }
    show(e.nearest_point(), pnt);
    show(e.as_point(), pnt);
    show(e.lower_distance_parameter(), real);
    show(e.lower_distance(), real);
    show(e.as_real(), real);
    let x = e.extrema()?;
    flag(x.is_done());
    show(x.nb_ext(), int);
    Ok(())
}

/// `curve mode x y z u0 u1 qx qy qz`: GeomAPI_ProjectPointOnCurve.
pub fn ppc_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let c = geom_curve(&mut r);
        let (mode, p) = (r.integer(), r.pnt());
        let (u0, u1, q) = (r.real(), r.real(), r.pnt());
        let e = match mode {
            0 => ProjectPointOnCurve::with_curve(p, &c)?,
            1 => ProjectPointOnCurve::with_range(p, &c, u0, u1)?,
            _ => {
                let mut e = ProjectPointOnCurve::new();
                if mode == 2 {
                    e.init_curve(&c, u0, u1)?;
                    e.perform(p)?;
                }
                if mode == 3 {
                    e.init(p, &c)?;
                    e.perform(q)?;
                }
                if mode == 5 {
                    e.init_curve(&c, u0, u1)?;
                }
                e
            }
        };
        ppc_print(&e)
    });
    end();
}

/// `ga_gapi_pps_print`: every accessor, indices 0..NbPoints+1.
fn pps_print(e: &ProjectPointOnSurf) -> Result<(), Failure> {
    flag(e.is_done());
    let n = e.nb_points()?;
    int(n);
    int(e.as_int()?);
    for i in 0..=n + 1 {
        show(e.point(i), pnt);
        show(e.parameters(i), |(u, v)| {
            real(u);
            real(v);
        });
        show(e.distance(i), real);
    }
    show(e.nearest_point(), pnt);
    show(e.as_point(), pnt);
    show(e.lower_distance_parameters(), |(u, v)| {
        real(u);
        real(v);
    });
    show(e.lower_distance(), real);
    show(e.as_real(), real);
    let x = e.extrema();
    flag(x.is_done());
    show(x.nb_ext(), int);
    Ok(())
}

fn ext_algo(a: i32) -> ExtAlgo {
    if a == 0 { ExtAlgo::Grad } else { ExtAlgo::Tree }
}

fn ext_flag(f: i32) -> ExtFlag {
    [ExtFlag::Min, ExtFlag::Max, ExtFlag::MinMax][f as usize]
}

type SurfaceInit = (i32, ExtAlgo, f64, (f64, f64, f64, f64));

// The constructor or Init `ctor` of ga_gapi_pps_case.
fn pps_build(
    s: &rustycad_tkernel::standard::Handle<dyn rustycad_tkg3d::geom::GeomSurface>,
    (ctor, algo, tol, bounds): SurfaceInit,
    p1: Point3,
) -> Result<ProjectPointOnSurf, Failure> {
    let mut e = match ctor {
        0 => return ProjectPointOnSurf::with_surface(p1, s, algo),
        1 => return ProjectPointOnSurf::with_tolerance(p1, s, tol, algo),
        2 => return ProjectPointOnSurf::with_bounds(p1, s, bounds, algo),
        3 => return ProjectPointOnSurf::with_bounds_tolerance(p1, s, bounds, tol, algo),
        _ => ProjectPointOnSurf::new(),
    };
    match ctor {
        4 => e.init_surface(s, bounds, algo)?,
        5 => e.init_surface_tolerance(s, bounds, tol, algo)?,
        6 => e.init(p1, s, algo)?,
        7 => e.init_tolerance(p1, s, tol, algo)?,
        8 => e.init_bounds(p1, s, bounds, algo)?,
        9 => e.init_bounds_tolerance(p1, s, bounds, tol, algo)?,
        _ => {}
    }
    Ok(e)
}

/// `surface ctor algo tol u1 u2 v1 v2 flag n (x y z a)*n`: GeomAPI_ProjectPointOnSurf.
pub fn pps_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = geom_surface(&mut r);
        let (ctor, algo, tol) = (r.integer(), ext_algo(r.integer()), r.real());
        let bounds = (r.real(), r.real(), r.real(), r.real());
        let (fl, n) = (r.integer(), r.integer());
        let steps: Vec<(Point3, i32)> = (0..n).map(|_| (r.pnt(), r.integer())).collect();
        let p1 = steps.first().map_or(Point3::default(), |step| step.0);
        let mut e = pps_build(&s, (ctor, algo, tol, bounds), p1)?;
        if fl >= 0 {
            e.set_extrema_flag(ext_flag(fl));
        }
        pps_print(&e)?;
        let first: usize = match ctor {
            4 | 5 => 0,
            10 => steps.len(),
            _ => 1,
        };
        for (p, a) in steps.iter().skip(first) {
            if *a >= 0 {
                e.set_extrema_algo(ext_algo(*a));
            }
            e.perform(*p)?;
            pps_print(&e)?;
        }
        Ok(())
    });
    end();
}

/// `ga_gapi_ecc_print`: the members of GeomAPI_ExtremaCurveCurve.
fn ecc_print(e: &ExtremaCurveCurve) -> Result<(), Failure> {
    let n = e.nb_extrema();
    int(n);
    show(e.as_int(), int);
    for i in 0..=n + 1 {
        show(e.points(i), pair);
        show(e.parameters(i), |(u, v)| {
            real(u);
            real(v);
        });
        show(e.distance(i), real);
    }
    show(e.is_parallel(), flag);
    show(e.nearest_points(), pair);
    show(e.lower_distance_parameters(), |(u, v)| {
        real(u);
        real(v);
    });
    show(e.lower_distance(), real);
    show(e.as_real(), real);
    let x = e.extrema()?;
    flag(x.is_done());
    show(x.is_parallel(), flag);
    show(x.nb_ext(), int);
    Ok(())
}

/// `ga_gapi_ecc_total`: the Total members, TotalLowerDistance first when `order` is 1.
fn ecc_total(e: &mut ExtremaCurveCurve, order: i32) {
    if order == 1 {
        show(e.total_lower_distance(), real);
    }
    show(e.total_nearest_points(), |points| {
        flag(points.is_some());
        if let Some(points) = points {
            pair(points);
        }
    });
    show(e.total_lower_distance_parameters(), |parameters| {
        flag(parameters.is_some());
        if let Some((u, v)) = parameters {
            real(u);
            real(v);
        }
    });
    show(e.total_lower_distance(), real);
}

/// `count (c1 c2 ranged u1 u2 v1 v2)*count order`: GeomAPI_ExtremaCurveCurve.
pub fn ecc_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let count = r.integer();
        let mut e = ExtremaCurveCurve::new();
        for k in 0..count {
            let (c1, c2) = (geom_curve(&mut r), geom_curve(&mut r));
            let ranged = r.flag();
            let (u, v) = ((r.real(), r.real()), (r.real(), r.real()));
            match (k, ranged) {
                (0, true) => e = ExtremaCurveCurve::with_ranges(&c1, &c2, u, v)?,
                (0, false) => e = ExtremaCurveCurve::with_curves(&c1, &c2)?,
                (_, true) => e.init_ranges(&c1, &c2, u, v)?,
                (_, false) => e.init(&c1, &c2)?,
            }
        }
        let order = r.integer();
        ecc_print(&e)?;
        ecc_total(&mut e, order);
        Ok(())
    });
    end();
}

/// `curve surface mode w1 w2 u1 u2 v1 v2`: GeomAPI_ExtremaCurveSurface.
pub fn ecs_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let (c, s, mode) = (geom_curve(&mut r), geom_surface(&mut r), r.integer());
        let w = (r.real(), r.real());
        let ranges = (w, (r.real(), r.real()), (r.real(), r.real()));
        let e = match mode {
            0 => ExtremaCurveSurface::with_curve(&c, &s)?,
            1 => ExtremaCurveSurface::with_ranges(&c, &s, ranges)?,
            _ => {
                let mut e = ExtremaCurveSurface::new();
                if mode == 3 {
                    e.init(&c, &s)?;
                }
                if mode == 4 {
                    e.init_ranges(&c, &s, ranges)?;
                }
                e
            }
        };
        let n = e.nb_extrema();
        int(n);
        int(e.as_int());
        for i in 0..=n + 1 {
            show(e.points(i), pair);
            show(e.parameters(i), |(w, u, v)| {
                real(w);
                real(u);
                real(v);
            });
            show(e.distance(i), real);
        }
        show(e.is_parallel(), flag);
        show(e.nearest_points(), pair);
        show(e.lower_distance_parameters(), |(w, u, v)| {
            real(w);
            real(u);
            real(v);
        });
        show(e.lower_distance(), real);
        show(e.as_real(), real);
        let x = e.extrema()?;
        flag(x.is_done());
        show(x.is_parallel(), flag);
        show(x.nb_ext(), int);
        Ok(())
    });
    end();
}

fn four(v: (f64, f64, f64, f64)) {
    real(v.0);
    real(v.1);
    real(v.2);
    real(v.3);
}

/// `s1 s2 mode u1 u2 v1 v2 u3 u4 v3 v4`: GeomAPI_ExtremaSurfaceSurface.
pub fn ess_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let (s1, s2, mode) = (geom_surface(&mut r), geom_surface(&mut r), r.integer());
        let b1 = (r.real(), r.real(), r.real(), r.real());
        let b2 = (r.real(), r.real(), r.real(), r.real());
        let e = match mode {
            0 => ExtremaSurfaceSurface::with_surfaces(&s1, &s2)?,
            1 => ExtremaSurfaceSurface::with_bounds(&s1, &s2, b1, b2)?,
            _ => {
                let mut e = ExtremaSurfaceSurface::new();
                if mode == 3 {
                    e.init(&s1, &s2)?;
                }
                if mode == 4 {
                    e.init_bounds(&s1, &s2, b1, b2)?;
                }
                e
            }
        };
        let n = e.nb_extrema();
        int(n);
        int(e.as_int());
        for i in 0..=n + 1 {
            show(e.points(i), pair);
            show(e.parameters(i), four);
            show(e.distance(i), real);
        }
        show(e.is_parallel(), flag);
        show(e.nearest_points(), pair);
        show(e.lower_distance_parameters(), four);
        show(e.lower_distance(), real);
        show(e.as_real(), real);
        let x = e.extrema()?;
        flag(x.is_done());
        show(x.is_parallel(), flag);
        show(x.nb_ext(), int);
        Ok(())
    });
    end();
}
