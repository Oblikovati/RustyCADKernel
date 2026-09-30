//! The static members of GeomInt and GeomInt_IntSS and GeomInt_ParameterAndOrientation, the
//! Rust twin of tools/oracle/tkgeomalgo_probe_geomint_statics.cxx.
use super::common::{Reader, attempt, end, flag, int, real};
use super::curve2d::curve2d;
use super::geom_int::{gi_curve, gi_curve2d};
use super::ics_geometry::{geom_curve, geom_surface, surface};
use super::int_patch::Fixture;
use super::int_patch_wltool::line;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom_int::{
    GeomIntIntSS, GeomIntParameterAndOrientation, GeomIntRLineCurves, adjust_periodic,
};
use rustycad_tkgeomalgo::int_patch::{IntPatchRLine, IntPatchWLine};
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::{Direction2, Point2};

/// `ga_gi_pcurves_case`.
pub fn pcurves_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let c = geom_curve(&mut r);
        let s = geom_surface(&mut r);
        let (f, l) = (r.real(), r.real());
        let mut tol = r.real();
        let bounded = r.flag();
        let b = if bounded {
            [r.real(), r.real(), r.real(), r.real()]
        } else {
            [0.0; 4]
        };
        let mut c2d: Handle<dyn Geom2dCurve> = if r.flag() {
            curve2d(&mut r)?
        } else {
            Handle::default()
        };
        if bounded {
            attempt(|| {
                GeomIntIntSS::build_pcurves_bounds(
                    f,
                    l,
                    (b[0], b[1], b[2], b[3]),
                    &mut tol,
                    &s,
                    &c,
                    &mut c2d,
                )
            });
        } else {
            attempt(|| GeomIntIntSS::build_pcurves(f, l, &mut tol, &s, &c, &mut c2d));
        }
        gi_curve2d(&c2d)?;
        real(tol);
        Ok(())
    });
    end();
}

/// `ga_gi_trim_case`.
pub fn trim_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let c1: Handle<dyn Geom2dCurve> = if r.flag() {
            curve2d(&mut r)?
        } else {
            Handle::default()
        };
        let c2: Handle<dyn Geom2dCurve> = if r.flag() {
            curve2d(&mut r)?
        } else {
            Handle::default()
        };
        let (mut b1, mut b2) = (BoundingBox2::default(), BoundingBox2::default());
        for b in [&mut b1, &mut b2] {
            let p = r.pnt2d();
            b.add_point(p);
            let q = r.pnt2d();
            b.add_point(q);
        }
        let mut prm: Vec<f64> = (0..r.integer()).map(|_| r.real()).collect();
        attempt(|| GeomIntIntSS::trim_iline_on_surf_boundaries(&c1, &c2, &b1, &b2, &mut prm));
        int(prm.len() as i32);
        prm.into_iter().for_each(real);
        Ok(())
    });
    end();
}

/// `ga_gi_mkbsp_case`.
pub fn mkbsp_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let fx = Fixture::new();
    attempt(|| {
        let w: Handle<IntPatchWLine> = line(&mut r, &fx)?.cast();
        let (a, b) = (r.integer(), r.integer());
        attempt(|| gi_curve(&GeomIntIntSS::make_bspline(&w, a, b)?));
        attempt(|| gi_curve2d(&GeomIntIntSS::make_bspline2d(&w, a, b, true)?.cast()));
        attempt(|| gi_curve2d(&GeomIntIntSS::make_bspline2d(&w, a, b, false)?.cast()));
        Ok(())
    });
    end();
}

/// `ga_gi_rline_case`.
pub fn rline_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let fx = Fixture::new();
    attempt(|| {
        let (s1, s2) = (surface(&mut r), surface(&mut r));
        let l: Handle<IntPatchRLine> = line(&mut r, &fx)?.cast();
        let (p1, p2) = (r.flag(), r.flag());
        let mut out = GeomIntRLineCurves {
            tol_reached: -1.0,
            ..Default::default()
        };
        if p1 {
            out.c2d1 = Handle::new(Geom2dLine::from_point(
                Point2::new(0.25, 0.0),
                Direction2::Y,
            ))
            .cast();
        }
        if p2 {
            out.c2d2 = Handle::new(Geom2dLine::from_point(
                Point2::new(0.0, 0.25),
                Direction2::X,
            ))
            .cast();
        }
        attempt(|| GeomIntIntSS::treat_rline(&l, &s1.cast(), &s2.cast(), &mut out));
        gi_curve(&out.c3d)?;
        gi_curve2d(&out.c2d1)?;
        gi_curve2d(&out.c2d2)?;
        real(out.tol_reached);
        Ok(())
    });
    end();
}

/// `ga_gi_adjust_case`.
pub fn adjust_case(values: &[f64]) {
    let mut r = Reader::new(values);
    for _ in 0..r.integer() {
        let (p, a, b, t, e) = (r.real(), r.real(), r.real(), r.real(), r.real());
        let (adjusted, new_par, offset) = adjust_periodic(p, a, b, t, e);
        flag(adjusted);
        real(new_par);
        real(offset);
    }
    end();
}

fn orientation(k: i32) -> Orientation {
    Orientation::ALL[k as usize]
}

/// `ga_gi_porient_case`.
pub fn porient_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let p = r.real();
    let (a, b, c, d) = (r.integer(), r.integer(), r.integer(), r.integer());
    let e = GeomIntParameterAndOrientation::new();
    real(e.parameter());
    int(e.orientation1() as i32);
    int(e.orientation2() as i32);
    let mut v = GeomIntParameterAndOrientation::with_values(p, orientation(a), orientation(b));
    real(v.parameter());
    int(v.orientation1() as i32);
    int(v.orientation2() as i32);
    v.set_orientation1(orientation(c));
    v.set_orientation2(orientation(d));
    int(v.orientation1() as i32);
    int(v.orientation2() as i32);
    end();
}
