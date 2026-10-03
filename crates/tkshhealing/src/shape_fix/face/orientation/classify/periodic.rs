use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::gp::Point2;
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;

#[derive(Clone, Copy)]
pub(super) struct PeriodicRanges {
    pub(super) u: Option<f64>,
    pub(super) v: Option<f64>,
}

pub(super) fn classify_vertex_periodically(
    point: Point2,
    periods: PeriodicRanges,
    classifier: &BRepTopAdaptorFClass2d,
    infinite: State,
) -> Result<State, Failure> {
    let mut state = classifier.perform(point, false)?;
    if state == infinite {
        state = positive_period_copy(point, periods.u, true, classifier)?;
    }
    if state == infinite {
        state = positive_period_copy(point, periods.v, false, classifier)?;
    }
    Ok(state)
}

fn positive_period_copy(
    point: Point2,
    period: Option<f64>,
    u_axis: bool,
    classifier: &BRepTopAdaptorFClass2d,
) -> Result<State, Failure> {
    let Some(period) = period else {
        return Ok(State::Unknown);
    };
    classifier.perform(shifted_point(point, period, u_axis), false)
}

fn shifted_point(point: Point2, period: f64, u_axis: bool) -> Point2 {
    if u_axis {
        return Point2::new(point.x() + period, point.y());
    }
    Point2::new(point.x(), point.y() + period)
}

pub(super) fn periodic_copy(
    point: Point2,
    periods: PeriodicRanges,
    classifier: &BRepTopAdaptorFClass2d,
    infinite: State,
) -> Result<Option<(State, Point2)>, Failure> {
    for shifted in periodic_candidates(point, periods) {
        let state = classifier.perform(shifted, false)?;
        if state != infinite {
            return Ok(Some((state, shifted)));
        }
    }
    Ok(None)
}

fn periodic_candidates(point: Point2, periods: PeriodicRanges) -> Vec<Point2> {
    let mut candidates = Vec::new();
    append_axis_candidates(point, periods.u, true, &mut candidates);
    append_axis_candidates(point, periods.v, false, &mut candidates);
    append_diagonal_candidates(point, periods, &mut candidates);
    candidates
}

fn append_axis_candidates(
    point: Point2,
    period: Option<f64>,
    u_axis: bool,
    result: &mut Vec<Point2>,
) {
    let Some(period) = period else {
        return;
    };
    for sign in [1.0, -1.0] {
        result.push(shifted_point(point, sign * period, u_axis));
    }
}

fn append_diagonal_candidates(point: Point2, periods: PeriodicRanges, result: &mut Vec<Point2>) {
    let (Some(u), Some(v)) = (periods.u, periods.v) else {
        return;
    };
    for us in [-1.0, 1.0] {
        for vs in [-1.0, 1.0] {
            result.push(Point2::new(point.x() + us * u, point.y() + vs * v));
        }
    }
}

pub(super) fn toggle_inside_outside(state: State) -> State {
    if state == State::In {
        return State::Out;
    }
    State::In
}
