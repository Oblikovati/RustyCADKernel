//! The `PICK` branch of `Draw_Display::DrawTo`: does the screen segment `p1 p2` pass
//! within `precision` pixels of the pick point? Records the hit and its parameter.
use super::state::PickState;
use rustycad_tkmath::gp::Point2;

/// Integer pixel segment and the pick query, as the C++ casts them.
struct PixelSegment {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

/// Runs the hit test of one segment when nothing is found yet.
pub(super) fn pick_segment(pick: &mut PickState, p1: Point2, p2: Point2) {
    if pick.found {
        return;
    }
    let segment = PixelSegment {
        x1: p1.x() as i32,
        y1: p1.y() as i32,
        x2: p2.x() as i32,
        y2: p2.y() as i32,
    };
    if entirely_outside(pick, &segment) {
        return;
    }
    let Some(inside) = crosses_window_edge(pick, &segment) else {
        return;
    };
    pick.found = pick.found || inside;
    if pick.found {
        pick.last_param = dominant_parameter(pick, &segment);
    }
}

/// True when both end points lie beyond the same side of the pick window.
fn entirely_outside(pick: &PickState, s: &PixelSegment) -> bool {
    let (x_high, x_low) = (pick.x + pick.precision, pick.x - pick.precision);
    let (y_high, y_low) = (pick.y + pick.precision, pick.y - pick.precision);
    (s.x1 >= x_high && s.x2 >= x_high)
        || (s.x1 <= x_low && s.x2 <= x_low)
        || (s.y1 >= y_high && s.y2 >= y_high)
        || (s.y1 <= y_low && s.y2 <= y_low)
}

/// Tests the four window edges in the C++ order. `None` means a hit was recorded
/// (the C++ `break`); otherwise whether the segment stays inside the window.
fn crosses_window_edge(pick: &mut PickState, s: &PixelSegment) -> Option<bool> {
    let mut inside = true;
    let precision = pick.precision;
    for (crosses, edge) in [
        (
            s.x1 > pick.x + precision || s.x2 > pick.x + precision,
            pick.x + precision,
        ),
        (
            s.x1 < pick.x - precision || s.x2 < pick.x - precision,
            pick.x - precision,
        ),
    ] {
        if !crosses {
            continue;
        }
        let y = s.y1 as f64 + (s.y2 - s.y1) as f64 * (edge - s.x1) as f64 / (s.x2 - s.x1) as f64;
        if y < (pick.y + precision) as f64 && y > (pick.y - precision) as f64 {
            record_hit(pick, (pick.x - s.x1) as f64 / (s.x2 - s.x1) as f64);
            return None;
        }
        inside = false;
    }
    crosses_horizontal_edges(pick, s, inside)
}

/// The two horizontal edges of `crosses_window_edge`.
fn crosses_horizontal_edges(
    pick: &mut PickState,
    s: &PixelSegment,
    mut inside: bool,
) -> Option<bool> {
    let precision = pick.precision;
    for (crosses, edge) in [
        (
            s.y1 > pick.y + precision || s.y2 > pick.y + precision,
            pick.y + precision,
        ),
        (
            s.y1 < pick.y - precision || s.y2 < pick.y - precision,
            pick.y - precision,
        ),
    ] {
        if !crosses {
            continue;
        }
        let x = s.x1 as f64 + (s.x2 - s.x1) as f64 * (edge - s.y1) as f64 / (s.y2 - s.y1) as f64;
        if x < (pick.x + precision) as f64 && x > (pick.x - precision) as f64 {
            record_hit(pick, (pick.y - s.y1) as f64 / (s.y2 - s.y1) as f64);
            return None;
        }
        inside = false;
    }
    Some(inside)
}

/// Marks the pick as found at parameter `param`.
fn record_hit(pick: &mut PickState, param: f64) {
    pick.found = true;
    pick.last_param = param;
}

/// The parameter along the longer coordinate direction (0 for a near-zero extent).
fn dominant_parameter(pick: &PickState, s: &PixelSegment) -> f64 {
    let (dx, dy) = ((s.x2 - s.x1).abs(), (s.y2 - s.y1).abs());
    if dx > dy {
        // The C++ compares the integer extent with 1e-5, i.e. tests for zero.
        return if (dx as f64) < 1e-5 {
            0.0
        } else {
            (pick.x - s.x1) as f64 / (s.x2 - s.x1) as f64
        };
    }
    if (dy as f64) < 1e-5 {
        0.0
    } else {
        (pick.y - s.y1) as f64 / (s.y2 - s.y1) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query() -> PickState {
        PickState {
            x: 10,
            y: 10,
            precision: 3,
            ..PickState::default()
        }
    }

    #[test]
    fn a_segment_through_the_window_is_found() {
        let mut pick = query();
        pick_segment(&mut pick, Point2::new(0.0, 10.0), Point2::new(20.0, 10.0));
        assert!(pick.found);
        assert_eq!(pick.last_param, 0.5);
    }

    #[test]
    fn a_far_segment_is_missed() {
        let mut pick = query();
        pick_segment(&mut pick, Point2::new(0.0, 0.0), Point2::new(20.0, 0.0));
        assert!(!pick.found);
    }

    #[test]
    fn a_segment_inside_the_window_uses_the_dominant_axis() {
        let mut pick = query();
        pick_segment(&mut pick, Point2::new(9.0, 9.0), Point2::new(11.0, 10.0));
        assert!(pick.found);
        assert_eq!(pick.last_param, 0.5);
    }
}
