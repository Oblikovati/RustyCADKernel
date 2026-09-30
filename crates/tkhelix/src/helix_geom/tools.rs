// Rust translation of ModelingAlgorithms/TKHelix/HelixGeom/HelixGeom_Tools.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `HelixGeom_Tools`: the B-spline approximation of a helix, or of any adapted 3D
//! curve, by AdvApprox with dichotomic cutting.
use super::errors::{array_failure, null_object};
use super::helix_curve::HelixGeomHelixCurve;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::adv_approx::{
    AdvApproxApproxAFunction, AdvApproxDichoCutting, AdvApproxEvaluatorFunction,
};
use rustycad_tkg3d::geom::GeomBSplineCurve;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point3, Vector3};

/// `HelixGeom_Tools`: static approximation helpers.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkhelix::helix_geom::HelixGeomTools;
/// let (mut bspline, mut max_error) = (Handle::default(), 0.0);
/// let code = HelixGeomTools::appr_helix(0.0, 6.0, 10.0, 5.0, 0.0, true, 1e-4, &mut bspline, &mut max_error)?;
/// assert_eq!(code, 0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct HelixGeomTools;

impl HelixGeomTools {
    /// `ApprHelix(aT1, aT2, aPitch, aRStart, aTaperAngle, aIsCW, aTol, theBSpl,
    /// theMaxError)`: loads a helix (Standard_ConstructionError for invalid
    /// parameters) and approximates it with C2 continuity, degree 8 and 150 segments
    /// at most; returns the code of [`appr_curve_3d`](Self::appr_curve_3d);
    /// `HelixGeomTools::appr_helix(0.0, 6.0, 10.0, 5.0, 0.0, true, 1e-4, &mut b, &mut e)?`.
    #[allow(clippy::too_many_arguments)] // The native member list.
    pub fn appr_helix(
        t1: f64,
        t2: f64,
        pitch: f64,
        r_start: f64,
        taper_angle: f64,
        is_cw: bool,
        tol: f64,
        the_bspl: &mut Handle<GeomBSplineCurve>,
        the_max_error: &mut f64,
    ) -> Result<i32, Failure> {
        let adaptor = HelixGeomHelixCurve::new();
        adaptor.load(t1, t2, pitch, r_start, taper_angle, is_cw)?;
        let (continuity, max_degree, max_seg): (Shape, i32, i32) = (Shape::C2, 8, 150);
        Self::appr_curve_3d(
            &adaptor.to_adaptor(),
            tol,
            continuity,
            max_seg,
            max_degree,
            the_bspl,
            the_max_error,
        )
    }

    /// `ApprCurve3D(theHC, theTol, theCont, theMaxSeg, theMaxDeg, theBSpl,
    /// theMaxError)`: 0 with the B-spline and its maximum error; 1 when AdvApprox is
    /// not done (the outputs untouched, even when it has a result out of tolerance); 2
    /// without a result (the error set to 0). An adaptor or AdvApprox exception
    /// propagates; `HelixGeomTools::appr_curve_3d(&adaptor, 1e-6, Shape::C1, 50, 6, &mut b, &mut e)?`.
    #[allow(clippy::too_many_arguments)] // The native member list.
    pub fn appr_curve_3d(
        the_hc: &Handle<dyn Adaptor3dCurve>,
        the_tol: f64,
        the_cont: Shape,
        the_max_seg: i32,
        the_max_deg: i32,
        the_bspl: &mut Handle<GeomBSplineCurve>,
        the_max_error: &mut f64,
    ) -> Result<i32, Failure> {
        let adaptor: &dyn Adaptor3dCurve = the_hc.get().ok_or_else(|| null_object("theHC"))?;
        let mut three_d_tol: Array1<'static, f64> =
            Array1::with_bounds(1, 1).map_err(array_failure("NCollection_HArray1"))?;
        three_d_tol.init(the_tol);
        let (first, last): (f64, f64) = (adaptor.first_parameter()?, adaptor.last_parameter()?);
        let mut ev = HelixGeomToolsEval::new(the_hc.clone());
        let approx = AdvApproxApproxAFunction::with_cutting(
            0,
            0,
            1,
            None,
            None,
            Some(&three_d_tol),
            first,
            last,
            the_cont,
            the_max_deg,
            the_max_seg,
            &mut ev,
            &AdvApproxDichoCutting::new(),
        );
        // An evaluator exception unwinds through AdvApprox natively; it comes first.
        ev.take_failure()?;
        Self::extract_result(&approx?, the_bspl, the_max_error)
    }

