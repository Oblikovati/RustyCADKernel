// Rust translation of Geom2dAPI_Interpolate.hxx/.cxx and GeomAPI_Interpolate.hxx/.cxx
// (the constructors, Load and the accessors) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dAPI_Interpolate` and `GeomAPI_Interpolate`: a B-spline curve through
//! points, with optional tangents; one generic body over the space of the points
//! (docs/geomalgo-design.md section 15).
use super::interpolate_build::{
    array, at, build_parameters, check_parameters, check_points, check_tangents, put,
    scale_tangents,
};
use super::interpolate_space::{Interpolate2dSpace, InterpolateSpace};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};

/// `Geom2dAPI_Interpolate`: the generic interpolation over `gp_Pnt2d`, `gp_Vec2d` and
/// `Geom2d_BSplineCurve`.
pub type Interpolate = InterpolateGen<Interpolate2dSpace>;

/// `Geom2dAPI_Interpolate` (with [`Interpolate2dSpace`]) and `GeomAPI_Interpolate`
/// (with `geom_api::Interpolate3dSpace`): the points (shared with the caller, as the
/// native handle), their parameters (chord lengths unless given), the tangents and
/// their flags. `Load(Tangents, TangentFlags)` shares the caller's flags, which
/// Perform completes (the end tangents it builds are flagged there, as natively).
///
/// ```
/// use rustycad_tkgeomalgo::geom2d_api::Interpolate;
/// use rustycad_tkernel::ncollection::{Array1, HArray1};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::Point2;
/// let mut points = Array1::with_bounds(1, 3).unwrap();
/// points[1] = Point2::new(0.0, 0.0);
/// points[2] = Point2::new(1.0, 1.0);
/// points[3] = Point2::new(2.0, 0.0);
/// let mut interp = Interpolate::new(&Handle::new(HArray1::from_owned(points)), false, 1e-7).unwrap();
/// interp.perform().unwrap();
/// assert!(interp.is_done());
/// ```
pub struct InterpolateGen<S: InterpolateSpace> {
    my_tolerance: f64,
    my_points: Handle<HArray1<S::Point>>,
    my_is_done: bool,
    my_curve: Handle<S::Curve>,
    my_tangents: Handle<HArray1<S::Vector>>,
    my_tangent_flags: Handle<HArray1<bool>>,
    my_parameters: Handle<HArray1<f64>>,
    my_periodic: bool,
    my_tangent_request: bool,
}

/// The Standard_ConstructionError of the constructors and Load.
fn construction<S: InterpolateSpace>(reason: &str) -> Failure {
    Failure::with_type(
        "Standard_ConstructionError",
        &format!("{} - {reason}", S::NAME),
        "",
    )
}

// The contents of a shared array (Standard_NullObject for a null handle).
fn shared<T: Send + Sync + 'static>(array: &Handle<HArray1<T>>) -> Result<&HArray1<T>, Failure> {
    array.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "Geom2dAPI_Interpolate/GeomAPI_Interpolate - null array; expected an array",
            "",
        )
    })
}

impl<S: InterpolateSpace> InterpolateGen<S> {
    // The fields of both constructors before their checks.
    fn with_fields(
        points: &Handle<HArray1<S::Point>>,
        periodic: bool,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let (lower, upper) = {
            let values = shared(points)?.array1();
            (values.lower(), values.upper())
        };
        Ok(Self {
            my_tolerance: tolerance,
            my_points: points.clone(),
            my_is_done: false,
            my_curve: Handle::default(),
            my_tangents: Handle::new(HArray1::from_owned(array(lower, upper)?)),
            my_tangent_flags: Handle::new(HArray1::from_owned(array(lower, upper)?)),
            my_parameters: Handle::default(),
            my_periodic: periodic,
            my_tangent_request: false,
        })
    }

