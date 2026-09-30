//! `ApproxInt_Approx.gxx`: the approximation of a walking line by B-spline curves (3d
//! and in the parameter planes), by Bezier pieces joined into a B-spline when the line
//! has enough points (docs/geomalgo-design.md section 20). The `Perform` overloads are
//! in `perform`, the private steps in `build`.
mod build;
mod perform;

pub use perform::ApproxIntRange;

use super::line::ApproxIntLine;
use super::multi_line::{ApproxIntMultiLine, ApproxIntOffsets};
use super::surface_tool::ApproxIntSurfaceTool;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays;
use rustycad_tkgeombase::app_par_curves::{Constraint, MultiBSpCurve};
use rustycad_tkgeombase::approx::{
    BSplComputeLine, ComputeLine, ComputeLineOptions, MCurvesToBSpCurve, ParametrizationType,
};
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::math::Vector;
use std::marker::PhantomData;

/// The ratio between the requested and the internal tolerances ("should be removed in
/// the future", natively).
const RATIO_TOL: f64 = 1.5;

/// The macros of an `ApproxInt_Approx` instance: `ThePSurfaceTool` and `TheWLine`.
pub trait ApproxIntApproxTypes {
    /// `ThePSurfaceTool`.
    type SurfaceTool: ApproxIntSurfaceTool + 'static;
    /// `TheWLine`.
    type Line: ApproxIntLine + 'static;
}

/// `Approx_Data`: the options and the offsets of the current approximation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ApproxIntApproxData {
    /// `myBezierApprox`: Bezier pieces joined (true) or one B-spline.
    pub bezier_approx: bool,
    /// The offsets of the points (`Xo`... `V2o`).
    pub offsets: ApproxIntOffsets,
    /// `ApproxXYZ`.
    pub approx_xyz: bool,
    /// `ApproxU1V1`.
    pub approx_u1v1: bool,
    /// `ApproxU2V2`.
    pub approx_u2v2: bool,
    /// `indicemin`.
    pub indicemin: i32,
    /// `indicemax`.
    pub indicemax: i32,
    /// `myNbPntMax`: the minimum number of points per knot interval.
    pub nb_pnt_max: i32,
    /// `parametrization`.
    pub parametrization: ParametrizationType,
}

impl Default for ApproxIntApproxData {
    fn default() -> Self {
        Self {
            bezier_approx: true,
            offsets: ApproxIntOffsets::default(),
            approx_xyz: true,
            approx_u1v1: true,
            approx_u2v2: true,
            indicemin: 0,
            indicemax: 0,
            nb_pnt_max: 30,
            parametrization: ParametrizationType::ChordLength,
        }
    }
}

/// `ApproxInt_Approx` over the types `B` (GeomInt_WLApprox over Adaptor3d_HSurfaceTool
/// and IntPatch_WLine).
///
/// ```ignore
/// let mut approx = GeomIntWLApprox::new();
/// approx.set_parameters(1e-7, 1e-7, 4, 8, 0, 30, true, ParametrizationType::ChordLength);
/// approx.perform_on_surfaces(&s1, &s2, &wline, (true, true, true), 1, 20)?;
/// ```
pub struct ApproxIntApprox<B: ApproxIntApproxTypes> {
    my_compute_line: BSplComputeLine,
    my_compute_line_bezier: ComputeLine,
    my_bez_to_bspl: MCurvesToBSpCurve,
    my_with_tangency: bool,
    my_tol3d: f64,
    my_tol2d: f64,
    my_deg_min: i32,
    my_deg_max: i32,
    my_nb_iter_max: i32,
    my_tol_reached3d: f64,
    my_tol_reached2d: f64,
    my_data: ApproxIntApproxData,
    my_knots: Vec<i32>,
    types: PhantomData<B>,
}

impl<B: ApproxIntApproxTypes> Default for ApproxIntApprox<B> {
    fn default() -> Self {
        Self::new()
    }
}

/// The options of the two compute lines: degrees, tolerances, iterations, cutting and
/// parametrization (`Squares` false).
fn compute_options(
    (deg_min, deg_max): (i32, i32),
    (tol3d, tol2d): (f64, f64),
    nb_iter: i32,
    (cutting, parametrization): (bool, ParametrizationType),
) -> ComputeLineOptions {
    ComputeLineOptions {
        degree_min: deg_min,
        degree_max: deg_max,
        tolerance_3d: tol3d,
        tolerance_2d: tol2d,
        nb_iterations: nb_iter,
        cutting,
        parametrization,
        squares: false,
    }
}

