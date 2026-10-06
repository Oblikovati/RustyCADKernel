// Rust translation of the members that GeomFill_UniformSection.cxx and
// GeomFill_EvolvedSection.cxx (ModelingAlgorithms/TKGeomAlgo/GeomFill) repeat, at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The single-curve section laws (uniform and evolved) share their constructor
//! conversion and the members that read their B-spline section; they are written
//! once here.
use super::sweep_failures::live;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::put;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::gcpnts::AbscissaPoint;
use rustycad_tkgeombase::geom_convert::curve_to_bspline_curve;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::gp::{Point3, Xyz};

/// The section curve of a single-curve law: the copy `mySection` and the B-spline
/// `myCurve` (the input itself when it is a B-spline, else its quasi-angular
/// conversion, a periodic result losing its first knot).
#[derive(Clone)]
pub(crate) struct SectionCurve {
    pub my_section: Handle<dyn GeomCurve>,
    pub my_curve: Handle<GeomBSplineCurve>,
}

impl SectionCurve {
    /// The constructor body of both laws.
    pub fn new(c: &Handle<dyn GeomCurve>) -> Result<Self, Failure> {
        let curve = live(c, "GeomFill_UniformSection", "C")?;
        let my_section: Handle<dyn GeomCurve> = curve.copy()?.cast();
        let mut my_curve: Handle<GeomBSplineCurve> = c.cast();
        if my_curve.is_null() {
            my_curve = curve_to_bspline_curve(c, ParameterisationType::QuasiAngular)?;
            let b = live(&my_curve, "GeomFill_UniformSection", "myCurve")?;
            if b.is_periodic() {
                let m = b.degree() / 2 + 1;
                b.remove_knot(1, m, precision::confusion())?;
            }
        }
        Ok(Self {
            my_section,
            my_curve,
        })
    }

    pub fn curve(&self) -> &GeomBSplineCurve {
        self.my_curve
            .get()
            .expect("the constructor builds the B-spline")
    }

    /// `Poles = myCurve->Poles(); Weights = myCurve->WeightsArray()` (assignments
    /// that take the curve's bounds).
    pub fn assign_poles(&self, poles: &mut Array1<'_, Point3>, weights: &mut Array1<'_, f64>) {
        poles.assign(&self.curve().poles());
        self.curve().weights_array(|w| {
            weights.assign(w);
        });
    }

    /// `SectionShape`: `(NbPoles, NbKnots, Degree)` of the B-spline.
    pub fn section_shape(&self) -> (i32, i32, i32) {
        let c = self.curve();
        (c.nb_poles(), c.nb_knots(), c.degree())
    }

    /// `Knots(TKnots)`: `TKnots = myCurve->Knots()`.
    pub fn knots(&self, knots: &mut Array1<'_, f64>) {
        knots.assign(&self.curve().knots());
    }

    /// `Mults(TMults)`: `TMults = myCurve->Multiplicities()`.
    pub fn mults(&self, mults: &mut Array1<'_, i32>) {
        mults.assign(&self.curve().multiplicities());
    }

    /// The sum of 21 points from the first parameter of the section, by twentieths
    /// of the range up to the last parameter of the B-spline.
    pub fn point_sum(&self) -> Result<Xyz, Failure> {
        let section = live(&self.my_section, "BarycentreOfSurf", "mySection")?;
        let mut u = section.first_parameter();
        let delta = (self.curve().last_parameter() - u) / 20.0;
        let mut bary = Xyz::new(0.0, 0.0, 0.0);
        for _ in 0..=20 {
            bary += self.curve().value(u)?.xyz();
            u += delta;
        }
        Ok(bary)
    }

    /// The length of the section curve (`GCPnts_AbscissaPoint::Length`).
    pub fn length(&self) -> Result<f64, Failure> {
        let ac = GeomAdaptorCurve::with_curve(&self.my_section)?;
        AbscissaPoint::length(&ac)
    }
}

/// `GetTolerance(BoundTol, SurfTol, AngleTol, Tol3d)`: `SurfTol` everywhere, and
/// `BoundTol` at both ends when it is smaller.
pub(crate) fn section_tolerance(
    bound_tol: f64,
    surf_tol: f64,
    tol3d: &mut Array1<'_, f64>,
) -> Result<(), Failure> {
    tol3d.init(surf_tol);
    if bound_tol < surf_tol {
        let (lower, upper) = (tol3d.lower(), tol3d.upper());
        put(tol3d, lower, bound_tol)?;
        put(tol3d, upper, bound_tol)?;
    }
    Ok(())
}
