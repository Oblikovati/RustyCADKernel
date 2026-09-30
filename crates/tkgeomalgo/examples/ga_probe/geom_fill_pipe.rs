//! GeomFill_SweepSectionGenerator, GeomFill_AppSweep and GeomFill_Pipe probe
//! operations, the Rust twin of tools/oracle/tkgeomalgo_probe_geomfill_pipe.cxx.
use super::common::{Reader, attempt, end, flag, int, real, show};
use super::curve2d::curve2d;
use super::geom_fill_sweep::{filled, pnts, reals, vecs};
use super::geom_fill_sweepalgo::surface;
use super::geom_fill_trihedron::SHAPES;
use super::geom_fill_trihedron::gft_curve;
use super::ics_geometry::{geom_curve, geom_surface};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomSurface};
use rustycad_tkgeomalgo::app_blend::{AppBlendApprox, AppBlendSection};
use rustycad_tkgeomalgo::geom_fill::*;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};

/// A section with `np` poles (and one 2d pole) preset to -1.
fn section_arrays(np: i32) -> AppBlendSection {
    AppBlendSection {
        poles: filled(1, np, Point3::new(-1.0, -1.0, -1.0)),
        d_poles: filled(1, np, Vector3::new(-1.0, -1.0, -1.0)),
        poles2d: filled(1, 1, Point2::default()),
        d_poles2d: filled(1, 1, Vector2::default()),
        weights: filled(1, np, -1.0),
        d_weights: filled(1, np, -1.0),
    }
}

/// `ga_gfg_section`.
fn gfg_section(g: &GeomFillSweepSectionGenerator, p: i32, np: i32) {
    let mut s = section_arrays(np);
    show(g.section(p, &mut s), |()| {
        int(s.poles.lower());
        pnts(&s.poles);
        reals(&s.weights);
    });
    let mut s = section_arrays(np);
    show(g.section_d1(p, &mut s), |b| {
        flag(b);
        pnts(&s.poles);
        vecs(&s.d_poles);
        reals(&s.weights);
        reals(&s.d_weights);
    });
    show(g.parameter(p), real);
}

fn gfg_trsf(g: &GeomFillSweepSectionGenerator, i: i32) {
    attempt(|| {
        let t = g.transformation(i)?;
        for r in 1..=3 {
            for c in 1..=4 {
                real(t.value(r, c)?);
            }
        }
        Ok(())
    });
}

/// The surface of an AppBlend approximation.
fn app_surface(app: &GeomFillAppSweep) -> Result<Handle<dyn GeomSurface>, Failure> {
    let s = GeomBSplineSurface::with_weights(
        app.surf_poles()?,
        app.surf_weights()?,
        (app.surf_u_knots()?, app.surf_u_mults()?),
        (app.surf_v_knots()?, app.surf_v_mults()?),
        (app.u_degree()?, app.v_degree()?),
        (false, false),
    )?;
    Ok(Handle::new(s).cast())
}

/// `ga_gfg_app`.
fn gfg_app(g: &mut GeomFillSweepSectionGenerator, with_parameters: bool) {
    let line = GeomFillLine::with_nb_points(g.nb_sections());
    let t3d = precision::approximation();
    let mut app =
        GeomFillAppSweep::new(4, 8, t3d, precision::p_approximation(), 0, with_parameters);
    attempt(|| app.perform_by_pieces(Some(&line), g, 30));
    flag(app.is_done());
    if !app.is_done() {
        return;
    }
    show(app.surf_shape(), |s| s.into_iter().for_each(int));
    show(app.tol_reached(), |(t3, t2)| {
        [t3, t2].into_iter().for_each(real)
    });
    show(app_surface(&app), |s| surface(&s));
}

/// `Init` of the kind read from `r` on `g` (the arguments are read first).
fn sweepsec_init(g: &mut GeomFillSweepSectionGenerator, r: &mut Reader, kind: i32) {
    if kind == 0 {
        let (p, c1, c2) = (gft_curve(r), gft_curve(r), gft_curve(r));
        let rad = r.real();
        return attempt(|| g.init_rails(&p, &c1, &c2, rad));
    }
    let p = geom_curve(r);
    match kind {
        1 => {
            let rad = r.real();
            attempt(|| g.init_radius(&p, rad));
        }
        2 => {
            let f = geom_curve(r);
            attempt(|| g.init_section(&p, &f));
        }
        _ => {
            let (f, l) = (geom_curve(r), geom_curve(r));
            attempt(|| g.init_sections(&p, &f, &l));
        }
    }
}

