// Rust twin of tools/oracle/tkgeomalgo_probe_gfill_fill.cxx: the generator, the filling
// schemes and the fillings of two to four curves.
use super::common::{Reader, attempt, end, flag, int, pnt, real};
use super::gfill::{dump_bspline, read_bezier, read_bspline, read_curve};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomBezierCurve, GeomBezierSurface, GeomSurface};
use rustycad_tkgeomalgo::geom_fill::{
    GeomFillBSplineCurves, GeomFillBezierCurves, GeomFillCoons, GeomFillCurved, GeomFillFilling,
    GeomFillFillingStyle, GeomFillGenerator, GeomFillStretch,
};
use rustycad_tkmath::gp::Point3;

/// The pole grid access both surface kinds share.
trait PoleGrid {
    fn degrees(&self) -> (i32, i32);
    fn rational(&self) -> (bool, bool);
    fn counts(&self) -> (i32, i32);
    fn pole_weight(&self, i: i32, j: i32) -> (Point3, f64);
}

impl PoleGrid for GeomBSplineSurface {
    fn degrees(&self) -> (i32, i32) {
        (self.u_degree(), self.v_degree())
    }
    fn rational(&self) -> (bool, bool) {
        (self.is_u_rational(), self.is_v_rational())
    }
    fn counts(&self) -> (i32, i32) {
        (self.nb_u_poles(), self.nb_v_poles())
    }
    fn pole_weight(&self, i: i32, j: i32) -> (Point3, f64) {
        (self.pole(i, j).unwrap(), self.weight(i, j).unwrap())
    }
}

impl PoleGrid for GeomBezierSurface {
    fn degrees(&self) -> (i32, i32) {
        (self.u_degree(), self.v_degree())
    }
    fn rational(&self) -> (bool, bool) {
        (self.is_u_rational(), self.is_v_rational())
    }
    fn counts(&self) -> (i32, i32) {
        (self.nb_u_poles(), self.nb_v_poles())
    }
    fn pole_weight(&self, i: i32, j: i32) -> (Point3, f64) {
        (self.pole(i, j).unwrap(), self.weight(i, j).unwrap())
    }
}

fn dump_surface_poles<S: PoleGrid + GeomSurface>(s: &S) {
    let (du, dv) = s.degrees();
    int(du);
    int(dv);
    flag(s.is_u_periodic());
    flag(s.is_v_periodic());
    let (ru, rv) = s.rational();
    flag(ru);
    flag(rv);
    let (nu, nv) = s.counts();
    int(nu);
    int(nv);
    for i in 1..=nu {
        for j in 1..=nv {
            let (p, w) = s.pole_weight(i, j);
            pnt(p);
            real(w);
        }
    }
}

pub fn dump_bsurface(s: &Handle<GeomBSplineSurface>) {
    let Some(s) = s.get() else {
        print!("null ");
        return;
    };
    dump_surface_poles(s);
    s.u_knots().iter().for_each(|&k| real(k));
    s.u_multiplicities().iter().for_each(|&m| int(m));
    s.v_knots().iter().for_each(|&k| real(k));
    s.v_multiplicities().iter().for_each(|&m| int(m));
}

fn dump_bzsurface(s: &Handle<GeomBezierSurface>) {
    match s.get() {
        Some(s) => dump_surface_poles(s),
        None => print!("null "),
    }
}

pub fn generator_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let ptol = r.real();
    let mut g = GeomFillGenerator::new();
    let n = r.integer();
    for _ in 0..n {
        let curve = read_curve(&mut r).expect("probe curve");
        attempt(|| g.add_curve(&curve));
    }
    dump_bsurface(&g.surface().cast());
    attempt(|| g.perform(ptol));
    attempt(|| g.degree().map(int));
    dump_bsurface(&g.surface().cast());
    end();
}

fn pnt_row(r: &mut Reader) -> Array1<'static, Point3> {
    let (lower, n) = (r.integer(), r.integer());
    let mut a = Array1::with_bounds(lower, lower + n - 1).unwrap();
    (lower..lower + n).for_each(|i| a[i] = r.pnt());
    a
}

fn real_row(r: &mut Reader) -> Array1<'static, f64> {
    let (lower, n) = (r.integer(), r.integer());
    let mut a = Array1::with_bounds(lower, lower + n - 1).unwrap();
    (lower..lower + n).for_each(|i| a[i] = r.real());
    a
}

fn dump_filling(f: &GeomFillFilling, weights: bool) -> Result<(), Failure> {
    flag(f.is_rational());
    int(f.nb_u_poles()?);
    int(f.nb_v_poles()?);
    let mut p: Array2<'static, Point3> = Array2::with_bounds(0, 0, 0, 0).unwrap();
    f.poles(&mut p)?;
    int(p.lower_row());
    int(p.lower_col());
    p.iter().for_each(|&x| pnt(x));
    if !weights {
        return Ok(());
    }
    let mut w: Array2<'static, f64> = Array2::with_bounds(0, 0, 0, 0).unwrap();
    f.weights(&mut w)?;
    int(w.col_length());
    int(w.row_length());
    w.iter().for_each(|&x| real(x));
    Ok(())
}

