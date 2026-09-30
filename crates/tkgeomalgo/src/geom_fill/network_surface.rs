// Rust translation of GeomFill_NetworkSurface.hxx/.cxx (the class, isReadyToBuild,
// the checks and prepareCurveFamily) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_NetworkSurface`: the Gordon surface of a prepared B-spline curve network.
use super::GeomFillProfiler;
use super::network_surface_seam::{apply_periodicity, make_network_surface, reparametrize_surface};
use super::network_surface_skin::{owned1, owned2};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBSplineSurface, GeomCurve};
use rustycad_tkmath::gp::Point3;

/// `GeomFill_NetworkSurface::ResultStatus`: the result of the last Perform.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum NetworkSurfaceStatus {
    /// Perform() has not been called since initialization.
    #[default]
    NotStarted,
    /// Surface has been constructed.
    Done,
    /// Prepared network does not satisfy builder requirements.
    InvalidInput,
    /// Curve families could not be converted to a compatible basis.
    CurveCompatibilityFailed,
    /// Profile or guide skin interpolation has failed.
    SkinningFailed,
    /// Intersection-grid reference surface could not be built.
    ReferenceSurfaceFailed,
    /// Intermediate surfaces could not be aligned to one knot basis.
    KnotAlignmentFailed,
    /// Exact rational product degree exceeds OCCT's B-spline limit.
    RationalDegreeOverflow,
    /// Exact rational numerator/denominator construction has failed.
    RationalConstructionFailed,
    /// Internal B-spline construction has failed.
    ConstructionFailed,
    /// Closed seam could not be converted to periodic form.
    PeriodicityFailed,
}

/// The prepared network of `Init`: the curve families, the parameters locating them,
/// the contact grid and its weights (the native arrays, with their bounds).
#[derive(Clone, Default)]
pub(crate) struct PreparedNetwork {
    pub(crate) profiles: Vec<Handle<GeomBSplineCurve>>,
    pub(crate) guides: Vec<Handle<GeomBSplineCurve>>,
    pub(crate) profile_parameters: Array1<'static, f64>,
    pub(crate) guide_parameters: Array1<'static, f64>,
    pub(crate) intersection_points: Array2<'static, Point3>,
    pub(crate) intersection_weights: Array2<'static, f64>,
}

/// `GeomFill_NetworkSurface`: the profile skin corrected by the deviation of the guide
/// skin from the reference surface of the contact grid, on one aligned knot basis
/// (exactly, through common denominators, for rational families). The input must be a
/// consistently ordered and reparametrized non-periodic network of at least two
/// profiles and two guides; GeomFill_Gordon prepares it.
///
/// ```
/// use rustycad_tkgeomalgo::geom_fill::{GeomFillNetworkSurface, NetworkSurfaceStatus};
/// let mut network = GeomFillNetworkSurface::new();
/// network.perform();
/// assert_eq!(network.status(), NetworkSurfaceStatus::InvalidInput);
/// ```
#[derive(Default)]
pub struct GeomFillNetworkSurface {
    my_network: PreparedNetwork,
    my_surface: Handle<GeomBSplineSurface>,
    my_tolerance: f64,
    my_is_u_closed: bool,
    my_is_v_closed: bool,
    my_status: NetworkSurfaceStatus,
}

/// The closed seams of a network: `(U closed, V closed)`.
pub type ClosedSeams = (bool, bool);

