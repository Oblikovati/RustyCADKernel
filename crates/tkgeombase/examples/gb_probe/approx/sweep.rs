//! The Rust twin of tools/oracle/tkgeombase_probe_approx_sweep.cxx.
use super::super::common::{self as p, Reader, attempt, end};
use super::shape;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkgeombase::approx::{
    SweepApproximation, SweepD1, SweepD2, SweepFunction, SweepOptions,
};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use std::sync::Mutex;

/// The settings of GbAxSweep.
#[derive(Clone, Copy, Default)]
struct Settings {
    nbpol: i32,
    nb2d: i32,
    rational: bool,
    d1mode: i32,
    d2mode: i32,
    resolution: bool,
    nbc2: i32,
    nbc3: i32,
    pfirst: f64,
    plast: f64,
}

/// What the approximation told the function: SetInterval and SetTolerance.
#[derive(Clone, Copy, Default)]
struct Record {
    first: f64,
    last: f64,
    tol3d: f64,
    tol2d: f64,
    nbinterval: i32,
}

/// GbAxSweep.
struct AxSweep {
    parent: StandardTransient,
    settings: Settings,
    record: Mutex<Record>,
}
impl_transient!(
    AxSweep,
    "GbAxSweep",
    dyn SweepFunction,
    parent,
    interfaces: [dyn SweepFunction]
);

fn pole(k: i32, t: f64) -> Point3 {
    let k = f64::from(k);
    Point3::new(
        t + 0.3 * k,
        0.5 * t * t + 0.1 * k * t,
        0.2 * k - 0.1 * t * t * t,
    )
}

fn dpole(k: i32, t: f64) -> Vector3 {
    Vector3::new(1.0, t + 0.1 * f64::from(k), -0.3 * t * t)
}

fn not_implemented(member: &str) -> Failure {
    Failure::with_type("Standard_NotImplemented", member, "")
}

impl AxSweep {
    fn weight(&self, k: i32, t: f64) -> f64 {
        if self.settings.rational {
            1.0 + 0.1 * f64::from(k) * t
        } else {
            1.0
        }
    }
    fn dweight(&self, k: i32) -> f64 {
        if self.settings.rational {
            0.1 * f64::from(k)
        } else {
            0.0
        }
    }
    fn values(
        &self,
        t: f64,
        poles: &mut Array1<'_, Point3>,
        poles2d: &mut Array1<'_, Point2>,
        weights: &mut Array1<'_, f64>,
    ) {
        for k in 1..=self.settings.nbpol {
            poles[k] = pole(k, t);
            weights[k] = self.weight(k, t);
        }
        for j in 1..=self.settings.nb2d {
            poles2d[j] = Point2::new(t, 0.5 * f64::from(j) + t * t);
        }
    }
    fn firsts(&self, t: f64, d: &mut SweepD1<'_, '_>) {
        self.values(t, d.poles, d.poles2d, d.weights);
        for k in 1..=self.settings.nbpol {
            d.d_poles[k] = dpole(k, t);
            d.d_weights[k] = self.dweight(k);
        }
        for j in 1..=self.settings.nb2d {
            d.d_poles2d[j] = Vector2::new(1.0, 2.0 * t);
        }
    }
}

