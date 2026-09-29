//! AppBlend probe operations, the Rust twin of tools/oracle/tkgeomalgo_probe_appblend.cxx:
//! `AppBlendAppSurf` on the same probe line and section generator.
use super::common::{Reader, end, flag, int, pnt, pnt2d, real, show};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::app_blend::*;
use rustycad_tkgeombase::approx::ParametrizationType;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};

/// `n` line points: Point(i) is the section index i.
struct Line(i32);

impl TheLine for Line {
    type Point = i32;
    fn nb_points(&self) -> i32 {
        self.0
    }
    fn point(&self, index: i32) -> i32 {
        index
    }
}

/// The probe sections of GaAbSections.
struct Sections {
    n: i32,
    np2d: i32,
    len: f64,
    abcd: [f64; 4],
    deriv: bool,
}

const BASE: [[f64; 3]; 3] = [[0.0, 0.0, 0.0], [1.0, 1.0, 0.2], [2.0, 0.0, 0.0]];

impl Sections {
    fn t(&self, i: i32) -> f64 {
        f64::from(i - 1) / f64::from(self.n - 1) * self.len
    }
    fn fill(&self, i: i32, s: &mut AppBlendSection) {
        let [a, b, c, d] = self.abcd;
        let t: f64 = self.t(i);
        for (j, [x, y, z]) in BASE.iter().copied().enumerate() {
            s.poles[j as i32 + 1] = Point3::new(x + a * t, y * (1.0 + b * t), c * t * t + z);
        }
        (s.weights[1], s.weights[2], s.weights[3]) = (1.0, 0.7 + d * t, 1.0);
        for k in 1..=self.np2d {
            s.poles2d[k] = Point2::new(t, f64::from(k) * t * t);
        }
    }
}

impl TheSectionGenerator for Sections {
    type Point = i32;
    fn get_shape(&self) -> [i32; 4] {
        [3, 2, 2, self.np2d]
    }
    fn knots(&self, knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
        let lower: i32 = knots.lower();
        (knots[lower], knots[lower + 1]) = (0.0, 1.0);
        Ok(())
    }
    fn mults(&self, mults: &mut Array1<'_, i32>) -> Result<(), Failure> {
        let lower: i32 = mults.lower();
        (mults[lower], mults[lower + 1]) = (3, 3);
        Ok(())
    }
    fn section_d1(&mut self, p: &i32, s: &mut AppBlendSection) -> Result<bool, Failure> {
        self.fill(*p, s);
        let [a, b, c, d] = self.abcd;
        let t: f64 = self.t(*p);
        for (j, base) in [0.0, 1.0, 0.0].into_iter().enumerate() {
            s.d_poles[j as i32 + 1] = Vector3::new(a, base * b, 2.0 * c * t);
        }
        (s.d_weights[1], s.d_weights[2], s.d_weights[3]) = (0.0, d, 0.0);
        for k in 1..=self.np2d {
            s.d_poles2d[k] = Vector2::new(1.0, 2.0 * f64::from(k) * t);
        }
        Ok(self.deriv)
    }
    fn section(&mut self, p: &i32, s: &mut AppBlendSection) -> Result<(), Failure> {
        self.fill(*p, s);
        Ok(())
    }
    fn parameter(&self, p: &i32) -> f64 {
        self.t(*p)
    }
}

fn report(s: &AppBlendAppSurf) {
    flag(s.is_done());
    show(s.surf_shape(), |shape| shape.into_iter().for_each(int));
    show(
        s.surf_poles().and_then(|p| Ok((p, s.surf_weights()?))),
        |(p, w)| {
            for i in p.lower_row()..=p.upper_row() {
                for j in p.lower_col()..=p.upper_col() {
                    pnt(p[(i, j)]);
                    real(w[(i, j)]);
                }
            }
        },
    );
    show(
        s.surf_v_knots().and_then(|k| Ok((k, s.surf_v_mults()?))),
        |(k, m)| {
            int(k.lower());
            k.iter().for_each(|&x| real(x));
            m.iter().for_each(|&x| int(x));
        },
    );
    show(
        s.surf_u_knots().and_then(|k| Ok((k, s.surf_u_mults()?))),
        |(k, m)| {
            k.iter().for_each(|&x| real(x));
            m.iter().for_each(|&x| int(x));
        },
    );
    show(s.tol_reached(), |(t3, t2)| {
        real(t3);
        real(t2);
    });
    real(s.tol_curve_on_surf(1));
    report_curves2d(s);
}