impl GeomFillNetworkSurface {
    /// `GeomFill_NetworkSurface()`: not started. `GeomFillNetworkSurface::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Init(Profiles, Guides, ProfileParameters, GuideParameters, IntersectionPoints,
    /// IntersectionWeights, Tolerance, IsUClosed, IsVClosed)`: copies of the network
    /// (the points and weights in (guide, profile) order), not started.
    /// `network.init((&profiles, &guides), (&v, &u), (&points, &weights), 1e-7, (false, false))`.
    pub fn init(
        &mut self,
        (the_profiles, the_guides): (
            &Array1<'_, Handle<GeomBSplineCurve>>,
            &Array1<'_, Handle<GeomBSplineCurve>>,
        ),
        (the_profile_parameters, the_guide_parameters): (&Array1<'_, f64>, &Array1<'_, f64>),
        (the_points, the_weights): (&Array2<'_, Point3>, &Array2<'_, f64>),
        the_tolerance: f64,
        (the_is_u_closed, the_is_v_closed): ClosedSeams,
    ) {
        self.my_network = PreparedNetwork {
            profiles: the_profiles.iter().cloned().collect(),
            guides: the_guides.iter().cloned().collect(),
            profile_parameters: owned1(the_profile_parameters),
            guide_parameters: owned1(the_guide_parameters),
            intersection_points: owned2(the_points),
            intersection_weights: owned2(the_weights),
        };
        self.my_tolerance = the_tolerance;
        (self.my_is_u_closed, self.my_is_v_closed) = (the_is_u_closed, the_is_v_closed);
        self.my_status = NetworkSurfaceStatus::NotStarted;
        self.my_surface = Handle::default();
    }
    /// `Perform()`: InvalidInput for a network that is not ready; the curve families made
    /// compatible (in place: a second Perform starts from them), the surface built, the
    /// closed seams made periodic and the surface reparametrized onto `[0, 1]^2`. Any
    /// exception gives ConstructionFailed, as the native try/catch. `network.perform()`.
    pub fn perform(&mut self) {
        self.my_status = NetworkSurfaceStatus::NotStarted;
        self.my_surface = Handle::default();
        if !is_ready_to_build(&self.my_network) {
            self.my_status = NetworkSurfaceStatus::InvalidInput;
            return;
        }
        match self.build() {
            Ok(status) => self.my_status = status,
            Err(_) => {
                self.my_surface = Handle::default();
                self.my_status = NetworkSurfaceStatus::ConstructionFailed;
            }
        }
    }
    // The body of the native try block: the status it leaves.
    fn build(&mut self) -> Result<NetworkSurfaceStatus, Failure> {
        if !prepare_curve_family(&mut self.my_network.profiles)?
            || !prepare_curve_family(&mut self.my_network.guides)?
        {
            return Ok(NetworkSurfaceStatus::CurveCompatibilityFailed);
        }
        let (surface, status) = make_network_surface(&self.my_network, self.my_is_u_closed)?;
        let Some(surface) = surface else {
            return Ok(status);
        };
        self.my_surface = surface;
        let closed = (self.my_is_u_closed, self.my_is_v_closed);
        if !apply_periodicity(&self.my_surface, closed, self.my_tolerance)? {
            self.my_surface = Handle::default();
            return Ok(NetworkSurfaceStatus::PeriodicityFailed);
        }
        reparametrize_surface(&self.my_surface)?;
        Ok(NetworkSurfaceStatus::Done)
    }
    /// `IsDone()`. `network.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_status == NetworkSurfaceStatus::Done
    }
    /// `Status()`. `network.status()`.
    pub fn status(&self) -> NetworkSurfaceStatus {
        self.my_status
    }
    /// `Surface()`: StdFail_NotDone unless done. `network.surface()?`.
    pub fn surface(&self) -> Result<Handle<GeomBSplineSurface>, Failure> {
        if !self.is_done() {
            return Err(Failure::with_type(
                "StdFail_NotDone",
                &format!(
                    "GeomFill_NetworkSurface::Surface - status {:?}; expected Done",
                    self.my_status
                ),
                "",
            ));
        }
        Ok(self.my_surface.clone())
    }
}

/// `checkParameters`: at least two parameters, each above the previous one by more
/// than Precision::PConfusion().
fn check_parameters(the_parameters: &Array1<'_, f64>) -> bool {
    let values: Vec<f64> = the_parameters.iter().copied().collect();
    // `if (At(i) <= At(i - 1) + PConfusion) return false`: a NaN passes, as natively.
    let too_close = |w: &[f64]| w[1] <= w[0] + precision::p_confusion();
    values.len() >= 2 && !values.windows(2).any(too_close)
}

