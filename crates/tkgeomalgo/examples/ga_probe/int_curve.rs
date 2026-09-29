//! IntCurve probe operations, the Rust twin of tools/oracle/tkgeomalgo_probe_intcurve.cxx.
use super::common::{Reader, attempt, end, flag, int, pnt2d, real, show, xy};
use super::int_imp_par_gen::v2;
use super::int_res2d::inter;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::int_curve::{
    IConicTool, IntConicConic, IntImpConicParConic, Interval, MyImpParToolOfIntImpConicParConic,
    PConic, PConicTool, PeriodicInterval, ProjectOnPConicTool,
};
use rustycad_tkgeomalgo::int_res2d::Domain;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{
    Axis22d, Circle2, Direction2, Ellipse2, Hyperbola2, Line2, Parabola2, Point2, Vector2,
};
use rustycad_tkmath::math::{Function, FunctionWithDerivative};

/// `GaConic`: a conic of kind 0..4 on a frame, with its radii.
#[derive(Clone, Copy)]
pub struct Conic {
    pub kind: i32,
    p: Point2,
    d: Direction2,
    sense: bool,
    r1: f64,
    r2: f64,
}

impl Conic {
    fn ax(&self) -> Result<Axis22d, Failure> {
        Axis22d::new(self.p, self.d, self.sense)
    }
    pub fn lin(&self) -> Line2 {
        Line2::new(self.p, self.d)
    }
    pub fn circ(&self) -> Result<Circle2, Failure> {
        Circle2::new(self.ax()?, self.r1)
    }
    pub fn elips(&self) -> Result<Ellipse2, Failure> {
        Ellipse2::new(self.ax()?, self.r1, self.r2)
    }
    pub fn parab(&self) -> Result<Parabola2, Failure> {
        Parabola2::new(self.ax()?, self.r1)
    }
    pub fn hypr(&self) -> Result<Hyperbola2, Failure> {
        Hyperbola2::new(self.ax()?, self.r1, self.r2)
    }
    pub fn value(&self, u: f64) -> Result<Point2, Failure> {
        Ok(match self.kind {
            0 => elclib::value(u, &self.lin()),
            1 => elclib::value(u, &self.circ()?),
            2 => elclib::value(u, &self.elips()?),
            3 => elclib::value(u, &self.parab()?),
            _ => elclib::value(u, &self.hypr()?),
        })
    }
}

pub fn read_conic(r: &mut Reader<'_>) -> Conic {
    let kind = r.integer();
    let p = r.pnt2d();
    let (dx, dy) = (r.real(), r.real());
    let d = Direction2::new(dx, dy).expect("probe direction");
    let (sense, r1, r2) = (r.flag(), r.real(), r.real());
    Conic {
        kind,
        p,
        d,
        sense,
        r1,
        r2,
    }
}

/// `GaDomainSpec`.
#[derive(Clone, Copy)]
pub struct DomainSpec {
    kind: i32,
    p1: f64,
    t1: f64,
    p2: f64,
    t2: f64,
    closed: bool,
    z: f64,
    zp: f64,
}

pub fn read_domain(r: &mut Reader<'_>) -> DomainSpec {
    let kind = r.integer();
    let (p1, t1, p2, t2) = (r.real(), r.real(), r.real(), r.real());
    let (closed, z, zp) = (r.flag(), r.real(), r.real());
    DomainSpec {
        kind,
        p1,
        t1,
        p2,
        t2,
        closed,
        z,
        zp,
    }
}

/// `ga_domain`: the domain of a spec on a curve evaluated by `value`.
pub fn domain(
    s: &DomainSpec,
    value: impl Fn(f64) -> Result<Point2, Failure>,
) -> Result<Domain, Failure> {
    let mut d = Domain::new();
    match s.kind {
        1 => d.set_values(value(s.p1)?, s.p1, s.t1, value(s.p2)?, s.p2, s.t2),
        2 => d.set_semi_infinite_values(value(s.p1)?, s.p1, s.t1, true),
        3 => d.set_semi_infinite_values(value(s.p2)?, s.p2, s.t2, false),
        _ => {}
    }
    if s.closed {
        d.set_equivalent_parameters(s.z, s.zp)?;
    }
    Ok(d)
}

fn conic_domain(s: &DomainSpec, c: &Conic) -> Result<Domain, Failure> {
    domain(s, |u| c.value(u))
}

fn itool(c: &Conic) -> Result<IConicTool, Failure> {
    Ok(match c.kind {
        0 => IConicTool::from_line(c.lin())?,
        1 => IConicTool::from_circle(c.circ()?),
        2 => IConicTool::from_ellipse(c.elips()?),
        3 => IConicTool::from_parabola(c.parab()?),
        4 => IConicTool::from_hyperbola(c.hypr()?),
        _ => IConicTool::new(),
    })
}

