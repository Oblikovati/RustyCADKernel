// Rust translation of ChFi2d_FilletAlgo::NbResults and Result (ChFi2d_FilletAlgo.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The fillets found: their number, and the arc nearest to a point with the edges cut
//! at its ends.
use super::fill::live;
use super::{ChFi2dFilletAlgo, FilletPoint};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve};
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
use rustycad_tkmath::elclib;
use rustycad_tkmath::elslib::{plane_parameters, plane_value};
use rustycad_tkmath::gp::{Axis2, Axis3, Circle3, Point2, Point3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
use std::f64::consts::FRAC_PI_2;

impl ChFi2dFilletAlgo {
    /// `NbResults(thePoint)`: the number of roots that are still solutions.
    pub fn nb_results(&mut self, point: Point3) -> Result<i32, Failure> {
        let _target = plane_parameters(self.position()?, point);
        let mut nb = 0;
        for i in 0..self.result_params.len() {
            if self.refilled(i)?.has_solution(self.radius) != 0 {
                nb += 1;
            }
        }
        Ok(nb)
    }

    /// The position of the plane.
    fn position(&self) -> Result<Axis3, Failure> {
        let plane = self.plane.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "ChFi2d_FilletAlgo: no plane; expected Init first",
            )
        })?;
        Ok(plane.pln().position())
    }

    /// The root `i` (0-based) filled again on its side (`myStartSide` keeps it).
    fn refilled(&mut self, i: usize) -> Result<FilletPoint, Failure> {
        self.start_side = self.result_orientation.get(i).copied().unwrap_or(0) != 0;
        let param = self.result_params[i];
        let mut point = FilletPoint::new(param);
        self.fill_point(&mut point, param + 1.0)?;
        Ok(point)
    }

    /// `Result(thePoint, theEdge1, theEdge2, iSolution)`: the fillet arc nearest to the
    /// point (or the solution `i_solution` when it is not -1), and the given edges cut
    /// at its ends; the null edge when there is none.
    pub fn result(
        &mut self,
        point: Point3,
        edge1: &mut Edge,
        edge2: &mut Edge,
        i_solution: i32,
    ) -> Result<Edge, Failure> {
        let (x, y) = plane_parameters(self.position()?, point);
        let Some(nearest) = self.nearest_solution(Point2::new(x, y), i_solution)? else {
            return Ok(Edge::default());
        };
        let position = self.position()?;
        // create circle edge
        let center = plane_value(nearest.get_center().x(), nearest.get_center().y(), position);
        let circ = Circle3::new(Axis2::new(center, position.direction())?, self.radius)?;
        let circle: Handle<dyn GeomCurve> = Handle::new(GeomCircle::from_circ(circ)).cast();
        let p1 = self.point_on(true, nearest.get_param(), position)?;
        let p2 = self.point_on(false, nearest.get_param2(), position)?;
        let ends = arc_ends(&circle, circ, point, (p1, p2))?;
        let (first, last) = if ends.is_out {
            (ends.param2, ends.param1)
        } else {
            (ends.param1, ends.param2)
        };
        let result = BRepBuilderAPIMakeEdge::from_circ_params(circ, first, last)?.edge()?;
        // divide edges
        let divided1 = self.divided(
            &self.edge1.clone(),
            (&circle, ends.param1),
            nearest.get_param(),
            !ends.is_out,
        )?;
        let divided2 = self.divided(
            &self.edge2.clone(),
            (&circle, ends.param2),
            nearest.get_param2(),
            ends.is_out,
        )?;
        if self.edges_exchanged {
            (*edge2, *edge1) = (divided1, divided2);
        } else {
            (*edge1, *edge2) = (divided1, divided2);
        }
        Ok(result)
    }

    /// The root that is still a solution nearest to the target (its center at the
    /// radius from it), or the solution `i_solution`.
    fn nearest_solution(
        &mut self,
        target: Point2,
        i_solution: i32,
    ) -> Result<Option<FilletPoint>, Failure> {
        let (mut nearest, mut distance, mut i_sol) = (None, 0.0, 1);
        for i in 0..self.result_params.len() {
            let point = self.refilled(i)?;
            if point.has_solution(self.radius) == 0 {
                continue;
            }
            let p = if i_solution == -1 {
                (point.get_center().distance(target) - self.radius).abs()
            } else if i_solution == i_sol {
                0.0
            } else {
                f64::MAX
            };
            if nearest.is_none() || p < distance {
                (nearest, distance) = (Some(point), p);
            }
            if i_solution == i_sol {
                break;
            }
            i_sol += 1;
        }
        Ok(nearest)
    }

    /// The point of the first (`on_first`) or second curve at `param`, in the space.
    fn point_on(&self, on_first: bool, param: f64, position: Axis3) -> Result<Point3, Failure> {
        let curve = if on_first { &self.curve1 } else { &self.curve2 };
        let p2d = live(curve)?.value(param)?;
        Ok(plane_value(p2d.x(), p2d.y(), position))
    }

    /// The edge cut at the end of the fillet: its 3d curve from `param` (kept as start
    /// when the circle and the curve run apart and `keep_start_when_apart`, or as end).
    fn divided(
        &self,
        edge: &Edge,
        (circle, circle_param): (&Handle<dyn GeomCurve>, f64),
        param: f64,
        keep_start_when_apart: bool,
    ) -> Result<Edge, Failure> {
        let (curve, mut start, mut end) = BRepTool::curve(edge)?;
        let (_, dir) = curve.get().ok_or_else(null_curve)?.d1(param)?;
        let (_, circle_dir) = circle.get().ok_or_else(null_curve)?.d1(circle_param)?;
        let apart = circle_dir.angle(dir)? > FRAC_PI_2;
        if if apart {
            keep_start_when_apart
        } else {
            !keep_start_when_apart
        } {
            start = param;
        } else {
            end = param;
        }
        // Check the case when start and end are identical (bug compatible with ww5)
        if (start - end).abs() < precision::confusion() {
            end = start + precision::confusion();
        }
        BRepBuilderAPIMakeEdge::from_curve_params(&curve, start, end)?.edge()
    }
}

