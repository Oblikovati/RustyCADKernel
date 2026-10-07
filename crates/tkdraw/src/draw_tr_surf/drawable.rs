// Rust translation of Draw/TKDraw/DrawTrSurf/DrawTrSurf_Drawable.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DrawTrSurf_Drawable`: the discretisation settings every geometric drawable shares
//! and the curve plotting they use.
use crate::draw::{DrawDisplay, Drawable3DBase};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dIsoCurve};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::gcpnts::UniformDeflection;
use rustycad_tkmath::geom_abs::{CurveType, IsoType, Shape};
use rustycad_tkmath::gp::{Point2, Point3};
use std::any::{Any, TypeId};
use std::cell::Cell;

/// `IsoRatio` of `PlotCurve`: a chord is kept when the midpoint is at most 0.1% off.
const ISO_RATIO: f64 = 1.001;

/// `DrawTrSurf_Drawable`: the common state (the drawable base, the draw mode, the
/// number of points per interval and the deflection) of the DrawTrSurf objects.
///
/// ```
/// use rustycad_tkdraw::draw_tr_surf::DrawTrSurfDrawable;
/// let drawable = DrawTrSurfDrawable::new(16, 0.01, 1);
/// drawable.set_discretisation(30);
/// assert_eq!((drawable.discretisation(), drawable.draw_mode()), (30, 1));
/// ```
#[derive(Debug)]
pub struct DrawTrSurfDrawable {
    base: Drawable3DBase,
    draw_mode: Cell<i32>,
    discret: Cell<i32>,
    deflection: Cell<f64>,
}

impl DrawTrSurfDrawable {
    /// The protected constructor `DrawTrSurf_Drawable(discret, deflection, DrawMode)`
    /// (the C++ defaults are 0.01 and 0).
    pub fn new(discret: i32, deflection: f64, draw_mode: i32) -> Self {
        Self {
            base: Drawable3DBase::new(),
            draw_mode: Cell::new(draw_mode),
            discret: Cell::new(discret),
            deflection: Cell::new(deflection),
        }
    }

    /// The `Draw_Drawable3D` part.
    pub fn base(&self) -> &Drawable3DBase {
        &self.base
    }

    /// The [`crate::draw::Drawable3D::ancestor`] answer of a drawable built on this
    /// part: this part when `type_id` names `DrawTrSurf_Drawable`.
    pub fn as_kind(&self, type_id: TypeId) -> Option<&dyn Any> {
        (type_id == TypeId::of::<Self>()).then_some(self as &dyn Any)
    }

    /// `SetDiscretisation`.
    pub fn set_discretisation(&self, discret: i32) {
        self.discret.set(discret);
    }

    /// `GetDiscretisation`.
    pub fn discretisation(&self) -> i32 {
        self.discret.get()
    }

    /// `SetDeflection`.
    pub fn set_deflection(&self, deflection: f64) {
        self.deflection.set(deflection);
    }

    /// `GetDeflection`.
    pub fn deflection(&self) -> f64 {
        self.deflection.get()
    }

    /// `SetDrawMode`: 1 draws by uniform deflection, 0 by `discret` points per interval.
    pub fn set_draw_mode(&self, draw_mode: i32) {
        self.draw_mode.set(draw_mode);
    }

    /// `GetDrawMode`.
    pub fn draw_mode(&self) -> i32 {
        self.draw_mode.get()
    }

    /// `DrawCurve2dOn`: plots a 2D curve, by uniform deflection in draw mode 1, else
    /// with `discret` steps on each CN interval (lines get only their end points).
    pub fn draw_curve2d_on(
        &self,
        curve: &dyn Adaptor2dCurve2d,
        display: &mut DrawDisplay<'_>,
    ) -> Result<(), Failure> {
        if self.draw_mode() == 1 {
            let deflection = self.deflection() / display.zoom();
            let line = UniformDeflection::with_curve(curve, deflection, true)?;
            // The C++ code reads the gp_Pnt of each point through a gp_Pnt2d pointer: x, y.
            let to_2d = |point: Point3| Point2::new(point.x(), point.y());
            return plot_deflection(
                &line,
                display,
                |d, p| d.move_to(to_2d(p)),
                |d, p| d.draw_to(to_2d(p)),
            );
        }
        let bounds = interval_bounds(curve.nb_intervals(Shape::CN)?, |b| {
            curve.intervals(b, Shape::CN)
        })?;
        display.move_to(curve.d0(curve.first_parameter()?)?);
        let is_line = curve.get_type()? == CurveType::Line;
        for interval in 1..bounds.len() as i32 {
            let mut t = bound(&bounds, interval)?;
            if !is_line {
                let step = (bound(&bounds, interval + 1)? - t) / f64::from(self.discretisation());
                for _ in 1..self.discretisation() {
                    t += step;
                    display.draw_to(curve.d0(t)?);
                }
            }
            display.draw_to(curve.d0(bound(&bounds, interval + 1)?)?);
        }
        Ok(())
    }

    /// `DrawCurveOn`: plots a 3D curve, by uniform deflection in draw mode 1, else
    /// adaptively per CN interval (circles and ellipses by equal steps, other curves by
    /// double steps refined while the chord midpoint is off the curve).
    pub fn draw_curve_on(
        &self,
        curve: &dyn Adaptor3dCurve,
        display: &mut DrawDisplay<'_>,
    ) -> Result<(), Failure> {
        if self.draw_mode() == 1 {
            let deflection = self.deflection() / display.zoom();
            let line = UniformDeflection::with_curve(curve, deflection, true)?;
            return plot_deflection(
                &line,
                display,
                |d, p| d.move_to_3d(p),
                |d, p| d.draw_to_3d(p),
            );
        }
        self.draw_intervals_on(curve, display)
    }