enum Scheme {
    Stretch(GeomFillStretch),
    Coons(GeomFillCoons),
    Curved(GeomFillCurved),
}

impl Scheme {
    fn filling(&self) -> &GeomFillFilling {
        match self {
            Scheme::Stretch(s) => s,
            Scheme::Coons(s) => s,
            Scheme::Curved(s) => s,
        }
    }
}

fn init_scheme(
    scheme: &mut Scheme,
    p: &[Array1<'static, Point3>],
    w: Option<&[Array1<'static, f64>]>,
) -> Result<(), Failure> {
    let two_rows = p.len() == 2;
    match (scheme, w) {
        (Scheme::Curved(s), Some(w)) if two_rows => {
            s.init_two_weighted_rows(&p[0], &p[1], &w[0], &w[1])
        }
        (Scheme::Curved(s), None) if two_rows => s.init_two_rows(&p[0], &p[1]),
        (Scheme::Stretch(s), Some(w)) => s.init_weighted(rows(p), rows(w)),
        (Scheme::Stretch(s), None) => s.init(rows(p)),
        (Scheme::Coons(s), Some(w)) => s.init_weighted(rows(p), rows(w)),
        (Scheme::Coons(s), None) => s.init(rows(p)),
        (Scheme::Curved(s), Some(w)) => s.init_weighted(rows(p), rows(w)),
        (Scheme::Curved(s), None) => s.init(rows(p)),
    }
}

fn rows<'a, T>(a: &'a [Array1<'static, T>]) -> [&'a Array1<'static, T>; 4] {
    [&a[0], &a[1], &a[2], &a[3]]
}

pub fn scheme_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (scheme_code, weighted) = (r.integer(), r.flag());
    let n = if scheme_code == 3 { 2 } else { 4 };
    let p: Vec<_> = (0..n).map(|_| pnt_row(&mut r)).collect();
    let w: Vec<_> = if weighted {
        (0..n).map(|_| real_row(&mut r)).collect()
    } else {
        Vec::new()
    };
    let again = r.flag();
    let mut scheme = match scheme_code {
        0 => Scheme::Stretch(GeomFillStretch::new()),
        1 => Scheme::Coons(GeomFillCoons::new()),
        _ => Scheme::Curved(GeomFillCurved::new()),
    };
    let weights = weighted.then_some(w.as_slice());
    let error = init_scheme(&mut scheme, &p, weights)
        .err()
        .map(|e| e.exception_type().to_string());
    if let Some(error) = &error {
        print!("{error} ");
    }
    let ok = error.is_none();
    if ok || error.as_deref() != Some("Standard_DomainError") {
        attempt(|| dump_filling(scheme.filling(), ok && weighted));
    }
    if again && ok {
        attempt(|| init_scheme(&mut scheme, &p, None));
        attempt(|| dump_filling(scheme.filling(), weighted));
    }
    end();
}

fn style(code: i32) -> GeomFillFillingStyle {
    [
        GeomFillFillingStyle::StretchStyle,
        GeomFillFillingStyle::CoonsStyle,
        GeomFillFillingStyle::CurvedStyle,
    ][code as usize]
}

pub fn bezier_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (style, n) = (style(r.integer()), r.integer());
    let c: Vec<Handle<GeomBezierCurve>> = (0..n)
        .map(|_| {
            r.integer();
            read_bezier(&mut r).expect("probe curve")
        })
        .collect();
    let mut f = GeomFillBezierCurves::new();
    dump_bzsurface(&f.surface());
    attempt(|| match n {
        4 => f.init_four(&c[0], &c[1], &c[2], &c[3], style),
        3 => f.init_three(&c[0], &c[1], &c[2], style),
        _ => f.init_two(&c[0], &c[1], style),
    });
    dump_bzsurface(&f.surface());
    c[0].poles().iter().for_each(|&p| pnt(p));
    end();
}

pub fn bspline_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (style, n) = (style(r.integer()), r.integer());
    let c: Vec<_> = (0..n)
        .map(|_| {
            r.integer();
            read_bspline(&mut r).expect("probe curve")
        })
        .collect();
    let mut f = GeomFillBSplineCurves::new();
    dump_bsurface(&f.surface());
    attempt(|| match n {
        4 => f.init_four(&c[0], &c[1], &c[2], &c[3], style),
        3 => f.init_three(&c[0], &c[1], &c[2], style),
        _ => f.init_two(&c[0], &c[1], style),
    });
    dump_bsurface(&f.surface());
    dump_bspline(&c[0]);
    end();
}
