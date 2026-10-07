// Rust translation of ChFi2d_AnaFilletAlgo::SegmentFilletSegment, SegmentFilletArc and
// ArcFilletSegment (ChFi2d_AnaFilletAlgo.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The cases of the analytical fillet with a segment: its center, its direction, the
//! shrinking of both neighbours, and the end point on an arc neighbour.
use super::{ChFi2dAnaFilletAlgo, Neighbour};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana2d::AnaIntersection;
use rustycad_tkmath::gp::{self, Axis2d, Circle2, Direction2, Line2, Point2, Vector2};
use std::f64::consts::FRAC_PI_2;

/// A fillet definition: center `(xc, yc)`, clockwise or not, the shrinking parameters
/// on both neighbours (`start`, `end`), and the points on arc neighbours (`xstart,
/// ystart`, `xend, yend`, DBL_MAX when not computed).
#[derive(Clone, Copy, Debug)]
pub(super) struct FilletDef {
    pub(super) center: (f64, f64),
    pub(super) cw: bool,
    pub(super) start: f64,
    pub(super) end: f64,
    pub(super) start_point: (f64, f64),
    pub(super) end_point: (f64, f64),
}

impl Default for FilletDef {
    fn default() -> Self {
        Self {
            center: (0.0, 0.0),
            cw: false,
            start: 0.0,
            end: 0.0,
            start_point: (f64::MAX, f64::MAX),
            end_point: (f64::MAX, f64::MAX),
        }
    }
}

/// The fillet between a segment and an arc (either order): the shrink of
/// the segment, the angle of the arc shrink, the end point on the arc, and the signed
/// angle from the segment to the arc tangent.
struct SegmentArc {
    segment_shrink: f64,
    arc_shrink: f64,
    arc_point: (f64, f64),
    angle: f64,
}

pub(super) fn pnt((x, y): (f64, f64)) -> Point2 {
    Point2::new(x, y)
}

/// `IsEqual(p1, p2)` in the plane.
fn is_equal(p1: Point2, p2: Point2) -> bool {
    p1.square_distance(p2) < precision::square_confusion()
}

/// The circle of center `center` and radius `radius` (X axis as reference).
pub(super) fn circle(center: Point2, radius: f64) -> Result<Circle2, Failure> {
    Circle2::from_x_axis(
        Axis2d::new(center, Direction2::new(1.0, 0.0)?),
        radius,
        true,
    )
}

impl ChFi2dAnaFilletAlgo {
    /// `SegmentFilletSegment(radius, xc, yc, cw, start, end)`: WW5 method, along the
    /// bisectrissa of the segments at their common point.
    pub(super) fn segment_fillet_segment(
        &self,
        radius: f64,
        def: &mut FilletDef,
    ) -> Result<bool, Failure> {
        let (p11, p12, p22) = (pnt(self.n1.p1), pnt(self.n1.p2), pnt(self.n2.p2));
        // Check length of segments.
        if is_equal(p12, p11) || is_equal(p12, p22) {
            return Ok(false);
        }
        let v1 = Vector2::between(p12, p11).normalized()?;
        let v2 = Vector2::between(p12, p22).normalized()?;
        // Make bisectrissa.
        let bisec = (v1 + v2) * 0.5;
        if bisec.square_magnitude() < precision::square_confusion() {
            return Ok(false);
        }
        let bisec = bisec.normalized()?;
        // Angle at bisectrissa, length along it till the center of fillet.
        let beta = v1.angle(bisec)?;
        let l = radius / beta.abs().sin();
        let pc = p12.translated(bisec * l);
        def.center = (pc.x(), pc.y());
        // Shrinking length along segments.
        def.start = (l * l - radius * radius).sqrt();
        def.end = def.start;
        def.cw = beta > 0.0;
        Ok(true)
    }

    /// `SegmentFilletArc(radius, xc, yc, cw, start, end, xend, yend)`.
    pub(super) fn segment_fillet_arc(
        &self,
        radius: f64,
        def: &mut FilletDef,
    ) -> Result<bool, Failure> {
        let (p12, p11, pc2) = (pnt(self.n1.p2), pnt(self.n1.p1), pnt(self.n2.center));
        if p11.square_distance(p12) < gp::resolution() {
            return Ok(false);
        }
        // Rotate the arc vector to become tangential at p21.
        let turn = if self.n2.cw { FRAC_PI_2 } else { -FRAC_PI_2 };
        let tangent = Vector2::between(p12, pc2).rotated(turn);
        let segment = (p11, Vector2::between(p12, p11));
        let tol = precision::confusion();
        let Some(r) = segment_arc(&self.n2, p12, segment, tangent, (radius, tol), def)? else {
            return Ok(false);
        };
        (def.start, def.end, def.end_point) = (r.segment_shrink, r.arc_shrink, r.arc_point);
        def.cw = r.angle > 0.0;
        Ok(true)
    }

    /// `ArcFilletSegment(radius, xc, yc, cw, start, end, xstart, ystart)`.
    pub(super) fn arc_fillet_segment(
        &self,
        radius: f64,
        def: &mut FilletDef,
    ) -> Result<bool, Failure> {
        let (p12, p22, pc1) = (pnt(self.n1.p2), pnt(self.n2.p2), pnt(self.n1.center));
        if p12.square_distance(p22) < gp::resolution() {
            return Ok(false);
        }
        let turn = if self.n1.cw { -FRAC_PI_2 } else { FRAC_PI_2 };
        let tangent = Vector2::between(p12, pc1).rotated(turn);
        let segment = (p22, Vector2::between(p12, p22));
        let tol = precision::square_confusion();
        let Some(r) = segment_arc(&self.n1, p12, segment, tangent, (radius, tol), def)? else {
            return Ok(false);
        };
        (def.end, def.start, def.start_point) = (r.segment_shrink, r.arc_shrink, r.arc_point);
        // Orientation of the fillet: the segment vector to the arc tangent.
        def.cw = r.angle < 0.0;
        Ok(true)
    }
}

