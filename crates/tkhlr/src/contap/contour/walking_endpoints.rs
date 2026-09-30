//! Native walking endpoint reconstruction, including reverse destination traversal.
use crate::contap::{
    ContapHCurve2dTool, ContapLine, ContapPoint, ContapSurfProps,
    ContapTheIWLineOfTheIWalking as WalkLine, ContapTheSearch,
};
use rustycad_tkernel::{ncollection::Array1, standard::Failure};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkgeomalgo::int_surf::{IntSurfPathPoint, IntSurfTransition, make_transition};
use rustycad_tkmath::gp::Direction3;
pub(super) struct EndpointBuilder<'a> {
    pub search: &'a ContapTheSearch,
    pub paths: &'a [IntSurfPathPoint],
    pub destination: &'a Array1<'static, i32>,
    pub surface: &'a Adaptor3dSurfaceHandle,
}
struct EndpointPosition {
    last: bool,
    rank: i32,
    parameter: i32,
    multiplicity: i32,
}
#[derive(Default)]
pub(super) struct EndpointState {
    pub(super) first: ContapPoint,
    last: ContapPoint,
    pub(super) line_transition: IntSurfTransition,
    pub(super) arc_transition: IntSurfTransition,
}
impl EndpointState {
    fn point(&mut self, last: bool) -> &mut ContapPoint {
        if last {
            &mut self.last
        } else {
            &mut self.first
        }
    }
}
impl EndpointBuilder<'_> {
    pub fn append(
        &self,
        line: &ContapLine,
        walk: &WalkLine,
        state: &mut EndpointState,
    ) -> Result<(), Failure> {
        self.endpoint(line, walk, state, false)?;
        self.endpoint(line, walk, state, true)
    }
    fn endpoint(
        &self,
        line: &ContapLine,
        walk: &WalkLine,
        state: &mut EndpointState,
        last: bool,
    ) -> Result<(), Failure> {
        let has_point = if last {
            walk.has_last_point()
        } else {
            walk.has_first_point()
        };
        let parameter = if last { walk.nb_points() } else { 1 };
        if !has_point {
            return self.simple_endpoint(line, walk, state, last, parameter);
        }
        let index = if last {
            walk.last_point_index()?
        } else {
            walk.first_point_index()?
        };
        self.boundary_group(line, walk, state, last, parameter, index)
    }
    fn path(&self, index: i32) -> Result<&IntSurfPathPoint, Failure> {
        self.paths
            .get(index.wrapping_sub(1) as usize)
            .ok_or_else(|| {
                Failure::out_of_range(&format!(
                    "Contour endpoint path {index}; expected 1..={}",
                    self.paths.len()
                ))
            })
    }
    fn boundary_group(
        &self,
        line: &ContapLine,
        walk: &WalkLine,
        state: &mut EndpointState,
        last: bool,
        parameter: i32,
        index: i32,
    ) -> Result<(), Failure> {
        let path = self.path(index)?;
        let mut multiplicity = path.multiplicity()?;
        for rank in (1..=self.search.nb_points()?).rev() {
            if self.destination[rank] != index {
                continue;
            }
            self.boundary_endpoint(
                line,
                walk,
                state,
                path,
                EndpointPosition {
                    last,
                    rank,
                    parameter,
                    multiplicity,
                },
            )?;
            multiplicity -= 1;
        }
        Ok(())
    }
    fn simple_endpoint(
        &self,
        line: &ContapLine,
        walk: &WalkLine,
        state: &mut EndpointState,
        last: bool,
        parameter: i32,
    ) -> Result<(), Failure> {
        let (u, v) = walk.value(parameter)?.parameters_on_s2();
        let point = state.point(last);
        point.set_value(line.point(parameter)?.value(), u, v);
        point.set_parameter(f64::from(parameter));
        line.add_vertex(point);
        Ok(())
    }
    fn boundary_endpoint(
        &self,
        line: &ContapLine,
        walk: &WalkLine,
        state: &mut EndpointState,
        path: &IntSurfPathPoint,
        position: EndpointPosition,
    ) -> Result<(), Failure> {
        let (u, v) = path.parameters(position.multiplicity)?;
        let point = state.point(position.last);
        point.set_value(path.value(), u, v);
        point.set_parameter(f64::from(position.parameter));
        let tangent = if position.last {
            walk.is_tangent_at_end()
        } else {
            walk.is_tangent_at_begining()
        };
        self.transitions(state, path, position.rank, position.last, tangent)?;
        self.attach_boundary(line, state, position)
    }
    fn attach_boundary(
        &self,
        line: &ContapLine,
        state: &mut EndpointState,
        position: EndpointPosition,
    ) -> Result<(), Failure> {
        let (rank, last) = (position.rank, position.last);
        let start = self.search.point(rank)?;
        let arc = start.arc();
        let (line_transition, arc_transition) = (state.line_transition, state.arc_transition);
        let point = state.point(last);
        point.set_arc(&arc, start.parameter(), line_transition, arc_transition);
        if !start.is_new() {
            point.set_vertex(&start.vertex()?);
        }
        line.add_vertex(point);
        Ok(())
    }
    fn transitions(
        &self,
        state: &mut EndpointState,
        path: &IntSurfPathPoint,
        rank: i32,
        last: bool,
        tangent: bool,
    ) -> Result<(), Failure> {
        if tangent {
            state.line_transition.set_undecided();
            state.arc_transition.set_undecided();
            return Ok(());
        }
        let start = self.search.point(rank)?;
        let (uv, d2) = ContapHCurve2dTool::d1(&start.arc(), start.parameter())?;
        let (_, du, dv, normal) = ContapSurfProps::deriv_and_norm(self.surface, uv.x(), uv.y())?;
        let boundary = du * d2.x() + dv * d2.y();
        let mut along = path.direction3d()?;
        if last {
            along.reverse();
        }
        let direction = Direction3::new(normal.x(), normal.y(), normal.z())?;
        make_transition(
            along,
            boundary,
            direction,
            &mut state.line_transition,
            &mut state.arc_transition,
        );
        Ok(())
    }
}