/// `kind ... polynomial withparams`.
pub fn sweepsec_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    let mut g = GeomFillSweepSectionGenerator::new();
    sweepsec_init(&mut g, &mut r, kind);
    let (poly, withp) = (r.flag(), r.flag());
    if let Err(error) = g.perform(poly) {
        print!("{} ", error.exception_type());
        return end();
    }
    let n = g.nb_sections();
    int(n);
    let mut shape = [0; 4];
    show(g.get_shape(), |s| {
        shape = s;
        s.into_iter().for_each(int);
    });
    let (np, nk) = (shape[0], shape[1]);
    let mut k = filled(1, nk, -1.0);
    show(g.knots(&mut k), |()| {
        int(k.lower());
        reals(&k);
    });
    let mut m = filled(1, nk, -1);
    show(g.mults(&mut m), |()| {
        m.as_slice().iter().for_each(|v| int(*v))
    });
    [0, 1, 2, n / 2, n, n + 1]
        .into_iter()
        .for_each(|p| gfg_section(&g, p, np));
    [0, 1, n / 2, n - 1, n]
        .into_iter()
        .for_each(|i| gfg_trsf(&g, i));
    gfg_app(&mut g, withp);
    end();
}

const TRIHEDRONS: [GeomFillTrihedron; 10] = [
    GeomFillTrihedron::IsCorrectedFrenet,
    GeomFillTrihedron::IsFixed,
    GeomFillTrihedron::IsFrenet,
    GeomFillTrihedron::IsConstantNormal,
    GeomFillTrihedron::IsDarboux,
    GeomFillTrihedron::IsGuideAC,
    GeomFillTrihedron::IsGuidePlan,
    GeomFillTrihedron::IsGuideACWithContact,
    GeomFillTrihedron::IsGuidePlanWithContact,
    GeomFillTrihedron::IsDiscreteTrihedron,
];

/// `ga_gfp_pipe`: the pipe of constructor `kind`.
fn gfp_pipe(r: &mut Reader) -> Result<GeomFillPipe, Failure> {
    let kind = r.integer();
    match kind {
        0 => return Ok(GeomFillPipe::new()),
        3 => {
            let p = curve2d(r)?;
            let s = geom_surface(r);
            return GeomFillPipe::on_surface(&p, &s, &geom_curve(r));
        }
        8 => {
            let (p, c1, c2) = (gft_curve(r), gft_curve(r), gft_curve(r));
            return Ok(GeomFillPipe::with_adaptor_rails(&p, &c1, &c2, r.real()));
        }
        _ => {}
    }
    let p = geom_curve(r);
    match kind {
        1 => GeomFillPipe::with_radius(&p, r.real()),
        2 => {
            let c = geom_curve(r);
            GeomFillPipe::with_section(&p, &c, TRIHEDRONS[r.integer() as usize])
        }
        4 => {
            let (f, l) = (geom_curve(r), geom_curve(r));
            GeomFillPipe::with_two_sections(&p, &f, &l)
        }
        5 => {
            let n = r.integer();
            let sections: Vec<_> = (0..n).map(|_| geom_curve(r)).collect();
            GeomFillPipe::with_sections(&p, &sections)
        }
        6 => {
            let c = geom_curve(r);
            GeomFillPipe::with_direction(&p, &c, r.dir())
        }
        7 => {
            let (c1, c2) = (geom_curve(r), geom_curve(r));
            GeomFillPipe::with_rails(&p, &c1, &c2, r.real())
        }
        _ => {
            let (g, c) = (gft_curve(r), geom_curve(r));
            let (acr, rot) = (r.flag(), r.flag());
            GeomFillPipe::with_guide(&p, &g, &c, acr, rot)
        }
    }
}

fn pipe_state(p: &GeomFillPipe) {
    flag(p.is_done());
    int(p.get_status() as i32);
    flag(p.generate_particular_case());
}

/// `ctor... kpart mode tol poly conti degmax nbseg withparams`.
pub fn pipe_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut p = match gfp_pipe(&mut r) {
        Ok(p) => p,
        Err(error) => {
            print!("{} ", error.exception_type());
            return end();
        }
    };
    let (kpart, mode, tol, poly) = (r.flag(), r.integer(), r.real(), r.flag());
    let (conti, degmax, nbseg, withp) = (r.integer(), r.integer(), r.integer(), r.flag());
    pipe_state(&p);
    p.set_generate_particular_case(kpart);
    if mode == 0 {
        attempt(|| p.perform(tol, poly, SHAPES[conti as usize], degmax, nbseg));
    } else {
        attempt(|| p.perform_with_parameters(withp, poly));
    }
    pipe_state(&p);
    flag(p.exchange_uv());
    real(p.error_on_surf());
    surface(&p.surface());
    end();
}

/// `curve1 curve2`: `GeomFill::Surface`.
pub fn surface_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (c1, c2) = (geom_curve(&mut r), geom_curve(&mut r));
    show(rustycad_tkgeomalgo::geom_fill::surface(&c1, &c2), |s| {
        surface(&s)
    });
    end();
}
