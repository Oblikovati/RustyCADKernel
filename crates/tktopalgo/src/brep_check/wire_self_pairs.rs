// Rust translation of the pair checks of BRepCheck_Wire::SelfIntersect (ModelingAlgorithms/TKTopAlgo/BRepCheck/BRepCheck_Wire.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The intersections of two edges of a wire in `SelfIntersect`: the inner points must
//! lie at a common vertex (or along the chord to the nearest one, skv OCC1764), and a
//! common segment must have an end at a common vertex without two coincident lines.
use super::wire_self_intersect::{SelfIntersect, point_on_edge};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, ShapeIterator, Vertex};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::{is_equal, real_last};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkgeomalgo::int_res2d::{IntersectionPoint, Position};
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{self, Direction3, Line3, Point3, Vector3};

/// `aPAR_T`: the fraction of a common segment tested for coincident lines.
const PAR_T: f64 = 0.43213918;

/// The two 3d points of one intersection point, on each edge.
struct PointPair {
    first: Point3,
    second: Point3,
}

/// The vertices common to the two edges (children of the second one found among the
/// children of the first one).
fn common_vertices(e1: &Edge, e2: &Edge) -> Result<Vec<Vertex>, Failure> {
    let mut vmap = MapOfShape::default();
    let mut iterator = ShapeIterator::new(e1, true, true);
    while let Ok(child) = iterator.value() {
        vmap.add(child.clone());
        iterator.next();
    }
    let mut common = Vec::new();
    iterator = ShapeIterator::new(e2, true, true);
    while let Ok(child) = iterator.value() {
        if vmap.contains(child) {
            common.push(Vertex::unchecked(child.clone()));
        }
        iterator.next();
    }
    Ok(common)
}

