// Rust translation of the pole and knot members and loops that the DrawTrSurf Bezier
// and BSpline drawables (curves, 2D curves, surfaces) repeat, at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The poles and knots of the spline drawables: their display settings, the control
//! polygon and the `FindPole`/`FindKnot` search.
use super::curve_common::PlotPoint;
use super::params::DrawTrSurfParams;
use crate::draw::{DrawColor, DrawColorKind, DrawDisplay, DrawMarkerShape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3};
use std::cell::Cell;

/// The control polygon settings (`drawPoles`, `polesLook`).
///
/// ```
/// use rustycad_tkdraw::draw::{DrawColor, DrawColorKind};
/// use rustycad_tkdraw::draw_tr_surf::PolesStyle;
/// let poles = PolesStyle::new(true, DrawColor::new(DrawColorKind::Rouge));
/// poles.clear();
/// assert!(!poles.shown());
/// ```
#[derive(Debug, Clone)]
pub struct PolesStyle {
    shown: Cell<bool>,
    color: Cell<DrawColor>,
}

impl PolesStyle {
    /// Poles shown or not, in `color`.
    pub fn new(shown: bool, color: DrawColor) -> Self {
        Self {
            shown: Cell::new(shown),
            color: Cell::new(color),
        }
    }

    /// The default of the one-argument constructors: shown, in red.
    pub fn default_shown() -> Self {
        Self::new(true, DrawColor::new(DrawColorKind::Rouge))
    }

    /// `ShowPoles`.
    pub fn show(&self) {
        self.shown.set(true);
    }

    /// `ClearPoles`.
    pub fn clear(&self) {
        self.shown.set(false);
    }

    /// Whether the control polygon is drawn (`drawPoles`).
    pub fn shown(&self) -> bool {
        self.shown.get()
    }

    /// `SetPolesColor`.
    pub fn set_color(&self, color: DrawColor) {
        self.color.set(color);
    }

    /// `PolesColor`.
    pub fn color(&self) -> DrawColor {
        self.color.get()
    }
}

/// The knot marker settings (`drawKnots`, `knotsForm`, `knotsLook`, `knotsDim`).
///
/// ```
/// use rustycad_tkdraw::draw::{DrawColor, DrawColorKind, DrawMarkerShape};
/// use rustycad_tkdraw::draw_tr_surf::KnotsStyle;
/// let knots = KnotsStyle::default_shown();
/// assert_eq!((knots.shape(), knots.size()), (DrawMarkerShape::Losange, 5));
/// ```
#[derive(Debug, Clone)]
pub struct KnotsStyle {
    shown: Cell<bool>,
    shape: Cell<DrawMarkerShape>,
    color: Cell<DrawColor>,
    size: i32,
}

impl KnotsStyle {
    /// Knots shown or not, as `shape` markers of `size` pixels in `color`.
    pub fn new(shown: bool, shape: DrawMarkerShape, color: DrawColor, size: i32) -> Self {
        Self {
            shown: Cell::new(shown),
            shape: Cell::new(shape),
            color: Cell::new(color),
            size,
        }
    }

    /// The default of the one-argument constructors: shown, violet diamonds of 5
    /// pixels.
    pub fn default_shown() -> Self {
        Self::new(
            true,
            DrawMarkerShape::Losange,
            DrawColor::new(DrawColorKind::Violet),
            5,
        )
    }

    /// `ShowKnots`.
    pub fn show(&self) {
        self.shown.set(true);
    }

    /// `ClearKnots`.
    pub fn clear(&self) {
        self.shown.set(false);
    }

    /// Whether the knots are marked (`drawKnots`).
    pub fn shown(&self) -> bool {
        self.shown.get()
    }

    /// `SetKnotsShape`.
    pub fn set_shape(&self, shape: DrawMarkerShape) {
        self.shape.set(shape);
    }

    /// `KnotsShape`.
    pub fn shape(&self) -> DrawMarkerShape {
        self.shape.get()
    }

    /// `SetKnotsColor`.
    pub fn set_color(&self, color: DrawColor) {
        self.color.set(color);
    }

    /// `KnotsColor`.
    pub fn color(&self) -> DrawColor {
        self.color.get()
    }

    /// `knotsDim`: the marker size in pixels.
    pub fn size(&self) -> i32 {
        self.size
    }

    /// Marks `points` in the knot color, shape and size.
    pub(crate) fn mark<P: PlotPoint>(
        &self,
        display: &mut DrawDisplay<'_>,
        points: impl IntoIterator<Item = Result<P, Failure>>,
    ) -> Result<(), Failure> {
        display.set_color(self.color());
        for point in points {
            point?.mark_on(display, self.shape(), self.size);
        }
        Ok(())
    }
}

/// The constructor arguments of a spline drawable beyond the geometry and the
/// discretisation: curve, pole and knot colors, knot marker, and what is shown.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplineLook {
    /// `CurvColor`.
    pub curve_color: DrawColor,
    /// `PolesColor`.
    pub poles_color: DrawColor,
    /// `KnotsColor`.
    pub knots_color: DrawColor,
    /// `KnotsShape`.
    pub knots_shape: DrawMarkerShape,
    /// `KnotsSize`.
    pub knots_size: i32,
    /// `ShowPoles`.
    pub show_poles: bool,
    /// `ShowKnots`.
    pub show_knots: bool,
}

