//! The AdvApp2Var_ApproxF2var probe, part 2; the counterpart of
//! tools/oracle/tkgeombase_probe_advapp2var_f2var2.cxx.
use super::adv_eval::Function;
use super::adv_f2var::{Square, roots};
use super::adv_math::{fill, print, reals};
use super::common::{Reader, end, int, real};
use rustycad_tkgeombase::adv_app2var::approx_f2var as f2;

pub fn mma2ce1_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let f = Function(r.integer());
    let (numdec, iu, iv) = (r.integer(), r.integer(), r.integer());
    let (nu, nv, ju, jv) = (r.integer(), r.integer(), r.integer(), r.integer());
    let (gu, gv, mu, mv) = (r.integer(), r.integer(), r.integer(), r.integer());
    let eps = r.real();
    let uv: Vec<f64> = reals(&mut r, 4);
    let nd = 3;
    let (ur, vr) = roots(nu, nv);
    let mut t = Square::new(nu, nv, nd, 0.);
    let (mut fp, mut tt, mut ier) = (fill(nd * nu.max(nv)), fill(nu.max(nv)), -1);
    let square = f2::SampledSquare {
        ndimen: nd,
        uintfn: [uv[0], uv[1]],
        vintfn: [uv[2], uv[3]],
        nbpntu: nu,
        nbpntv: nv,
        urootb: &ur,
        vrootb: &vr,
        isofav: 2,
    };
    f2::mma2ds1_(&square, &f, &mut t.tables(), &mut fp, &mut tt, &mut ier);
    let request = f2::PatchApproxRequest {
        numdec,
        ndimen: nd,
        nbsesp: 1,
        ndimse: &[3],
        ndminu: mu,
        ndminv: mv,
        ndguli: gu,
        ndgvli: gv,
        ndjacu: ju,
        ndjacv: jv,
        iordru: iu,
        iordrv: iv,
        nbpntu: nu,
        nbpntv: nv,
        epsapr: &[eps],
    };
    let samples = f2::SquareSamples {
        sosotb: &t.soso,
        disotb: &t.diso,
        soditb: &t.sodi,
        diditb: &t.didi,
    };
    let (mut pj, mut emax, mut emoy) = (fill((ju + 1) * (jv + 1) * nd), [7.5], [7.5]);
    let mut output = f2::PatchApproxOutput {
        patjac: &mut pj,
        errmax: &mut emax,
        errmoy: &mut emoy,
        ndegpu: -1,
        ndegpv: -1,
        itydec: -1,
    };
    ier = -9;
    f2::mma2ce1_(&request, &samples, &mut output, &mut ier);
    int(ier);
    int(output.itydec);
    int(output.ndegpu);
    int(output.ndegpv);
    real(emax[0]);
    real(emoy[0]);
    print(&pj);
    end();
}

pub fn mma2fnc_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let f = Function(r.integer());
    let (iso, tc) = (r.integer(), r.real());
    let uv: Vec<f64> = reals(&mut r, 4);
    let (io, id, nb, ndg, ncf, mx) = (
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
    );
    let eps = r.real();
    let nd = 3;
    let (rootlg, _) = roots(nb, nb);
    let request = f2::IsoApproxRequest {
        ndimen: nd,
        nbsesp: 1,
        ndimse: &[3],
        uvfonc: [uv[0], uv[1], uv[2], uv[3]],
        tconst: tc,
        isofav: iso,
        nbroot: nb,
        rootlg: &rootlg,
        iordre: io,
        ideriv: id,
        ndgjac: ndg,
        nbcrmx: mx,
        ncflim: ncf,
        epsapr: &[eps],
    };
    let mut ncoef: Vec<i32> = vec![-1; mx as usize];
    let (mut crb, mut som, mut dif) = (
        fill(ncf * nd * mx),
        fill((nb / 2 + 1) * nd * mx),
        fill((nb / 2 + 1) * nd * mx),
    );
    let (mut c1, mut c2, mut tab) = (
        fill(nd * (io + 2) * mx),
        fill(nd * (io + 2) * mx),
        fill(mx + 1),
    );
    let (mut emax, mut emoy) = (fill(mx), fill(mx));
    let mut output = f2::IsoApproxOutput {
        ncoeff: &mut ncoef,
        courbe: &mut crb,
        nbcrbe: -1,
        somtab: &mut som,
        diftab: &mut dif,
        contr1: &mut c1,
        contr2: &mut c2,
        tabdec: &mut tab,
        errmax: &mut emax,
        errmoy: &mut emoy,
    };
    let mut ier = -9;
    f2::mma2fnc_(&request, &f, &mut output, &mut ier);
    int(ier);
    int(output.nbcrbe);
    ncoef.iter().for_each(|n| int(*n));
    for table in [&tab, &emax, &emoy, &crb, &som, &dif, &c1, &c2] {
        print(table);
    }
    end();
}

pub fn mma2fx6_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (mu, mv, nd, pu, pv, iu, iv) = (
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
    );
    let eps = r.real();
    let fro = reals(&mut r, 8);
    let mut cu: Vec<i32> = (0..pu * pv).map(|_| r.integer()).collect();
    let mut cv: Vec<i32> = (0..pu * pv).map(|_| r.integer()).collect();
    let em = reals(&mut r, pu * pv);
    let pat = reals(&mut r, mu * mv * nd * pu * pv);
    let grid = f2::PatchGrid {
        ncfmxu: mu,
        ncfmxv: mv,
        ndimen: nd,
        nbsesp: 1,
        ndimse: &[nd],
        nbupat: pu,
        nbvpat: pv,
        iordru: iu,
        iordrv: iv,
        epsapr: &[eps],
        epsfro: &fro,
        patcan: &pat,
        errmax: &em,
    };
    f2::mma2fx6_(&grid, &mut cu, &mut cv);
    cu.iter().chain(&cv).for_each(|n| int(*n));
    end();
}
