//! Geom2dInt curve-curve probe operations, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_ginter.cxx.
use super::common::{Reader, attempt, end, flag, int, pnt2d, real, show};
use super::curve2d::curve2d;
use super::int_curve::{DomainSpec, domain, read_domain};
use super::int_res2d::inter;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeomalgo::geom2d_api::InterCurveCurve;
use rustycad_tkgeomalgo::geom2d_int::{
    ExactIntersectionPointOfTheIntPCurvePCurveOfGInter, GInter, TheIntPCurvePCurveOfGInter,
    ThePolygon2dOfTheIntPCurvePCurveOfGInter as Poly,
};
use rustycad_tkgeomalgo::int_res2d::Domain;
use rustycad_tkgeomalgo::intf::Polygon2d;
use rustycad_tkmath::bnd::BoundingBox2;

fn curve_domain(s: &DomainSpec, a: &Geom2dAdaptorCurve) -> Result<Domain, Failure> {
    domain(s, |u| a.value(u))
}

fn box2d(b: &BoundingBox2) -> Result<(), Failure> {
    flag(b.is_void());
    if b.is_void() {
        return Ok(());
    }
    for v in b.get()? {
        real(v);
    }
    Ok(())
}

// Every public member of the polygon, on the indices whose arrays are defined natively.
fn poly(p: &Poly) -> Result<(), Failure> {
    let nbseg: i32 = p.nb_segments();
    let npin: i32 = if p.closed() { nbseg } else { nbseg + 1 };
    real(p.deflection_over_estimation());
    int(nbseg);
    flag(p.closed());
    box2d(p.bounding())?;
    show(p.inf_parameter(), real);
    show(p.sup_parameter(), real);
    show(p.auto_intersection_is_possible(), flag);
    for i in 0..=npin {
        show(p.segment(i), |(b, e)| {
            pnt2d(b);
            pnt2d(e);
        });
    }
    for i in 0..npin {
        show(p.approx_param_on_curve(i, 0.5), real);
    }
    show(p.approx_param_on_curve(npin, 0.0), real);
    int(p.calcul_region(-2., 3., -1., 1., -1., 1.));
    int(p.calcul_region(0.5, -3., -1., 1., -1., 1.));
    Ok(())
}

/// `curve domain n tol curve2 domain2 n2 tol2 deflection closed`: the polygon members.
pub fn polygon_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let c = curve2d(&mut r)?;
        let s = read_domain(&mut r);
        let (n, tol) = (r.integer(), r.real());
        let c2 = curve2d(&mut r)?;
        let s2 = read_domain(&mut r);
        let (n2, tol2) = (r.integer(), r.real());
        let (dfl, closed) = (r.real(), r.flag());
        let (a, a2) = (adaptor(&c)?, adaptor(&c2)?);
        let mut p = Poly::new(&a, n, &curve_domain(&s, &a)?, tol)?;
        poly(&p)?;
        attempt(|| {
            let q = Poly::new(&a2, n2, &curve_domain(&s2, &a2)?, tol2)?;
            box2d(q.bounding())?;
            p.compute_with_box(&a, q.bounding())?;
            poly(&p)
        });
        p.set_deflection_over_estimation(dfl);
        p.set_closed(closed);
        poly(&p)?;
        p.dump();
        Ok(())
    });
    end();
}

fn adaptor(c: &Handle<dyn Geom2dCurve>) -> Result<Geom2dAdaptorCurve, Failure> {
    Geom2dAdaptorCurve::with_curve(c)
}

fn eip_out(e: &ExactIntersectionPointOfTheIntPCurvePCurveOfGInter<'_>) {
    int(e.nb_roots());
    if e.nb_roots() > 0 {
        let (u, v) = e.roots();
        real(u);
        real(v);
    }
    flag(e.an_error_occurred());
}

/// `curve1 domain1 n1 curve2 domain2 n2 tol seg1 par1 seg2 par2 uo vo uinf vinf usup vsup`.
pub fn exact_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let c1 = curve2d(&mut r)?;
        let s1 = read_domain(&mut r);
        let n1 = r.integer();
        let c2 = curve2d(&mut r)?;
        let s2 = read_domain(&mut r);
        let (n2, tol) = (r.integer(), r.real());
        let mut seat1 = (r.integer(), r.real());
        let mut seat2 = (r.integer(), r.real());
        let (uo, vo, ui, vi, us, vs) = (r.real(), r.real(), r.real(), r.real(), r.real(), r.real());
        let (a1, a2) = (adaptor(&c1)?, adaptor(&c2)?);
        let p1 = Poly::new(&a1, n1, &curve_domain(&s1, &a1)?, tol)?;
        let p2 = Poly::new(&a2, n2, &curve_domain(&s2, &a2)?, tol)?;
        let mut e = ExactIntersectionPointOfTheIntPCurvePCurveOfGInter::new(&a1, &a2, tol)?;
        attempt(|| {
            e.perform_polygons(&p1, &p2, &mut seat1, &mut seat2)?;
            int(seat1.0);
            real(seat1.1);
            int(seat2.0);
            real(seat2.1);
            eip_out(&e);
            Ok(())
        });
        attempt(|| {
            e.perform(uo, vo, ui, vi, us, vs)?;
            eip_out(&e);
            Ok(())
        });
        Ok(())
    });
    end();
}

