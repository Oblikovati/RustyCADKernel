//! The AdvApp2Var class probe (Context, Node, Iso, Patch, Network, Framework and
//! ApproxAFunc2Var); the counterpart of tools/oracle/tkgeombase_probe_advapp2var_class.cxx.
use super::adv_eval::Function;
use super::common::{Reader, attempt, end, flag, int, pnt, real, xy};
use rustycad_tkernel::ncollection::{Array1, Array2, Sequence};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adv_approx::{AdvApproxCutting, AdvApproxDichoCutting, AdvApproxPrefAndRec};
use rustycad_tkgeombase::adv_app2var::{
    AdvApp2VarApproxAFunc2Var, AdvApp2VarContext, AdvApp2VarCriterion,
    AdvApp2VarCriterionRepartition, AdvApp2VarCriterionType, AdvApp2VarFramework, AdvApp2VarIso,
    AdvApp2VarNetwork, AdvApp2VarNode, AdvApp2VarPatch, FrontierTolerances, SubspaceTolerances,
};
use rustycad_tkmath::geom_abs::{IsoType, Shape};
use rustycad_tkmath::gp::Xy;

const ISO_TYPES: [IsoType; 3] = [IsoType::IsoU, IsoType::IsoV, IsoType::NoneIso];
const SHAPES: [Shape; 7] = [
    Shape::C0,
    Shape::G1,
    Shape::C1,
    Shape::G2,
    Shape::C2,
    Shape::C3,
    Shape::CN,
];

fn iso_index(iso_type: IsoType) -> i32 {
    ISO_TYPES
        .iter()
        .position(|t| *t == iso_type)
        .map_or(-1, |i| i as i32)
}

fn tol(n: i32, d: i32) -> Option<Array1<'static, f64>> {
    if n <= 0 {
        return None;
    }
    let mut t: Array1<'static, f64> = Array1::with_bounds(1, n).ok()?;
    for k in 1..=n {
        t.set_value(k, 1.0e-3 * f64::from(k) * f64::from(d)).ok()?;
    }
    Some(t)
}

fn tof(n: i32, d: i32) -> Option<Array2<'static, f64>> {
    if n <= 0 {
        return None;
    }
    let mut t: Array2<'static, f64> = Array2::with_bounds(1, n, 1, 4).ok()?;
    for k in 1..=n {
        for j in 1..=4 {
            let value = 1.0e-3 * f64::from(k) * f64::from(d) * (1. + f64::from(j) / 10.);
            t.set_value(k, j, value).ok()?;
        }
    }
    Some(t)
}

fn harray(a: Option<&Array1<'static, f64>>) {
    let Some(a) = a else {
        return int(-1);
    };
    int(a.length());
    a.as_slice().iter().for_each(|v| real(*v));
}

fn harray2(a: Option<&Array2<'static, f64>>) {
    let Some(a) = a else {
        return int(-1);
    };
    int(a.col_length());
    int(a.row_length());
    for i in a.lower_row()..=a.upper_row() {
        for j in a.lower_col()..=a.upper_col() {
            real(*a.value(i, j).expect("in range"));
        }
    }
}

fn head(a: &Array1<'static, f64>, n: i32) {
    a.as_slice()[..n.max(0) as usize]
        .iter()
        .for_each(|v| real(*v));
}

/// A Gauss table of `n` points without the zero-root row that mmapptt_ leaves
/// unwritten for an even count.
fn gauss(a: &Array1<'static, f64>, n: i32) {
    int(a.length());
    for (k, v) in (0..).zip(a.as_slice()) {
        if n % 2 == 1 || k % (n / 2 + 1) != 0 {
            real(*v);
        }
    }
}

/// A 3D context with one subspace, as the native `aa_patch` builds it.
fn context_3d(
    f: i32,
    orders: [i32; 2],
    limits: [i32; 2],
    precis: i32,
) -> Result<AdvApp2VarContext, Failure> {
    let (t, tf) = (tol(1, 3), tof(1, 3));
    let tolerances = SubspaceTolerances {
        one_d: None,
        two_d: None,
        three_d: t.as_ref(),
    };
    let fronts = FrontierTolerances {
        one_d: None,
        two_d: None,
        three_d: tf.as_ref(),
    };
    AdvApp2VarContext::new(
        f,
        orders[0],
        orders[1],
        limits[0],
        limits[1],
        precis,
        [0, 0, 1],
        tolerances,
        fronts,
    )
}