    /// The discrete mode of `DrawCurveOn`: circles and ellipses take `discret` steps,
    /// the other non-linear curves `discret / 2` double steps refined by `PlotCurve`.
    ///
    /// Native quirk kept: the previous point of the refinement is not moved to the end
    /// of an interval, so the next interval starts its first chord there.
    fn draw_intervals_on(
        &self,
        curve: &dyn Adaptor3dCurve,
        display: &mut DrawDisplay<'_>,
    ) -> Result<(), Failure> {
        let bounds = interval_bounds(curve.nb_intervals(Shape::CN)?, |b| {
            curve.intervals(b, Shape::CN)
        })?;
        let first = curve.d0(curve.first_parameter()?)?;
        display.move_to_3d(first);
        let curve_type = curve.get_type()?;
        let mut previous = first;
        for interval in 1..bounds.len() as i32 {
            let mut t = bound(&bounds, interval)?;
            let end = bound(&bounds, interval + 1)?;
            let step = (end - t) / f64::from(self.discretisation());
            match curve_type {
                CurveType::Line => {}
                CurveType::Circle | CurveType::Ellipse => {
                    for _ in 1..self.discretisation() {
                        t += step;
                        display.draw_to_3d(curve.d0(t)?);
                    }
                }
                _ => {
                    for _ in 1..self.discretisation() / 2 {
                        let t1 = t + step * 2.;
                        let next = curve.d0(t1)?;
                        plot_curve(display, curve, t, step, previous, next)?;
                        previous = next;
                        t = t1;
                    }
                }
            }
            let last = curve.d0(end)?;
            plot_curve(display, curve, t, step, previous, last)?;
            display.draw_to_3d(last);
        }
        Ok(())
    }

    /// `DrawIsoCurveOn`: loads the iso `iso` at `parameter` on `[first, last]` and plots
    /// it; a Bezier or BSpline iso is plotted through a `GeomAdaptor_Curve` on it.
    pub fn draw_iso_curve_on(
        &self,
        curve: &Adaptor3dIsoCurve,
        iso: IsoType,
        parameter: f64,
        (first, last): (f64, f64),
        display: &mut DrawDisplay<'_>,
    ) -> Result<(), Failure> {
        curve.load_iso_range(iso, parameter, (first, last))?;
        let geometry = match curve.get_type()? {
            CurveType::BezierCurve => curve.bezier()?.cast::<dyn GeomCurve>(),
            CurveType::BSplineCurve => curve.bspline()?.cast::<dyn GeomCurve>(),
            _ => return self.draw_curve_on(curve, display),
        };
        let adaptor = GeomAdaptorCurve::default();
        adaptor.load_range(&geometry, first, last)?;
        self.draw_curve_on(&adaptor, display)
    }
}

/// Draws the points of a uniform deflection: a move to the first, lines to the others.
fn plot_deflection<P, D>(
    line: &UniformDeflection,
    display: &mut DrawDisplay<'_>,
    mut move_to: P,
    mut draw_to: D,
) -> Result<(), Failure>
where
    P: FnMut(&mut DrawDisplay<'_>, Point3),
    D: FnMut(&mut DrawDisplay<'_>, Point3),
{
    if !line.is_done() {
        return Ok(());
    }
    move_to(display, line.value(1)?);
    for index in 2..=line.nb_points()? {
        draw_to(display, line.value(index)?);
    }
    Ok(())
}

/// `PlotCurve`: draws to `last` when the curve midpoint is within `ISO_RATIO` of the
/// chord `first`-`last` (or the chord is shorter than the confusion), else halves the
/// step on both halves.
fn plot_curve(
    display: &mut DrawDisplay<'_>,
    curve: &dyn Adaptor3dCurve,
    first_param: f64,
    half_step: f64,
    first: Point3,
    last: Point3,
) -> Result<(), Failure> {
    let middle = curve.d0(first_param + half_step)?;
    let length = first.distance(&last);
    if length < precision::confusion()
        || middle.distance(&first) + middle.distance(&last) <= ISO_RATIO * length
    {
        display.draw_to_3d(last);
        return Ok(());
    }
    plot_curve(display, curve, first_param, half_step / 2., first, middle)?;
    let local_first = first_param + half_step;
    plot_curve(display, curve, local_first, half_step / 2., middle, last)
}

/// `NCollection_Array1<double> TI(1, nbintv + 1); C.Intervals(TI, GeomAbs_CN);`.
pub(crate) fn interval_bounds(
    nb_intervals: i32,
    fill: impl FnOnce(&mut Array1<'static, f64>) -> Result<(), Failure>,
) -> Result<Array1<'static, f64>, Failure> {
    let mut bounds: Array1<'static, f64> = Array1::with_bounds(1, nb_intervals + 1)
        .map_err(|error| error.into_failure("NCollection_Array1"))?;
    fill(&mut bounds)?;
    Ok(bounds)
}

/// `TI(index)`; Standard_OutOfRange outside the bounds.
pub(crate) fn bound(bounds: &Array1<'_, f64>, index: i32) -> Result<f64, Failure> {
    bounds
        .value(index)
        .copied()
        .map_err(|error| error.into_failure("NCollection_Array1::Value"))
}
