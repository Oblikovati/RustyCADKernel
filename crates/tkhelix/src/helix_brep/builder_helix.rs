// Rust translation of ModelingAlgorithms/TKHelix/HelixBRep/HelixBRep_BuilderHelix.hxx/.cxx
// (constructor, parameters and accessors) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `HelixBRep_BuilderHelix`: the parameters of a composite helix wire (the build is in
//! `builder_helix_perform.rs`).
use crate::helix_geom::errors::{array_failure, construction_error};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Axis3;

/// `HelixBRep_BuilderHelix`: a wire of helix parts along an axis, each part with its
/// height, pitch (or number of turns) and start and end diameters (docs/helix-design.md
/// sections 3 and 4).
///
/// ```
/// use rustycad_tkernel::ncollection::Array1;
/// use rustycad_tkhelix::helix_brep::HelixBRepBuilderHelix;
/// use rustycad_tkmath::gp::Axis3;
/// let (mut heights, mut pitches, mut is_pitches) =
///     (Array1::with_bounds(1, 1)?, Array1::with_bounds(1, 1)?, Array1::with_bounds(1, 1)?);
/// (heights[1], pitches[1], is_pitches[1]) = (100.0, 20.0, true);
/// let mut builder = HelixBRepBuilderHelix::new();
/// builder.set_parameters_diam(&Axis3::default(), 100.0, &heights, &pitches, &is_pitches)?;
/// builder.perform()?;
/// assert_eq!(builder.error_status(), 0);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone)]
pub struct HelixBRepBuilderHelix {
    pub(super) axis3: Axis3,
    pub(super) diams: Option<Array1<'static, f64>>,
    pub(super) heights: Option<Array1<'static, f64>>,
    pub(super) pitches: Option<Array1<'static, f64>>,
    pub(super) is_pitches: Option<Array1<'static, bool>>,
    pub(super) tolerance: f64,
    pub(super) tol_reached: f64,
    pub(super) continuity: Continuity,
    pub(super) max_degree: i32,
    pub(super) max_segments: i32,
    pub(super) error_status: i32,
    pub(super) warning_status: i32,
    pub(super) shape: Shape,
    pub(super) n_parts: i32,
}

impl Default for HelixBRepBuilderHelix {
    fn default() -> Self {
        Self::new()
    }
}

// The one-based array of `count` values `value` (NCollection_Array1(1, n) then Init).
fn filled<T: Clone + Default>(count: i32, value: T) -> Result<Array1<'static, T>, Failure> {
    let mut array: Array1<'static, T> =
        Array1::with_bounds(1, count).map_err(array_failure("NCollection_Array1"))?;
    array.init(value);
    Ok(array)
}

// Standard_ConstructionError of mismatched array lengths.
fn wrong_dimension(lengths: &[i32]) -> Failure {
    construction_error(&format!(
        "HelixBRep_BuilderHelix::SetParameters: wrong array dimension; lengths {lengths:?}, \
         expected the number of parts for every array"
    ))
}

// `aHeights(i) = thePitches(ip) * theNbTurns(in)` from each array's lower bound.
fn heights_of_turns(
    pitches: &Array1<'_, f64>,
    nb_turns: &Array1<'_, f64>,
) -> Result<Array1<'static, f64>, Failure> {
    let mut heights: Array1<'static, f64> = filled(pitches.length(), 0.0)?;
    let at = array_failure("NCollection_Array1::operator()");
    for i in 1..=pitches.length() {
        let pitch: f64 = *pitches.value(pitches.lower() + i - 1).map_err(&at)?;
        let turns: f64 = *nb_turns.value(nb_turns.lower() + i - 1).map_err(&at)?;
        *heights.change_value(i).map_err(&at)? = pitch * turns;
    }
    Ok(heights)
}

impl HelixBRepBuilderHelix {
    /// `HelixBRep_BuilderHelix()`: the standard axis, no parameters (error and warning
    /// status 1), tolerance 1e-4, C1, degree 8, 1000 segments, reached tolerance 99;
    /// `HelixBRepBuilderHelix::new()`.
    pub fn new() -> Self {
        Self {
            // SetDirection(gp::DZ()) and SetLocation(origin) on the default gp_Ax3 leave
            // it the standard frame.
            axis3: Axis3::default(),
            diams: None,
            heights: None,
            pitches: None,
            is_pitches: None,
            tolerance: 0.0001,
            tol_reached: 99.,
            continuity: Continuity::C1,
            max_degree: 8,
            max_segments: 1000,
            error_status: 1,
            warning_status: 1,
            shape: Shape::default(),
            n_parts: 1,
        }
    }