fn context_body(r: &mut Reader) -> Result<(), Failure> {
    let (f, iu, iv, nu, nv, p) = (
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
    );
    let (n1, n2, n3) = (r.integer(), r.integer(), r.integer());
    let (t1, t2, t3) = (tol(n1, 1), tol(n2, 2), tol(n3, 3));
    let (f1, f2, f3) = (tof(n1, 1), tof(n2, 2), tof(n3, 3));
    let tolerances = SubspaceTolerances {
        one_d: t1.as_ref(),
        two_d: t2.as_ref(),
        three_d: t3.as_ref(),
    };
    let fronts = FrontierTolerances {
        one_d: f1.as_ref(),
        two_d: f2.as_ref(),
        three_d: f3.as_ref(),
    };
    let c = AdvApp2VarContext::new(f, iu, iv, nu, nv, p, [n1, n2, n3], tolerances, fronts)?;
    for v in [
        c.total_dimension(),
        c.total_number_ssp(),
        c.favor_iso(),
        c.u_order(),
        c.v_order(),
    ] {
        int(v);
    }
    for v in [c.u_limit(), c.v_limit(), c.u_jac_deg(), c.v_jac_deg()] {
        int(v);
    }
    harray(c.u_jac_max());
    harray(c.v_jac_max());
    let (ur, vr) = (c.u_roots().expect("roots"), c.v_roots().expect("roots"));
    int(ur.length());
    int(vr.length());
    head(ur, ur.length() - if iu > -1 { 2 } else { 0 });
    head(vr, vr.length() - if iv > -1 { 2 } else { 0 });
    let pu = ur.length() - if iu > -1 { 2 } else { 0 };
    let pv = vr.length() - if iv > -1 { 2 } else { 0 };
    gauss(c.u_gauss().expect("gauss"), pu);
    gauss(c.v_gauss().expect("gauss"), pv);
    harray(c.i_toler());
    harray2(c.f_toler());
    harray2(c.c_toler());
    let copy: AdvApp2VarContext = c.clone();
    int(copy.u_limit());
    harray(copy.i_toler());
    Ok(())
}

/// aa_context: ifav iu iv nlimu nlimv iprecis nb1 nb2 nb3.
pub fn context_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| context_body(&mut r));
    end();
}

/// The one-patch grid of `InitGrid` with the cuts of the request applied.
struct Grid {
    net: AdvApp2VarNetwork,
    frame: AdvApp2VarFramework,
}

impl Grid {
    fn new(iu: i32, iv: i32, d: [f64; 4]) -> Result<Self, Failure> {
        let mut p: Sequence<Handle<AdvApp2VarPatch>> = Sequence::default();
        p.append(Handle::new(AdvApp2VarPatch::with_domain(d, iu, iv)));
        let (mut us, mut vs) = (Sequence::default(), Sequence::default());
        us.append(d[0]);
        us.append(d[1]);
        vs.append(d[2]);
        vs.append(d[3]);
        let net = AdvApp2VarNetwork::new(p, us, vs);
        let mut bag: Sequence<Handle<AdvApp2VarNode>> = Sequence::default();
        for (u, v) in [(d[0], d[2]), (d[1], d[2]), (d[0], d[3]), (d[1], d[3])] {
            bag.append(Handle::new(AdvApp2VarNode::with_coord(
                Xy::new(u, v),
                iu,
                iv,
            )?));
        }
        let iso = |t: IsoType, cte: f64, pos: i32| {
            Handle::new(AdvApp2VarIso::with_domain(t, cte, d, pos, iu, iv))
        };
        let (mut bu, mut bv) = (Sequence::default(), Sequence::default());
        bu.append(iso(IsoType::IsoV, d[2], 1));
        bu.append(iso(IsoType::IsoV, d[3], 2));
        bv.append(iso(IsoType::IsoU, d[0], 3));
        bv.append(iso(IsoType::IsoU, d[1], 4));
        let (mut su, mut sv) = (Sequence::default(), Sequence::default());
        su.append(bu);
        sv.append(bv);
        Ok(Self {
            net,
            frame: AdvApp2VarFramework::new(bag, su, sv),
        })
    }

    fn cut(&mut self, r: &mut Reader) -> Result<(), Failure> {
        for _ in 0..r.integer() {
            let (dir, x) = (r.integer(), r.real());
            if dir == 0 {
                self.net.update_in_u(x)?;
                self.frame.update_in_u(x)?;
            } else {
                self.net.update_in_v(x)?;
                self.frame.update_in_v(x)?;
            }
        }
        Ok(())
    }
}

fn print_iso(i: &AdvApp2VarIso) {
    int(iso_index(i.iso_type()));
    for v in [
        i.constante(),
        i.t0(),
        i.t1(),
        i.u0(),
        i.u1(),
        i.v0(),
        i.v1(),
    ] {
        real(v);
    }
    int(i.u_order());
    int(i.v_order());
    int(i.position());
    flag(i.is_approximated());
    flag(i.has_result());
}