/// `checkWeights`: a non-empty grid of weights above Precision::Confusion().
fn check_weights(the_weights: &Array2<'_, f64>) -> bool {
    // `if (At(i) <= Confusion) return false`: a NaN weight passes, as natively.
    !the_weights.is_empty() && !the_weights.iter().any(|w| *w <= precision::confusion())
}

/// `hasRationalCurve`: one non-null curve is rational.
pub(crate) fn has_rational_curve(the_curves: &[Handle<GeomBSplineCurve>]) -> bool {
    the_curves
        .iter()
        .any(|c| c.get().is_some_and(|c| c.is_rational()))
}

/// `hasPeriodicCurve`: one curve is null or periodic.
fn has_periodic_curve(the_curves: &[Handle<GeomBSplineCurve>]) -> bool {
    the_curves
        .iter()
        .any(|c| c.get().is_none_or(|c| c.is_periodic()))
}

/// `isReadyToBuild`: two profiles and guides at least, none periodic or null, one
/// parameter per curve, a (guides x profiles) contact grid, increasing parameters and
/// positive weights.
fn is_ready_to_build(n: &PreparedNetwork) -> bool {
    let (nb_p, nb_g) = (n.profiles.len(), n.guides.len());
    nb_p >= 2
        && nb_g >= 2
        && !has_periodic_curve(&n.profiles)
        && !has_periodic_curve(&n.guides)
        && nb_p == n.profile_parameters.len()
        && nb_g == n.guide_parameters.len()
        && n.intersection_points.nb_rows() == n.guide_parameters.len()
        && n.intersection_points.nb_columns() == n.profile_parameters.len()
        && n.intersection_weights.nb_rows() == n.guide_parameters.len()
        && n.intersection_weights.nb_columns() == n.profile_parameters.len()
        && check_parameters(&n.profile_parameters)
        && check_parameters(&n.guide_parameters)
        && check_weights(&n.intersection_weights)
}

/// `isCompatibleCurveFamily`: every curve with the rationality, degree, pole count,
/// knot count, multiplicities and knots (within Precision::PConfusion()) of the first.
fn is_compatible_curve_family(the_curves: &[Handle<GeomBSplineCurve>]) -> Result<bool, Failure> {
    let Some(a_reference) = the_curves.first().and_then(|c| c.get()) else {
        return Ok(false);
    };
    for a_handle in &the_curves[1..] {
        let Some(a_curve) = a_handle.get() else {
            return Ok(false);
        };
        if a_curve.is_rational() != a_reference.is_rational()
            || a_curve.degree() != a_reference.degree()
            || a_curve.nb_poles() != a_reference.nb_poles()
            || a_curve.nb_knots() != a_reference.nb_knots()
        {
            return Ok(false);
        }
        for a_knot in 1..=a_curve.nb_knots() {
            if a_curve.multiplicity(a_knot)? != a_reference.multiplicity(a_knot)?
                || (a_curve.knot(a_knot)? - a_reference.knot(a_knot)?).abs()
                    > precision::p_confusion()
            {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

/// `prepareCurveFamily`: a rational family must already be compatible; a polynomial
/// family is replaced by the compatible curves of GeomFill_Profiler (false for a null
/// curve or a result that is not a B-spline).
pub(crate) fn prepare_curve_family(
    the_curves: &mut [Handle<GeomBSplineCurve>],
) -> Result<bool, Failure> {
    if has_rational_curve(the_curves) {
        return is_compatible_curve_family(the_curves);
    }
    let mut a_profiler = GeomFillProfiler::new();
    for a_curve in the_curves.iter() {
        if a_curve.is_null() {
            return Ok(false);
        }
        a_profiler.add_curve(&a_curve.cast())?;
    }
    a_profiler.perform(precision::p_confusion())?;
    for (index, a_curve) in the_curves.iter_mut().enumerate() {
        *a_curve = a_profiler.curve(index as i32 + 1)?.cast();
        if a_curve.is_null() {
            return Ok(false);
        }
    }
    Ok(true)
}
