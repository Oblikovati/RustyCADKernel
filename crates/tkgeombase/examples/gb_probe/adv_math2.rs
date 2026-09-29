//! The AdvApp2Var_MathBase probe, part 2; the counterpart of
//! tools/oracle/tkgeombase_probe_advapp2var_math2.cxx.
use super::adv_math::{fill, print, reals};
use super::common::{Reader, end, int};
use rustycad_tkgeombase::adv_app2var::math_base as mb;

pub fn mmhjcan_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (nd, nk, oc, nl) = (r.integer(), r.integer(), r.integer(), r.integer());
    let ncf: Vec<i32> = (0..nk.max(1)).map(|_| r.integer()).collect();
    let (t, old) = (reals(&mut r, nk + 1), reals(&mut r, nd * nl * nk));
    let (mut new, mut ier) = (fill(nd * nl * nk), -1);
    mb::mmhjcan_(nd, nk, &ncf, oc, nl, &old, &t, &mut new, &mut ier);
    int(ier);
    print(&new);
    end();
}

pub fn mminltt_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (nc, nl, ncol, mut used) = (r.integer(), r.integer(), r.integer(), r.integer());
    let mut t = reals(&mut r, nc * nl);
    let m = r.integer();
    for _ in 0..m {
        let line = reals(&mut r, ncol);
        let mut ier = -1;
        mb::mminltt_(nc, nl, &mut t, ncol, &mut used, &line, 0., &mut ier);
        int(used);
        int(ier);
    }
    print(&t);
    end();
}

pub fn mmjacan_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (id, nd) = (r.integer(), r.integer());
    let p = reals(&mut r, (nd / 2 + 1) * 2);
    let mut c = fill(nd + 1);
    mb::mmjacan_(id, nd, &p, &mut c);
    print(&c);
    end();
}

pub fn mmjaccv_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (nc, nd, id) = (r.integer(), r.integer(), r.integer());
    let l = reals(&mut r, nc * nd);
    let (mut aux, mut c) = (fill(((nc - 1) / 2 + 1) * 2), fill(nc * nd));
    mb::mmjaccv_(nc, nd, id, &l, &mut aux, &mut c);
    print(&c);
    end();
}

pub fn mmpobas_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (t, io, nc, nv) = (r.real(), r.integer(), r.integer(), r.integer());
    let (mut v, mut ier) = (fill(nc * (nv + 1)), -1);
    mb::mmpobas_(t, io, nc, nv, &mut v, &mut ier);
    int(ier);
    print(&v);
    end();
}

pub fn mmpocrb_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (nx, nc, nd, t) = (r.integer(), r.integer(), r.integer(), r.real());
    let c = reals(&mut r, nx * nc);
    let mut p = fill(nd);
    mb::mmpocrb_(nx, nc, &c, nd, t, &mut p);
    print(&p);
    end();
}

pub fn mmmpocur_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (nx, nd, ng, t) = (r.integer(), r.integer(), r.integer(), r.real());
    let c = reals(&mut r, nx * nd);
    let mut p = fill(nd);
    mb::mmmpocur_(nx, nd, ng, &c, t, &mut p);
    print(&p);
    end();
}

pub fn mmposui_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let n = r.integer();
    let a: Vec<i32> = (0..2 * n).map(|_| r.integer()).collect();
    let nstoc = a[(2 * n - 1) as usize];
    let (mut p, mut ier) = (vec![-9; nstoc.max(1) as usize], -1);
    mb::mmposui_(n, nstoc, &a, &mut p, &mut ier);
    int(ier);
    p.iter().for_each(|x| int(*x));
    end();
}

/// The system of the aa_mmresol request (GbAaResol of the native probe).
#[derive(Default)]
struct Resol {
    h: i32,
    g: i32,
    hn: i32,
    gn: i32,
    mn: i32,
    mh: Vec<f64>,
    mg: Vec<f64>,
    vh: Vec<f64>,
    vg: Vec<f64>,
    hpos: Vec<i32>,
    hsui: Vec<i32>,
    gpos: Vec<i32>,
    msui: Vec<i32>,
    mpos: Vec<i32>,
}