fn grid_patches(n: &mut AdvApp2VarNetwork, iu: i32, iv: i32) -> Result<(), Failure> {
    int(n.nb_patch());
    int(n.nb_patch_in_u());
    int(n.nb_patch_in_v());
    for i in 1..=n.nb_patch_in_u() + 1 {
        real(n.u_parameter(i)?);
    }
    for j in 1..=n.nb_patch_in_v() + 1 {
        real(n.v_parameter(j)?);
    }
    for j in 1..=n.nb_patch_in_v() {
        for i in 1..=n.nb_patch_in_u() {
            let p = n.patch(i, j)?;
            [p.u0(), p.u1(), p.v0(), p.v1()].into_iter().for_each(real);
            int(p.u_order());
            int(p.v_order());
        }
    }
    let first = n.first_not_approx();
    flag(first.is_some());
    int(first.unwrap_or(0));
    for i in 1..=n.nb_patch() {
        n.change_patch(i)?.change_nb_coeff(i + 2, (2 * i) % 7 + 1);
    }
    let (cu, cv) = n.same_degree(iu, iv);
    int(cu);
    int(cv);
    for i in 1..=n.nb_patch() {
        int(n.change_patch(i)?.nb_coeff_in_u());
        int(n.change_patch(i)?.nb_coeff_in_v());
    }
    Ok(())
}

fn grid_frame(n: &AdvApp2VarNetwork, f: &AdvApp2VarFramework) -> Result<(), Failure> {
    let nodes = (n.nb_patch_in_u() + 1) * (n.nb_patch_in_v() + 1);
    for i in 1..=nodes {
        xy(f.node(i)?.coord());
        int(f.node(i)?.u_order());
    }
    for j in 1..=n.nb_patch_in_v() + 1 {
        for i in 1..=n.nb_patch_in_u() {
            print_iso(f.iso_v(n.u_parameter(i)?, n.u_parameter(i + 1)?, n.v_parameter(j)?)?);
            int(f.first_node(IsoType::IsoV, j, i));
            int(f.last_node(IsoType::IsoV, j, i));
        }
    }
    for i in 1..=n.nb_patch_in_u() + 1 {
        for j in 1..=n.nb_patch_in_v() {
            print_iso(f.iso_u(n.u_parameter(i)?, n.v_parameter(j)?, n.v_parameter(j + 1)?)?);
            int(f.first_node(IsoType::IsoU, i, j));
            int(f.last_node(IsoType::IsoU, i, j));
        }
    }
    xy(
        f.node_at(n.u_parameter(1)?, n.v_parameter(n.nb_patch_in_v() + 1)?)?
            .coord(),
    );
    let first = f.first_not_approx();
    flag(first.is_some());
    let (i1, i2) = first.map_or((0, 0), |(_, a, b)| (a, b));
    int(i1);
    int(i2);
    Ok(())
}

fn grid_body(r: &mut Reader) -> Result<(), Failure> {
    let (iu, iv) = (r.integer(), r.integer());
    let d = [r.real(), r.real(), r.real(), r.real()];
    let mut g = Grid::new(iu, iv, d)?;
    g.cut(r)?;
    grid_patches(&mut g.net, iu, iv)?;
    grid_frame(&g.net, &g.frame)
}

/// aa_grid: iu iv u0 u1 v0 v1 cuts.
pub fn grid_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| grid_body(&mut r));
    end();
}

/// The probe criterion: 2 * max error + 1e-4 * coefficient count, satisfied under the bound.
struct Criterion {
    max: f64,
    kind: AdvApp2VarCriterionType,
    repartition: AdvApp2VarCriterionRepartition,
}

impl AdvApp2VarCriterion for Criterion {
    fn value(&self, p: &AdvApp2VarPatch, _c: &AdvApp2VarContext) -> Result<(), Failure> {
        let errors = p
            .max_errors()
            .ok_or_else(|| Failure::with_message("Standard_NullObject", "no errors"))?;
        let count = f64::from(p.nb_coeff_in_u()) * f64::from(p.nb_coeff_in_v());
        p.set_crit_value(2. * *errors.value(1).expect("one subspace") + 1.0e-4 * count);
        Ok(())
    }
    fn is_satisfied(&self, p: &AdvApp2VarPatch) -> bool {
        p.crit_value() < self.max
    }
    fn max_value(&self) -> f64 {
        self.max
    }
    fn criterion_type(&self) -> AdvApp2VarCriterionType {
        self.kind
    }
    fn repartition(&self) -> AdvApp2VarCriterionRepartition {
        self.repartition
    }
}