impl SplineLook {
    /// The look `DrawTrSurf::Set` and the restore factories give new splines.
    pub fn from_params(params: &DrawTrSurfParams) -> Self {
        Self {
            curve_color: params.curv_color,
            poles_color: params.poles_color,
            knots_color: params.knots_color,
            knots_shape: params.knots_marker,
            knots_size: params.knots_size,
            show_poles: params.is_show_poles,
            show_knots: params.is_show_knots,
        }
    }

    /// The look of an existing drawable (for `Copy`).
    pub(crate) fn of(curve_color: DrawColor, poles: &PolesStyle, knots: &KnotsStyle) -> Self {
        Self {
            curve_color,
            poles_color: poles.color(),
            knots_color: knots.color(),
            knots_shape: knots.shape(),
            knots_size: knots.size(),
            show_poles: poles.shown(),
            show_knots: knots.shown(),
        }
    }

    /// The pole settings.
    pub(crate) fn poles(&self) -> PolesStyle {
        PolesStyle::new(self.show_poles, self.poles_color)
    }

    /// The knot settings.
    pub(crate) fn knots(&self) -> KnotsStyle {
        KnotsStyle::new(
            self.show_knots,
            self.knots_shape,
            self.knots_color,
            self.knots_size,
        )
    }
}

/// The control polygon: a move to the first pole and lines to the others, in `color`.
pub(crate) fn draw_polygon<P: PlotPoint>(
    display: &mut DrawDisplay<'_>,
    color: DrawColor,
    poles: &[P],
) {
    display.set_color(color);
    let Some((first, rest)) = poles.split_first() else {
        return;
    };
    first.move_on(display);
    for pole in rest {
        pole.draw_on(display);
    }
}

/// The pole net of a surface: each V column of poles along U (closed when the surface
/// is U-periodic), then each U row along V (closed when V-periodic), in `color`.
pub(crate) fn draw_pole_net(
    display: &mut DrawDisplay<'_>,
    color: DrawColor,
    (nb_u, nb_v): (i32, i32),
    (u_periodic, v_periodic): (bool, bool),
    pole: impl Fn(i32, i32) -> Result<Point3, Failure>,
) -> Result<(), Failure> {
    display.set_color(color);
    for j in 1..=nb_v {
        display.move_to_3d(pole(1, j)?);
        for i in 2..=nb_u {
            display.draw_to_3d(pole(i, j)?);
        }
        if u_periodic {
            display.draw_to_3d(pole(1, j)?);
        }
    }
    for i in 1..=nb_u {
        display.move_to_3d(pole(i, 1)?);
        for j in 2..=nb_v {
            display.draw_to_3d(pole(i, j)?);
        }
        if v_periodic {
            display.draw_to_3d(pole(i, 1)?);
        }
    }
    Ok(())
}

/// The `FindPole` loop of the surfaces: the first pole after `(u_index, v_index)` in
/// U-then-V order whose projection is within `precision` of `target`; `(0, 0)` when
/// none is. Both indices advance first, so the search resumes at the next V column.
pub(crate) fn find_net_index(
    display: &DrawDisplay<'_>,
    (target, precision): (Point2, f64),
    (u_index, v_index): (i32, i32),
    (nb_u, nb_v): (i32, i32),
    pole: impl Fn(i32, i32) -> Result<Point3, Failure>,
) -> Result<(i32, i32), Failure> {
    let mut u = u_index + 1;
    for v in v_index + 1..=nb_v {
        while u <= nb_u {
            if display.project(pole(u, v)?).distance(target) <= precision {
                return Ok((u, v));
            }
            u += 1;
        }
        u = 1;
    }
    Ok((0, 0))
}

/// The search loop of `FindPole` and `FindKnot`: the first index after `index`, up to
/// `count`, whose point projects within `precision` of `target`; 0 when none does.
///
/// ```text
/// FindPole: target (X / zoom, Y / zoom), precision XPrec / zoom
/// FindKnot: target (X, Y), precision Prec (the native code does not scale them)
/// ```
pub(crate) fn find_index<P: PlotPoint>(
    display: &DrawDisplay<'_>,
    (target, precision): (Point2, f64),
    (index, count): (i32, i32),
    point_at: impl Fn(i32) -> Result<P, Failure>,
) -> Result<i32, Failure> {
    for candidate in index + 1..=count {
        if point_at(candidate)?.project_on(display).distance(target) <= precision {
            return Ok(candidate);
        }
    }
    Ok(0)
}

/// The `FindPole` target and precision: the pick point and tolerance in model units.
pub(crate) fn scaled_target(
    display: &DrawDisplay<'_>,
    (x, y): (f64, f64),
    precision: f64,
) -> (Point2, f64) {
    let zoom = display.zoom();
    (Point2::new(x / zoom, y / zoom), precision / zoom)
}
