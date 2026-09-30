// Rust translation of the static IsOverlapPartEdges and CheckOverlapping of
// ModelingAlgorithms/TKShHealing/ShapeAnalysis/ShapeAnalysis_Edge.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The overlap of two edges: points of the shorter one along its length, each within
//! the tolerance of the other edge.
use super::ShapeAnalysisEdge;
use crate::shape_extend::ShapeExtendStatus;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::gcpnts::AbscissaPoint;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_extrema::{BRepExtremaDistShapeShape, BRepExtremaSupportType};

/// The length of the edge curve (`GCPnts_AbscissaPoint::Length`).
fn edge_length(curve: &BRepAdaptorCurve) -> Result<f64, Failure> {
    let adaptor: &dyn Adaptor3dCurve = curve;
    AbscissaPoint::length(adaptor)
}

/// The point of the first edge at the curvilinear length `s`: its first vertex (with its
/// orientation) near 0, else the point of the abscissa search (`None` when it fails).
fn point_at_length(
    first_edge: &Edge,
    curve: &BRepAdaptorCurve,
    s: f64,
) -> Result<Option<Point3>, Failure> {
    if s <= precision::confusion() {
        return Ok(Some(BRepTool::pnt(&topexp::first_vertex(
            first_edge, true,
        ))?));
    }
    let adaptor: &dyn Adaptor3dCurve = curve;
    let point = AbscissaPoint::with_tolerance(
        precision::confusion(),
        adaptor,
        s,
        curve.first_parameter()?,
    )?;
    if !point.is_done() {
        return Ok(None);
    }
    Ok(Some(curve.value(point.parameter()?)?))
}

/// The static `IsOverlapPartEdges`: every point of the first edge at lengths from
/// `start` to `end` by half steps is within `tolerance` of the second edge.
fn is_overlap_part_edges(
    first_edge: &Edge,
    sec_edge: &Edge,
    tolerance: f64,
    step: f64,
    (start, end): (f64, f64),
) -> Result<bool, Failure> {
    let curve = BRepAdaptorCurve::with_edge(first_edge)?;
    let mut distance = BRepExtremaDistShapeShape::new();
    distance.load_s1(sec_edge)?;
    let mut length = start;
    while length <= end {
        if sample_exceeds_tolerance(first_edge, &curve, length, &mut distance, tolerance)? {
            return Ok(false);
        }
        length += step / 2.0;
    }
    Ok(true)
}

impl ShapeAnalysisEdge {
    /// `CheckOverlapping(theEdge1, theEdge2, theTolOverlap, theDomainDist)`: the shorter
    /// edge lies within `tol_overlap` of the other one along its whole length (DONE3), or,
    /// when `domain_dist` is not 0, along a part of that length around each closest point
    /// (DONE4). On the second path `tol_overlap` receives the distance between the edges;
    /// `sae.check_overlapping(&e1, &e2, &mut tol, 0.0)?`.
    pub fn check_overlapping(
        &mut self,
        edge1: &Edge,
        edge2: &Edge,
        tol_overlap: &mut f64,
        domain_dist: f64,
    ) -> Result<bool, Failure> {
        let span = SpanCheck::from_edges(edge1, edge2, *tol_overlap, domain_dist)?;
        if span.whole_edge_overlaps()? {
            self.add_status(ShapeExtendStatus::Done3);
            return Ok(true);
        }
        if domain_dist == 0.0 {
            return Ok(false);
        }
        let (overlap, resulting_tolerance) = span.partial_overlap()?;
        if overlap {
            self.add_status(ShapeExtendStatus::Done4);
        }
        *tol_overlap = resulting_tolerance;
        Ok(overlap)
    }
}

/// The part of the shorter edge checked around one closest point.
struct SpanCheck<'a> {
    first_edge: &'a Edge,
    sec_edge: &'a Edge,
    length: f64,
    domain_tol: f64,
    tol: f64,
}

