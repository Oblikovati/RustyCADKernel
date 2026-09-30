//! GeomFill sweep probe operations (the GeomFill class, the arc convertors and the
//! tensor), the Rust twin of tools/oracle/tkgeomalgo_probe_geomfill_sweep.cxx.
use super::common::{Reader, end, flag, int, pnt, real, show, xyz};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::geom_fill::*;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkmath::math::{Matrix, Vector};

pub fn conv_of(code: i32) -> ParameterisationType {
    use ParameterisationType::*;
    [
        TgtThetaOver2,
        TgtThetaOver2_1,
        TgtThetaOver2_2,
        TgtThetaOver2_3,
        TgtThetaOver2_4,
        QuasiAngular,
        RationalC1,
        Polynomial,
    ][code as usize]
}

pub fn pnts(a: &Array1<'_, Point3>) {
    a.as_slice().iter().for_each(|p| pnt(*p));
}
pub fn vecs(a: &Array1<'_, Vector3>) {
    a.as_slice().iter().for_each(|v| xyz(*v.xyz()));
}
pub fn reals(a: &Array1<'_, f64>) {
    a.as_slice().iter().for_each(|v| real(*v));
}
pub fn vec(r: &mut Reader) -> Vector3 {
    Vector3::from(r.xyz())
}
/// An array `lower..=upper` filled with `value`.
pub fn filled<T: Clone + Default>(lower: i32, upper: i32, value: T) -> Array1<'static, T> {
    let mut a: Array1<'static, T> = Array1::with_bounds(lower, upper).expect("probe array");
    a.init(value);
    a
}
fn show_err(result: Result<(), Failure>) {
    show(result, |()| ());
}

/// `max_ang conv lower n min_ang radius ang_tol sp_tol`.
pub fn shape_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (max_ang, mut conv) = (r.real(), conv_of(r.integer()));
    let (lower, n) = (r.integer(), r.integer());
    let (min_ang, radius, ang_tol, sp_tol) = (r.real(), r.real(), r.real(), r.real());
    let (np, nk, deg) = get_shape(max_ang, &mut conv);
    [np, nk, deg, conv as i32].into_iter().for_each(int);
    let mut k = filled(lower, lower + n - 1, -1.0);
    show_err(knots(conv, &mut k));
    reals(&k);
    let mut m = filled(lower, lower + n - 1, -1);
    show_err(mults(conv, &mut m));
    m.as_slice().iter().for_each(|v| int(*v));
    let mut w = filled(1, if np > 0 { np } else { 1 }, -1.0);
    show(get_minimal_weights(conv, min_ang, max_ang, &mut w), |()| {
        int(w.lower());
        int(w.upper());
        reals(&w);
    });
    show(get_tolerance(conv, min_ang, radius, ang_tol, sp_tol), real);
    end();
}

fn read_circle(r: &mut Reader) -> CircleD2 {
    let (ns1, ns2, nplan) = (vec(r), vec(r), vec(r));
    let (pts1, pts2, rayon, center) = (r.pnt(), r.pnt(), r.real(), r.pnt());
    let circle = CircleD0 {
        ns1,
        ns2,
        nplan,
        pts1,
        pts2,
        rayon,
        center,
    };
    let (dn1w, dn2w, dnplan, tang1, tang2) = (vec(r), vec(r), vec(r), vec(r), vec(r));
    let (d_rayon, d_center) = (r.real(), vec(r));
    let d1 = CircleD1 {
        circle,
        dn1w,
        dn2w,
        dnplan,
        tang1,
        tang2,
        d_rayon,
        d_center,
    };
    let (d2n1w, d2n2w, d2nplan, dtang1, dtang2) = (vec(r), vec(r), vec(r), vec(r), vec(r));
    let (d2_rayon, d2_center) = (r.real(), vec(r));
    CircleD2 {
        d1,
        d2n1w,
        d2n2w,
        d2nplan,
        dtang1,
        dtang2,
        d2_rayon,
        d2_center,
    }
}

/// The six output arrays of the D2 members, reset to -1.
pub struct Outputs {
    pub p: Array1<'static, Point3>,
    pub dp: Array1<'static, Vector3>,
    pub d2p: Array1<'static, Vector3>,
    pub w: Array1<'static, f64>,
    pub dw: Array1<'static, f64>,
    pub d2w: Array1<'static, f64>,
}
impl Outputs {
    pub fn new(lower: i32, upper: i32) -> Self {
        let neg = Vector3::new(-1.0, -1.0, -1.0);
        Self {
            p: filled(lower, upper, Point3::new(-1.0, -1.0, -1.0)),
            dp: filled(lower, upper, neg),
            d2p: filled(lower, upper, neg),
            w: filled(lower, upper, -1.0),
            dw: filled(lower, upper, -1.0),
            d2w: filled(lower, upper, -1.0),
        }
    }
    pub fn d1(&mut self) -> ArcPolesD1<'_, 'static> {
        ArcPolesD1 {
            poles: &mut self.p,
            d_poles: &mut self.dp,
            weights: &mut self.w,
            d_weights: &mut self.dw,
        }
    }
    pub fn d2(&mut self) -> ArcPolesD2<'_, 'static> {
        ArcPolesD2 {
            d1: ArcPolesD1 {
                poles: &mut self.p,
                d_poles: &mut self.dp,
                weights: &mut self.w,
                d_weights: &mut self.dw,
            },
            d2_poles: &mut self.d2p,
            d2_weights: &mut self.d2w,
        }
    }
}

