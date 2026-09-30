// Rust translation of ModelingData/TKGeomBase/Geom2dConvert/Geom2dConvert_ApproxArcsSegments.cxx
// (makeCircle, makeArc, makeLine, checkCurve) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The elementary builders: the arc through three curve points, the arc from a
//! tangent, the segment, and the deflection check of a built element.
use super::{ApproxArcsSegments, MAXPOINTS, seq_at};
use crate::gc::{MakeArcOfCircle2d, MakeSegment2d, Root};
use crate::geom2d_convert::ppoint::PPoint;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::StandardDirection2;
use rustycad_tkmath::gp::{self, Axis22d, Circle2, Direction2, Line2, Point2, Xy};

// The geometry checkCurve projects onto: the basis line or circle of the element.
enum ElementGeometry {
    Line(Line2),
    Circle(Circle2),
}

impl ApproxArcsSegments<'_> {
    /// `makeCircle`: the arc through the two points and the curve point at the middle
    /// parameter (the curve is a circle); null when GC_MakeArcOfCircle2d fails.
    pub(super) fn make_circle(
        &self,
        first: &PPoint,
        last: &PPoint,
    ) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let para_m: f64 = (first.parameter() + last.parameter()) * 0.5;
        let point_m: Point2 = self.my_curve.d0(para_m)?;
        let make_arc = MakeArcOfCircle2d::through_points(
            to_point(first.point()),
            point_m,
            to_point(last.point()),
        )?;
        if make_arc.is_done() {
            return Ok(make_arc.value()?.cast());
        }
        Ok(Handle::default())
    }

    /// `makeArc`: the arc from `param1` to `param2` tangent to the derivative at
    /// `param1` (reversed unless `is_first`), or None when the tangent is along the
    /// chord or normal to it. On success the derivative of `param2` becomes the arc's
    /// tangent there.
    pub(super) fn make_arc(
        &self,
        param1: &PPoint,
        param2: &mut PPoint,
        is_first: bool,
    ) -> Result<Option<Handle<Geom2dTrimmedCurve>>, Failure> {
        let (p1, p2): (Xy, Xy) = (param1.point(), param2.point());
        let vec: Xy = if is_first { param1.d1() } else { -param1.d1() };
        // Detect the sense (CCW means positive).
        let delta: Xy = p2 - p1;
        let mut sense: f64 = vec ^ delta;
        if sense > precision::angular() {
            sense = 1.0;
        } else if sense < -precision::angular() {
            sense = -1.0;
        } else {
            return Ok(None);
        }
        // Find the centre of the circle.
        let middle: Xy = (p2 + p1) * 0.5;
        let prod_p1v: f64 = p1 * vec;
        let prod_dem: f64 = delta * middle;
        let vprod_vd: f64 = vec ^ delta;
        let resolution: f64 = gp::resolution();
        if !(vprod_vd < -resolution || vprod_vd > resolution) {
            // The tangent is normal to the chord.
            return Ok(None);
        }
        let center = Point2::new(
            (prod_p1v * delta.y() - prod_dem * vec.y()) / vprod_vd,
            (prod_dem * vec.x() - prod_p1v * delta.x()) / vprod_vd,
        );
        let arc = ArcSpec {
            center,
            p1,
            p2,
            is_first,
            sense,
        };
        arc.build(param2)
    }

    /// `makeLine`: the segment between the two points, checked against the curve;
    /// with `is_check`, also refused when a derivative is null or an inner end turns
    /// by more than max(angle tolerance, 0.01). On success both derivatives become
    /// the chord.
    pub(super) fn make_line(
        &self,
        first: &mut PPoint,
        last: &mut PPoint,
        is_check: bool,
    ) -> Result<Handle<Geom2dTrimmedCurve>, Failure> {
        let slope: Xy = last.point() - first.point();
        if slope.square_modulus().abs() < gp::resolution() {
            return Ok(Handle::default());
        }
        let dir_line: Direction2 = Direction2::from_xy(slope)?;
        if is_check && !self.line_ends_agree(first, last, dir_line)? {
            return Ok(Handle::default());
        }
        // Building the segment of line.
        let make_seg = MakeSegment2d::from_points(to_point(first.point()), to_point(last.point()))?;
        if !make_seg.is_done() {
            return Ok(Handle::default());
        }
        let curve: Handle<Geom2dTrimmedCurve> = make_seg.value()?;
        if !self.check_curve(&curve.cast(), first.parameter(), last.parameter())? {
            return Ok(Handle::default());
        }
        // Correct the derivatives in both arguments.
        let new_d1: Xy = last.point() - first.point();
        first.set_d1(new_d1);
        last.set_d1(new_d1);
        Ok(curve)
    }

    // The isCheck tests of makeLine: non-null derivatives, and angular continuity (G1)
    // checked only where the line does not end at an extremity of the curve.
    fn line_ends_agree(
        &self,
        first: &PPoint,
        last: &PPoint,
        dir_line: Direction2,
    ) -> Result<bool, Failure> {
        if first.d1().square_modulus() < gp::resolution()
            || last.d1().square_modulus() < gp::resolution()
        {
            return Ok(false);
        }
        let mut abs_angle: [f64; 2] = [0.0, 0.0];
        if *first != self.my_ext[0] {
            let ang: f64 = dir_line.angle(Direction2::from_xy(first.d1())?);
            abs_angle[0] = if ang > 0.0 { ang } else { -ang };
        }
        if *last != self.my_ext[1] {
            let ang: f64 = dir_line.angle(Direction2::from_xy(last.d1())?);
            abs_angle[1] = if ang > 0.0 { ang } else { -ang };
        }
        // Derivatives farther from the line than the continuity tolerance ask for a
        // biarc instead of a line.
        let cont_tolerance: f64 = std_max(self.my_angle_tolerance, 0.01);
        Ok(!(abs_angle[0] > cont_tolerance || abs_angle[1] > cont_tolerance))
    }

    /// `checkCurve`: whether every sample of the curve between the two parameters
    /// (the stored scan parameters, else MAXPOINTS steps) projects on the basis line
    /// or circle of `curve` within the tolerance; false for a null or other curve.
    pub(super) fn check_curve(
        &self,
        curve: &Handle<dyn Geom2dCurve>,
        first_param: f64,
        last_param: f64,
    ) -> Result<bool, Failure> {
        let Some(geometry) = element_geometry(curve)? else {
            return Ok(false);
        };
        let is_uniform_done: bool = !self.my_seq_params.is_empty();
        let nb_pnts: i32 = if is_uniform_done {
            self.my_seq_params.len() as i32
        } else {
            MAXPOINTS
        };
        let param_step: f64 = (last_param - first_param) / f64::from(MAXPOINTS);
        let mut is_less: bool = true;
        let mut i: i32 = 1;
        while i <= nb_pnts && is_less {
            let param: f64 = if is_uniform_done {
                seq_at(&self.my_seq_params, i).parameter()
            } else {
                first_param + f64::from(i) * param_step
            };
            i += 1;
            if param < (first_param - precision::p_confusion())
                || param > (last_param + precision::p_confusion())
            {
                continue;
            }
            is_less = self.projects_within(&geometry, param)?;
        }
        Ok(is_less)
    }

    // The distance from the curve point at `param` to its projection on the element.
    fn projects_within(&self, geometry: &ElementGeometry, param: f64) -> Result<bool, Failure> {
        let point_adaptor: Point2 = self.my_curve.d0(param)?;
        let proj_point: Point2 = match geometry {
            ElementGeometry::Line(line) => {
                elclib::value(elclib::parameter(line, point_adaptor)?, line)
            }
            ElementGeometry::Circle(circle) => {
                elclib::value(elclib::parameter(circle, point_adaptor)?, circle)
            }
        };
        Ok(proj_point.distance(point_adaptor) < self.my_tolerance + precision::p_confusion())
    }
}