/// Whether both points are within 1.1 times the tolerance of one common vertex.
fn at_common_vertex(pair: &PointPair, common: &[Vertex]) -> Result<bool, Failure> {
    for vertex in common {
        let point = BRepTool::pnt(vertex)?;
        let tol = 1.1 * BRepTool::vertex_tolerance(vertex)?;
        let tol2 = tol * tol;
        if point.square_distance(&pair.first) <= tol2 && point.square_distance(&pair.second) <= tol2
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The nearest common vertex to `point` and its parameters on both edges; a vertex at
/// the same distance replaces it when its parameters are nearer the intersection
/// (eap: a closed edge).
fn nearest_common(
    point: &Point3,
    common: &[Vertex],
    (e1, e2): (&Edge, &Edge),
    (p1, p2): (f64, f64),
) -> Result<(Point3, f64, f64), Failure> {
    let (mut nearest, mut v1, mut v2) = (Point3::new(0.0, 0.0, 0.0), 0.0, 0.0);
    let mut distance: f64 = real_last();
    for vertex in common {
        let candidate = BRepTool::pnt(vertex)?;
        let d: f64 = point.distance(&candidate);
        if d < distance {
            nearest = candidate;
            distance = d;
            v1 = BRepTool::parameter(vertex, e1)?;
            v2 = BRepTool::parameter(vertex, e2)?;
        } else if is_equal(distance, d) {
            let n1 = BRepTool::parameter(vertex, e1)?;
            let n2 = BRepTool::parameter(vertex, e2)?;
            if (p1 - v1).abs() + (p2 - v2).abs() > (p1 - n1).abs() + (p2 - n2).abs() {
                nearest = candidate;
                v1 = n1;
                v2 = n2;
            }
        }
    }
    Ok((nearest, v1, v2))
}

impl SelfIntersect<'_> {
    /// The 3d points of the parameters `(p1, p2)` on the two edges.
    fn point_pair(
        &self,
        (e1, e2): (&Edge, &Edge),
        (p1, p2): (f64, f64),
    ) -> Result<PointPair, Failure> {
        Ok(PointPair {
            first: point_on_edge(e1, p1, &self.c1, &self.hs)?,
            second: point_on_edge(e2, p2, &self.c2, &self.hs)?,
        })
    }

    /// The points and segments of the last pair intersection: false when one is an
    /// intersection of the wire.
    pub(super) fn pair_ok(&self, e1: &Edge, e2: &Edge) -> Result<bool, Failure> {
        let common = common_vertices(e1, e2)?;
        for p in 1..=self.inter.nb_points()? {
            if !self.point_ok(&self.inter.point(p)?, (e1, e2), &common)? {
                return Ok(false);
            }
        }
        for s in 1..=self.inter.nb_segments()? {
            let segment = self.inter.segment(s)?;
            if !(segment.has_first_point() && segment.has_last_point()) {
                continue;
            }
            let ends = [segment.first_point()?, segment.last_point()?];
            if !self.segment_ok(&ends, (e1, e2), &common)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// One intersection point: an end point, or one out of the edge ranges, is
    /// accepted; an inner one must be at a common vertex or along its chord.
    fn point_ok(
        &self,
        ip: &IntersectionPoint,
        edges: (&Edge, &Edge),
        common: &[Vertex],
    ) -> Result<bool, Failure> {
        if ip.transition_of_first().position_on_curve() != Position::Middle
            && ip.transition_of_second().position_on_curve() != Position::Middle
        {
            return Ok(true);
        }
        let params = (ip.param_on_first(), ip.param_on_second());
        // gka: protect against working out of the edge range.
        let (_, f1, l1) = BRepTool::curve(edges.0)?;
        let (_, f2, l2) = BRepTool::curve(edges.1)?;
        let eps = precision::p_confusion();
        if f1 - params.0 > eps || params.0 - l1 > eps || f2 - params.1 > eps || params.1 - l2 > eps
        {
            return Ok(true);
        }
        let pair = self.point_pair(edges, params)?;
        if at_common_vertex(&pair, common)? {
            return Ok(true);
        }
        if common.is_empty() {
            return Ok(false);
        }
        self.along_chords(&pair, edges, params, common)
    }

    /// The maximum yawn between the two edges: the points of each edge between the
    /// nearest common vertex and the intersection must stay within twice the edge
    /// tolerance of the chord from the vertex to the intersection point.
    fn along_chords(
        &self,
        pair: &PointPair,
        (e1, e2): (&Edge, &Edge),
        params: (f64, f64),
        common: &[Vertex],
    ) -> Result<bool, Failure> {
        let (nearest, v1, v2) = nearest_common(&pair.first, common, (e1, e2), params)?;
        // Patch: extraordinary situation (a vertex tolerance of 0), skv OCC1764.
        if nearest.distance(&pair.first) <= gp::resolution()
            || nearest.distance(&pair.second) <= gp::resolution()
        {
            return Ok(true);
        }
        let mut chord = Line3::new(
            nearest,
            Direction3::from_vector(Vector3::between(nearest, pair.first))?,
        );
        let du1: f64 = 0.1 * (params.0 - v1);
        if !self.edge_near_chord(e1, &chord, (v1, du1), false)? {
            return Ok(false);
        }
        chord.set_direction(Direction3::from_xyz(pair.second.xyz() - nearest.xyz())?);
        let du2: f64 = 0.1 * (params.1 - v2);
        self.edge_near_chord(e2, &chord, (v2, du2), true)
    }

    /// The points `v + k du` (k = 2..8) of the edge within twice its tolerance of
    /// `chord`.
    fn edge_near_chord(
        &self,
        edge: &Edge,
        chord: &Line3,
        (start, du): (f64, f64),
        second: bool,
    ) -> Result<bool, Failure> {
        let tole: f64 = BRepTool::edge_tolerance(edge)?;
        let pcurve = if second { &self.c2 } else { &self.c1 };
        for k in 2..9 {
            let u: f64 = start + f64::from(k) * du;
            let point = point_on_edge(edge, u, pcurve, &self.hs)?;
            if chord.distance(point) > tole * 2.0 {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// One common segment with both ends: accepted when an end at the ends of both
    /// curves is not inside two coincident lines, or when an end is at a common vertex.
    fn segment_ok(
        &self,
        ends: &[IntersectionPoint; 2],
        edges: (&Edge, &Edge),
        common: &[Vertex],
    ) -> Result<bool, Failure> {
        for end in ends {
            let at_ends = end.transition_of_first().position_on_curve() != Position::Middle
                && end.transition_of_second().position_on_curve() != Position::Middle;
            if at_ends {
                // ZZ: two lines are checked for coincidence.
                return Ok(!self.coincident_lines(ends, edges)?);
            }
            let pair = self.point_pair(edges, (end.param_on_first(), end.param_on_second()))?;
            if at_common_vertex(&pair, common)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Whether both pcurves are lines and the segment's inner point at `PAR_T` of the
    /// first one lies on the second, inside its segment, with parallel directions.
    fn coincident_lines(
        &self,
        ends: &[IntersectionPoint; 2],
        (e1, e2): (&Edge, &Edge),
    ) -> Result<bool, Failure> {
        if self.c1.get_type()? != CurveType::Line || self.c2.get_type()? != CurveType::Line {
            return Ok(false);
        }
        let tol = BRepTool::edge_tolerance(e1)? + BRepTool::edge_tolerance(e2)?;
        let tol2 = tol * tol;
        let (l1, l2) = (self.c1.line()?, self.c2.line()?);
        let (t11, t12) = (ends[0].param_on_first(), ends[1].param_on_first());
        let (t21, t22) = (ends[0].param_on_second(), ends[1].param_on_second());
        let t1m: f64 = (1.0 - PAR_T) * t11 + PAR_T * t12;
        let p1m = self.c1.value(t1m)?;
        if l2.square_distance(p1m) < tol2 {
            let t2m: f64 = elclib::parameter(&l2, p1m)?;
            if t2m > t21 && t2m < t22 {
                return Ok(l1
                    .direction()
                    .is_parallel(l2.direction(), precision::angular()));
            }
        }
        Ok(false)
    }
}