    /// `SetParameters(theAxis, theDiams, theHeights, thePitches, theIsPitches)`: the
    /// general composite helix of `theDiams.Length() - 1` parts; each flag says whether
    /// the pitch value is a pitch (else a number of turns). Standard_ConstructionError
    /// when the lengths differ, with the axis stored, the shape and the diameter,
    /// height and pitch arrays cleared (not the flags) and both statuses 1. The arrays
    /// keep their bounds; `builder.set_parameters(&axis, &diams, &heights, &pitches, &flags)?`.
    pub fn set_parameters(
        &mut self,
        axis: &Axis3,
        diams: &Array1<'_, f64>,
        heights: &Array1<'_, f64>,
        pitches: &Array1<'_, f64>,
        is_pitches: &Array1<'_, bool>,
    ) -> Result<(), Failure> {
        self.n_parts = diams.length() - 1;
        self.axis3 = *axis;
        (self.diams, self.heights, self.pitches) = (None, None, None);
        self.shape.nullify();
        (self.error_status, self.warning_status) = (1, 1);
        let n = self.n_parts;
        if n != heights.length() || n != pitches.length() || n != is_pitches.length() {
            return Err(wrong_dimension(&[
                diams.length(),
                heights.length(),
                pitches.length(),
                is_pitches.length(),
            ]));
        }
        self.diams = Some(copied(n + 1, diams)?);
        self.heights = Some(copied(n, heights)?);
        self.pitches = Some(copied(n, pitches)?);
        self.is_pitches = Some(copied(n, is_pitches)?);
        (self.error_status, self.warning_status) = (0, 0);
        Ok(())
    }

    /// `SetParameters(theAxis, theDiam, theHeights, thePitches, theIsPitches)`: a pure
    /// helix of one diameter; `builder.set_parameters_diam(&axis, 100.0, &h, &p, &f)?`.
    pub fn set_parameters_diam(
        &mut self,
        axis: &Axis3,
        diam: f64,
        heights: &Array1<'_, f64>,
        pitches: &Array1<'_, f64>,
        is_pitches: &Array1<'_, bool>,
    ) -> Result<(), Failure> {
        let diams: Array1<'static, f64> = filled(heights.length() + 1, diam)?;
        self.set_parameters(axis, &diams, heights, pitches, is_pitches)
    }

    /// `SetParameters(theAxis, theDiam1, theDiam2, theHeights, thePitches,
    /// theIsPitches)`: a pure spiral whose diameter goes linearly with the height from
    /// `diam1` to `diam2`. Native quirk kept: after the sum over the whole array, the
    /// partial heights restart from the absolute index 1 (another element, or
    /// Standard_OutOfRange, for an array not based at 1);
    /// `builder.set_parameters_spiral(&axis, 100.0, 20.0, &h, &p, &f)?`.
    pub fn set_parameters_spiral(
        &mut self,
        axis: &Axis3,
        diam1: f64,
        diam2: f64,
        heights: &Array1<'_, f64>,
        pitches: &Array1<'_, f64>,
        is_pitches: &Array1<'_, bool>,
    ) -> Result<(), Failure> {
        let diams: Array1<'static, f64> = spiral_diameters(diam1, diam2, heights)?;
        self.set_parameters(axis, &diams, heights, pitches, is_pitches)
    }

    /// `SetParameters(theAxis, theDiams, thePitches, theNbTurns)`: the general
    /// composite helix with each height `pitch * turns`;
    /// `builder.set_parameters_turns(&axis, &diams, &pitches, &turns)?`.
    pub fn set_parameters_turns(
        &mut self,
        axis: &Axis3,
        diams: &Array1<'_, f64>,
        pitches: &Array1<'_, f64>,
        nb_turns: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let n_parts: i32 = diams.length() - 1;
        if n_parts != pitches.length() || n_parts != nb_turns.length() {
            return Err(wrong_dimension(&[
                diams.length(),
                pitches.length(),
                nb_turns.length(),
            ]));
        }
        let heights = heights_of_turns(pitches, nb_turns)?;
        let is_pitches: Array1<'static, bool> = filled(n_parts, true)?;
        self.set_parameters(axis, diams, &heights, pitches, &is_pitches)
    }

    /// `SetParameters(theAxis, theDiam, thePitches, theNbTurns)`: a pure helix with
    /// each height `pitch * turns`; `builder.set_parameters_diam_turns(&axis, 100.0, &p, &t)?`.
    pub fn set_parameters_diam_turns(
        &mut self,
        axis: &Axis3,
        diam: f64,
        pitches: &Array1<'_, f64>,
        nb_turns: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let (heights, is_pitches) = Self::turns_parts(pitches, nb_turns)?;
        self.set_parameters_diam(axis, diam, &heights, pitches, &is_pitches)
    }

