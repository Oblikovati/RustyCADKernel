use super::*;
struct SeamChoice {
    neighbour: Point2,
    candidate: Point2,
    vector: Vector2,
    switched: bool,
    check_angle: bool,
}
impl SeamChoice {
    fn resolve_small_vector(&mut self, point: &mut IntSurfPntOn2S, surface: usize) {
        if self.vector.square_magnitude() < gp::resolution() && self.check_angle {
            if self.switched {
                point.set_parameters_on(surface == 0, self.candidate.x(), self.candidate.y());
            }
            self.check_angle = false;
        }
    }
}
impl Decomposition<'_> {
    pub(super) fn correct_periodic(
        &self,
        point: &mut IntSurfPntOn2S,
        original: &IntSurfPntOn2S,
        group: &WalkingGroup,
        first: bool,
        surface: usize,
        state: &BoundaryState,
    ) -> Result<bool, Failure> {
        let tolerance = 0.5 * precision::confusion();
        let axis = usize::from(!state.u_axis);
        let bound = BoundaryAxis::new(self.surfaces[surface], axis, tolerance)?;
        let uv = point.value_on_surface(surface == 0);
        let neighbour = self.neighbour(group, first, surface)?;
        let mut choice = seam_choice(uv, neighbour, &bound, state);
        choice.resolve_small_vector(point, surface);
        if !choice.check_angle || !self.accept_seam_angle(&choice, group, first, surface)? {
            return Ok(true);
        }
        if !choice.switched {
            return Ok(false);
        }
        if !self.matches_surfaces(point, original, choice.candidate, surface)? {
            return Ok(true);
        }
        point.set_parameters_on(surface == 0, choice.candidate.x(), choice.candidate.y());
        Ok(false)
    }
    fn accept_seam_angle(
        &self,
        choice: &SeamChoice,
        group: &WalkingGroup,
        first: bool,
        surface: usize,
    ) -> Result<bool, Failure> {
        let (low, high) = (group.indices[0], *group.indices.last().unwrap());
        let mut index = if first { low } else { high };
        while index <= high && index >= low {
            index += if first { 1 } else { -1 };
            let previous = self.line.point(index)?.value_on_surface(surface == 0);
            let old = Vector2::between(previous, choice.neighbour);
            if old.square_magnitude() <= gp::resolution() {
                continue;
            }
            let angle = choice.vector.angle(old)?;
            return Ok(angle.abs() < std::f64::consts::PI * 0.25 && choice.vector.dot(old) > 0.);
        }
        Ok(false)
    }
    fn matches_surfaces(
        &self,
        point: &IntSurfPntOn2S,
        original: &IntSurfPntOn2S,
        candidate: Point2,
        surface: usize,
    ) -> Result<bool, Failure> {
        let mut trial = *point;
        trial.set_parameters_on(surface == 0, candidate.x(), candidate.y());
        let (u1, v1) = trial.parameters_on_s1();
        let (u2, v2) = trial.parameters_on_s2();
        let a = self.surfaces[0].value(u1, v1)?;
        let b = self.surfaces[1].value(u2, v2)?;
        let tolerance = 0.5 * precision::confusion();
        Ok(original.value().is_equal(&a, tolerance)
            && original.value().is_equal(&b, tolerance)
            && a.is_equal(&b, tolerance))
    }
}
fn seam_choice(
    uv: Point2,
    neighbour: Point2,
    bound: &BoundaryAxis,
    state: &BoundaryState,
) -> SeamChoice {
    let value = coordinate(uv, state.u_axis);
    let alternate = reflected_parameter(value, bound, state.first_bound);
    let neighbour_value = coordinate(neighbour, state.u_axis);
    let d1 = (neighbour_value - value).abs();
    let d2 = (neighbour_value - alternate).abs();
    let quarter = bound.period.unwrap() / 4.;
    let switched = d1 - d2 > precision::p_confusion() && d2 < quarter;
    let candidate = if switched {
        replace_coordinate(uv, alternate, state.u_axis)
    } else {
        uv
    };
    SeamChoice {
        neighbour,
        candidate,
        vector: Vector2::between(neighbour, candidate),
        switched,
        check_angle: switched || d1 < quarter,
    }
}

fn coordinate(point: Point2, u_axis: bool) -> f64 {
    if u_axis {
        return point.x();
    }
    point.y()
}
fn replace_coordinate(point: Point2, value: f64, u_axis: bool) -> Point2 {
    if u_axis {
        return Point2::new(value, point.y());
    }
    Point2::new(point.x(), value)
}
fn reflected_parameter(value: f64, bound: &BoundaryAxis, first: bool) -> f64 {
    let (adjusted, offset) = bound.adjusted(value);
    let distance = if first {
        (adjusted - bound.lower).abs()
    } else {
        (adjusted - bound.upper).abs()
    };
    let alternate = if first {
        bound.upper - distance
    } else {
        bound.lower + distance
    };
    alternate + offset
}
