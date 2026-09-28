//! The IntAna2d package probe; the counterpart of tools/oracle/tkgeombase_probe_intana2d.cxx.
use super::common::{self as p, Reader, attempt, end, flag, int, real};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana2d::{
    AnaIntersection, Conic, IntPoint, MyDirectPolynomialRoots, coord_ancien_repere,
    points_confondus, traitement_points_confondus,
};
use rustycad_tkmath::gp::{Circle2, Ellipse2, Hyperbola2, Line2, Parabola2};

/// One 2D element of the corpus (GbConic2d).
enum Conic2d {
    Line(Line2),
    Circle(Circle2),
    Ellipse(Ellipse2),
    Parabola(Parabola2),
    Hyperbola(Hyperbola2),
}

fn conic2d(r: &mut Reader) -> Result<Conic2d, Failure> {
    Ok(match r.integer() {
        0 => {
            let point = r.pnt2d();
            Conic2d::Line(Line2::new(point, r.dir2d()))
        }
        1 => {
            let a = r.ax22d();
            Conic2d::Circle(Circle2::new(a, r.real())?)
        }
        2 => {
            let (a, major) = (r.ax22d(), r.real());
            Conic2d::Ellipse(Ellipse2::new(a, major, r.real())?)
        }
        3 => {
            let a = r.ax22d();
            Conic2d::Parabola(Parabola2::new(a, r.real())?)
        }
        _ => {
            let (a, major) = (r.ax22d(), r.real());
            Conic2d::Hyperbola(Hyperbola2::new(a, major, r.real())?)
        }
    })
}

fn implicit(element: &Conic2d) -> Conic {
    match *element {
        Conic2d::Line(l) => Conic::from_line(l),
        Conic2d::Circle(c) => Conic::from_circle(c),
        Conic2d::Ellipse(e) => Conic::from_ellipse(e),
        Conic2d::Parabola(x) => Conic::from_parabola(x),
        Conic2d::Hyperbola(h) => Conic::from_hyperbola(h),
    }
}

fn int_point(point: IntPoint) {
    p::pnt2d(point.value());
    real(point.param_on_first());
    flag(point.second_is_implicit());
    p::show(point.param_on_second(), real);
}

fn ana(a: &AnaIntersection) -> Result<(), Failure> {
    flag(a.is_done());
    if !a.is_done() {
        p::show(a.nb_points(), int);
        return Ok(());
    }
    flag(a.is_empty()?);
    flag(a.identical_elements()?);
    flag(a.parallel_elements()?);
    let n = a.nb_points()?;
    int(n);
    for i in 1..=n {
        int_point(a.point(i)?);
    }
    p::show(a.point(n + 1), int_point);
    Ok(())
}

pub fn ia2_lines_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let point = r.pnt2d();
        let l1 = Line2::new(point, r.dir2d());
        let point = r.pnt2d();
        let l2 = Line2::new(point, r.dir2d());
        ana(&AnaIntersection::lines(l1, l2))
    });
    end();
}

pub fn ia2_circles_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let a1 = r.ax22d();
        let c1 = Circle2::new(a1, r.real())?;
        let a2 = r.ax22d();
        let c2 = Circle2::new(a2, r.real())?;
        let mut a = AnaIntersection::new();
        a.perform_circles(c1, c2)?;
        ana(&a)
    });
    end();
}

pub fn ia2_line_circle_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let point = r.pnt2d();
        let l = Line2::new(point, r.dir2d());
        let a1 = r.ax22d();
        let c = Circle2::new(a1, r.real())?;
        let mut a = AnaIntersection::new();
        a.perform_line_circle(l, c)?;
        ana(&a)
    });
    end();
}

pub fn ia2_conic_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let first = conic2d(&mut r)?;
        let conic = implicit(&conic2d(&mut r)?);
        let mut a = AnaIntersection::new();
        match first {
            Conic2d::Line(l) => a.perform_line_conic(l, &conic)?,
            Conic2d::Circle(c) => a.perform_circle_conic(c, &conic)?,
            Conic2d::Ellipse(e) => a.perform_ellipse_conic(e, &conic)?,
            Conic2d::Parabola(x) => a.perform_parabola_conic(x, &conic)?,
            Conic2d::Hyperbola(h) => a.perform_hyperbola_conic(h, &conic)?,
        }
        ana(&a)
    });
    end();
}

pub fn ia2_conic_eval_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let conic = implicit(&conic2d(&mut r)?);
        let (x, y) = (r.real(), r.real());
        let axis = r.ax2d();
        conic.coefficients().into_iter().for_each(real);
        real(conic.value(x, y));
        p::xy(conic.grad(x, y));
        let (value, gradient) = conic.val_and_grad(x, y);
        real(value);
        p::xy(gradient);
        conic
            .new_coefficients(conic.coefficients(), axis)
            .into_iter()
            .for_each(real);
        Ok(())
    });
    end();
}

pub fn ia2_roots_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let roots = if r.integer() == 0 {
            let (a4, a3, a2, a1) = (r.real(), r.real(), r.real(), r.real());
            MyDirectPolynomialRoots::quartic(a4, a3, a2, a1, r.real())?
        } else {
            let (a2, a1) = (r.real(), r.real());
            MyDirectPolynomialRoots::quadratic(a2, a1, r.real())?
        };
        flag(roots.is_done());
        flag(roots.infinite_roots());
        let n = roots.nb_solutions();
        int(n);
        for i in 1..=n {
            real(roots.value(i)?);
        }
        Ok(())
    });
    end();
}

pub fn ia2_outils_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (x1, y1, x2, y2) = (r.real(), r.real(), r.real(), r.real());
    flag(points_confondus(x1, y1, x2, y2));
    let (x, y) = (r.real(), r.real());
    let (x, y) = coord_ancien_repere(x, y, r.ax2d());
    real(x);
    real(y);
    let n = r.integer();
    let mut pts = [IntPoint::default(); 4];
    for point in pts.iter_mut().take(n as usize) {
        let (px, py, u1) = (r.real(), r.real(), r.real());
        point.set_value(px, py, u1, r.real());
    }
    int(traitement_points_confondus(n, &mut pts));
    pts.into_iter().for_each(int_point);
    end();
}

pub fn ia2_int_point_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (x, y, u1, u2) = (r.real(), r.real(), r.real(), r.real());
    let mut d = IntPoint::default();
    int_point(d);
    int_point(IntPoint::new(x, y, u1, u2));
    int_point(IntPoint::implicit(x, y, u1));
    d.set_implicit_value(y, x, u2);
    int_point(d);
    d.set_value(y, x, u2, u1);
    int_point(d);
    let empty = AnaIntersection::new();
    attempt(|| ana(&empty));
    p::show(empty.is_empty(), flag);
    p::show(empty.identical_elements(), flag);
    p::show(empty.parallel_elements(), flag);
    p::show(empty.point(1), int_point);
    end();
}