fn pconic(c: &Conic) -> Result<PConic, Failure> {
    Ok(match c.kind {
        0 => PConic::from_line(c.lin())?,
        1 => PConic::from_circle(c.circ()?),
        2 => PConic::from_ellipse(c.elips()?),
        3 => PConic::from_parabola(c.parab()?),
        _ => PConic::from_hyperbola(c.hypr()?),
    })
}

type Pair<'a> = (&'a Conic, &'a Domain, &'a Conic, &'a Domain);

fn perform_pair(
    i: &mut IntConicConic,
    (a, da, b, db): Pair<'_>,
    tc: f64,
    t: f64,
) -> Result<(), Failure> {
    match a.kind * 10 + b.kind {
        0 => i.perform_lines(a.lin(), da, b.lin(), db, tc, t),
        1 => i.perform_line_circle(a.lin(), da, b.circ()?, db, tc, t),
        2 => i.perform_line_ellipse(a.lin(), da, b.elips()?, db, tc, t),
        3 => i.perform_line_parabola(a.lin(), da, b.parab()?, db, tc, t),
        4 => i.perform_line_hyperbola(a.lin(), da, b.hypr()?, db, tc, t),
        11 => i.perform_circles(a.circ()?, da, b.circ()?, db, tc, t),
        12 => i.perform_circle_ellipse(a.circ()?, da, b.elips()?, db, tc, t),
        13 => i.perform_circle_parabola(a.circ()?, da, b.parab()?, db, tc, t),
        14 => i.perform_circle_hyperbola(a.circ()?, da, b.hypr()?, db, tc, t),
        22 => i.perform_ellipses(a.elips()?, da, b.elips()?, db, tc, t),
        23 => i.perform_ellipse_parabola(a.elips()?, da, b.parab()?, db, tc, t),
        24 => i.perform_ellipse_hyperbola(a.elips()?, da, b.hypr()?, db, tc, t),
        33 => i.perform_parabolas(a.parab()?, da, b.parab()?, db, tc, t),
        34 => i.perform_parabola_hyperbola(a.parab()?, da, b.hypr()?, db, tc, t),
        _ => i.perform_hyperbolas(a.hypr()?, da, b.hypr()?, db, tc, t),
    }
}

/// `conicA domainA conicB domainB tolconf tol reversed`.
pub fn conics_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (a, sa) = (read_conic(&mut r), read_domain(&mut r));
    let (b, sb) = (read_conic(&mut r), read_domain(&mut r));
    let (tc, t, rev) = (r.real(), r.real(), r.flag());
    attempt(|| {
        let (da, db) = (conic_domain(&sa, &a)?, conic_domain(&sb, &b)?);
        let mut result = IntConicConic::new();
        result.set_reversed_parameters(rev);
        perform_pair(&mut result, (&a, &da, &b, &db), tc, t)?;
        inter(&result)
    });
    end();
}

/// `conicA domainA conicB domainB tolconf tol reversed accuracy epsx`.
pub fn imppar_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (a, sa) = (read_conic(&mut r), read_domain(&mut r));
    let (b, sb) = (read_conic(&mut r), read_domain(&mut r));
    let (tc, t, rev) = (r.real(), r.real(), r.flag());
    let (accuracy, epsx) = (r.integer(), r.real());
    attempt(|| {
        let (da, db) = (conic_domain(&sa, &a)?, conic_domain(&sb, &b)?);
        let mut pc = pconic(&b)?;
        pc.set_accuracy(accuracy);
        pc.set_eps_x(epsx);
        let mut result = IntImpConicParConic::new();
        result.set_reversed_parameters(rev);
        result.perform(&itool(&a)?, &da, &pc, &db, tc, t)?;
        inter(&result)
    });
    end();
}

/// `conic u x y`: every member of IConicTool, D1/D2 outputs set to sentinels first.
pub fn itool_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let c = read_conic(&mut r);
    let u = r.real();
    let p = r.pnt2d();
    attempt(|| {
        let tool = itool(&c)?;
        let copy = tool;
        pnt2d(copy.value(u));
        let (mut q, mut t, mut n) = (
            Point2::new(-7.0, -8.0),
            Vector2::new(-9.0, -10.0),
            Vector2::new(-11.0, -12.0),
        );
        tool.d1(u, &mut q, &mut t);
        pnt2d(q);
        v2(t);
        (q, t) = (Point2::new(-7.0, -8.0), Vector2::new(-9.0, -10.0));
        tool.d2(u, &mut q, &mut t, &mut n);
        pnt2d(q);
        v2(t);
        v2(n);
        real(tool.distance(p));
        v2(tool.grad_distance(p));
        show(tool.find_parameter(p), real);
        Ok(())
    });
    end();
}