/// `curve1 domain1 curve2 domain2 tolconf tol minnb reversed mode nbiter`.
pub fn polypoly_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let c1 = curve2d(&mut r)?;
        let s1 = read_domain(&mut r);
        let c2 = curve2d(&mut r)?;
        let s2 = read_domain(&mut r);
        let (tc, t, minnb, rev) = (r.real(), r.real(), r.integer(), r.flag());
        let (mode, nbiter) = (r.integer(), r.integer());
        let (a1, a2) = (adaptor(&c1)?, adaptor(&c2)?);
        let (d1, d2) = (curve_domain(&s1, &a1)?, curve_domain(&s2, &a2)?);
        let mut i = TheIntPCurvePCurveOfGInter::new();
        int(i.get_min_nb_samples());
        i.set_min_nb_samples(minnb);
        int(i.get_min_nb_samples());
        i.set_reversed_parameters(rev);
        match mode {
            0 => i.perform(&a1, &d1, &a2, &d2, tc, t)?,
            1 => i.perform_self(&a1, &d1, tc, t)?,
            2 => i.perform_iter(&a1, &d1, &a2, &d2, tc, t, nbiter, 1., 1.)?,
            _ => i.perform_self_iter(&a1, &d1, tc, t, nbiter, 1., 1.)?,
        }
        inter(&i)
    });
    end();
}

fn domain_out(d: &Domain) -> Result<(), Failure> {
    flag(d.has_first_point());
    if d.has_first_point() {
        real(d.first_parameter()?);
        pnt2d(d.first_point()?);
        real(d.first_tolerance()?);
    }
    flag(d.has_last_point());
    if d.has_last_point() {
        real(d.last_parameter()?);
        pnt2d(d.last_point()?);
        real(d.last_tolerance()?);
    }
    flag(d.is_closed());
    if d.is_closed() {
        let (a, b) = d.equivalent_parameters();
        real(a);
        real(b);
    }
    Ok(())
}

/// `curve1 domain1 curve2 domain2 tolconf tol minnb reversed mode ctor`: Geom2dInt_GInter.
pub fn ginter_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let c1 = curve2d(&mut r)?;
        let s1 = read_domain(&mut r);
        let c2 = curve2d(&mut r)?;
        let s2 = read_domain(&mut r);
        let (tc, t, minnb, rev) = (r.real(), r.real(), r.integer(), r.flag());
        let (mode, ctor) = (r.integer(), r.flag());
        let (a1, a2) = (adaptor(&c1)?, adaptor(&c2)?);
        let (d1, d2) = (curve_domain(&s1, &a1)?, curve_domain(&s2, &a2)?);
        let (c1, c2): (&dyn Adaptor2dCurve2d, &dyn Adaptor2dCurve2d) = (&a1, &a2);
        if ctor {
            attempt(|| {
                let i = match mode {
                    0 => GInter::between(c1, c2, tc, t)?,
                    1 => GInter::with_first_domain(c1, &d1, c2, tc, t)?,
                    2 => GInter::with_second_domain(c1, c2, &d2, tc, t)?,
                    3 => GInter::with_domains(c1, &d1, c2, &d2, tc, t)?,
                    4 => GInter::of_self(c1, tc, t)?,
                    _ => GInter::of_self_on(c1, &d1, tc, t)?,
                };
                inter(&i)
            });
            return Ok(());
        }
        let mut i = GInter::new();
        int(i.get_min_nb_samples());
        i.set_min_nb_samples(minnb);
        int(i.get_min_nb_samples());
        i.set_reversed_parameters(rev);
        attempt(|| {
            match mode {
                0 => i.perform_between(c1, c2, tc, t)?,
                1 => i.perform_first_domain(c1, &d1, c2, tc, t)?,
                2 => i.perform_second_domain(c1, c2, &d2, tc, t)?,
                3 => i.perform(c1, &d1, c2, &d2, tc, t)?,
                4 => i.perform_self(c1, tc, t)?,
                _ => i.perform_self_on(c1, &d1, tc, t)?,
            }
            inter(&i)
        });
        attempt(|| domain_out(&i.compute_domain(c1, tc)?));
        attempt(|| domain_out(&i.compute_domain(c2, t)?));
        Ok(())
    });
    end();
}

fn trimmed_out(c: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
    let curve = c.get().expect("a trimmed curve");
    real(curve.first_parameter());
    real(curve.last_parameter());
    show(curve.eval_d0(curve.first_parameter()), pnt2d);
    show(curve.eval_d0(curve.last_parameter()), pnt2d);
    Ok(())
}

/// `curve1 self curve2 tol`: Geom2dAPI_InterCurveCurve.
pub fn intercc_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let c1 = curve2d(&mut r)?;
        let self_ = r.flag();
        let c2 = curve2d(&mut r)?;
        let tol = r.real();
        let mut e = InterCurveCurve::new();
        int(e.nb_points());
        int(e.nb_segments());
        if self_ {
            e.init_self(&c1, tol)?;
        } else {
            e.init(&c1, &c2, tol)?;
        }
        let np = e.nb_points();
        int(np);
        for i in 0..=np + 1 {
            show(e.point(i), pnt2d);
        }
        let ns = e.nb_segments();
        int(ns);
        for i in 0..=ns + 1 {
            attempt(|| {
                let (t1, t2) = e.segment(i)?;
                trimmed_out(&t1)?;
                trimmed_out(&t2)
            });
        }
        inter(e.intersector())
    });
    end();
}
