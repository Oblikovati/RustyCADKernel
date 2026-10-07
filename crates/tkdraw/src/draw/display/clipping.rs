//! `CalculRegion` and `Trim`: the segment clipping of `Draw_Display::DrawTo`.
//!
//! `Trim` is not an exact clipper: it rejects a segment whose end points lie on the same
//! outer side of the frame, rejects one whose closest point to the frame centre is
//! farther than twice the frame diagonal, and otherwise shortens the far ends to that
//! distance (so that coordinates fit the 16-bit window segments). The floating-point
//! operations keep the C++ order.
use rustycad_tkmath::gp::Point2;

/// `CalculRegion`: the Cohen-Sutherland outcode of `(x, y)` against the frame
/// `[x1, x2] x [y1, y2]` (1 left, 2 right, 4 below, 8 above).
fn region(x: f64, y: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> i32 {
    let horizontal = if x < x1 {
        1
    } else if x > x2 {
        2
    } else {
        0
    };
    let vertical = if y < y1 {
        4
    } else if y > y2 {
        8
    } else {
        0
    };
    horizontal | vertical
}

/// `Trim(P1, P2, x0, y0, x1, y1)`: false when the segment is rejected; otherwise the
/// points may have been moved closer to the frame.
///
/// ```
/// use rustycad_tkdraw::draw::trim_segment;
/// use rustycad_tkmath::gp::Point2;
/// let (mut a, mut b) = (Point2::new(-5.0, 1.0), Point2::new(-2.0, 1.0));
/// assert!(!trim_segment(&mut a, &mut b, [0.0, 0.0, 10.0, 10.0]));
/// ```
pub fn trim_segment(p1: &mut Point2, p2: &mut Point2, frame: [f64; 4]) -> bool {
    let [x0, y0, x1, y1] = frame;
    let (xa, ya, xb, yb) = (p1.x(), p1.y(), p2.x(), p2.y());
    if region(xa, ya, x0, y0, x1, y1) & region(xb, yb, x0, y0, x1, y1) != 0 {
        return false;
    }
    let (mut dx, mut dy) = (xb - xa, yb - ya);
    let dab = (dx * dx + dy * dy).sqrt();
    if dab < 1e-10 {
        return false;
    }
    dx /= dab;
    dy /= dab;
    let (frame_x, frame_y) = (0.5 * (x0 + x1), 0.5 * (y0 + y1));
    let (width, height) = (x1 - x0, y1 - y0);
    let d = (width * width + height * height).sqrt() * 2.0;
    let p = (frame_x - xa) * dx + (frame_y - ya) * dy;
    let closest = Point2::new(xa + p * dx, ya + p * dy);
    if Point2::new(frame_x, frame_y).square_distance(closest) > d * d {
        return false;
    }
    shorten_ends(p1, p2, closest, [dx, dy, d]);
    true
}

/// Moves the end points that lie farther than `d` from `closest` (the foot of the frame
/// centre on the segment's line) back to distance `d`, as the three C++ cases do.
fn shorten_ends(p1: &mut Point2, p2: &mut Point2, closest: Point2, [dx, dy, d]: [f64; 3]) {
    let (xm, ym) = (closest.x(), closest.y());
    let (xa, ya, xb, yb) = (p1.x(), p1.y(), p2.x(), p2.y());
    let to_p1 = closest.distance(*p1);
    let to_p2 = closest.distance(*p2);
    let amab = (xm - xa) * (xb - xa) + (ym - ya) * (yb - ya);
    let move_p1 = |p1: &mut Point2| p1.set_coordinates(xm - d * dx, ym - d * dy);
    let move_p2 = |p2: &mut Point2| p2.set_coordinates(xm + d * dx, ym + d * dy);
    if amab > 0.0 {
        // The foot lies between the end points.
        if to_p1 > d {
            move_p1(p1);
        }
        if to_p2 > d {
            move_p2(p2);
        }
    } else if to_p1 < to_p2 {
        // The foot lies before P1: only P2 can be too far.
        if to_p2 > d {
            move_p2(p2);
        }
    } else if to_p1 > d {
        // The foot lies after P2: only P1 can be too far.
        move_p1(p1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcodes_follow_the_frame_sides() {
        assert_eq!(region(-1.0, 5.0, 0.0, 0.0, 10.0, 10.0), 1);
        assert_eq!(region(11.0, 11.0, 0.0, 0.0, 10.0, 10.0), 10);
        assert_eq!(region(5.0, -1.0, 0.0, 0.0, 10.0, 10.0), 4);
    }

    #[test]
    fn a_crossing_segment_is_kept_and_far_ends_are_shortened() {
        let (mut a, mut b) = (Point2::new(-1e6, 5.0), Point2::new(1e6, 5.0));
        assert!(trim_segment(&mut a, &mut b, [0.0, 0.0, 10.0, 10.0]));
        let d = (200.0f64).sqrt() * 2.0;
        assert_eq!(a.x(), 5.0 - d);
        assert_eq!(b.x(), 5.0 + d);
    }

    #[test]
    fn a_degenerate_segment_is_rejected() {
        let (mut a, mut b) = (Point2::new(5.0, 5.0), Point2::new(5.0, 5.0));
        assert!(!trim_segment(&mut a, &mut b, [0.0, 0.0, 10.0, 10.0]));
    }
}
