// Rust translation of ChFi2d_AnaFilletAlgo::ArcFilletArc (ChFi2d_AnaFilletAlgo.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The analytical fillet of two arcs of circle.
use super::ChFi2dAnaFilletAlgo;
use super::cases::{FilletDef, circle, pnt};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana2d::AnaIntersection;
use rustycad_tkmath::gp::{self, Point2, Vector2};
use std::f64::consts::FRAC_PI_2;

impl ChFi2dAnaFilletAlgo {
    /// `ArcFilletArc(radius, xc, yc, cw, start, end)`: WW5 method, the intersection of
    /// the circles of both arc radii plus or minus the fillet radius nearest to a check
    /// point on the bisectrissa of the tangents at the common point.
    pub(super) fn arc_fillet_arc(&self, radius: f64, def: &mut FilletDef) -> Result<bool, Failure> {
        let (pc1, pc2, p12) = (pnt(self.n1.center), pnt(self.n2.center), pnt(self.n1.p2));
        // Rotate the vectors so that they are tangent to circles at p12.
        let v1 =
            Vector2::between(pc1, p12).rotated(if self.n1.cw { FRAC_PI_2 } else { -FRAC_PI_2 });
        let v2 =
            Vector2::between(pc2, p12).rotated(if self.n2.cw { -FRAC_PI_2 } else { FRAC_PI_2 });
        // Make a "check" point for choosing an offset circle.
        let (v1, v2) = (v1.normalized()?, v2.normalized()?);
        let bisec = (v1 + v2) * 0.5;
        if bisec.square_magnitude() < gp::resolution() {
            return Ok(false);
        }
        let checkp = p12.translated(bisec * radius);
        let c1 = self.offset_arc(pc1, self.n1.radius, radius, checkp)?;
        let c2 = self.offset_arc(pc2, self.n2.radius, radius, checkp)?;
        // The intersection of these two circles closer to the "check" point.
        let inter = AnaIntersection::circles(c1, c2)?;
        if !inter.is_done() || inter.nb_points()? == 0 {
            return Ok(false);
        }
        let pc = nearest_to(&inter, checkp)?;
        def.center = (pc.x(), pc.y());
        // Orientation of fillet.
        let angle = v1.angle(v2)?;
        def.cw = if angle.abs() < precision::angular() {
            Vector2::between(pc, pc1).angle(Vector2::between(pc, pc2))? < 0.0
        } else {
            angle > 0.0
        };
        // Shrinking of circles.
        def.start = Vector2::between(pc1, p12)
            .angle(Vector2::between(pc1, pc))?
            .abs();
        def.end = Vector2::between(pc2, p12)
            .angle(Vector2::between(pc2, pc))?
            .abs();
        Ok(true)
    }

    /// The circle of center `center` and radius `arc_radius + radius`, or
    /// `arc_radius - radius` when the arc is larger and the first one passes far from
    /// the check point.
    fn offset_arc(
        &self,
        center: Point2,
        arc_radius: f64,
        radius: f64,
        checkp: Point2,
    ) -> Result<rustycad_tkmath::gp::Circle2, Failure> {
        let mut c = circle(center, arc_radius + radius)?;
        if arc_radius > radius && c.distance(checkp) > radius {
            c.set_radius(arc_radius - radius)?;
        }
        Ok(c)
    }
}

/// The intersection point nearest to `checkp` (square distances, the first one on ties);
/// the origin when every distance is DBL_MAX or more.
fn nearest_to(inter: &AnaIntersection, checkp: Point2) -> Result<Point2, Failure> {
    let (mut min_dist, mut pc) = (f64::MAX, Point2::default());
    for i in 1..=inter.nb_points()? {
        let p = inter.point(i)?.value();
        let d = checkp.square_distance(p);
        if d < min_dist {
            (min_dist, pc) = (d, p);
        }
    }
    Ok(pc)
}