fn report_curves2d(s: &AppBlendAppSurf) {
    show(s.nb_curves2d(), int);
    show(s.curves2d_shape(), |shape| shape.into_iter().for_each(int));
    show(s.curves2d_degree(), int);
    for i in 1..=2 {
        show(s.curve2d_poles(i), |p| p.iter().for_each(|&q| pnt2d(q)));
    }
    show(s.curve2d(1), |c| {
        int(c.poles.length());
        int(c.knots.length());
        int(c.mults.length());
    });
    show(s.surface(), |surface| {
        int(surface.poles.col_length());
        int(surface.poles.row_length());
        int(surface.v_knots.length());
    });
}

const PARAMETRIZATIONS: [ParametrizationType; 3] = [
    ParametrizationType::ChordLength,
    ParametrizationType::Centripetal,
    ParametrizationType::IsoParametric,
];
const SHAPES: [Shape; 7] = [
    Shape::C0,
    Shape::G1,
    Shape::C1,
    Shape::G2,
    Shape::C2,
    Shape::C3,
    Shape::CN,
];

/// `n len a b c d np2d deriv mode dmin dmax tol3d tol2d nbit knownp partype cont nbmaxp
/// notgt w1 w2 w3`: GaAbAppSurf computed twice, then on a null line.
pub fn appsurf_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (n, len) = (r.integer(), r.real());
    let abcd = [r.real(), r.real(), r.real(), r.real()];
    let (np2d, deriv, mode) = (r.integer(), r.flag(), r.integer());
    let (dmin, dmax, t3, t2, nbit) = (r.integer(), r.integer(), r.real(), r.real(), r.integer());
    let (knownp, partype, cont, nbmaxp, notgt) =
        (r.flag(), r.integer(), r.integer(), r.integer(), r.flag());
    let w = [r.real(), r.real(), r.real()];
    let mut f = Sections {
        n,
        np2d,
        len,
        abcd,
        deriv,
    };
    let line = Line(n);
    let mut s = AppBlendAppSurf::new(dmin, dmax, t3, t2, nbit, knownp);
    s.set_par_type(PARAMETRIZATIONS[partype as usize]);
    s.set_continuity(SHAPES[cont as usize]);
    if let Err(error) = s.set_criterium_weight(w[0], w[1], w[2]) {
        print!("{} ", error.exception_type());
    }
    s.criterium_weight().into_iter().for_each(real);
    int(s.par_type() as i32);
    int(s.continuity() as i32);
    set_context_approx_with_no_tgt(notgt);
    for _ in 0..2 {
        let result = match mode {
            0 => s.perform(Some(&line), &mut f, false),
            1 => s.perform(Some(&line), &mut f, true),
            2 => s.perform_smoothing(Some(&line), &mut f),
            _ => s.perform_by_pieces(Some(&line), &mut f, nbmaxp),
        };
        if let Err(error) = result {
            print!("{} ", error.exception_type());
        }
        report(&s);
    }
    set_context_approx_with_no_tgt(false);
    if let Err(error) = s.perform(None::<&Line>, &mut f, false) {
        print!("{} ", error.exception_type());
    }
    flag(s.is_done());
    end();
}

/// `b1 b2`: the two context switches set, read and reset.
pub fn context_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (b1, b2) = (r.flag(), r.flag());
    set_context_spline_approx(b1);
    set_context_approx_with_no_tgt(b2);
    flag(get_context_spline_approx());
    flag(get_context_approx_with_no_tgt());
    set_context_spline_approx(false);
    set_context_approx_with_no_tgt(false);
    flag(get_context_spline_approx());
    flag(get_context_approx_with_no_tgt());
    end();
}