impl SweepFunction for AxSweep {
    fn d0(
        &self,
        t: f64,
        _: f64,
        _: f64,
        poles: &mut Array1<'_, Point3>,
        poles2d: &mut Array1<'_, Point2>,
        weights: &mut Array1<'_, f64>,
    ) -> Result<bool, Failure> {
        self.values(t, poles, poles2d, weights);
        Ok(true)
    }
    fn d1(&self, t: f64, _: f64, _: f64, mut values: SweepD1<'_, '_>) -> Result<bool, Failure> {
        if self.settings.d1mode == 0 {
            return Err(not_implemented("Approx_SweepFunction::D1"));
        }
        self.firsts(t, &mut values);
        Ok(self.settings.d1mode == 1)
    }
    fn d2(&self, t: f64, _: f64, _: f64, mut values: SweepD2<'_, '_>) -> Result<bool, Failure> {
        if self.settings.d2mode == 0 {
            return Err(not_implemented("Approx_SweepFunction::D2"));
        }
        self.firsts(t, &mut values.d1);
        for k in 1..=self.settings.nbpol {
            values.d2_poles[k] = Vector3::new(0.0, 1.0, -0.6 * t);
            values.d2_weights[k] = 0.0;
        }
        for j in 1..=self.settings.nb2d {
            values.d2_poles2d[j] = Vector2::new(0.0, 2.0);
        }
        Ok(self.settings.d2mode == 1)
    }
    fn nb_2d_curves(&self) -> Result<i32, Failure> {
        Ok(self.settings.nb2d)
    }
    fn section_shape(&self) -> Result<(i32, i32, i32), Failure> {
        Ok((self.settings.nbpol, 2, self.settings.nbpol - 1))
    }
    fn knots(&self, knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
        let (lower, upper) = (knots.lower(), knots.upper());
        knots[lower] = 0.0;
        knots[upper] = 1.0;
        Ok(())
    }
    fn mults(&self, mults: &mut Array1<'_, i32>) -> Result<(), Failure> {
        let (lower, upper) = (mults.lower(), mults.upper());
        mults[lower] = self.settings.nbpol;
        mults[upper] = self.settings.nbpol;
        Ok(())
    }
    fn is_rational(&self) -> Result<bool, Failure> {
        Ok(self.settings.rational)
    }
    fn nb_intervals(&self, s: Shape) -> Result<i32, Failure> {
        Ok(if s >= Shape::C3 {
            self.settings.nbc3
        } else {
            self.settings.nbc2
        })
    }
    fn intervals(&self, t: &mut Array1<'_, f64>, s: Shape) -> Result<(), Failure> {
        let n = self.nb_intervals(s)?;
        let (a, b) = (self.settings.pfirst, self.settings.plast);
        for i in 1..=n + 1 {
            t[i] = a + (b - a) * f64::from(i - 1) / f64::from(n);
        }
        Ok(())
    }
    fn set_interval(&self, first: f64, last: f64) -> Result<(), Failure> {
        let mut record = self.record.lock().unwrap();
        (record.first, record.last) = (first, last);
        record.nbinterval += 1;
        Ok(())
    }
    fn resolution(&self, index: i32, tol: f64) -> Result<(f64, f64), Failure> {
        if !self.settings.resolution {
            return Err(not_implemented("Approx_SweepFunction::Resolution"));
        }
        Ok((tol * f64::from(index) * 0.75, tol))
    }
    fn get_tolerance(
        &self,
        _: f64,
        surf: f64,
        _: f64,
        t: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        for i in t.lower()..=t.upper() {
            t[i] = surf / (1.0 + 0.25 * f64::from(i - 1));
        }
        Ok(())
    }
    fn set_tolerance(&self, tol3d: f64, tol2d: f64) -> Result<(), Failure> {
        let mut record = self.record.lock().unwrap();
        (record.tol3d, record.tol2d) = (tol3d, tol2d);
        Ok(())
    }
    fn barycentre_of_surf(&self) -> Result<Point3, Failure> {
        if !self.settings.rational {
            return Err(not_implemented("Approx_SweepFunction::BarycentreOfSurf"));
        }
        Ok(Point3::new(0.5, 0.25, 0.1))
    }
    fn maximal_section(&self) -> Result<f64, Failure> {
        if !self.settings.rational {
            return Err(not_implemented("Approx_SweepFunction::MaximalSection()"));
        }
        Ok(2.0)
    }
    fn get_minimal_weight(&self, weights: &mut Array1<'_, f64>) -> Result<(), Failure> {
        if !self.settings.rational {
            return Err(not_implemented("Approx_SweepFunction::GetMinimalWeight"));
        }
        for k in weights.lower()..=weights.upper() {
            let (a, b) = (
                self.weight(k, self.settings.pfirst),
                self.weight(k, self.settings.plast),
            );
            weights[k] = if b < a { b } else { a };
        }
        Ok(())
    }
}

