// Rust translation of the drawing steps that DrawTrSurf_Curve.cxx and
// DrawTrSurf_Curve2d.cxx repeat for 3D and 2D curves, at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The steps that `DrawTrSurf_Curve::DrawOn` and `DrawTrSurf_Curve2d::DrawOn` share:
//! the finite range of an infinite curve, the orientation arrow and the curvature
//! radii.
use super::drawable::bound;
use super::style::CurveStyle;
use crate::draw::{DrawDisplay, DrawMarkerShape};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::geom_lprop::{CLProps, CLProps2d};
use rustycad_tkmath::gp::{self, Direction2, Point2, Point3, Vector2, Vector3};

/// `DrawTrSurf_CurveLimit`: an infinite curve is drawn until its ends are this far
/// apart.
pub(crate) const CURVE_LIMIT: f64 = 400.0;

/// `Resolution` of the curvature display: smaller curvatures are not drawn.
const CURVATURE_RESOLUTION: f64 = 1.0e-9;

/// A point of the plane or of space as the curve drawings use it.
pub(crate) trait PlotPoint: Copy {
    /// `Distance`.
    fn distance_to(self, other: Self) -> f64;
    /// `Draw_Display::MoveTo`.
    fn move_on(self, display: &mut DrawDisplay<'_>);
    /// `Draw_Display::DrawTo`.
    fn draw_on(self, display: &mut DrawDisplay<'_>);
    /// `P1.Translated(ratio * gp_Vec(P1, P2))`.
    fn toward(self, target: Self, ratio: f64) -> Self;
    /// `Draw_Display::Project`; a 2D point is projected as `(x, y, 0)`.
    fn project_on(self, display: &DrawDisplay<'_>) -> Point2;
    /// `Draw_Display::DrawMarker(P, shape, size)` with a pixel size.
    fn mark_on(self, display: &mut DrawDisplay<'_>, shape: DrawMarkerShape, size: i32);
}

impl PlotPoint for Point3 {
    fn distance_to(self, other: Self) -> f64 {
        self.distance(&other)
    }
    fn move_on(self, display: &mut DrawDisplay<'_>) {
        display.move_to_3d(self);
    }
    fn draw_on(self, display: &mut DrawDisplay<'_>) {
        display.draw_to_3d(self);
    }
    fn toward(self, target: Self, ratio: f64) -> Self {
        self.translated(Vector3::between(self, target) * ratio)
    }
    fn project_on(self, display: &DrawDisplay<'_>) -> Point2 {
        display.project(self)
    }
    fn mark_on(self, display: &mut DrawDisplay<'_>, shape: DrawMarkerShape, size: i32) {
        display.draw_marker_3d(self, shape, size);
    }
}

impl PlotPoint for Point2 {
    fn distance_to(self, other: Self) -> f64 {
        self.distance(other)
    }
    fn move_on(self, display: &mut DrawDisplay<'_>) {
        display.move_to(self);
    }
    fn draw_on(self, display: &mut DrawDisplay<'_>) {
        display.draw_to(self);
    }
    fn toward(self, target: Self, ratio: f64) -> Self {
        self.translated(Vector2::between(self, target) * ratio)
    }
    fn project_on(self, display: &DrawDisplay<'_>) -> Point2 {
        display.project(Point3::new(self.x(), self.y(), 0.0))
    }
    fn mark_on(self, display: &mut DrawDisplay<'_>, shape: DrawMarkerShape, size: i32) {
        display.draw_marker(self, shape, size);
    }
}

/// The parameter range drawn for a curve on `[first, last]`: an infinite end is
/// replaced by doubling a range around the finite end (or around 0) until the end
/// points are [`CURVE_LIMIT`] apart.
///
/// Native quirk kept: a curve whose points never get that far apart loops forever.
/// A NaN distance ends the loop, as the C++ `while (d < limit)` does.
pub(crate) fn finite_range<P: PlotPoint>(
    (mut first, mut last): (f64, f64),
    d0: impl Fn(f64) -> Result<P, Failure>,
) -> Result<(f64, f64), Failure> {
    let first_inf = precision::is_negative_infinite(first);
    let last_inf = precision::is_positive_infinite(last);
    let mut delta = 1.0;
    if first_inf && last_inf {
        loop {
            delta *= 2.0;
            (first, last) = (-delta, delta);
            if reached_limit(d0(first)?.distance_to(d0(last)?)) {
                break;
            }
        }
    } else if first_inf {
        let end = d0(last)?;
        loop {
            delta *= 2.0;
            first = last - delta;
            if reached_limit(d0(first)?.distance_to(end)) {
                break;
            }
        }
    } else if last_inf {
        let start = d0(first)?;
        loop {
            delta *= 2.0;
            last = first + delta;
            if reached_limit(start.distance_to(d0(last)?)) {
                break;
            }
        }
    }
    Ok((first, last))
}

/// The end test of the range doubling: `!(distance < CURVE_LIMIT)`, true for NaN.
fn reached_limit(distance: f64) -> bool {
    distance >= CURVE_LIMIT || distance.is_nan()
}

/// The orientation arrow at `tip` along `direction` (screen-sized, 20 by 10 pixels),
/// drawn without growing the bounds (`Draw_Bounds = false`); nothing when
/// `direction` is shorter than `gp::Resolution()`.
pub(crate) fn draw_orientation_arrow(
    display: &mut DrawDisplay<'_>,
    tip: Point2,
    direction: Vector2,
) -> Result<(), Failure> {
    display.set_draw_bounds(false);
    let drawn = draw_arrow(display, tip, direction);
    display.set_draw_bounds(true);
    drawn
}

/// The arrow of [`draw_orientation_arrow`] with the bounds left as they are (the
/// DBRep edge arrows).
pub(crate) fn draw_arrow(
    display: &mut DrawDisplay<'_>,
    tip: Point2,
    direction: Vector2,
) -> Result<(), Failure> {
    // `if (v.Magnitude() > gp::Resolution())`: nothing for NaN either.
    let long_enough = direction.magnitude() > gp::resolution();
    if !long_enough {
        return Ok(());
    }
    let length = 20.0 / display.zoom();
    let half_width = 10.0 / display.zoom();
    let d = Direction2::from_vector(direction)?;
    let (x, y) = (tip.x(), tip.y());
    display.move_to(Point2::new(
        x - length * d.x() - half_width * d.y(),
        y - length * d.y() + half_width * d.x(),
    ));
    display.draw_to(tip);
    display.draw_to(Point2::new(
        x - length * d.x() + half_width * d.y(),
        y - length * d.y() - half_width * d.x(),
    ));
    Ok(())
}

/// The local properties a curvature display reads at one parameter.
pub(crate) trait CurvatureProps {
    /// `gp_Pnt` or `gp_Pnt2d`.
    type Point: PlotPoint;
    /// `SetParameter(t)` then `IsTangentDefined()` and `|Curvature()|`; `None` when
    /// the tangent is not defined.
    fn curvature_at(&mut self, t: f64) -> Result<Option<f64>, Failure>;
    /// `CentreOfCurvature` at the last parameter.
    fn centre(&mut self) -> Result<Self::Point, Failure>;
}

macro_rules! curvature_props {
    ($props:ty, $point:ty) => {
        impl CurvatureProps for $props {
            type Point = $point;
            fn curvature_at(&mut self, t: f64) -> Result<Option<f64>, Failure> {
                self.set_parameter(t)?;
                if !self.is_tangent_defined()? {
                    return Ok(None);
                }
                Ok(Some(self.curvature()?.abs()))
            }
            fn centre(&mut self) -> Result<$point, Failure> {
                self.centre_of_curvature()
            }
        }
    };
}
curvature_props!(CLProps, Point3);
curvature_props!(CLProps2d, Point2);

/// What the curvature display of one curve draws with.
pub(crate) struct CurvatureDisplay {
    /// The number of samples per CN interval (`GetDiscretisation`).
    samples: i32,
    /// `radiusmax`: longer radii are scaled down to this length.
    radius_max: f64,
    /// `radiusratio`: the drawn part of each radius.
    radius_ratio: f64,
}

impl CurvatureDisplay {
    /// The display of a curve with `samples` points per interval and `style`.
    pub(crate) fn of(samples: i32, style: &CurveStyle) -> Self {
        Self {
            samples,
            radius_max: style.radius_max(),
            radius_ratio: style.radius_ratio(),
        }
    }

    /// The `GeomLProp_CLProps(curv, 2, 1e-9)` resolution, below which a curvature is
    /// not drawn.
    pub(crate) const fn resolution(&self) -> f64 {
        CURVATURE_RESOLUTION
    }

    /// The curvature radii: at `samples` parameters of each CN interval, a segment from
    /// the curve point toward the centre of curvature, of `radius_ratio` times the
    /// radius (capped at `radius_max`).
    pub(crate) fn draw<P: CurvatureProps>(
        &self,
        display: &mut DrawDisplay<'_>,
        bounds: &Array1<'static, f64>,
        props: &mut P,
        d0: impl Fn(f64) -> Result<P::Point, Failure>,
    ) -> Result<(), Failure> {
        for interval in 1..bounds.len() as i32 {
            let mut t = bound(bounds, interval)?;
            let step = (bound(bounds, interval + 1)? - t) / f64::from(self.samples);
            for _ in 1..=self.samples {
                self.draw_radius(display, props, &d0, t)?;
                t += step;
            }
        }
        Ok(())
    }

    /// One radius at `t`, when the tangent is defined and the curvature is not null.
    fn draw_radius<P: CurvatureProps>(
        &self,
        display: &mut DrawDisplay<'_>,
        props: &mut P,
        d0: &impl Fn(f64) -> Result<P::Point, Failure>,
        t: f64,
    ) -> Result<(), Failure> {
        let Some(curvature) = props.curvature_at(t)? else {
            return Ok(());
        };
        if curvature <= CURVATURE_RESOLUTION {
            return Ok(());
        }
        let point = d0(t)?;
        point.move_on(display);
        let radius = 1. / curvature;
        let mut ratio = if radius > self.radius_max {
            self.radius_max / radius
        } else {
            1.0
        };
        ratio *= self.radius_ratio;
        point.toward(props.centre()?, ratio).draw_on(display);
        Ok(())
    }
}