    /// `SetParameters(theAxis, theDiam1, theDiam2, thePitches, theNbTurns)`: a pure
    /// spiral with each height `pitch * turns`;
    /// `builder.set_parameters_spiral_turns(&axis, 100.0, 20.0, &p, &t)?`.
    pub fn set_parameters_spiral_turns(
        &mut self,
        axis: &Axis3,
        diam1: f64,
        diam2: f64,
        pitches: &Array1<'_, f64>,
        nb_turns: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let (heights, is_pitches) = Self::turns_parts(pitches, nb_turns)?;
        self.set_parameters_spiral(axis, diam1, diam2, &heights, pitches, &is_pitches)
    }

    // The heights and all-true flags of the one-diameter turn overloads, after their
    // length check.
    fn turns_parts(
        pitches: &Array1<'_, f64>,
        nb_turns: &Array1<'_, f64>,
    ) -> Result<(Array1<'static, f64>, Array1<'static, bool>), Failure> {
        let n_parts: i32 = pitches.length();
        if n_parts != nb_turns.length() {
            return Err(wrong_dimension(&[pitches.length(), nb_turns.length()]));
        }
        Ok((heights_of_turns(pitches, nb_turns)?, filled(n_parts, true)?))
    }

    /// `SetApproxParameters(theTolerance, theMaxDegree, theContinuity)`;
    /// `builder.set_approx_parameters(1e-4, 8, Continuity::C1)`.
    pub fn set_approx_parameters(
        &mut self,
        tolerance: f64,
        max_degree: i32,
        continuity: Continuity,
    ) {
        (self.tolerance, self.max_degree, self.continuity) = (tolerance, max_degree, continuity);
    }

    /// `ToleranceReached()`: the largest approximation error of the parts;
    /// `builder.tolerance_reached()`.
    pub fn tolerance_reached(&self) -> f64 {
        self.tol_reached
    }

    /// `ErrorStatus()`: 0 on success, 1 without valid parameters, 2 when a coil
    /// approximation failed, 3 when an edge could not be made, 10 when the start point
    /// is on the axis, 11 for a pitch, 12 for a height below the tolerance, 13 for a
    /// taper angle too close to PI/2; `builder.error_status()`.
    pub fn error_status(&self) -> i32 {
        self.error_status
    }

    /// `WarningStatus()`: 1 when the reached tolerance exceeds the tolerance (or
    /// without valid parameters); `builder.warning_status()`.
    pub fn warning_status(&self) -> i32 {
        self.warning_status
    }

    /// `Shape()`: the helix wire, null before a successful `Perform`; `builder.shape()`.
    pub fn shape(&self) -> &Shape {
        &self.shape
    }
}

// `new NCollection_HArray1(1, count)` then `ChangeArray1() = source`: the copy takes the
// bounds of the source (NCollection_Array1::Assign).
fn copied<T: Clone + Default>(
    count: i32,
    source: &Array1<'_, T>,
) -> Result<Array1<'static, T>, Failure> {
    let mut array: Array1<'static, T> =
        Array1::with_bounds(1, count).map_err(array_failure("NCollection_HArray1"))?;
    array.assign(source);
    Ok(array)
}

// The diameters of the spiral overload, in the native order: the sum of all heights,
// the slope, both ends, then the partial sums restarting from the absolute index 1.
fn spiral_diameters(
    diam1: f64,
    diam2: f64,
    heights: &Array1<'_, f64>,
) -> Result<Array1<'static, f64>, Failure> {
    let n_parts: i32 = heights.length();
    let mut diams: Array1<'static, f64> = filled(n_parts + 1, 0.0)?;
    let at = array_failure("NCollection_Array1::operator()");
    let mut an_h: f64 = heights.iter().fold(0., |sum, h| sum + h);
    let k: f64 = (diam2 - diam1) / an_h;
    *diams.change_value(1).map_err(&at)? = diam1;
    *diams.change_value(n_parts + 1).map_err(&at)? = diam2;
    an_h = *heights.value(1).map_err(&at)?;
    // j runs from 2 beside i, as the native `for (i = Lower() + 1, j = 2; ...)`.
    for (j, i) in (2_i32..).zip((heights.lower() + 1)..=heights.upper()) {
        *diams.change_value(j).map_err(&at)? = diam1 + k * an_h;
        an_h += *heights.value(i).map_err(&at)?;
    }
    Ok(diams)
}