fn surface_results(a: &SweepApproximation) -> Result<(), Failure> {
    let shape = a.surf_shape()?;
    [
        shape.u_degree,
        shape.v_degree,
        shape.nb_u_poles,
        shape.nb_v_poles,
        shape.nb_u_knots,
        shape.nb_v_knots,
    ]
    .into_iter()
    .for_each(p::int);
    let (poles, weights) = (a.surf_poles()?, a.surf_weights()?);
    for i in poles.lower_row()..=poles.upper_row() {
        for j in poles.lower_col()..=poles.upper_col() {
            p::pnt(*poles.value(i, j).unwrap());
            p::real(*weights.value(i, j).unwrap());
        }
    }
    a.surf_u_knots()?.iter().for_each(|&k| p::real(k));
    a.surf_v_knots()?.iter().for_each(|&k| p::real(k));
    a.surf_u_mults()?.iter().for_each(|&m| p::int(m));
    a.surf_v_mults()?.iter().for_each(|&m| p::int(m));
    p::real(a.max_error_on_surf()?);
    p::real(a.average_error_on_surf()?);
    p::int(a.nb_curves_2d()?);
    Ok(())
}

fn curves_results(a: &SweepApproximation) -> Result<(), Failure> {
    attempt(|| {
        let (d, n, k) = a.curves_2d_shape()?;
        [d, n, k].into_iter().for_each(p::int);
        Ok(())
    });
    for j in 1..=a.nb_curves_2d()? {
        attempt(|| {
            a.curve_2d_poles(j)?.iter().for_each(|&q| p::pnt2d(q));
            Ok(())
        });
        attempt(|| a.max_2d_error(j).map(p::real));
        attempt(|| a.average_2d_error(j).map(p::real));
        attempt(|| a.tol_curve_on_surf(j).map(p::real));
    }
    attempt(|| a.curves_2d_degree().map(p::int));
    attempt(|| {
        a.curves_2d_knots()?.iter().for_each(|&k| p::real(k));
        Ok(())
    });
    attempt(|| {
        a.curves_2d_mults()?.iter().for_each(|&m| p::int(m));
        Ok(())
    });
    Ok(())
}

fn results(a: &SweepApproximation, f: &AxSweep) -> Result<(), Failure> {
    p::flag(a.is_done());
    if !a.is_done() {
        attempt(|| a.u_degree().map(p::int));
        return Ok(());
    }
    surface_results(a)?;
    curves_results(a)?;
    let record = *f.record.lock().unwrap();
    p::int(record.nbinterval);
    [record.first, record.last, record.tol3d, record.tol2d]
        .into_iter()
        .for_each(p::real);
    Ok(())
}

fn read_settings(r: &mut Reader<'_>) -> Settings {
    Settings {
        nbpol: r.integer(),
        nb2d: r.integer(),
        rational: r.flag(),
        d1mode: r.integer(),
        d2mode: r.integer(),
        resolution: r.flag(),
        nbc2: r.integer(),
        nbc3: r.integer(),
        pfirst: r.real(),
        plast: r.real(),
    }
}

pub fn sweep_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let settings = read_settings(&mut r);
    let (first, last) = (r.real(), r.real());
    let mut options = SweepOptions::new(r.real(), r.real(), r.real(), r.real());
    options.continuity = shape(r.integer());
    (options.degmax, options.segmax) = (r.integer(), r.integer());
    let count = r.integer();
    let f = Handle::new(AxSweep {
        parent: StandardTransient::default(),
        settings,
        record: Mutex::new(Record::default()),
    });
    attempt(|| {
        let mut a = SweepApproximation::new(&f.clone().cast::<dyn SweepFunction>());
        a.perform(first, last, options)?;
        let s = settings;
        let dim = (if s.rational { s.nbpol } else { 0 }) + 2 * s.nb2d + 3 * s.nbpol;
        results(&a, f.get().expect("probe sweep"))?;
        for _ in 0..count {
            let (u, order, ef, el) = (r.real(), r.integer(), r.real(), r.real());
            let mut res = vec![-7.0; dim as usize];
            attempt(|| {
                p::int(a.eval(u, order, ef, el, &mut res)?);
                res.iter().for_each(|&v| p::real(v));
                Ok(())
            });
        }
        Ok(())
    });
    end();
}
