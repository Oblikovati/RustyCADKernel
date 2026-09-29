//! Geom2dHatch probe operations, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_geom2dhatch.cxx (the hatcher is in geom2d_hatcher.rs).
use super::common::{Reader, attempt, end, flag, int, pnt2d, real, show};
use super::curve2d::curve2d;
use super::gcc_ent::{dir2d, lin2d};
use super::int_res2d::inter;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom2d_hatch::{
    Classifier, Element, Elements, FClass2dOfClassifier, Intersector,
};
use rustycad_tkmath::gp::{Direction2, Line2, Point2, Vector2};

/// `curve u0 u1`: the adaptor bounded to [u0, u1] when u0 < u1, else on the whole curve.
pub fn hatch_adaptor(r: &mut Reader<'_>) -> Result<Geom2dAdaptorCurve, Failure> {
    let c = curve2d(r)?;
    let (u0, u1) = (r.real(), r.real());
    adaptor_of(&c, u0, u1)
}
pub fn adaptor_of(
    c: &Handle<dyn Geom2dCurve>,
    u0: f64,
    u1: f64,
) -> Result<Geom2dAdaptorCurve, Failure> {
    if u0 < u1 {
        return Geom2dAdaptorCurve::with_range(c, u0, u1);
    }
    Geom2dAdaptorCurve::with_curve(c)
}
pub fn orient(r: &mut Reader<'_>) -> Orientation {
    Orientation::ALL[r.integer() as usize]
}
pub fn adaptor_out(a: &Geom2dAdaptorCurve) {
    flag(!a.curve().is_null());
    if a.curve().is_null() {
        return;
    }
    real(a.first_parameter().expect("bound"));
    real(a.last_parameter().expect("bound"));
}

/// `curve u0 u1 u`: LocalGeometry at `u`.
pub fn local_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let a = hatch_adaptor(&mut r)?;
        let u = r.real();
        let (t, n, c) = Intersector::new(1e-7, 1e-7).local_geometry(&a, u)?;
        dir2d(t);
        dir2d(n);
        real(c);
        Ok(())
    });
    end();
}

/// `line p tol curve u0 u1`: Perform of the segment, a copy and an assigned intersector.
pub fn perform_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let l = r.lin2d();
        let (p, tol) = (r.real(), r.real());
        let a = hatch_adaptor(&mut r)?;
        let mut i = Intersector::default();
        real(i.confusion_tolerance());
        real(i.tangency_tolerance());
        i.set_reversed_parameters(true);
        attempt(|| {
            i.perform(&l, p, tol, &a)?;
            inter(&i)
        });
        inter(&i.clone())?;
        let mut s = Intersector::new(1.0, 2.0);
        s.assign(&i);
        inter(&s)?;
        real(s.confusion_tolerance());
        real(s.tangency_tolerance());
        Ok(())
    });
    end();
}

/// `curve1 u0 u1 curve2 v0 v1 conf tang reversed`: Intersect before and after the setters.
pub fn intersect_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let (a1, a2) = (hatch_adaptor(&mut r)?, hatch_adaptor(&mut r)?);
        let (conf, tang, rev) = (r.real(), r.real(), r.flag());
        let mut i = Intersector::new(conf, tang);
        i.set_reversed_parameters(rev);
        attempt(|| {
            i.intersect(&a1, &a2)?;
            inter(&i)
        });
        i.set_confusion_tolerance(conf * 10.);
        i.set_tangency_tolerance(tang * 10.);
        real(i.confusion_tolerance());
        real(i.tangency_tolerance());
        attempt(|| {
            i.intersect(&a1, &a2)?;
            inter(&i)
        });
        Ok(())
    });
    end();
}

/// `n (key orientation curve u0 u1)*n`: the elements, bound in order; the largest key.
pub fn read_elements(r: &mut Reader<'_>, e: &mut Elements) -> Result<i32, Failure> {
    let mut maxkey = 0;
    for _ in 0..r.integer() {
        let key = r.integer();
        let o = orient(r);
        let a = hatch_adaptor(r)?;
        flag(e.bind(key, Element::new(a, o)));
        maxkey = maxkey.max(key);
    }
    Ok(maxkey)
}
fn segment_out((ok, l, par): (bool, Line2, f64)) {
    flag(ok);
    lin2d(l);
    real(par);
}
fn elements_edges(e: &mut Elements) {
    e.init_wires();
    flag(e.more_wires());
    e.next_wire();
    flag(e.more_wires());
    e.init_edges();
    while e.more_edges() {
        let (c, o) = e.current_edge().expect("current edge");
        adaptor_out(&c);
        int(o as i32);
        e.next_edge();
    }
    attempt(|| e.current_edge().map(|_| ()));
    e.next_edge();
    flag(e.more_edges());
}

