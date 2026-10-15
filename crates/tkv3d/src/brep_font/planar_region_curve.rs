//! `BRepFont_PlanarRegion::Curve`: one planar line, quadratic or cubic Bezier segment.
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::{Point2, Xy};
use rustycad_tkmath::math_poly::quadratic;
use rustycad_tkmath::math_utils::Status;
use rustycad_tkservice::font::{FontGlyphOutlineSegment, FontGlyphOutlineSegmentKind as Kind};

/// The power-basis coefficients of a curve (`PowerCoefficients`), lowest degree first.
#[derive(Default)]
struct BRepFontPowerCoefficients {
    x: [f64; 4],
    y: [f64; 4],
    degree: usize,
}

/// `BRepFont_PlanarRegion::Curve`: a planar Bezier segment of a glyph outline.
/// Example: `BRepFontPlanarRegionCurve::new(&FontGlyphOutlineSegment::line(a, b)).value(0.5)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BRepFontPlanarRegionCurve {
    kind: Kind,
    start: Xy,
    control1: Xy,
    control2: Xy,
    end: Xy,
}

impl BRepFontPlanarRegionCurve {
    /// `Curve(theSegment)`: copies the kind and the four points of an outline segment;
    /// `BRepFontPlanarRegionCurve::new(&segment)`.
    pub fn new(segment: &FontGlyphOutlineSegment) -> Self {
        Self {
            kind: segment.kind(),
            start: segment.start(),
            control1: segment.control1(),
            control2: segment.control2(),
            end: segment.end(),
        }
    }
    /// `Type`; `curve.kind()`.
    pub fn kind(&self) -> Kind {
        self.kind
    }
    /// `Start`; `curve.start()`.
    pub fn start(&self) -> &Xy {
        &self.start
    }
    /// `Control1`; `curve.control1()`.
    pub fn control1(&self) -> &Xy {
        &self.control1
    }
    /// `Control2`; `curve.control2()`.
    pub fn control2(&self) -> &Xy {
        &self.control2
    }
    /// `End`; `curve.end()`.
    pub fn end(&self) -> &Xy {
        &self.end
    }
    /// `WithEndpoints`: a copy whose start and end are replaced; `curve.with_endpoints(a, b)`.
    pub fn with_endpoints(&self, start: Xy, end: Xy) -> Self {
        Self {
            start,
            end,
            ..*self
        }
    }

    /// `Value(theParameter)`: the Bernstein form at `t`; `curve.value(0.5)`.
    pub fn value(&self, t: f64) -> Xy {
        let u = 1.0 - t;
        match self.kind {
            Kind::Line => self.start * u + self.end * t,
            Kind::QuadraticBezier => {
                self.start * (u * u) + self.control1 * (2.0 * u * t) + self.end * (t * t)
            }
            Kind::CubicBezier => {
                self.start * (u * u * u)
                    + self.control1 * (3.0 * u * u * t)
                    + self.control2 * (3.0 * u * t * t)
                    + self.end * (t * t * t)
            }
        }
    }

    /// `D1(theParameter)`: the first derivative at `t`; `curve.d1(0.5)`.
    pub fn d1(&self, t: f64) -> Xy {
        let u = 1.0 - t;
        match self.kind {
            Kind::Line => self.end - self.start,
            Kind::QuadraticBezier => {
                (self.control1 - self.start) * (2.0 * u) + (self.end - self.control1) * (2.0 * t)
            }
            Kind::CubicBezier => {
                (self.control1 - self.start) * (3.0 * u * u)
                    + (self.control2 - self.control1) * (6.0 * u * t)
                    + (self.end - self.control2) * (3.0 * t * t)
            }
        }
    }

    /// `SignedArea`: the curve's contribution `1/2 * integral(x dy - y dx)` to a loop area,
    /// computed exactly from the power basis; `curve.signed_area()`.
    pub fn signed_area(&self) -> f64 {
        let coefficients = self.power_coefficients();
        let mut integral = 0.0;
        for i in 0..=coefficients.degree {
            for j in 1..=coefficients.degree {
                integral += (coefficients.x[i] * j as f64 * coefficients.y[j]
                    - coefficients.y[i] * j as f64 * coefficients.x[j])
                    / (i + j) as f64;
            }
        }
        0.5 * integral
    }

