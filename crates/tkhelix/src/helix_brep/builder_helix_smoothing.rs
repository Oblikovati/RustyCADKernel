// Rust translation of ModelingAlgorithms/TKHelix/HelixBRep/HelixBRep_BuilderHelix.cxx
// (Smoothing and SmoothingEdges) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The tangent smoothing of `HelixBRep_BuilderHelix` at the joints of its parts.
use super::builder_helix::HelixBRepBuilderHelix;
use super::builder_helix_perform::no_such_object;
use crate::helix_geom::errors::null_object;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topods::{Edge, Wire};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkmath::gp::{Point3, Vector3};

/// The end of a joint: the edge, its B-spline curve and the parameter of the joint.
struct JointEnd<'e> {
    edge: &'e Edge,
    curve: Handle<GeomBSplineCurve>,
    parameter: f64,
}

impl JointEnd<'_> {
    // The B-spline of `edge` (shared with the edge, as BRep_Tool::Curve returns it at
    // the identity location) and its last (`at_end`) or first parameter.
    fn of(edge: &Edge, at_end: bool) -> Result<JointEnd<'_>, Failure> {
        let (curve, first, last) = BRepTool::curve(edge)?;
        let curve: Handle<GeomBSplineCurve> = curve.cast();
        let parameter: f64 = if at_end { last } else { first };
        Ok(JointEnd {
            edge,
            curve,
            parameter,
        })
    }

    fn bspline(&self) -> Result<&GeomBSplineCurve, Failure> {
        self.curve.get().ok_or_else(|| null_object("aCPrev/aCNext"))
    }
}

// The last edge of `explorer` from its current edge on (aPrevEdge after the loop).
fn last_edge(explorer: &mut WireExplorer, mut last: Edge) -> Result<Edge, Failure> {
    while explorer.more() {
        last = explorer.current().clone();
        explorer.next()?;
    }
    Ok(last)
}

impl HelixBRepBuilderHelix {
    /// `Smoothing(theParts)`: at each joint of two parts, the tangents of the last edge
    /// of one and the first edge of the next are averaged. Standard_NoSuchObject
    /// without a part.
    pub(super) fn smoothing(&self, parts: &[Wire]) -> Result<(), Failure> {
        if parts.len() == 1 {
            return Ok(());
        }
        let first: &Wire = parts.first().ok_or_else(no_such_object)?;
        let mut explorer = WireExplorer::new();
        explorer.init(first)?;
        let mut prev_edge: Edge = last_edge(&mut explorer, Edge::default())?;
        for next_wire in &parts[1..] {
            explorer.clear();
            explorer.init(next_wire)?;
            let next_edge: Edge = explorer.current().clone();
            self.smoothing_edges(&prev_edge, &next_edge)?;
            prev_edge = last_edge(&mut explorer, prev_edge)?;
        }
        Ok(())
    }

    /// `SmoothingEdges(thePrev, theNext)`: nothing when the tangents at the joint make
    /// an angle below 1e-7; otherwise each curve gets the average tangent at the joint
    /// by MovePointAndTangent, raising its degree up to the maximum until the move
    /// succeeds, and the edge takes the curve when it did. The curves are the edges'
    /// own: a degree raised by a failed attempt stays.
    fn smoothing_edges(&self, prev: &Edge, next: &Edge) -> Result<(), Failure> {
        const EPS_ANG: f64 = 1.0e-7;
        let prev_end = JointEnd::of(prev, true)?;
        let next_end = JointEnd::of(next, false)?;
        let (p1, v1): (Point3, Vector3) = prev_end.bspline()?.d1(prev_end.parameter)?;
        let (p2, v2): (Point3, Vector3) = next_end.bspline()?.d1(next_end.parameter)?;
        if v1.angle(v2)? < EPS_ANG {
            return Ok(());
        }
        let tangent: Vector3 = (v1 + v2) * 0.5;
        self.move_joint(&prev_end, p1, tangent, (1, -1))?;
        self.move_joint(&next_end, p2, tangent, (-1, 1))
    }

    // MovePointAndTangent at the joint, then with the degree raised one by one up to
    // Geom_BSplineCurve::MaxDegree() until it succeeds; UpdateEdge on success.
    fn move_joint(
        &self,
        end: &JointEnd<'_>,
        point: Point3,
        tangent: Vector3,
        conditions: (i32, i32),
    ) -> Result<(), Failure> {
        let curve: &GeomBSplineCurve = end.bspline()?;
        let degree: i32 = curve.degree();
        let mut moved: bool = self.move_once(curve, end.parameter, point, tangent, conditions)?;
        let mut target: i32 = degree + 1;
        while !moved && target <= GeomBSplineCurve::max_degree() {
            curve.increase_degree(target)?;
            moved = self.move_once(curve, end.parameter, point, tangent, conditions)?;
            target += 1;
        }
        if !moved {
            return Ok(());
        }
        let tolerance: f64 = BRepTool::edge_tolerance(end.edge)?;
        let as_curve: Handle<dyn GeomCurve> = end.curve.cast();
        BRepBuilder::default().update_edge_curve(end.edge, &as_curve, tolerance)
    }

    // One MovePointAndTangent with the builder tolerance; true when its status is 0.
    fn move_once(
        &self,
        curve: &GeomBSplineCurve,
        u: f64,
        point: Point3,
        tangent: Vector3,
        conditions: (i32, i32),
    ) -> Result<bool, Failure> {
        let status: i32 = curve.move_point_and_tangent(
            u,
            point,
            tangent,
            self.tolerance,
            conditions.0,
            conditions.1,
        )?;
        Ok(status == 0)
    }
}