/// `elements px py nseg`: the explorer members, the segments, the map, the copy.
pub fn elements_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let mut e = Elements::new();
        attempt(|| e.current_edge().map(|_| ()));
        flag(e.more_edges());
        let maxkey = read_elements(&mut r, &mut e)?;
        let p = r.pnt2d();
        let nseg = r.integer();
        elements_edges(&mut e);
        explorer_members(&mut e, p, nseg)?;
        map_members(&mut e, p, maxkey)?;
        Ok(())
    });
    end();
}
fn explorer_members(e: &mut Elements, p: Point2, nseg: i32) -> Result<(), Failure> {
    let mut q = p;
    flag(e.check_point(&mut q));
    pnt2d(q);
    flag(e.reject(&p));
    let mut seg = e.segment(&p)?;
    segment_out(seg);
    flag(e.reject_wire(&seg.1, seg.2));
    flag(e.reject_edge(&seg.1, seg.2));
    let mut k = 0;
    while k < nseg && seg.0 {
        seg = e.other_segment(&p)?;
        segment_out(seg);
        k += 1;
    }
    Ok(())
}
fn map_members(e: &mut Elements, p: Point2, maxkey: i32) -> Result<(), Failure> {
    for key in 0..=maxkey + 1 {
        flag(e.is_bound(key));
        attempt(|| {
            int(e.find(key)?.orientation() as i32);
            adaptor_out(e.change_find(key)?.change_curve());
            Ok(())
        });
    }
    attempt(|| {
        e.change_find(maxkey)?
            .set_orientation(Orientation::Reversed);
        int(e.find(maxkey)?.orientation() as i32);
        Ok(())
    });
    let mut c = Elements::copy_of(e);
    flag(c.is_bound(maxkey));
    segment_out(c.other_segment(&p)?);
    element_members(e, maxkey);
    flag(e.unbind(maxkey));
    flag(e.unbind(maxkey));
    elements_edges(e);
    e.clear();
    flag(e.is_bound(1));
    elements_edges(e);
    Ok(())
}

// The default element, then an element on the curve of `maxkey` with its default
// orientation changed.
fn element_members(e: &Elements, maxkey: i32) {
    adaptor_out(Element::default().curve());
    attempt(|| {
        let curve = e.find(maxkey)?.curve().shallow_adaptor()?;
        let mut f = Element::new(curve, Orientation::Forward);
        int(f.orientation() as i32);
        f.set_orientation(Orientation::Internal);
        int(f.orientation() as i32);
        adaptor_out(f.change_curve());
        Ok(())
    });
}

/// A unit square (FORWARD, counterclockwise) around (0.5, 0.5).
pub fn square_elements(e: &mut Elements) -> Result<(), Failure> {
    let (x, y) = ([0., 1., 1., 0., 0.], [0., 0., 1., 1., 0.]);
    for k in 0..4 {
        let (a, b) = (Point2::new(x[k], y[k]), Point2::new(x[k + 1], y[k + 1]));
        let dir = Direction2::from_vector(Vector2::between(a, b))?;
        let l: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(a, dir)).cast();
        let adaptor = Geom2dAdaptorCurve::with_range(&l, 0., 1.)?;
        e.bind(k as i32 + 1, Element::new(adaptor, Orientation::Forward));
    }
    Ok(())
}
fn classifier_out(c: &Classifier) {
    int(c.state() as i32);
    flag(c.rejected());
    flag(c.no_wires());
    show(c.edge(), adaptor_out);
    show(c.edge_parameter(), real);
    int(c.position() as i32);
}

/// `elements px py tol`: the primed classifier, the default one and the constructor.
pub fn classify_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let mut e = Elements::new();
        read_elements(&mut r, &mut e)?;
        let (p, tol) = (r.pnt2d(), r.real());
        let d = Classifier::new();
        int(d.state() as i32);
        flag(d.rejected());
        flag(d.no_wires());
        show(d.edge_parameter(), real);
        let mut c = Classifier::new();
        let mut square = Elements::new();
        square_elements(&mut square)?;
        c.perform(&mut square, &Point2::new(0.5, 0.5), 1e-7)?;
        classifier_out(&c);
        attempt(|| {
            c.perform(&mut e, &p, tol)?;
            classifier_out(&c);
            Ok(())
        });
        attempt(|| {
            let mut e2 = Elements::new();
            square_elements(&mut e2)?;
            let k = Classifier::with_point(&mut e2, &p, tol)?;
            int(k.state() as i32);
            flag(k.no_wires());
            Ok(())
        });
        Ok(())
    });
    end();
}

/// `line par tol n (orientation curve u0 u1)*n`: FClass2d Reset then Compare.
pub fn fclass_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let l = r.lin2d();
        let (par, tol, n) = (r.real(), r.real(), r.integer());
        let mut f = FClass2dOfClassifier::new();
        real(f.parameter());
        int(f.closest_intersection());
        int(f.state() as i32);
        flag(f.is_head_or_end());
        f.reset(l, par, tol);
        real(f.parameter());
        int(f.state() as i32);
        for _ in 0..n {
            let o = orient(&mut r);
            let a = hatch_adaptor(&mut r)?;
            attempt(|| {
                f.compare(&a, o)?;
                int(f.closest_intersection());
                int(f.state() as i32);
                flag(f.is_head_or_end());
                real(f.parameter());
                flag(f.intersector().is_done());
                Ok(())
            });
        }
        Ok(())
    });
    end();
}