    /// `Bounds`: the tight box (endpoints and interior coordinate extrema) widened by a
    /// relative 64-ulp gap; the control hull when the extremum solve fails.
    /// Example: `curve.bounds().get()?`.
    pub fn bounds(&self) -> BoundingBox2 {
        let mut bounds = BoundingBox2::default();
        bounds.add_point(Point2::new(self.start.x(), self.start.y()));
        bounds.add_point(Point2::new(self.end.x(), self.end.y()));
        if self.kind == Kind::Line {
            return bounds;
        }
        let c = self.power_coefficients();
        let has_x = add_coordinate_extrema(&mut bounds, self, c.x[1], c.x[2], c.x[3]);
        let has_y = add_coordinate_extrema(&mut bounds, self, c.y[1], c.y[2], c.y[3]);
        if !has_x || !has_y {
            // Preserve the conservative control-hull bound if polynomial solving fails.
            self.add_control_hull(&mut bounds);
            return bounds;
        }
        bounds.set_gap(64.0 * f64::EPSILON * self.coordinate_scale());
        bounds
    }

    fn add_control_hull(&self, bounds: &mut BoundingBox2) {
        bounds.add_point(Point2::new(self.control1.x(), self.control1.y()));
        if self.kind == Kind::CubicBezier {
            bounds.add_point(Point2::new(self.control2.x(), self.control2.y()));
        }
    }

    // std::max({1.0, |each coordinate of the four points|}).
    fn coordinate_scale(&self) -> f64 {
        let points = [self.start, self.control1, self.control2, self.end];
        points
            .iter()
            .flat_map(|point| [point.x().abs(), point.y().abs()])
            .fold(1.0, f64::max)
    }

    fn power_coefficients(&self) -> BRepFontPowerCoefficients {
        let (p0, p1, p2, p3) = (self.start, self.control1, self.control2, self.end);
        let mut result = BRepFontPowerCoefficients::default();
        result.x[0] = p0.x();
        result.y[0] = p0.y();
        match self.kind {
            Kind::Line => {
                result.degree = 1;
                result.x[1] = p3.x() - p0.x();
                result.y[1] = p3.y() - p0.y();
            }
            Kind::QuadraticBezier => {
                result.degree = 2;
                result.x[1] = 2.0 * (p1.x() - p0.x());
                result.y[1] = 2.0 * (p1.y() - p0.y());
                result.x[2] = p0.x() - 2.0 * p1.x() + p3.x();
                result.y[2] = p0.y() - 2.0 * p1.y() + p3.y();
            }
            Kind::CubicBezier => cubic_coefficients(&mut result, [p0, p1, p2, p3]),
        }
        result
    }
}

fn cubic_coefficients(result: &mut BRepFontPowerCoefficients, [p0, p1, p2, p3]: [Xy; 4]) {
    result.degree = 3;
    result.x[1] = 3.0 * (p1.x() - p0.x());
    result.y[1] = 3.0 * (p1.y() - p0.y());
    result.x[2] = 3.0 * (p0.x() - 2.0 * p1.x() + p2.x());
    result.y[2] = 3.0 * (p0.y() - 2.0 * p1.y() + p2.y());
    result.x[3] = -p0.x() + 3.0 * p1.x() - 3.0 * p2.x() + p3.x();
    result.y[3] = -p0.y() + 3.0 * p1.y() - 3.0 * p2.y() + p3.y();
}

// Adds the curve points at the interior roots of one coordinate derivative
// (`addCoordinateExtrema`); false when MathPoly::Quadratic fails.
fn add_coordinate_extrema(
    bounds: &mut BoundingBox2,
    curve: &BRepFontPlanarRegionCurve,
    linear: f64,
    quadratic_term: f64,
    cubic_term: f64,
) -> bool {
    let roots = quadratic(3.0 * cubic_term, 2.0 * quadratic_term, linear);
    if !matches!(
        roots.status,
        Status::Ok | Status::NoSolution | Status::InfiniteSolutions
    ) {
        return false;
    }
    for &parameter in &roots.roots[..roots.nb_roots] {
        if parameter > 0.0 && parameter < 1.0 {
            let point = curve.value(parameter);
            bounds.add_point(Point2::new(point.x(), point.y()));
        }
    }
    true
}

#[cfg(test)]
#[path = "planar_region_curve_tests.rs"]
mod tests;
