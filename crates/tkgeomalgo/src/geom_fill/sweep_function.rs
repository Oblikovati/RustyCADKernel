// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_SweepFunction.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_SweepFunction`: a section law placed by a location law, as the
//! `Approx_SweepFunction` that `Approx_SweepApproximation` approximates
//! (docs/geomalgo-design.md, section 14).
use super::location_law::{GeomFillLocationLaw, LocationD0, LocationD2};
use super::section_law::GeomFillSectionLaw;
use super::sweep_failures::{live, sweep_failure};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{Array1, LinearVector};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkgeombase::approx::SweepFunction;
use rustycad_tkgeombase::geom_lib;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Matrix3, Point3, Vector3};

/// `GeomFill_SweepFunction`: the poles of the section at `myfOnS + (Param - myf) *
/// myRatio`, moved by the placement of the location law at `Param`.
///
/// The placement of the last evaluation (`M`, `V` and their derivatives) is kept,
/// as the native members, and handed back to the next evaluation.
/// `let f = GeomFillSweepFunction::new(&section, &location, 0.0, 0.0, 1.0);`.
pub struct GeomFillSweepFunction {
    parent: StandardTransient,
    my_loc: Handle<dyn GeomFillLocationLaw>,
    my_sec: Handle<dyn GeomFillSectionLaw>,
    pub(super) myf: f64,
    myf_on_s: f64,
    pub(super) my_ratio: f64,
    pub(super) placement: SharedState<LocationD2>,
}

impl_transient!(
    GeomFillSweepFunction,
    "GeomFill_SweepFunction",
    dyn SweepFunction,
    parent,
    interfaces: [dyn SweepFunction]
);

/// `aux *= M; aux += V` on the poles `1..=L`.
pub(super) fn place_poles(
    poles: &mut Array1<'_, Point3>,
    l: i32,
    p: &LocationD0,
) -> Result<(), Failure> {
    for ii in 1..=l {
        place_pole(poles, ii, &p.m, p.v)?;
    }
    Ok(())
}

/// `P *= M; P += V` on pole `ii`.
pub(super) fn place_pole(
    poles: &mut Array1<'_, Point3>,
    ii: i32,
    m: &Matrix3,
    v: Vector3,
) -> Result<(), Failure> {
    let pole = poles
        .change_value(ii)
        .map_err(|e| index_error("Poles", ii, e))?;
    let mut aux = pole.xyz();
    aux *= *m;
    aux += *v.xyz();
    *pole = Point3::from(aux);
    Ok(())
}

/// The Standard_OutOfRange of an index outside an output array.
pub(super) fn index_error(array: &str, index: i32, error: impl std::fmt::Display) -> Failure {
    sweep_failure(
        "Standard_OutOfRange",
        &format!(
            "GeomFill_SweepFunction: {array}({index}): {error}; expected an index in the array bounds"
        ),
    )
}

impl GeomFillSweepFunction {
    /// `GeomFill_SweepFunction(Section, Location, FirstParameter,
    /// FirstParameterOnS, RatioParameterOnS)`;
    /// `GeomFillSweepFunction::new(&section, &location, 0.0, 0.0, 1.0)`.
    pub fn new(
        section: &Handle<dyn GeomFillSectionLaw>,
        location: &Handle<dyn GeomFillLocationLaw>,
        first_parameter: f64,
        first_parameter_on_s: f64,
        ratio_parameter_on_s: f64,
    ) -> Self {
        Self {
            parent: StandardTransient::default(),
            my_loc: location.clone(),
            my_sec: section.clone(),
            myf: first_parameter,
            myf_on_s: first_parameter_on_s,
            my_ratio: ratio_parameter_on_s,
            placement: SharedState::new(LocationD2::default()),
        }
    }

    pub(super) fn loc(&self) -> Result<&dyn GeomFillLocationLaw, Failure> {
        live(&self.my_loc, "GeomFill_SweepFunction", "myLoc")
    }

    pub(super) fn sec(&self) -> Result<&dyn GeomFillSectionLaw, Failure> {
        live(&self.my_sec, "GeomFill_SweepFunction", "mySec")
    }

    /// The section parameter of the path parameter `param`.
    pub(super) fn section_parameter(&self, param: f64) -> f64 {
        self.myf_on_s + (param - self.myf) * self.my_ratio
    }

    /// The path parameter of the section parameter `t`.
    pub(super) fn path_parameter(&self, t: f64) -> f64 {
        (t - self.myf_on_s) / self.my_ratio + self.myf
    }

    /// The section intervals mapped to the path and the path intervals, fused.
    pub(super) fn fused_intervals(
        &self,
        nb_sec: i32,
        nb_loc: i32,
        shape: Shape,
    ) -> Result<LinearVector<f64>, Failure> {
        let mut int_s: Array1<'static, f64> = one_based(nb_sec + 1)?;
        let mut int_l: Array1<'static, f64> = one_based(nb_loc + 1)?;
        self.sec()?.intervals(&mut int_s, shape)?;
        for ii in 1..=nb_sec + 1 {
            int_s[ii] = self.path_parameter(int_s[ii]);
        }
        self.loc()?.intervals(&mut int_l, shape)?;
        let mut inter = LinearVector::with_capacity(0);
        let tol = precision::p_confusion() * 0.99;
        geom_lib::fuse_intervals(&int_s, &int_l, &mut inter, tol, false);
        Ok(inter)
    }
}
