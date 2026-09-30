// Rust translation of the B-spline periodicity checks of BRepTools::AddUVBounds(F, E, B)
// (ModelingData/TKBRep/BRepTools/BRepTools.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The U and V branches of `AddUVBounds(F, E, B)`, written once over an [`Axis`].
//!
//! Natively the two branches are copies with U and V swapped; here the coordinate
//! along the checked direction is `along` and the other one `across`, and every
//! arithmetic expression keeps the native operation order.
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomSurface, SurfaceBounds};
use rustycad_tkgeombase::scan_step::advanced_scan_parameter;

/// A parametric direction of a surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Axis {
    /// The U direction (the X coordinate of the p-curve box).
    U,
    /// The V direction (the Y coordinate of the p-curve box).
    V,
}

impl Axis {
    /// `IsUPeriodic()` or `IsVPeriodic()`.
    pub(crate) fn is_periodic(self, surface: &dyn GeomSurface) -> bool {
        match self {
            Axis::U => surface.is_u_periodic(),
            Axis::V => surface.is_v_periodic(),
        }
    }

    fn is_closed(self, surface: &dyn GeomSurface) -> Result<bool, Failure> {
        match self {
            Axis::U => surface.is_u_closed(),
            Axis::V => surface.is_v_closed(),
        }
    }

    /// `(min, max)` along this direction and across it.
    fn split(self, bounds: SurfaceBounds) -> ((f64, f64), (f64, f64)) {
        let (umin, umax, vmin, vmax) = bounds;
        match self {
            Axis::U => ((umin, umax), (vmin, vmax)),
            Axis::V => ((vmin, vmax), (umin, umax)),
        }
    }

    /// The surface point at `along` in this direction and `across` in the other one.
    fn value(
        self,
        surface: &dyn GeomSurface,
        along: f64,
        across: f64,
    ) -> Result<rustycad_tkmath::gp::Point3, Failure> {
        match self {
            Axis::U => surface.value(along, across),
            Axis::V => surface.value(across, along),
        }
    }
}

/// The `aXmin, aYmin, aXmax, aYmax` box of the p-curve of an edge.
#[derive(Clone, Copy, Debug)]
pub(crate) struct EdgeBox {
    pub(crate) xmin: f64,
    pub(crate) ymin: f64,
    pub(crate) xmax: f64,
    pub(crate) ymax: f64,
}

impl EdgeBox {
    pub(crate) fn new(xmin: f64, ymin: f64, xmax: f64, ymax: f64) -> Self {
        Self {
            xmin,
            ymin,
            xmax,
            ymax,
        }
    }

    fn range(&self, axis: Axis) -> (f64, f64) {
        match axis {
            Axis::U => (self.xmin, self.xmax),
            Axis::V => (self.ymin, self.ymax),
        }
    }

    fn set_range(&mut self, axis: Axis, (low, high): (f64, f64)) {
        match axis {
            Axis::U => (self.xmin, self.xmax) = (low, high),
            Axis::V => (self.ymin, self.ymax) = (low, high),
        }
    }

    /// The clamp of a non-periodic direction: the low end moves to the surface's low
    /// bound when that bound is strictly inside the box, then the high end likewise
    /// (tested against the already moved low end, as natively).
    pub(crate) fn clamp(&mut self, axis: Axis, bounds: SurfaceBounds) {
        let ((smin, smax), _) = axis.split(bounds);
        let (mut low, mut high) = self.range(axis);
        if low < smin && smin < high {
            low = smin;
        }
        if low < smax && smax < high {
            high = smax;
        }
        self.set_range(axis, (low, high));
    }
}

/// `100 * Precision::Confusion()^2`, the squared distance under which two points of
/// the periodicity test are the same.
fn periodic_tolerance() -> f64 {
    100.0 * precision::confusion() * precision::confusion()
}