    /// `Geom[2d]API_Interpolate(Points, PeriodicFlag, Tolerance)`: the parameters are
    /// the chord lengths; Standard_ConstructionError when two consecutive points are
    /// closer than `tolerance`. `Interpolate::new(&points, false, 1e-7)?`.
    pub fn new(
        points: &Handle<HArray1<S::Point>>,
        periodic_flag: bool,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let result: bool = check_points::<S>(&shared(points)?.array1(), tolerance)?;
        let mut interpolate: Self = Self::with_fields(points, periodic_flag, tolerance)?;
        if !result {
            return Err(construction::<S>(
                "two consecutive points are closer than the tolerance",
            ));
        }
        let parameters = build_parameters::<S>(periodic_flag, &shared(points)?.array1())?;
        interpolate.my_parameters = Handle::new(HArray1::from_owned(parameters));
        interpolate.clear_flags()?;
        Ok(interpolate)
    }

    /// `Geom[2d]API_Interpolate(Points, Parameters, PeriodicFlag, Tolerance)`: one more
    /// parameter than points for a periodic curve; Standard_ConstructionError on a
    /// wrong count, close points or parameters not increasing by RealSmall().
    /// `Interpolate::with_parameters(&points, &parameters, false, 1e-7)?`.
    pub fn with_parameters(
        points: &Handle<HArray1<S::Point>>,
        parameters: &Handle<HArray1<f64>>,
        periodic_flag: bool,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let result: bool = check_points::<S>(&shared(points)?.array1(), tolerance)?;
        let nb_points: usize = shared(points)?.array1().len();
        if periodic_flag && nb_points + 1 != shared(parameters)?.array1().len() {
            return Err(construction::<S>(
                "a periodic curve needs one more parameter than points",
            ));
        }
        let mut interpolate: Self = Self::with_fields(points, periodic_flag, tolerance)?;
        interpolate.my_parameters = parameters.clone();
        if !result {
            return Err(construction::<S>(
                "two consecutive points are closer than the tolerance",
            ));
        }
        if !check_parameters(&shared(parameters)?.array1())? {
            return Err(construction::<S>("the parameters do not increase"));
        }
        interpolate.clear_flags()?;
        Ok(interpolate)
    }

    fn clear_flags(&mut self) -> Result<(), Failure> {
        let mut flags = shared(&self.my_tangent_flags)?.change_array1();
        let (lower, upper) = (flags.lower(), flags.upper());
        for ii in lower..=upper {
            put(&mut flags, ii, false)?;
        }
        Ok(())
    }

    /// `Load(InitialTangent, FinalTangent, Scale)`: tangents at the first and last
    /// points (indices 1 and NbPoints), scaled to the Lagrange derivatives when
    /// `scale`; Standard_ConstructionError for a tangent shorter than the tolerance.
    /// `interp.load_ends(t0, t1, true)?`.
    pub fn load_ends(
        &mut self,
        initial_tangent: S::Vector,
        final_tangent: S::Vector,
        scale: bool,
    ) -> Result<(), Failure> {
        self.my_tangent_request = true;
        let length: i32 = shared(&self.my_points)?.array1().len() as i32;
        {
            let mut flags = shared(&self.my_tangent_flags)?.change_array1();
            put(&mut flags, 1, true)?;
            put(&mut flags, length, true)?;
            let mut tangents = shared(&self.my_tangents)?.change_array1();
            put(&mut tangents, 1, initial_tangent)?;
            put(&mut tangents, length, final_tangent)?;
        }
        let result: bool = check_tangents::<S>(
            &shared(&self.my_tangents)?.array1(),
            &shared(&self.my_tangent_flags)?.array1(),
            self.my_tolerance,
        )?;
        if !result {
            return Err(construction::<S>("a tangent is shorter than the tolerance"));
        }
        if scale {
            self.scale_loaded_tangents(&self.my_tangent_flags.clone())?;
        }
        Ok(())
    }