// The basis line or circle of a (trimmed) element, None for a null or other curve.
fn element_geometry(curve: &Handle<dyn Geom2dCurve>) -> Result<Option<ElementGeometry>, Failure> {
    if curve.is_null() {
        // The check fails on an empty input.
        return Ok(None);
    }
    let basis: Handle<dyn Geom2dCurve> = match curve.cast::<Geom2dTrimmedCurve>().get() {
        Some(trimmed) => trimmed.basis_curve(),
        None => curve.clone(),
    };
    if let Some(line) = basis.cast::<Geom2dLine>().get() {
        return Ok(Some(ElementGeometry::Line(line.lin2d())));
    }
    match basis.cast::<Geom2dCircle>().get() {
        Some(circle) => Ok(Some(ElementGeometry::Circle(circle.circ2d()?))),
        None => Ok(None),
    }
}

// The arc of makeArc once its centre is known.
struct ArcSpec {
    center: Point2,
    p1: Xy,
    p2: Xy,
    is_first: bool,
    sense: f64,
}

impl ArcSpec {
    fn build(mut self, param2: &mut PPoint) -> Result<Option<Handle<Geom2dTrimmedCurve>>, Failure> {
        let rad: f64 = (self.center.distance(to_point(self.p1))
            + self.center.distance(to_point(self.p2)))
            * 0.5;
        let x = Direction2::from_standard(StandardDirection2::X);
        let y = Direction2::from_standard(StandardDirection2::Y);
        let cir = Circle2::new(Axis22d::with_directions(self.center, x, y)?, rad)?;
        let circ = Geom2dCircle::from_circ2d(cir);
        // The parameters of the first and last points of the arc.
        let (mut alpha1, mut alpha2): (f64, f64) = if self.is_first {
            (
                elclib::parameter(&cir, to_point(self.p1))?,
                elclib::parameter(&cir, to_point(self.p2))?,
            )
        } else {
            self.sense = -self.sense;
            (
                elclib::parameter(&cir, to_point(self.p2))?,
                elclib::parameter(&cir, to_point(self.p1))?,
            )
        };
        if (alpha1 - alpha2).abs() < 1e-100 {
            // A very small value, just to avoid an exact match.
            return Ok(None);
        }
        // Reverse the circle if the sense is negative.
        if self.sense < 0.0 {
            alpha1 = circ.reversed_parameter(alpha1)?;
            alpha2 = circ.reversed_parameter(alpha2)?;
            circ.reverse()?;
        }
        let basis: Handle<dyn Geom2dCurve> = Handle::new(circ).cast();
        let curve = Geom2dTrimmedCurve::new(&basis, alpha1, alpha2, true, true)?;
        // Correct the direction in the opposite point.
        let rad_v: Xy = param2.point() - self.center.xy();
        param2.set_d1(Xy::new(-rad_v.y() * self.sense, rad_v.x() * self.sense));
        Ok(Some(Handle::new(curve)))
    }
}

/// `gp_Pnt2d(const gp_XY&)`.
pub(super) fn to_point(xy: Xy) -> Point2 {
    Point2::new(xy.x(), xy.y())
}