/// `conv n lower` then the circle data.
pub fn circle_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (conv, n, lower) = (conv_of(r.integer()), r.integer(), r.integer());
    let c = read_circle(&mut r);
    let u = lower + n - 1;
    let mut o = Outputs::new(lower, u);
    show_err(get_circle(conv, &c.d1.circle, &mut o.p, &mut o.w));
    pnts(&o.p);
    reals(&o.w);
    let mut o = Outputs::new(lower, u);
    show(get_circle_d1(conv, &c.d1, o.d1()), flag);
    pnts(&o.p);
    vecs(&o.dp);
    reals(&o.w);
    reals(&o.dw);
    let mut o = Outputs::new(lower, u);
    show(get_circle_d2(conv, &c, o.d2()), flag);
    pnts(&o.p);
    vecs(&o.dp);
    vecs(&o.d2p);
    reals(&o.w);
    reals(&o.dw);
    reals(&o.d2w);
    end();
}

fn read_arc(r: &mut Reader) -> ArcSectionD2 {
    let (fp, dfp, d2fp) = (r.pnt(), vec(r), vec(r));
    let (c, dc, d2c) = (r.pnt(), vec(r), vec(r));
    let (dir, ddir, d2dir) = (vec(r), vec(r), vec(r));
    let (a, da, d2a) = (r.real(), r.real(), r.real());
    let arc = ArcSectionD0 {
        first_pnt: fp,
        center: c,
        dir,
        angle: a,
    };
    let d1 = ArcSectionD1 {
        arc,
        d_first_pnt: dfp,
        d_center: dc,
        d_dir: ddir,
        d_angle: da,
    };
    ArcSectionD2 {
        d1,
        d2_first_pnt: d2fp,
        d2_center: d2c,
        d2_dir: d2dir,
        d2_angle: d2a,
    }
}

fn polynomial(init: bool, arc: &ArcSectionD2, lower: i32, u: i32) {
    let mut pc = GeomFillPolynomialConvertor::new().expect("probe convertor");
    flag(pc.initialized());
    if init {
        pc.init().expect("probe init");
    }
    flag(pc.initialized());
    let mut o = Outputs::new(lower, u);
    show_err(pc.section(&arc.d1.arc, &mut o.p));
    pnts(&o.p);
    let mut o = Outputs::new(lower, u);
    show_err(pc.section_d1(&arc.d1, &mut o.p, &mut o.dp));
    pnts(&o.p);
    vecs(&o.dp);
    let mut o = Outputs::new(lower, u);
    show_err(pc.section_d2(arc, &mut o.p, &mut o.dp, &mut o.d2p));
    pnts(&o.p);
    vecs(&o.dp);
    vecs(&o.d2p);
}

fn quasi(init: bool, arc: &ArcSectionD2, lower: i32, u: i32) {
    let mut qc = GeomFillQuasiAngularConvertor::new().expect("probe convertor");
    flag(qc.initialized());
    if init {
        qc.init().expect("probe init");
    }
    flag(qc.initialized());
    let mut o = Outputs::new(lower, u);
    show_err(qc.section(&arc.d1.arc, &mut o.p, &mut o.w));
    pnts(&o.p);
    reals(&o.w);
    let mut o = Outputs::new(lower, u);
    show_err(qc.section_d1(&arc.d1, o.d1()));
    pnts(&o.p);
    vecs(&o.dp);
    reals(&o.w);
    reals(&o.dw);
    let mut o = Outputs::new(lower, u);
    show_err(qc.section_d2(arc, o.d2()));
    pnts(&o.p);
    vecs(&o.dp);
    vecs(&o.d2p);
    reals(&o.w);
    reals(&o.dw);
    reals(&o.d2w);
}

/// `kind init n lower` then the arc and its two derivatives.
pub fn convert_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, init, n, lower) = (r.integer(), r.flag(), r.integer(), r.integer());
    let arc = read_arc(&mut r);
    if kind == 0 {
        polynomial(init, &arc, lower, lower + n - 1);
    } else {
        quasi(init, &arc, lower, lower + n - 1);
    }
    end();
}

/// `nrow ncol nmat init n (row col mat value)... nvec v...`.
pub fn tensor_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (nr, nc, nm, init) = (r.integer(), r.integer(), r.integer(), r.real());
    let mut t = GeomFillTensor::new(nr, nc, nm).expect("probe tensor");
    t.init(init);
    for _ in 0..r.integer() {
        let (i, j, m, v) = (r.integer(), r.integer(), r.integer(), r.real());
        show(t.change_value(i, j, m), |slot| *slot = v);
    }
    for i in 0..=nr + 1 {
        for j in 1..=nc {
            for m in 1..=nm {
                show(t.value(i, j, m), real);
            }
        }
    }
    let v = r.reals();
    let mut right = Vector::with_bounds(1, v.len() as i32).expect("probe vector");
    for (k, x) in v.iter().enumerate() {
        right[k as i32 + 1] = *x;
    }
    let mut prod = Matrix::filled(1, nr, 1, nc, 0.0).expect("probe matrix");
    show(t.multiply(&right, &mut prod), |()| {
        for i in 1..=nr {
            for j in 1..=nc {
                real(prod[(i, j)]);
            }
        }
    });
    end();
}