impl<B: ApproxIntApproxTypes> ApproxIntApprox<B> {
    /// `ApproxInt_Approx()`: degrees 4..8, tolerances 1e-3, 5 iterations, continuity 2
    /// for the B-spline line; `GeomIntWLApprox::new()`.
    pub fn new() -> Self {
        let options = compute_options(
            (4, 8),
            (0.001, 0.001),
            5,
            (true, ParametrizationType::ChordLength),
        );
        let mut my_compute_line = BSplComputeLine::new(options);
        my_compute_line.set_continuity(2);
        Self {
            my_compute_line,
            my_compute_line_bezier: ComputeLine::new(options),
            my_bez_to_bspl: MCurvesToBSpCurve::new(),
            my_with_tangency: true,
            my_tol3d: 0.001,
            my_tol2d: 0.001,
            my_deg_min: 4,
            my_deg_max: 8,
            my_nb_iter_max: 5,
            my_tol_reached3d: 0.0,
            my_tol_reached2d: 0.0,
            my_data: ApproxIntApproxData::default(),
            my_knots: Vec::new(),
            types: PhantomData,
        }
    }

    /// `SetParameters(Tol3d, Tol2d, DegMin, DegMax, NbIterMax, NbPntMax,
    /// ApproxWithTangency, Parametrization)`: the tolerances are divided by 1.5; without
    /// tangency both lines pass through the end points (and keep that constraint);
    /// `approx.set_parameters(1e-7, 1e-7, 4, 8, 0, 30, true, ParametrizationType::ChordLength)`.
    #[allow(clippy::too_many_arguments)]
    pub fn set_parameters(
        &mut self,
        tol3d: f64,
        tol2d: f64,
        deg_min: i32,
        deg_max: i32,
        nb_iter_max: i32,
        nb_pnt_max: i32,
        approx_with_tangency: bool,
        parametrization: ParametrizationType,
    ) {
        self.my_data.nb_pnt_max = nb_pnt_max;
        self.my_with_tangency = approx_with_tangency;
        self.my_tol3d = tol3d / RATIO_TOL;
        self.my_tol2d = tol2d / RATIO_TOL;
        self.my_deg_min = deg_min;
        self.my_deg_max = deg_max;
        self.my_nb_iter_max = nb_iter_max;
        self.init_compute_lines(true, parametrization);
        if !approx_with_tangency {
            self.my_compute_line
                .set_constraints(Constraint::PassPoint, Constraint::PassPoint);
            self.my_compute_line_bezier
                .set_constraints(Constraint::PassPoint, Constraint::PassPoint);
        }
        self.my_data.bezier_approx = true;
    }

    /// `Init` of both compute lines with the current options.
    fn init_compute_lines(&mut self, cutting: bool, parametrization: ParametrizationType) {
        let options = compute_options(
            (self.my_deg_min, self.my_deg_max),
            (self.my_tol3d, self.my_tol2d),
            self.my_nb_iter_max,
            (cutting, parametrization),
        );
        self.my_compute_line.init(options);
        self.my_compute_line_bezier.init(options);
    }

    /// `TolReached3d()`: the reached 3d tolerance times 1.5; `approx.tol_reached3d()`.
    pub fn tol_reached3d(&self) -> f64 {
        self.my_tol_reached3d * RATIO_TOL
    }

    /// `TolReached2d()`: the reached 2d tolerance times 1.5; `approx.tol_reached2d()`.
    pub fn tol_reached2d(&self) -> f64 {
        self.my_tol_reached2d * RATIO_TOL
    }

    /// `IsDone()`: some Bezier piece was made, or the B-spline reached the tolerance;
    /// `approx.is_done()`.
    pub fn is_done(&self) -> bool {
        if self.my_data.bezier_approx {
            self.my_compute_line_bezier.nb_multi_curves() > 0
        } else {
            self.my_compute_line.is_tolerance_reached()
        }
    }

    /// `NbMultiCurves()`: always 1; `approx.nb_multi_curves()`.
    pub fn nb_multi_curves(&self) -> i32 {
        1
    }

    /// `Value(Index)`: the joined Bezier pieces or the B-spline (the index is ignored);
    /// `approx.value(1)`.
    pub fn value(&self, _index: i32) -> &MultiBSpCurve {
        if self.my_data.bezier_approx {
            self.my_bez_to_bspl.value()
        } else {
            self.my_compute_line.value()
        }
    }