/// The ends of the fillet arc on the circle and whether the arc runs outside the
/// parameter of the point nearest to the target.
struct ArcEnds {
    param1: f64,
    param2: f64,
    is_out: bool,
}

/// The parameters of the ends of the fillet on the circle, and whether the arc between
/// them contains the projection of the target point (checked manually at 0, "a bug in
/// Open CASCADE in calculation of nearest point to a circle near the parameter 0.0").
fn arc_ends(
    circle: &Handle<dyn GeomCurve>,
    circ: Circle3,
    target: Point3,
    (p1, p2): (Point3, Point3),
) -> Result<ArcEnds, Failure> {
    let mut proj = ProjectPointOnCurve::with_curve(target, circle)?;
    let mut target_param = proj.lower_distance_parameter()?;
    let point_on_circle = proj.nearest_point()?;
    let p0 = elclib::circle_value(0.0, circ.position(), circ.radius());
    if p0.distance(&target) < point_on_circle.distance(&target) {
        target_param = 0.0;
    }
    proj.perform(p1)?;
    let param1 = proj.lower_distance_parameter()?;
    proj.perform(p2)?;
    let param2 = proj.lower_distance_parameter()?;
    let mut is_out = (param1 < target_param && param2 < target_param)
        || (param1 > target_param && param2 > target_param);
    if param1 > param2 {
        is_out = !is_out;
    }
    Ok(ArcEnds {
        param1,
        param2,
        is_out,
    })
}

fn null_curve() -> Failure {
    Failure::with_message("Standard_NullObject", "ChFi2d_FilletAlgo: null curve")
}