    // The result part of ApprCurve3D after the approximation.
    fn extract_result(
        approx: &AdvApproxApproxAFunction,
        the_bspl: &mut Handle<GeomBSplineCurve>,
        the_max_error: &mut f64,
    ) -> Result<i32, Failure> {
        if !approx.is_done() {
            return Ok(1);
        }
        *the_max_error = 0.;
        if !approx.has_result() {
            return Ok(2);
        }
        let mut poles: Array1<'static, Point3> = Array1::with_bounds(1, approx.nb_poles()?)
            .map_err(array_failure("NCollection_Array1"))?;
        approx.poles_of(1, &mut poles)?;
        let knots = approx.knots().ok_or_else(|| null_object("Knots"))?;
        let mults = approx
            .multiplicities()
            .ok_or_else(|| null_object("Multiplicities"))?;
        let curve = GeomBSplineCurve::new(&poles, knots, mults, approx.degree(), false)?;
        *the_bspl = Handle::new(curve);
        *the_max_error = approx.max_error(3, 1)?;
        Ok(0)
    }
}

/// `HelixGeom_Tools_Eval`, the local AdvApprox evaluator of an adapted curve: its value,
/// first or second derivative (error code 3 with zeros for a higher order, 1 for a
/// dimension other than 3, and the evaluation goes on). An adaptor exception unwinds
/// natively; here it stops AdvApprox and is returned after it.
struct HelixGeomToolsEval {
    fonct: Handle<dyn Adaptor3dCurve>,
    failure: Option<Failure>,
}

impl HelixGeomToolsEval {
    fn new(fonct: Handle<dyn Adaptor3dCurve>) -> Self {
        Self {
            fonct,
            failure: None,
        }
    }

    // The first adaptor exception of the approximation, if any.
    fn take_failure(&mut self) -> Result<(), Failure> {
        match self.failure.take() {
            Some(failure) => Err(failure),
            None => Ok(()),
        }
    }

    // The derivative of `order` as three values; zeros beyond order 2.
    fn derivative(&self, par: f64, order: i32) -> Result<[f64; 3], Failure> {
        let fonct: &dyn Adaptor3dCurve = self.fonct.get().ok_or_else(|| null_object("fonct"))?;
        let vector_values = |v: Vector3| [v.x(), v.y(), v.z()];
        match order {
            0 => fonct.value(par).map(|p: Point3| [p.x(), p.y(), p.z()]),
            1 => fonct.d1(par).map(|(_, v1)| vector_values(v1)),
            2 => fonct.d2(par).map(|(_, _, v2)| vector_values(v2)),
            _ => Ok([0., 0., 0.]),
        }
    }
}

impl AdvApproxEvaluatorFunction for HelixGeomToolsEval {
    fn evaluate(
        &mut self,
        dimension: i32,
        _start_end: [f64; 2],
        parameter: f64,
        derivative_request: i32,
        result: &mut [f64],
    ) -> i32 {
        let dimension_code: i32 = if dimension != 3 { 1 } else { 0 };
        let values: [f64; 3] = match self.derivative(parameter, derivative_request) {
            Ok(values) => values,
            Err(failure) => {
                self.failure.get_or_insert(failure);
                return 1;
            }
        };
        result[..3].copy_from_slice(&values);
        // The default branch overwrites the dimension code with 3.
        if !(0..=2).contains(&derivative_request) {
            return 3;
        }
        dimension_code
    }
}