    /// The data of the last approximation (the private `myData`).
    pub fn data(&self) -> &ApproxIntApproxData {
        &self.my_data
    }

    /// The knot indices of the last approximation (the private `myKnots`).
    pub fn knots(&self) -> &[i32] {
        &self.my_knots
    }

    /// `Parameters(Line, firstP, lastP, Par, TheParameters)`: the parameters of the
    /// points `first_p..=last_p` of the line, normalized to `[0, 1]`: the cumulated
    /// distances in the approximated spaces (chord length) or their square roots
    /// (centripetal), else uniform;
    /// `GeomIntWLApprox::parameters(&line, 1, 20, ParametrizationType::ChordLength, &mut pars)?`.
    pub fn parameters(
        line: &ApproxIntMultiLine<B::Line>,
        first_p: i32,
        last_p: i32,
        par: ParametrizationType,
        the_parameters: &mut Vector<'_>,
    ) -> Result<(), Failure> {
        if par != ParametrizationType::ChordLength && par != ParametrizationType::Centripetal {
            for i in first_p..=last_p {
                the_parameters[i] =
                    (f64::from(i) - f64::from(first_p)) / (f64::from(last_p) - f64::from(first_p));
            }
            return Ok(());
        }
        the_parameters[first_p] = 0.0;
        let mut tabs = ParameterTabs::new(line.nb_p3d(), line.nb_p2d())?;
        for i in first_p + 1..=last_p {
            let dist: f64 = tabs.distance(line, i)?;
            let step: f64 = if par == ParametrizationType::ChordLength {
                dist
            } else {
                dist.sqrt()
            };
            the_parameters[i] = the_parameters[i - 1] + step;
        }
        let total: f64 = the_parameters[last_p];
        for i in first_p..=last_p {
            the_parameters[i] /= total;
        }
        Ok(())
    }
}

/// The point arrays of `Parameters` (`tabP`, `tabPP`, `tabP2d`, `tabPP2d`), kept from
/// one point to the next as natively.
struct ParameterTabs {
    nb_p3d: i32,
    nb_p2d: i32,
    tab_p: Array1<'static, Point3>,
    tab_pp: Array1<'static, Point3>,
    tab_p2d: Array1<'static, Point2>,
    tab_pp2d: Array1<'static, Point2>,
}

impl ParameterTabs {
    fn new(nb_p3d: i32, nb_p2d: i32) -> Result<Self, Failure> {
        Ok(Self {
            nb_p3d,
            nb_p2d,
            tab_p: arrays::one_based(nb_p3d.max(1))?,
            tab_pp: arrays::one_based(nb_p3d.max(1))?,
            tab_p2d: arrays::one_based(nb_p2d.max(1))?,
            tab_pp2d: arrays::one_based(nb_p2d.max(1))?,
        })
    }

    /// Reads the multipoint `i` into the given arrays, as the native `if` chain.
    fn read<L: ApproxIntLine>(
        (nb_p3d, nb_p2d): (i32, i32),
        line: &ApproxIntMultiLine<L>,
        i: i32,
        (tab, tab2d): (&mut Array1<'static, Point3>, &mut Array1<'static, Point2>),
    ) -> Result<(), Failure> {
        if nb_p3d != 0 && nb_p2d != 0 {
            line.value(i, tab, tab2d)
        } else if nb_p2d != 0 {
            line.value_2d(i, tab2d)
        } else if nb_p3d != 0 {
            line.value_3d(i, tab)
        } else {
            Ok(())
        }
    }

    /// The distance between the multipoints `i - 1` and `i`.
    fn distance<L: ApproxIntLine>(
        &mut self,
        line: &ApproxIntMultiLine<L>,
        i: i32,
    ) -> Result<f64, Failure> {
        let counts: (i32, i32) = (self.nb_p3d, self.nb_p2d);
        Self::read(counts, line, i - 1, (&mut self.tab_p, &mut self.tab_p2d))?;
        Self::read(counts, line, i, (&mut self.tab_pp, &mut self.tab_pp2d))?;
        let mut dist: f64 = 0.0;
        for j in 1..=self.nb_p3d {
            dist += arrays::at(&self.tab_pp, j)?.square_distance(&arrays::at(&self.tab_p, j)?);
        }
        for j in 1..=self.nb_p2d {
            dist += arrays::at(&self.tab_pp2d, j)?.square_distance(arrays::at(&self.tab_p2d, j)?);
        }
        Ok(dist.sqrt())
    }
}