fn print_patch(
    p: &AdvApp2VarPatch,
    c: &AdvApp2VarContext,
    crit: &Criterion,
) -> Result<(), Failure> {
    flag(p.is_discretised());
    flag(p.is_approximated());
    flag(p.has_result());
    int(p.cut_sense());
    int(p.cut_sense_with(crit, 3));
    int(p.nb_coeff_in_u());
    int(p.nb_coeff_in_v());
    if !p.has_result() {
        return Ok(());
    }
    harray(p.max_errors().as_ref());
    harray(p.average_errors().as_ref());
    harray2(p.iso_errors().as_ref());
    head(
        &p.coefficients(1, c)?.expect("equation"),
        c.u_limit() * c.v_limit() * 3,
    );
    let poles = p.poles(1, c)?;
    int(poles.col_length());
    int(poles.row_length());
    for i in poles.lower_row()..=poles.upper_row() {
        for j in poles.lower_col()..=poles.upper_col() {
            pnt(*poles.value(i, j).expect("in range"));
        }
    }
    attempt(|| p.coefficients(2, c).map(|_| ()));
    Ok(())
}

fn approximate_isos(
    g: &mut Grid,
    c: &AdvApp2VarContext,
    f: &Function,
    d: [f64; 4],
    orders: [i32; 2],
) -> Result<bool, Failure> {
    let (n1, n2) = (
        AdvApp2VarNode::with_orders(orders[0], orders[1]),
        AdvApp2VarNode::with_orders(orders[0], orders[1]),
    );
    while let Some((iso, i1, i2)) = g.frame.first_not_approx() {
        let (a, b) = (
            g.frame.first_node(iso.iso_type(), i1, i2),
            g.frame.last_node(iso.iso_type(), i1, i2),
        );
        n1.assign(g.frame.node(a)?);
        n2.assign(g.frame.node(b)?);
        iso.make_approx(c, d[0], d[1], d[2], d[3], f, &n1, &n2)?;
        print_iso(&iso);
        int(iso.nb_coeff());
        // An iso without result leaves null tables that Discretise dereferences natively.
        if !iso.has_result() {
            int(-9);
            return Ok(false);
        }
        harray(iso.polynom().as_ref());
        harray(iso.som_tab().as_ref());
        harray(iso.dif_tab().as_ref());
        harray2(iso.max_errors().as_ref());
        harray2(iso.moy_errors().as_ref());
        iso.overwrite_approx();
        g.frame.change_iso(i1, i2, &iso)?;
        g.frame.node(a)?.assign(&n1);
        g.frame.node(b)?.assign(&n2);
        for u in 0..=orders[0] {
            for v in 0..=orders[1] {
                pnt(n1.point(u, v)?);
                real(n1.error(u, v)?);
                pnt(n2.point(u, v)?);
            }
        }
    }
    Ok(true)
}

fn patch_body(r: &mut Reader) -> Result<(), Failure> {
    let func = Function(r.integer());
    let (f, iu, iv, nu, nv, pr) = (
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
    );
    let d = [r.real(), r.real(), r.real(), r.real()];
    let numdec = r.integer();
    let crit = Criterion {
        max: r.real(),
        kind: AdvApp2VarCriterionType::Relative,
        repartition: AdvApp2VarCriterionRepartition::Incremental,
    };
    let c = context_3d(f, [iu, iv], [nu, nv], pr)?;
    let mut g = Grid::new(iu, iv, d)?;
    g.cut(r)?;
    if !approximate_isos(&mut g, &c, &func, d, [iu, iv])? {
        return Ok(());
    }
    for i in 1..=g.net.nb_patch() {
        let p = g.net.change_patch(i)?;
        p.discretise(&c, &g.frame, &func)?;
        if p.is_discretised() {
            p.make_approx(&c, &g.frame, numdec)?;
            if p.has_result() {
                crit.value(p, &c)?;
                real(p.crit_value());
            }
        }
        print_patch(p, &c, &crit)?;
    }
    Ok(())
}

/// aa_patch: function ifav iu iv ndu ndv iprecis u0 u1 v0 v1 numdec critmax cuts.
pub fn patch_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| patch_body(&mut r));
    end();
}

fn fnv(text: &[u8]) -> u64 {
    text.iter().fold(1_469_598_103_934_665_603_u64, |h, ch| {
        (h ^ u64::from(*ch)).wrapping_mul(1_099_511_628_211)
    })
}