impl SpanCheck<'_> {
    /// The length along the shorter edge of solution `i` (at a vertex: 0 at its first
    /// vertex, else its whole length); `None` for a solution inside a face.
    fn solution_length(
        &self,
        min_dist: &BRepExtremaDistShapeShape,
        i: i32,
    ) -> Result<Option<f64>, Failure> {
        match min_dist.support_type_shape1(i)? {
            BRepExtremaSupportType::IsVertex => {
                let support = min_dist.support_on_shape1(i)?;
                let (v1, _) = topexp::vertices(self.first_edge, true);
                Ok(Some(if v1.is_same(&support) {
                    0.0
                } else {
                    self.length
                }))
            }
            BRepExtremaSupportType::IsOnEdge => {
                self.interior_solution_length(min_dist, i).map(Some)
            }
            BRepExtremaSupportType::IsInFace => Ok(None),
        }
    }

    /// The overlap check on `[length_p - domain/2, length_p + domain/2]`, moved inside
    /// the edge, by fifths of it.
    fn check_solution(
        &self,
        min_dist: &BRepExtremaDistShapeShape,
        i: i32,
    ) -> Result<bool, Failure> {
        let Some(length_p) = self.solution_length(min_dist, i)? else {
            return Ok(false);
        };
        let mut start = length_p - self.domain_tol / 2.0;
        if start < 0.0 {
            // Natively the end set here is overwritten just below.
            start = 0.0;
        }
        let mut end = length_p + self.domain_tol / 2.0;
        if end > self.length {
            end = self.length;
            start = end - self.domain_tol;
        }
        let step = (end - start) / 5.0;
        is_overlap_part_edges(self.first_edge, self.sec_edge, self.tol, step, (start, end))
    }
}

fn sample_exceeds_tolerance(
    edge: &Edge,
    curve: &BRepAdaptorCurve,
    length: f64,
    distance: &mut BRepExtremaDistShapeShape,
    tolerance: f64,
) -> Result<bool, Failure> {
    let Some(point) = point_at_length(edge, curve, length)? else {
        return Ok(false);
    };
    let mut vertex = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut vertex, point, precision::confusion())?;
    distance.load_s2(&vertex)?;
    distance.perform(&ProgressRange::default())?;
    Ok(distance.is_done() && distance.value()? >= tolerance)
}

impl<'a> SpanCheck<'a> {
    fn from_edges(
        edge1: &'a Edge,
        edge2: &'a Edge,
        tol: f64,
        domain: f64,
    ) -> Result<Self, Failure> {
        let length1 = edge_length(&BRepAdaptorCurve::with_edge(edge1)?)?;
        let length2 = edge_length(&BRepAdaptorCurve::with_edge(edge2)?)?;
        let (first_edge, sec_edge) = if length1 >= length2 {
            (edge2, edge1)
        } else {
            (edge1, edge2)
        };
        let length = length1.min(length2);
        let domain_tol = if domain > length { length } else { domain };
        Ok(Self {
            first_edge,
            sec_edge,
            length,
            domain_tol,
            tol,
        })
    }

    fn whole_edge_overlaps(&self) -> Result<bool, Failure> {
        is_overlap_part_edges(
            self.first_edge,
            self.sec_edge,
            self.tol,
            self.length / 2.0,
            (0.0, self.length),
        )
    }

    fn partial_overlap(&self) -> Result<(bool, f64), Failure> {
        let distance = BRepExtremaDistShapeShape::with_deflection(
            self.first_edge,
            self.sec_edge,
            self.tol,
            Default::default(),
            &ProgressRange::default(),
        )?;
        if !distance.is_done() {
            return Ok((false, self.tol));
        }
        let measured = distance.value()?;
        if measured >= self.tol {
            return Ok((false, self.tol));
        }
        for index in 1..=distance.nb_solution() {
            if self.check_solution(&distance, index)? {
                return Ok((true, measured));
            }
        }
        Ok((false, measured))
    }

    fn interior_solution_length(
        &self,
        distance: &BRepExtremaDistShapeShape,
        index: i32,
    ) -> Result<f64, Failure> {
        let parameter = distance.par_on_edge_s1(index)?;
        let (first, _) = BRepTool::range(self.first_edge)?;
        let curve = BRepAdaptorCurve::with_edge(self.first_edge)?;
        AbscissaPoint::length_between(&curve as &dyn Adaptor3dCurve, first, parameter)
    }
}
