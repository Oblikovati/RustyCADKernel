//! TopOpeBRepTool_SolidClassifier, TopOpeBRepTool_ShapeClassifier, FSC_* and
//! TOOL::TggeomE probe operations, the Rust twin of
//! tools/oracle/tkbool_probe_topopebreptool_classifier.cxx (the records are documented
//! there).
use super::brep_fill_generator::failed;
use super::brep_fill_location_law::at;
use super::common::{Reader, attempt, end, flag, int, pnt, pnt2d, xyz};
use super::shapes::{TpShapes, tp_shapes};
use rustycad_tkbool::top_ope_brep_tool::{
    TopOpeBRepToolShapeClassifier, TopOpeBRepToolSolidClassifier, fsc_get_psc,
    fsc_get_psc_with_reference, fsc_state_eonface, fsc_state_ponface, tggeom_e,
};
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::gp::Vector3;

/// The program of a request, `None` (printed) when it fails.
fn program(r: &mut Reader<'_>) -> Option<TpShapes> {
    tp_shapes(r).map_err(failed).ok()
}

/// `tt_solid_classifier program shell solid tol n (x y z)...`.
pub fn solid_classifier_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(t) = program(&mut r) else { return };
    let (ish, iso, tol, n) = (r.integer(), r.integer(), r.real(), r.integer());
    let shapes = at(&t, ish)
        .and_then(|s| topods::shell(&s))
        .and_then(|sh| Ok((sh, topods::solid(&at(&t, iso)?)?)));
    let (sh, so) = match shapes {
        Ok(s) => s,
        Err(error) => return failed(error),
    };
    let mut sc = TopOpeBRepToolSolidClassifier::new();
    for _ in 0..n {
        let p = r.pnt();
        attempt(|| sc.classify_shell(&sh, p, tol).map(|st| int(st as i32)));
        attempt(|| sc.classify_solid(&so, p, tol).map(|st| int(st as i32)));
        int(sc.state() as i32);
    }
    end();
}

/// `bl_tt_sc_result`.
fn sc_result(sc: &TopOpeBRepToolShapeClassifier, st: State) {
    int(st as i32);
    int(sc.state() as i32);
    attempt(|| sc.p3d().map(pnt));
    attempt(|| sc.p2d().map(pnt2d));
}

/// The classifications of `tt_shape_classifier` once the shapes are read.
fn shape_classifications(s: &Shape, rf: &Shape, sd: i32, avs: Option<&Shape>) {
    let mut sc = TopOpeBRepToolShapeClassifier::new();
    attempt(|| {
        let st = sc.state_shape_shape(s, rf, sd)?;
        {
            sc_result(&sc, st);
            Ok(())
        }
    });
    if let Some(avs) = avs {
        attempt(|| {
            let st = sc.state_shape_shape_avoiding(s, avs, rf)?;
            {
                sc_result(&sc, st);
                Ok(())
            }
        });
        let mut l = List::default();
        l.append(avs.clone());
        attempt(|| {
            let st = sc.state_shape_shape_avoiding_list(s, &l, rf)?;
            {
                sc_result(&sc, st);
                Ok(())
            }
        });
    }
    let null = Shape::default();
    attempt(|| {
        sc.set_same_domain(sd);
        sc.set_reference(rf)?;
        let st = sc.state_shape_reference(s, avs.unwrap_or(&null))?;
        {
            sc_result(&sc, st);
            Ok(())
        }
    });
}

/// `tt_shape_classifier program s ref samedomain avoid`.
pub fn shape_classifier_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(t) = program(&mut r) else { return };
    let (is, ir, sd, av) = (r.integer(), r.integer(), r.integer(), r.integer());
    let read = || -> Result<(Shape, Shape, Option<Shape>), Failure> {
        Ok((
            at(&t, is)?,
            at(&t, ir)?,
            if av >= 0 { Some(at(&t, av)?) } else { None },
        ))
    };
    match read() {
        Ok((s, rf, avs)) => shape_classifications(&s, &rf, sd, avs.as_ref()),
        Err(error) => return failed(error),
    }
    end();
}

/// `tt_fsc program e f t x y z`.
pub fn fsc_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(t) = program(&mut r) else { return };
    let (ie, iface, tt, p) = (r.integer(), r.integer(), r.real(), r.pnt());
    let run = || -> Result<(), Failure> {
        let (e, f) = (at(&t, ie)?, at(&t, iface)?);
        attempt(|| fsc_get_psc(|psc| fsc_state_eonface(&e, tt, &f, psc)).map(|st| int(st as i32)));
        attempt(|| fsc_get_psc(|psc| fsc_state_ponface(p, &f, psc)).map(|st| int(st as i32)));
        attempt(|| {
            fsc_get_psc_with_reference(&f, |sc| sc.state_shape_reference(&e, &Shape::default()))?
                .map(|st| int(st as i32))
        });
        Ok(())
    };
    attempt(run);
    end();
}

/// `tt_tggeom program e n par...`.
pub fn tggeom_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(t) = program(&mut r) else { return };
    let (ie, n) = (r.integer(), r.integer());
    let e = match at(&t, ie).and_then(|s| topods::edge(&s)) {
        Ok(e) => e,
        Err(error) => return failed(error),
    };
    for _ in 0..n {
        let par = r.real();
        attempt(|| {
            let tg = tggeom_e(par, &e)?;
            flag(tg.is_some());
            xyz(*tg.unwrap_or(Vector3::new(0., 0., 0.)).xyz());
            Ok(())
        });
    }
    end();
}
