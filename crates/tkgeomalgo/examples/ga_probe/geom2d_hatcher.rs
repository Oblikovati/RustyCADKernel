//! Geom2dHatch_Hatcher and Geom2dHatch_Hatching probe operations, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_geom2dhatcher.cxx.
use super::common::{Reader, attempt, captured, end, flag, int, pnt2d, real, show};
use super::curve2d::curve2d;
use super::geom2d_hatch::{adaptor_of, adaptor_out, hatch_adaptor, orient};
use super::hatch_gen::{domain_out, ph};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkgeomalgo::geom2d_hatch::{Hatcher, Hatching, Intersector};
use rustycad_tkgeomalgo::hatch_gen::{Domain, ErrorStatus, PointOnElement, PointOnHatching};
use rustycad_tkmath::gp::{Direction2, Point2};

fn hatching_out(h: &Hatcher, i: i32) {
    show(h.hatching_curve(i), adaptor_out);
    attempt(|| {
        flag(h.trim_done(i)?);
        flag(h.trim_failed(i)?);
        flag(h.is_done(i)?);
        int(h.status(i)? as i32);
        let np = h.nb_points(i)?;
        int(np);
        for k in 0..=np + 1 {
            show(h.point(i, k), ph);
        }
        Ok(())
    });
    attempt(|| {
        let nd = h.nb_domains(i)?;
        int(nd);
        for k in 0..=nd + 1 {
            show(h.domain(i, k), domain_out);
        }
        Ok(())
    });
}
fn report(h: &Hatcher, ne: i32, nh: i32) {
    real(h.confusion2d());
    real(h.confusion3d());
    flag(h.keep_points());
    flag(h.keep_segments());
    for i in 0..=ne + 1 {
        show(h.element_curve(i), adaptor_out);
    }
    for i in 0..=nh + 1 {
        hatching_out(h, i);
    }
    captured(|text| {
        h.dump(text).expect("text");
        Ok(())
    });
}

// One operation `op value`; the index added by 14 and 15, else -1.
fn hatcher_op(h: &mut Hatcher, op: i32, v: f64) -> Result<i32, Failure> {
    let i = v as i32;
    let x_axis = Direction2::new(1., 0.)?;
    let line: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::from_point(Point2::new(0., v), x_axis)).cast();
    match op {
        0 => h.trim()?,
        1 => h.compute_domains()?,
        2 => h.trim_hatching(i)?,
        3 => h.compute_domains_of(i)?,
        4 => h.rem_element(i),
        5 => h.rem_hatching(i)?,
        6 => h.set_confusion2d(v),
        7 => h.set_confusion3d(v),
        8 => h.set_keep_points(v != 0.),
        9 => h.set_keep_segments(v != 0.),
        10 => h.set_intersector(&Intersector::new(v, v)),
        11 => h.clear(),
        12 => h.clr_elements(),
        13 => h.clr_hatchings(),
        14 => return Ok(h.add_hatching(Geom2dAdaptorCurve::with_curve(&line)?)),
        15 => return h.trim_curve(Geom2dAdaptorCurve::with_curve(&line)?),
        16 => h.element_mut(i)?.set_orientation(Orientation::Reversed),
        _ => h.hatching_mut(i)?.set_trim_done(false),
    }
    Ok(-1)
}

/// `conf2d conf3d keepP keepS ne elements nh hatchings nops ops`: a hatcher and its report.
pub fn hatcher_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let (c2, c3, kp, ks) = (r.real(), r.real(), r.flag(), r.flag());
        let mut h = Hatcher::new(&Intersector::new(1e-7, 1e-7), c2, c3, kp, ks);
        let ne = r.integer();
        for _ in 0..ne {
            let o = orient(&mut r);
            let c = curve2d(&mut r)?;
            let (u0, u1) = (r.real(), r.real());
            if u0 > u1 {
                int(h.add_element_curve(&c, o)?);
            } else {
                int(h.add_element(adaptor_of(&c, u0, u1)?, o));
            }
        }
        let nh = r.integer();
        for _ in 0..nh {
            int(h.add_hatching(hatch_adaptor(&mut r)?));
        }
        for _ in 0..r.integer() {
            let (op, v) = (r.integer(), r.real());
            attempt(|| {
                let k = hatcher_op(&mut h, op, v)?;
                if k >= 0 {
                    int(k);
                }
                Ok(())
            });
        }
        real(h.intersector().confusion_tolerance());
        real(h.change_intersector().tangency_tolerance());
        report(&h, ne, nh);
        Ok(())
    });
    end();
}

fn read_hatching_point(r: &mut Reader<'_>, conf: f64) -> PointOnHatching {
    let mut p = PointOnHatching::new();
    p.set_parameter(r.real());
    p.set_position(orient(r));
    p.set_state_before(State::ALL[r.integer() as usize]);
    p.set_state_after(State::ALL[r.integer() as usize]);
    p.set_segment_beginning(r.flag());
    p.set_segment_end(r.flag());
    for _ in 0..r.integer() {
        let mut e = PointOnElement::new();
        e.set_index(r.integer());
        e.set_parameter(r.real());
        p.add_point(&e, conf);
    }
    p
}

/// `curve u0 u1 conf n points rem`: the Hatching members.
pub fn hatching_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let mut h = Hatching::new(hatch_adaptor(&mut r)?);
        let (conf, n) = (r.real(), r.integer());
        for _ in 0..n {
            let p = read_hatching_point(&mut r, conf);
            h.add_point(&p, conf);
        }
        let rem = r.integer();
        flags_out(&h);
        let np = h.nb_points();
        int(np);
        for k in 0..=np + 1 {
            show(h.point(k), ph);
        }
        status_members(&mut h);
        domain_members(&mut h, np, rem);
        Ok(())
    });
    end();
}
fn flags_out(h: &Hatching) {
    flag(h.trim_done());
    flag(h.trim_failed());
    flag(h.is_done());
    int(h.status() as i32);
}
fn status_members(h: &mut Hatching) {
    h.set_trim_done(true);
    h.set_trim_failed(true);
    int(h.status() as i32);
    h.set_trim_failed(false);
    int(h.status() as i32);
    h.set_status(ErrorStatus::IncoherentParity);
    int(h.status() as i32);
    h.set_is_done(true);
}
fn domain_members(h: &mut Hatching, np: i32, rem: i32) {
    for k in 1..=np {
        attempt(|| {
            let domain = Domain::semi_infinite(h.point(k)?, k % 2 == 1);
            h.add_domain(domain);
            Ok(())
        });
    }
    h.add_domain(Domain::new());
    let nd = h.nb_domains();
    int(nd);
    for k in 0..=nd + 1 {
        show(h.domain(k), domain_out);
    }
    attempt(|| h.rem_domain(1));
    attempt(|| h.rem_domain(nd + 5));
    int(h.nb_domains());
    attempt(|| h.rem_point(rem));
    flag(h.is_done());
    int(h.nb_domains());
    int(h.nb_points());
    attempt(|| {
        h.change_point(1)?.set_parameter(-7.);
        ph(h.point(1)?);
        Ok(())
    });
    h.set_is_done(true);
    h.add_domain(Domain::new());
    h.clr_points();
    flag(h.trim_done());
    flag(h.trim_failed());
    flag(h.is_done());
    int(h.nb_domains());
    int(h.nb_points());
    int(h.status() as i32);
    show(h.classification_point(), pnt2d);
    adaptor_out(h.curve());
    adaptor_out(Hatching::default().curve());
}
