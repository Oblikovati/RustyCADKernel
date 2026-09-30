use super::*;
mod periodic;
mod projection;
#[derive(Default)]
struct BoundaryState {
    count: i32,
    near: bool,
    u_axis: bool,
    first_bound: bool,
}
impl Decomposition<'_> {
    pub(super) fn correct_point(
        &self,
        point: &mut IntSurfPntOn2S,
        group: &WalkingGroup,
        first: bool,
    ) -> Result<(), Failure> {
        let original = *point;
        for surface in 0..2 {
            let state =
                boundary_state(point.value_on_surface(surface == 0), self.surfaces[surface])?;
            if self.compute_line_end(point, &original, group, first, surface, &state)? {
                self.recompute_point(point, group, first, surface, state.near)?;
            }
        }
        Ok(())
    }
    fn compute_line_end(
        &self,
        point: &mut IntSurfPntOn2S,
        original: &IntSurfPntOn2S,
        group: &WalkingGroup,
        first: bool,
        surface: usize,
        state: &BoundaryState,
    ) -> Result<bool, Failure> {
        if state.count == 2 {
            return Ok(true);
        }
        if state.count != 1 {
            return Ok(state.near);
        }
        let axis = usize::from(!state.u_axis);
        if !boundary::is_periodic(self.surfaces[surface], axis)? {
            return Ok(false);
        }
        self.correct_periodic(point, original, group, first, surface, state)
    }
    fn recompute_point(
        &self,
        point: &mut IntSurfPntOn2S,
        group: &WalkingGroup,
        first: bool,
        surface: usize,
        near: bool,
    ) -> Result<(), Failure> {
        let adaptor = self.surfaces[surface];
        let bounds = adaptor.surface().bounds();
        let neighbour = self.neighbour(group, first, surface)?;
        let candidate = if near {
            self.near_boundary(point, neighbour, surface, bounds)?
        } else {
            let other = self.distinct_neighbour(group, first, surface, neighbour)?;
            find_point::find_point(other, neighbour, bounds)?
        };
        if let Some(candidate) = candidate {
            self.project_candidate(point, group, first, surface, candidate, bounds)?;
        }
        Ok(())
    }
    fn near_boundary(
        &self,
        point: &mut IntSurfPntOn2S,
        neighbour: Point2,
        surface: usize,
        bounds: SurfaceBounds,
    ) -> Result<Option<Point2>, Failure> {
        let adaptor = self.surfaces[surface];
        if !adaptor.is_u_periodic()? && !adaptor.is_v_periodic()? {
            return Ok(None);
        }
        let adjusted =
            adjust_by_neighbour(neighbour, point.value_on_surface(surface == 0), adaptor)?;
        if adjusted.x() < bounds.0
            || adjusted.x() > bounds.1
            || adjusted.y() < bounds.2
            || adjusted.y() > bounds.3
        {
            return find_point::find_point(neighbour, adjusted, bounds);
        }
        point.set_parameters_on(surface == 0, adjusted.x(), adjusted.y());
        Ok(None)
    }
    fn distinct_neighbour(
        &self,
        group: &WalkingGroup,
        first: bool,
        surface: usize,
        neighbour: Point2,
    ) -> Result<Point2, Failure> {
        let (low, high) = (group.indices[0], *group.indices.last().unwrap());
        let mut index = if first { low } else { high };
        let mut other = neighbour;
        while index <= high && index >= low {
            // Native increments before reading, including the point beyond the group.
            index += if first { 1 } else { -1 };
            other = self.line.point(index)?.value_on_surface(surface == 0);
            if neighbour.square_distance(other) > gp::resolution() {
                break;
            }
        }
        Ok(other)
    }
}
fn boundary_state(point: Point2, surface: &GeomAdaptorSurface) -> Result<BoundaryState, Failure> {
    let mut state = BoundaryState::default();
    for axis in 0..2 {
        let bound = BoundaryAxis::new(surface, axis, 0.5 * precision::confusion())?;
        let value = bound
            .adjusted(if axis == 0 { point.x() } else { point.y() })
            .0;
        if let Some(first) = bound.location(value, bound.resolution) {
            state.u_axis = axis == 0;
            state.first_bound = first;
            state.count += 1;
        } else if bound.period.is_some() {
            let epsilon = bound.resolution * 100.;
            let part = (bound.upper - bound.lower) * 0.1;
            state.near = bound
                .location(value, if epsilon > part { part } else { epsilon })
                .is_some();
        }
    }
    Ok(state)
}