    /// `Load(Tangents, TangentFlags, Scale)`: the flagged tangents (the flags are
    /// shared with the caller); Standard_ConstructionError on a count different from
    /// the points' or a flagged tangent shorter than the tolerance.
    /// `interp.load(&tangents, &flags, true)?`.
    pub fn load(
        &mut self,
        tangents: &Array1<'_, S::Vector>,
        tangent_flags: &Handle<HArray1<bool>>,
        scale: bool,
    ) -> Result<(), Failure> {
        self.my_tangent_request = true;
        self.my_tangent_flags = tangent_flags.clone();
        let nb_points: usize = shared(&self.my_points)?.array1().len();
        if tangents.len() != nb_points || shared(tangent_flags)?.array1().len() != nb_points {
            return Err(construction::<S>(
                "the tangents and flags must match the points",
            ));
        }
        if !check_tangents::<S>(
            tangents,
            &shared(tangent_flags)?.array1(),
            self.my_tolerance,
        )? {
            return Err(construction::<S>("a tangent is shorter than the tolerance"));
        }
        let mut copy: Array1<'static, S::Vector> = array(tangents.lower(), tangents.upper())?;
        for ii in tangents.lower()..=tangents.upper() {
            put(&mut copy, ii, at(tangents, ii)?)?;
        }
        self.my_tangents = Handle::new(HArray1::from_owned(copy));
        if scale {
            self.scale_loaded_tangents(tangent_flags)?;
        }
        Ok(())
    }

    fn scale_loaded_tangents(&self, flags: &Handle<HArray1<bool>>) -> Result<(), Failure> {
        scale_tangents::<S>(
            &shared(&self.my_points)?.array1(),
            &mut shared(&self.my_tangents)?.change_array1(),
            &shared(flags)?.array1(),
            &shared(&self.my_parameters)?.array1(),
        )
    }

    /// `Perform()`: the periodic or the non-periodic interpolation; the exceptions of
    /// BSplCLib and the B-spline curve are returned as `Err`. `interp.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        if self.my_periodic {
            return self.perform_periodic();
        }
        self.perform_non_periodic()
    }

    /// `Curve()`: StdFail_NotDone when the interpolation failed. `interp.curve()?`.
    pub fn curve(&self) -> Result<Handle<S::Curve>, Failure> {
        if !self.my_is_done {
            return Err(super::not_done(&format!("{}::Curve", S::NAME)));
        }
        Ok(self.my_curve.clone())
    }
    /// `operator occ::handle<Geom*_BSplineCurve>()`: the curve, null when not done.
    /// `interp.as_curve()`.
    pub fn as_curve(&self) -> Handle<S::Curve> {
        self.my_curve.clone()
    }
    /// `IsDone()`. `interp.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_is_done
    }

    pub(super) fn arrays(&self) -> InterpolateArrays<S> {
        InterpolateArrays {
            points: self.my_points.clone(),
            tangents: self.my_tangents.clone(),
            flags: self.my_tangent_flags.clone(),
            parameters: self.my_parameters.clone(),
        }
    }
    pub(super) fn tangent_request(&self) -> bool {
        self.my_tangent_request
    }
    pub(super) fn periodic(&self) -> bool {
        self.my_periodic
    }
    pub(super) fn set_result(&mut self, curve: S::Curve) {
        self.my_curve = Handle::new(curve);
        self.my_is_done = true;
    }
}

/// The shared arrays of an interpolation.
pub(super) struct InterpolateArrays<S: InterpolateSpace> {
    points: Handle<HArray1<S::Point>>,
    tangents: Handle<HArray1<S::Vector>>,
    flags: Handle<HArray1<bool>>,
    parameters: Handle<HArray1<f64>>,
}

impl<S: InterpolateSpace> InterpolateArrays<S> {
    pub(super) fn points(&self) -> Result<&HArray1<S::Point>, Failure> {
        shared(&self.points)
    }
    pub(super) fn tangents(&self) -> Result<&HArray1<S::Vector>, Failure> {
        shared(&self.tangents)
    }
    pub(super) fn flags(&self) -> Result<&HArray1<bool>, Failure> {
        shared(&self.flags)
    }
    pub(super) fn parameters(&self) -> Result<&HArray1<f64>, Failure> {
        shared(&self.parameters)
    }
}