impl Resol {
    fn build(h: i32, g: i32, bw: i32) -> Self {
        let mut s = Resol {
            h,
            g,
            ..Default::default()
        };
        for i in 1..=h {
            let c = (i - 1).min(bw);
            for j in i - c..=i {
                s.mh.push(if i == j {
                    2. * f64::from(bw) + 1. + f64::from(i) * 0.001
                } else {
                    -1. / (f64::from(i - j) + 1.)
                });
            }
            s.hn += c + 1;
            s.hpos.extend([c, s.hn]);
            s.vh.push(1. + 0.5 * f64::from(i));
        }
        for i in 1..=g {
            let c = 5.min(h - i + 1);
            for j in i..i + c {
                s.mg.push(if j == i { 3. } else { 1. / f64::from(i + j) });
            }
            s.gn += c;
            s.gpos.extend([c, s.gn, i]);
            s.vg.push(0.25 * f64::from(i));
        }
        for i in 1..=g {
            s.mn += i;
            s.mpos.extend([i - 1, s.mn]);
        }
        s.hsui = vec![0; s.hn.max(1) as usize];
        s.msui = vec![0; s.mn.max(1) as usize];
        let mut ier = 0;
        mb::mmposui_(h, s.hn, &s.hpos, &mut s.hsui, &mut ier);
        if g > 0 {
            mb::mmposui_(g, s.mn, &s.mpos, &mut s.msui, &mut ier);
        } else {
            s.gpos = vec![0; 3];
            s.mpos = vec![0; 2];
            s.mg = vec![0.];
            s.vg = vec![0.];
        }
        s
    }
}

pub fn mmresol_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (h, g, bw) = (r.integer(), r.integer(), r.integer());
    let s = Resol::build(h, g, bw);
    let system = mb::ProfileSystem {
        hdimen: s.h,
        gdimen: s.g,
        hnstoc: s.hn,
        gnstoc: s.gn,
        mnstoc: s.mn,
        matsyh: &s.mh,
        matsyg: &s.mg,
        vecsyh: &s.vh,
        vecsyg: &s.vg,
        hposit: &s.hpos,
        hposui: &s.hsui,
        gposit: &s.gpos,
        mmposui: &s.msui,
        mposit: &s.mpos,
    };
    let (mut sol, mut ier) = (fill(h), -1);
    mb::mmresol_(&system, &mut sol, &mut ier);
    int(ier);
    print(&sol);
    end();
}

pub fn mmrtptt_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let n = r.integer();
    let mut v = fill(n / 2);
    mb::mmrtptt_(n, &mut v);
    print(&v);
    end();
}

pub fn mmsrre2_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (t, n) = (r.real(), r.integer());
    let v = reals(&mut r, n);
    let eps = r.real();
    let (mut ni, mut it, mut ier) = (-1, -1, -1);
    mb::mmsrre2_(t, n, &v, eps, &mut ni, &mut it, &mut ier);
    int(ni);
    int(it);
    int(ier);
    end();
}

pub fn mmunivt_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (n, eps) = (r.integer(), r.real());
    let v = reals(&mut r, n);
    let (mut o, mut ier) = (fill(n), -1);
    mb::mmunivt_(n, &v, &mut o, eps, &mut ier);
    int(ier);
    print(&o);
    end();
}

pub fn mmvncol_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let n = r.integer();
    let mut v = reals(&mut r, n);
    v.resize(n.max(3) as usize, 0.);
    let (mut o, mut ier) = (fill(3), -1);
    mb::mmvncol_(n, &v, &mut o, &mut ier);
    int(ier);
    print(&o);
    end();
}

pub fn mmarcin_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (nx, nd, nc, u0, u1) = (r.integer(), r.integer(), r.integer(), r.real(), r.real());
    let o = reals(&mut r, nc * nd);
    let (mut c, mut ier) = (fill(nx * nc), -1);
    mb::mmarcin_(nx, nd, nc, &o, u0, u1, &mut c, &mut ier);
    int(ier);
    print(&c);
    end();
}

pub fn mmcvinv_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (nx, nc, nd) = (r.integer(), r.integer(), r.integer());
    let o = reals(&mut r, nc * nd);
    let mut c = fill(nx * nc);
    mb::mmcvinv_(nx, nc, nd, &o, &mut c);
    print(&c);
    end();
}

pub fn mmmrslwd_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (nx, no, nd, eps) = (r.integer(), r.integer(), r.integer(), r.real());
    let (a, b) = (reals(&mut r, nx * no), reals(&mut r, nx * nd));
    let (mut aux, mut x, mut ier) = (fill((no + nd) * no), fill(nx * nd), -1);
    mb::mmmrslwd_(nx, no, nd, &a, &b, eps, &mut aux, &mut x, &mut ier);
    int(ier);
    print(&x);
    end();
}