fn print_approx(a: &AdvApp2VarApproxAFunc2Var) -> Result<(), Failure> {
    flag(a.is_done());
    flag(a.has_result());
    int(a.u_degree());
    int(a.v_degree());
    for k in 1..=3 {
        int(a.num_sub_spaces(k)?);
    }
    real(a.max_error(3, 1)?);
    real(a.average_error(3, 1)?);
    real(a.u_front_error(3, 1)?);
    real(a.v_front_error(3, 1)?);
    real(a.crit_error(3, 1)?);
    flag(a.max_errors(1)?.is_none());
    flag(a.v_front_errors(2)?.is_none());
    attempt(|| a.max_errors(4).map(|_| ()));
    attempt(|| a.average_errors(0).map(|_| ()));
    attempt(|| a.u_front_error(3, 2).map(|_| ()));
    attempt(|| a.crit_error(1, 1).map(|_| ()));
    let s = a.surface(1)?;
    [s.u_degree(), s.v_degree(), s.nb_u_knots(), s.nb_v_knots()]
        .into_iter()
        .for_each(int);
    for i in 1..=s.nb_u_knots() {
        real(s.u_knot(i)?);
        int(s.u_multiplicity(i)?);
    }
    for i in 1..=s.nb_v_knots() {
        real(s.v_knot(i)?);
        int(s.v_multiplicity(i)?);
    }
    int(s.nb_u_poles());
    int(s.nb_v_poles());
    for i in 1..=s.nb_u_poles() {
        for j in 1..=s.nb_v_poles() {
            pnt(s.pole(i, j)?);
        }
    }
    let mut text: Vec<u8> = Vec::new();
    a.dump(&mut text)
        .map_err(|e| Failure::with_message("Standard_Failure", &e.to_string()))?;
    print!("{} ", fnv(&text));
    Ok(())
}

fn approx_body(r: &mut Reader) -> Result<(), Failure> {
    let func = Function(r.integer());
    let (fav, cu, cv, pr) = (r.integer(), r.integer(), r.integer(), r.integer());
    let (mu, mv, mp) = (r.integer(), r.integer(), r.integer());
    let d = [r.real(), r.real(), r.real(), r.real()];
    let (cut, ck, cr, cm) = (r.integer(), r.integer(), r.integer(), r.real());
    let mut rec: Array1<'static, f64> = Array1::with_bounds(1, 1).expect("bounds");
    rec.set_value(1, 0.5 * (d[0] + d[1])).expect("in range");
    let mut pref: Array1<'static, f64> = Array1::with_bounds(1, 2).expect("bounds");
    pref.set_value(1, 0.3).expect("in range");
    pref.set_value(2, 0.7).expect("in range");
    let (dicho, pref_cut) = (
        AdvApproxDichoCutting::new(),
        AdvApproxPrefAndRec::new(&rec, &pref, 5.)?,
    );
    let ch: &dyn AdvApproxCutting = if cut == 0 { &dicho } else { &pref_cut };
    let kinds = [
        AdvApp2VarCriterionType::Absolute,
        AdvApp2VarCriterionType::Relative,
    ];
    let reps = [
        AdvApp2VarCriterionRepartition::Regular,
        AdvApp2VarCriterionRepartition::Incremental,
    ];
    let crit = Criterion {
        max: cm,
        kind: kinds[usize::from(ck == 2)],
        repartition: reps[cr as usize],
    };
    let (t, tf) = (tol(1, 3), tof(1, 3));
    let (fav, cu, cv) = (
        ISO_TYPES[fav as usize],
        SHAPES[cu as usize],
        SHAPES[cv as usize],
    );
    let a = if ck == 0 {
        AdvApp2VarApproxAFunc2Var::new(
            0,
            0,
            1,
            None,
            None,
            t.as_ref(),
            None,
            None,
            tf.as_ref(),
            d[0],
            d[1],
            d[2],
            d[3],
            fav,
            cu,
            cv,
            pr,
            mu,
            mv,
            mp,
            &func,
            ch,
            ch,
        )?
    } else {
        AdvApp2VarApproxAFunc2Var::with_criterion(
            0,
            0,
            1,
            None,
            None,
            t.as_ref(),
            None,
            None,
            tf.as_ref(),
            d[0],
            d[1],
            d[2],
            d[3],
            fav,
            cu,
            cv,
            pr,
            mu,
            mv,
            mp,
            &func,
            &crit,
            ch,
            ch,
        )?
    };
    print_approx(&a)
}

/// aa_approx: function favor contU contV iprecis maxdegU maxdegV maxpatch u0 u1 v0 v1
/// cutkind critkind critrep critmax.
pub fn approx_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| approx_body(&mut r));
    end();
}
