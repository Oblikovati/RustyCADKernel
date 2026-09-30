use super::*;
#[derive(Clone, Copy)]
pub(super) struct BoundaryAxis {
    pub lower: f64,
    pub upper: f64,
    pub resolution: f64,
    pub period: Option<f64>,
}
impl BoundaryAxis {
    pub fn new(surface: &GeomAdaptorSurface, axis: usize, tolerance: f64) -> Result<Self, Failure> {
        let bounds = surface.surface().bounds();
        let (lower, upper) = if axis == 0 {
            (bounds.0, bounds.1)
        } else {
            (bounds.2, bounds.3)
        };
        let resolution = if axis == 0 {
            surface.u_resolution(tolerance)?
        } else {
            surface.v_resolution(tolerance)?
        };
        let period = axis_period(surface, axis)?;
        Ok(Self {
            lower,
            upper,
            resolution,
            period,
        })
    }
    pub fn adjusted(&self, parameter: f64) -> (f64, f64) {
        let Some(period) = self.period else {
            return (parameter, 0.);
        };
        let (_, value, offset) = adjust_periodic(parameter, self.lower, self.upper, period, 0.);
        (value, offset)
    }
    pub fn location(&self, parameter: f64, tolerance: f64) -> Option<bool> {
        if (parameter - self.lower).abs() < tolerance {
            return Some(true);
        }
        if (parameter - self.upper).abs() < tolerance {
            return Some(false);
        }
        None
    }
}
pub(super) fn point_on_boundary(
    point: IntSurfPntOn2S,
    surfaces: &[&Handle<GeomAdaptorSurface>; 2],
) -> Result<bool, Failure> {
    for (index, surface) in surfaces.iter().enumerate() {
        let uv = point.value_on_surface(index == 0);
        for axis in 0..2 {
            if !is_periodic(surface, axis)? {
                continue;
            }
            let bound = BoundaryAxis::new(surface, axis, 0.5 * precision::confusion())?;
            let parameter = if axis == 0 { uv.x() } else { uv.y() };
            if bound
                .location(bound.adjusted(parameter).0, bound.resolution)
                .is_some()
            {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
pub(super) fn is_periodic(surface: &GeomAdaptorSurface, axis: usize) -> Result<bool, Failure> {
    if axis == 0 {
        return surface.is_u_periodic();
    }
    surface.is_v_periodic()
}
pub(super) fn adjust_by_neighbour(
    neighbour: Point2,
    original: Point2,
    surface: &GeomAdaptorSurface,
) -> Result<Point2, Failure> {
    let mut adjusted = original;
    if surface.is_u_periodic()? {
        adjust_axis(neighbour, original, &mut adjusted, surface.u_period()?, 0);
    }
    if surface.is_v_periodic()? {
        adjust_axis(neighbour, original, &mut adjusted, surface.v_period()?, 1);
    }
    Ok(adjusted)
}
fn adjust_axis(
    neighbour: Point2,
    original: Point2,
    adjusted: &mut Point2,
    period: f64,
    axis: usize,
) {
    let mut candidate = *adjusted;
    let mut best = 1e100;
    for index in -1..=1 {
        if axis == 0 {
            candidate.set_x(original.x() + period * f64::from(index));
        } else {
            candidate.set_y(original.y() + period * f64::from(index));
        }
        let distance = neighbour.square_distance(candidate);
        if distance < best {
            *adjusted = candidate;
            best = distance;
        }
    }
}

fn axis_period(surface: &GeomAdaptorSurface, axis: usize) -> Result<Option<f64>, Failure> {
    if !is_periodic(surface, axis)? {
        return Ok(None);
    }
    Ok(Some(if axis == 0 {
        surface.u_period()?
    } else {
        surface.v_period()?
    }))
}