/// `conic u x y low high accuracy epsx`: PConic, PConicTool and ProjectOnPConicTool.
pub fn ptool_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let c = read_conic(&mut r);
    let u = r.real();
    let p = r.pnt2d();
    let (low, high, accuracy, epsx) = (r.real(), r.real(), r.integer(), r.real());
    attempt(|| {
        let mut pc = pconic(&c)?;
        let copy = pc;
        int(copy.type_curve() as i32);
        real(copy.param1());
        real(copy.param2());
        int(copy.accuracy());
        real(copy.eps_x());
        pnt2d(copy.axis2().location());
        xy(copy.axis2().x_direction().xy());
        xy(copy.axis2().y_direction().xy());
        pc.set_accuracy(accuracy);
        pc.set_eps_x(epsx);
        int(PConicTool::nb_samples(&pc));
        int(PConicTool::nb_samples_in(&pc, low, high));
        real(PConicTool::eps_x(&pc));
        pnt2d(PConicTool::value(&pc, u));
        let (mut q, mut t, mut n) = (
            Point2::new(-7.0, -8.0),
            Vector2::new(-9.0, -10.0),
            Vector2::new(-11.0, -12.0),
        );
        PConicTool::d1(&pc, u, &mut q, &mut t);
        pnt2d(q);
        v2(t);
        (q, t) = (Point2::new(-7.0, -8.0), Vector2::new(-9.0, -10.0));
        PConicTool::d2(&pc, u, &mut q, &mut t, &mut n);
        pnt2d(q);
        v2(t);
        v2(n);
        show(ProjectOnPConicTool::find_parameter(&pc, p, 1e-7), real);
        show(
            ProjectOnPConicTool::find_parameter_in(&pc, p, low, high, 1e-7),
            real,
        );
        Ok(())
    });
    end();
}

/// `conicA conicB u`: the function of IntImpConicParConic.
pub fn function_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (a, b) = (read_conic(&mut r), read_conic(&mut r));
    let u = r.real();
    attempt(|| {
        let pc = pconic(&b)?;
        let tool = itool(&a)?;
        let mut f = MyImpParToolOfIntImpConicParConic::new(&tool, &pc);
        let (mut v, mut d) = (-1.0, -2.0);
        flag(f.value(u, &mut v));
        real(v);
        flag(f.derivative(u, &mut d));
        real(d);
        (v, d) = (-1.0, -2.0);
        flag(f.values(u, &mut v, &mut d));
        real(v);
        real(d);
        Ok(())
    });
    end();
}

fn pr(i: &mut PeriodicInterval) {
    flag(i.is_null());
    real(i.binf);
    real(i.bsup);
    real(i.length());
}
fn ir(i: &mut Interval) {
    flag(i.is_null);
    flag(i.has_first_bound);
    flag(i.has_last_bound);
    real(i.binf);
    real(i.bsup);
    real(i.length());
}

/// `domain a b ha hb`: Interval and PeriodicInterval.
pub fn interval_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = read_domain(&mut r);
    let (a, b, ha, hb) = (r.real(), r.real(), r.flag(), r.flag());
    attempt(|| {
        let d = domain(&s, |u| Ok(Point2::new(u, 0.0)))?;
        let mut id = Interval::from_domain(&d)?;
        ir(&mut id);
        let mut ab = Interval::new(a, b);
        ir(&mut ab);
        let mut hab = Interval::with_bounds(a, ha, b, hb);
        ir(&mut hab);
        ir(&mut hab.intersection_with_bounded(&ab));
        ir(&mut id.intersection_with_bounded(&ab));
        ir(&mut Interval::default());
        let mut pd = PeriodicInterval::from_domain(&d)?;
        pr(&mut pd);
        let mut pab = PeriodicInterval::new(a, b);
        pr(&mut pab);
        let mut c = pab;
        c.complement();
        pr(&mut c);
        let mut n2 = pd;
        n2.normalize();
        pr(&mut n2);
        let mut sv = PeriodicInterval::default();
        sv.set_values(b, a);
        pr(&mut sv);
        let mut x = pab;
        pr(&mut n2.first_intersection(&mut x));
        pr(&mut x);
        pr(&mut n2.second_intersection(&mut x));
        let mut sn = pab;
        sn.set_null();
        pr(&mut sn);
        Ok(())
    });
    end();
}