/// The periodicity test of a non-periodic `Geom_BSplineSurface` whose p-curve box leaves
/// its bounds in `axis`: the surface must be closed in `axis` (or repeat at the two
/// ends of the across range) and repeat, one period away, at the 3 or 6 sample
/// parameters outside its bounds. False for any other surface or a box inside the
/// bounds.
pub(crate) fn is_bspline_periodic(
    surface: &Handle<dyn GeomSurface>,
    axis: Axis,
    bounds: SurfaceBounds,
    edge_box: &EdgeBox,
) -> Result<bool, Failure> {
    let Some(stored) = surface.get() else {
        return Ok(false);
    };
    let ((smin, smax), _) = axis.split(bounds);
    let (low, high) = edge_box.range(axis);
    // The native test is on the exact dynamic type.
    let exact = stored.dynamic_type().name() == GeomBSplineSurface::TYPE_NAME;
    if !exact || !(low < smin || high > smax) {
        return Ok(false);
    }
    if !axis.is_closed(stored)? && !ends_repeat(stored, axis, bounds)? {
        return Ok(false);
    }
    repeats_outside(stored, axis, bounds, (low, high))
}

/// Step 1: the points at the two bounds of `axis` coincide at the start and the end of
/// the across range (`for (aV = aVmin; aV <= aVmax; aV += aVStep)`).
///
/// When the increment does not move the across parameter (an empty range, a range of
/// a few ulps, an infinite bound) the native loop never advances; it stops here at
/// that sample, whose result it would repeat forever.
fn ends_repeat(
    surface: &dyn GeomSurface,
    axis: Axis,
    bounds: SurfaceBounds,
) -> Result<bool, Failure> {
    let ((smin, smax), (cmin, cmax)) = axis.split(bounds);
    let step = cmax - cmin;
    let mut across = cmin;
    while across <= cmax {
        let first = axis.value(surface, smin, across)?;
        let second = axis.value(surface, smax, across)?;
        if first.square_distance(&second) > periodic_tolerance() {
            return Ok(false);
        }
        // The native increment leaves `aV` unchanged (an empty range, a sub-ulp step or
        // an infinite bound): the loop would repeat this sample forever.
        let Some(next) = advanced_scan_parameter(across, step) else {
            break;
        };
        across = next;
    }
    Ok(true)
}

/// Step 2: at mid-range across, the points at the parameters outside the bounds (the
/// low end, its midpoint with the bound, the bound; likewise at the high end) match
/// the points one period away.
fn repeats_outside(
    surface: &dyn GeomSurface,
    axis: Axis,
    bounds: SurfaceBounds,
    (low, high): (f64, f64),
) -> Result<bool, Failure> {
    let ((smin, smax), (cmin, cmax)) = axis.split(bounds);
    let across = (cmin + cmax) * 0.5;
    for (outside, shifted) in outside_samples(smin, smax, low, high) {
        let first = axis.value(surface, outside, across)?;
        let second = axis.value(surface, shifted, across)?;
        if first.square_distance(&second) > periodic_tolerance() {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The `aU[]`/`aUpp[]` pairs: each sample outside the bounds and its image one period
/// (`smax - smin`) away, in the native order and operation order.
fn outside_samples(smin: f64, smax: f64, low: f64, high: f64) -> Vec<(f64, f64)> {
    let mut samples = Vec::with_capacity(6);
    if low < smin {
        for sample in [low, (low + smin) * 0.5, smin] {
            samples.push((sample, sample + smax - smin));
        }
    }
    if high > smax {
        for sample in [smax, (high + smax) * 0.5, high] {
            samples.push((sample, sample - smax + smin));
        }
    }
    samples
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkg3d::geom::GeomCylindricalSurface;
    use rustycad_tkmath::gp::{Axis3, Direction3, Point3};

    #[test]
    fn ends_repeat_stops_at_an_infinite_across_bound() {
        // Regression: over V in [0, +inf] the step is +inf, the second sample is at
        // +inf and `aV += aVStep` then leaves it there, so the native loop never ends.
        let z = Direction3::new(0., 0., 1.).unwrap();
        let x = Direction3::new(1., 0., 0.).unwrap();
        let frame = Axis3::new(Point3::new(0., 0., 0.), z, x).unwrap();
        let cylinder = GeomCylindricalSurface::new(frame, 2.).unwrap();
        let bounds: SurfaceBounds = (0., 2. * std::f64::consts::PI, 0., f64::INFINITY);
        assert!(ends_repeat(&cylinder, Axis::U, bounds).unwrap());
    }
}