/// The shared text of SegmentFilletArc and ArcFilletSegment: a line parallel to the
/// segment (`(end, vector)` from the common point `p12`) at the side of the center of
/// the fillet, intersected with the circle of the arc radius plus or minus the fillet
/// radius; the center is the intersection nearest to the bisectrissa. The center is
/// written in `def` as soon as it is found. `tol` compares the end point on the arc.
fn segment_arc(
    arc: &Neighbour,
    p12: Point2,
    (end, segment): (Point2, Vector2),
    tangent: Vector2,
    (radius, tol): (f64, f64),
    def: &mut FilletDef,
) -> Result<Option<SegmentArc>, Failure> {
    // If vectors coincide (segment and arc are tangent), the algorithm doesn't work...
    if segment.angle(tangent)?.abs() < precision::angular() {
        return Ok(None);
    }
    let (seg_u, tan_u) = (segment.normalized()?, tangent.normalized()?);
    // If segment and arc look in opposite direction, no fillet is possible.
    let bisec = (seg_u + tan_u) * 0.5;
    if bisec.square_magnitude() < gp::resolution() {
        return Ok(None);
    }
    let bisec = bisec.normalized()?;
    let near = p12.translated(bisec * radius);
    let near_line = Line2::new(p12, Direction2::from_vector(bisec)?);
    let line = parallel_line(end, seg_u, radius, near)?;
    let circ = offset_circle(arc, radius, near)?;
    let Some(pc) = nearest_intersection(&AnaIntersection::line_circle(line, circ)?, &near_line)?
    else {
        return Ok(None);
    };
    def.center = (pc.x(), pc.y());
    let segment_shrink = (pc.square_distance(p12) - radius * radius).sqrt();
    let pcc = Vector2::between(pnt(arc.center), pc);
    let arc_shrink = Vector2::between(pnt(arc.center), p12).angle(pcc)?.abs();
    let Some(arc_point) = arc_end_point(arc, pc, pcc, (radius, tol))? else {
        return Ok(None);
    };
    let angle = seg_u.angle(tan_u)?;
    Ok(Some(SegmentArc {
        segment_shrink,
        arc_shrink,
        arc_point,
        angle,
    }))
}

/// A line parallel to the segment of unit direction `d` through its end `end`, moved by
/// the fillet radius to the side of the point `near`.
fn parallel_line(end: Point2, d: Vector2, radius: f64, near: Point2) -> Result<Line2, Failure> {
    let mut line = Line2::new(end, Direction2::from_vector(-d)?);
    let normal = d.rotated(FRAC_PI_2);
    line.translate(normal * radius);
    if line.distance(near) > radius {
        line.translate(normal * (-2.0 * radius));
    }
    Ok(line)
}

/// The circle of the arc radius plus the fillet radius, or minus it when the arc is
/// larger and the first one passes far from `near`.
fn offset_circle(arc: &Neighbour, radius: f64, near: Point2) -> Result<Circle2, Failure> {
    let mut circ = circle(pnt(arc.center), arc.radius + radius)?;
    if arc.radius > radius && circ.distance(near) > radius {
        circ.set_radius(arc.radius - radius)?;
    }
    Ok(circ)
}

/// The intersection point nearest to the line `near` (the first one on ties); `None`
/// when the intersection failed or is empty.
pub(super) fn nearest_intersection(
    inter: &AnaIntersection,
    near: &Line2,
) -> Result<Option<Point2>, Failure> {
    if !inter.is_done() || inter.nb_points()? == 0 {
        return Ok(None);
    }
    let (mut min_dist, mut best) = (f64::MAX, None);
    for i in 1..=inter.nb_points()? {
        let p = inter.point(i)?.value();
        let d = near.distance(p);
        if d < min_dist {
            (min_dist, best) = (d, Some(p));
        }
    }
    Ok(best)
}

/// Duplicate the information on the shrink of the arc: its point on the ray from its
/// center towards the fillet center `pc`, the first intersection whose square distance
/// to `pc` is the square fillet radius within `tol` (DBL_MAX when none); `None` when the
/// intersection fails or is empty.
fn arc_end_point(
    arc: &Neighbour,
    pc: Point2,
    pcc: Vector2,
    (radius, tol): (f64, f64),
) -> Result<Option<(f64, f64)>, Failure> {
    let line = Line2::new(pnt(arc.center), Direction2::from_vector(pcc)?);
    let inter = AnaIntersection::line_circle(line, circle(pnt(arc.center), arc.radius)?)?;
    if !inter.is_done() || inter.nb_points()? == 0 {
        return Ok(None);
    }
    let rf2 = radius * radius;
    for i in 1..=inter.nb_points()? {
        let p = inter.point(i)?.value();
        if (p.square_distance(pc) - rf2).abs() < tol {
            return Ok(Some((p.x(), p.y())));
        }
    }
    Ok(Some((f64::MAX, f64::MAX)))
}
