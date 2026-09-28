//! The AdvApp2Var_ApproxF2var probe, part 1; the counterpart of
//! tools/oracle/tkgeombase_probe_advapp2var_f2var.cxx.
use super::adv_eval::Function;
use super::adv_math::{fill, print, reals};
use super::common::{Reader, end, int};
use rustycad_tkgeombase::adv_app2var::approx_f2var as f2;

pub fn mma1her_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let io = r.integer();
    let (mut h, mut ier) = (fill((2 * io + 2) * (2 * io + 2)), -1);
    f2::mma1her_(io, &mut h, &mut ier);
    int(ier);
    print(&h);
    end();
}

pub fn mma2roo_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (nu, nv) = (r.integer(), r.integer());
    let (mut u, mut v) = (fill(nu), fill(nv));
    f2::mma2roo_(nu, nv, &mut u, &mut v);
    print(&u);
    print(&v);
    end();
}

pub fn mma2jmx_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (nd, io) = (r.integer(), r.integer());
    let mut x = fill(nd - 2 * (io + 1) + 1);
    f2::mma2jmx_(nd, io, &mut x);
    print(&x);
    end();
}

pub fn mmapptt_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (nd, np, jo) = (r.integer(), r.integer(), r.integer());
    let (mut c, mut ier) = (fill((np / 2 + 1) * (nd + 1)), -1);
    f2::mmapptt_(nd, np, jo, &mut c, &mut ier);
    int(ier);
    print(&c);
    end();
}

fn hermite(order: i32) -> Vec<f64> {
    let (mut h, mut ier) = (fill((2 * order + 2) * (2 * order + 2)), 0);
    f2::mma1her_(order, &mut h, &mut ier);
    h
}

pub fn mma2ac1_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (nd, mu, mv, iu, iv) = (
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
    );
    let nc = nd * (iu + 2) * (iv + 2);
    let c: Vec<Vec<f64>> = (0..4).map(|_| reals(&mut r, nc)).collect();
    let mut pat = reals(&mut r, (mu + 1) * (mv + 1) * nd);
    let corners = f2::CornerConstraints {
        contr1: &c[0],
        contr2: &c[1],
        contr3: &c[2],
        contr4: &c[3],
    };
    f2::mma2ac1_(
        nd,
        mu,
        mv,
        iu,
        iv,
        &corners,
        &hermite(iu),
        &hermite(iv),
        &mut pat,
    );
    print(&pat);
    end();
}

fn mma2ac23(values: &[f64], v: bool) {
    let mut r = Reader::new(values);
    let (nd, mu, mv, io, nl) = (
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
    );
    let n1: Vec<i32> = (0..=io).map(|_| r.integer()).collect();
    let b1 = reals(&mut r, nl * nd * (io + 1));
    let n2: Vec<i32> = (0..=io).map(|_| r.integer()).collect();
    let b2 = reals(&mut r, nl * nd * (io + 1));
    let mut pat = reals(&mut r, (mu + 1) * (mv + 1) * nd);
    let isos = f2::IsoConstraints {
        ncf1: &n1,
        crb1: &b1,
        ncf2: &n2,
        crb2: &b2,
    };
    if v {
        f2::mma2ac2_(nd, mu, mv, io, nl, &isos, &hermite(io), &mut pat);
    } else {
        f2::mma2ac3_(nd, mu, mv, io, nl, &isos, &hermite(io), &mut pat);
    }
    print(&pat);
    end();
}
pub fn mma2ac2_case(values: &[f64]) {
    mma2ac23(values, true);
}
pub fn mma2ac3_case(values: &[f64]) {
    mma2ac23(values, false);
}

pub fn mma2can_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (mu, mv, nd, iu, iv, nu, nv) = (
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
    );
    let pj = reals(&mut r, nu * nv * nd);
    let (mut aux, mut can, mut ier) = (fill(2 * nu * nv * nd), fill(mu * mv * nd), -1);
    f2::mma2can_(
        mu, mv, nd, iu, iv, nu, nv, &pj, &mut aux, &mut can, &mut ier,
    );
    int(ier);
    print(&can);
    print(&aux);
    end();
}

