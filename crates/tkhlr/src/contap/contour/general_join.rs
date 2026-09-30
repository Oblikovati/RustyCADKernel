//! Native final contour vertex joins and walking/restriction associations.
use crate::contap::{
    ContapHContTool, ContapHCurve2dTool, ContapIType, ContapLine, ContapPoint, ContapSurfProps,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkgeomalgo::int_surf::{IntSurfTransition, make_transition};
use rustycad_tkmath::gp::{Direction3, Point2, Point3, Vector3};
pub(super) fn mark_multiple(lines: &[ContapLine], tolerance: f64) -> Result<(), Failure> {
    for (index, line) in lines.iter().enumerate() {
        for rank in 1..=line.nb_vertex() {
            let point = line.vertex(rank)?;
            if point.is_on_arc() {
                continue;
            }
            mark_matches(line, rank, point.value(), &lines[index + 1..], tolerance)?;
        }
    }
    Ok(())
}
fn mark_matches(
    line: &ContapLine,
    rank: i32,
    point: Point3,
    others: &[ContapLine],
    tolerance: f64,
) -> Result<(), Failure> {
    for other in others {
        for slot in 1..=other.nb_vertex() {
            let candidate = other.vertex(slot)?;
            if !candidate.is_on_arc() && point.distance(&candidate.value()) <= tolerance {
                line.change_vertex(rank, ContapPoint::set_multiple)?;
                other.change_vertex(slot, ContapPoint::set_multiple)?;
            }
        }
    }
    Ok(())
}
pub(super) struct RestrictionJoin<'a> {
    pub surface: &'a Adaptor3dSurfaceHandle,
    pub precision: f64,
    pub scratch: &'a mut ContapPoint,
    pub line_transition: &'a mut IntSurfTransition,
    pub arc_transition: &'a mut IntSurfTransition,
}
impl RestrictionJoin<'_> {
    pub fn perform(&mut self, lines: &[ContapLine]) -> Result<(), Failure> {
        for line in lines {
            if line.type_contour() != ContapIType::Walking {
                continue;
            }
            for rank in 1..=line.nb_vertex() {
                self.join_vertex(line, rank, lines)?;
            }
        }
        Ok(())
    }
    fn join_vertex(
        &mut self,
        line: &ContapLine,
        rank: i32,
        lines: &[ContapLine],
    ) -> Result<(), Failure> {
        let point = line.vertex(rank)?;
        if point.is_on_arc() || point.is_multiple() {
            return Ok(());
        }
        let (u, v) = point.parameters();
        let uv = Point2::new(u, v);
        for other in lines {
            if other.type_contour() != ContapIType::Restriction {
                continue;
            }
            let Some((parameter, projection)) = ContapHContTool::project(&other.arc()?, uv)? else {
                continue;
            };
            if projection.distance(uv) <= self.precision {
                self.attach(line, rank, point, other, parameter)?;
                break;
            }
        }
        Ok(())
    }
    fn attach(
        &mut self,
        line: &ContapLine,
        rank: i32,
        mut point: ContapPoint,
        restriction: &ContapLine,
        parameter: f64,
    ) -> Result<(), Failure> {
        let arc = restriction.arc()?;
        let (uv, d2) = ContapHCurve2dTool::d1(&arc, parameter)?;
        let (position, du, dv, normal) =
            ContapSurfProps::deriv_and_norm(self.surface, uv.x(), uv.y())?;
        let boundary = du * d2.x() + dv * d2.y();
        let tangent = walking_tangent(line, &point)?;
        let direction = Direction3::new(normal.x(), normal.y(), normal.z())?;
        make_transition(
            tangent,
            boundary,
            direction,
            self.line_transition,
            self.arc_transition,
        );
        point.set_arc(&arc, parameter, *self.line_transition, *self.arc_transition);
        point.set_multiple();
        line.change_vertex(rank, |vertex| *vertex = point)?;
        self.scratch.set_value(position, uv.x(), uv.y());
        self.scratch.set_parameter(parameter);
        self.scratch.set_multiple();
        restriction.add_vertex(self.scratch);
        Ok(())
    }
}
fn walking_tangent(line: &ContapLine, point: &ContapPoint) -> Result<Vector3, Failure> {
    let parameter = point.parameter_on_line() as i32;
    if parameter == line.nb_pnts()? {
        return Ok(Vector3::from(
            point.value().xyz() - line.point(parameter - 1)?.value().xyz(),
        ));
    }
    Ok(Vector3::from(
        line.point(parameter + 1)?.value().xyz() - point.value().xyz(),
    ))
}