/// The four square tables of the native GbAaSquare.
pub struct Square {
    pub soso: Vec<f64>,
    pub diso: Vec<f64>,
    pub sodi: Vec<f64>,
    pub didi: Vec<f64>,
}

impl Square {
    pub fn new(nu: i32, nv: i32, nd: i32, value: f64) -> Self {
        let full = ((nu / 2 + 1) * (nv / 2 + 1) * nd) as usize;
        let half = (nu / 2 * (nv / 2) * nd).max(1) as usize;
        Square {
            soso: vec![value; full],
            diso: vec![value; half],
            sodi: vec![value; half],
            didi: vec![value; full],
        }
    }
    pub fn tables(&mut self) -> f2::SquareTables<'_> {
        f2::SquareTables {
            sosotb: &mut self.soso,
            disotb: &mut self.diso,
            soditb: &mut self.sodi,
            diditb: &mut self.didi,
        }
    }
    pub fn print(&self) {
        print(&self.soso);
        print(&self.diso);
        print(&self.sodi);
        print(&self.didi);
    }
}

pub fn roots(nu: i32, nv: i32) -> (Vec<f64>, Vec<f64>) {
    let (mut u, mut v) = (fill(nu), fill(nv));
    f2::mma2roo_(nu, nv, &mut u, &mut v);
    (u, v)
}

pub fn mma2cdi_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (nd, nu, nv, iu, iv) = (
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
    );
    let (nc, su, sv) = (
        nd * (iu + 2) * (iv + 2),
        (nv / 2 + 1) * nd * (iu + 1),
        (nu / 2 + 1) * nd * (iv + 1),
    );
    let mut a: Vec<Vec<f64>> = Vec::new();
    for size in [nc; 4].into_iter().chain([su; 4]).chain([sv; 4]) {
        let mut v = reals(&mut r, size);
        if v.is_empty() {
            v.push(0.);
        }
        a.push(v);
    }
    let (ur, vr) = roots(nu, nv);
    let mut t = Square::new(nu, nv, nd, 7.5);
    let grid = f2::SquareGrid {
        ndimen: nd,
        nbpntu: nu,
        urootl: &ur,
        nbpntv: nv,
        vrootl: &vr,
        iordru: iu,
        iordrv: iv,
    };
    let corners = f2::CornerConstraints {
        contr1: &a[0],
        contr2: &a[1],
        contr3: &a[2],
        contr4: &a[3],
    };
    let isos_u = f2::IsoTables {
        sotb1: &a[4],
        sotb2: &a[5],
        ditb1: &a[6],
        ditb2: &a[7],
    };
    let isos_v = f2::IsoTables {
        sotb1: &a[8],
        sotb2: &a[9],
        ditb1: &a[10],
        ditb2: &a[11],
    };
    let mut ier = -1;
    f2::mma2cdi_(&grid, &corners, &isos_u, &isos_v, &mut t.tables(), &mut ier);
    int(ier);
    t.print();
    end();
}

pub fn mma2ds1_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (nd, nu, nv, iso) = (3, r.integer(), r.integer(), r.integer());
    let uv: Vec<f64> = reals(&mut r, 4);
    let f = Function(r.integer());
    let (ur, vr) = roots(nu, nv);
    let mut t = Square::new(nu, nv, nd, 0.25);
    let (mut fp, mut tt, mut ier) = (fill(nd * nu.max(nv)), fill(nu.max(nv)), -1);
    let square = f2::SampledSquare {
        ndimen: nd,
        uintfn: [uv[0], uv[1]],
        vintfn: [uv[2], uv[3]],
        nbpntu: nu,
        nbpntv: nv,
        urootb: &ur,
        vrootb: &vr,
        isofav: iso,
    };
    f2::mma2ds1_(&square, &f, &mut t.tables(), &mut fp, &mut tt, &mut ier);
    int(ier);
    t.print();
    end();
}
