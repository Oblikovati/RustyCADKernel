// Rust translation of BRepMesh_Classifier.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepMesh_Classifier`: point-in-face classification against the registered wires.
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::cslib::{Class2d, Class2dResult};
use rustycad_tkmath::gp::{Point2, Vector2};
use std::f64::consts::PI;

/// The registered wires: one CSLib classifier each and whether its accumulated turning
/// angle is not negative (an outer, counter-clockwise wire).
#[derive(Default)]
struct ClassifierWires {
    tab_class: Vec<Class2d>,
    tab_orient: Vec<bool>,
}

/// `BRepMesh_Classifier`: a point is IN when no registered wire rejects it; a point ON
/// a wire counts as OUT.
///
/// ```
/// use rustycad_tkg3d::top_abs::State;
/// use rustycad_tkmath::gp::Point2;
/// use rustycad_tkmesh::brep_mesh::BRepMeshClassifier;
/// let square = [Point2::new(0.0, 0.0), Point2::new(1.0, 0.0), Point2::new(1.0, 1.0), Point2::new(0.0, 1.0)];
/// let classifier = BRepMeshClassifier::new();
/// classifier.register_wire(&square, (1e-7, 1e-7), (0.0, 1.0), (0.0, 1.0))?;
/// assert_eq!(classifier.perform(Point2::new(0.5, 0.5)), State::In);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Default)]
pub struct BRepMeshClassifier {
    transient: StandardTransient,
    wires: SharedState<ClassifierWires>,
}
impl_transient!(
    BRepMeshClassifier,
    "BRepMesh_Classifier",
    StandardTransient,
    transient
);

impl BRepMeshClassifier {
    /// `BRepMesh_Classifier()`: no wire; `BRepMeshClassifier::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Perform(thePoint)`: OUT as soon as one wire classifies the point outside (or on
    /// it), IN otherwise; `classifier.perform(point)`.
    pub fn perform(&self, point: Point2) -> State {
        let wires = self.wires.read();
        for (class, orient) in wires.tab_class.iter().zip(&wires.tab_orient) {
            let current = class.si_dans(point);
            let is_out = match current {
                Class2dResult::Uncertain => true,
                _ if *orient => current == Class2dResult::Outside,
                _ => current == Class2dResult::Inside,
            };
            if is_out {
                return State::Out;
            }
        }
        State::In
    }

    /// `RegisterWire(theWire, theTolUV, theRangeU, theRangeV)`: a wire of fewer than two
    /// points is ignored; otherwise its classifier and orientation are appended;
    /// `classifier.register_wire(&points, (tol_u, tol_v), range_u, range_v)?`.
    pub fn register_wire(
        &self,
        wire: &[Point2],
        tol_uv: (f64, f64),
        range_u: (f64, f64),
        range_v: (f64, f64),
    ) -> Result<(), Failure> {
        if wire.len() < 2 {
            return Ok(());
        }
        let (angle, points) = accumulate_angle(wire)?;
        let class = Class2d::from_array(
            &points, tol_uv.0, tol_uv.1, range_u.0, range_v.0, range_u.1, range_v.1,
        );
        let mut wires = self.wires.write();
        wires.tab_class.push(class);
        wires
            .tab_orient
            .push(angle.partial_cmp(&0.0) != Some(std::cmp::Ordering::Less));
        Ok(())
    }
}

/// The native turning-angle loop of RegisterWire: the signed angles between successive
/// non-degenerate, non-collinear segments (wrapping to the first two points), zeroed
/// below Precision::Angular(); also returns the points copied into a 1-based array.
fn accumulate_angle(wire: &[Point2]) -> Result<(f64, Array1<'static, Point2>), Failure> {
    let count = wire.len();
    let mut points: Array1<'static, Point2> =
        Array1::with_bounds(1, count as i32).map_err(|error| {
            Failure::with_message(
                "Standard_RangeError",
                &format!("BRepMesh_Classifier::RegisterWire: {error}; expected {count} points"),
            )
        })?;
    points[1] = wire[0];
    points[2] = wire[1];
    let (mut p1, mut p2) = (wire[0], wire[1]);
    let mut angle = 0.0;
    for i in 1..=count {
        let ii = i + 2;
        let p3 = if ii > count {
            points[(ii - count) as i32]
        } else {
            points[ii as i32] = wire[ii - 1];
            wire[ii - 1]
        };
        if let Some(turn) = turning_angle(p1, p2, p3)? {
            angle += turn;
            p1 = p2;
        }
        p2 = p3;
    }
    if angle.abs() < precision::angular() {
        angle = 0.0;
    }
    Ok((angle, points))
}

/// The signed angle from `p1 -> p2` to `p2 -> p3` when both segments are longer than
/// Precision::PConfusion() and neither collinear nor opposite within
/// Precision::Angular().
fn turning_angle(p1: Point2, p2: Point2, p3: Point2) -> Result<Option<f64>, Failure> {
    let (a, b) = (Vector2::between(p1, p2), Vector2::between(p2, p3));
    let sq_confusion = precision::p_confusion() * precision::p_confusion();
    if !(a.square_magnitude() > sq_confusion && b.square_magnitude() > sq_confusion) {
        return Ok(None);
    }
    let current = a.angle(b)?;
    let current_abs = current.abs();
    let tolerance = precision::angular();
    Ok((current_abs > tolerance && (PI - current_abs) > tolerance).then_some(current))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(clockwise: bool) -> Vec<Point2> {
        let mut points = vec![
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
            Point2::new(1.0, 1.0),
            Point2::new(0.0, 1.0),
        ];
        if clockwise {
            points.reverse();
        }
        points
    }

    #[test]
    fn outer_wire_keeps_inner_points() {
        let classifier = BRepMeshClassifier::new();
        classifier
            .register_wire(&square(false), (1e-7, 1e-7), (0.0, 1.0), (0.0, 1.0))
            .unwrap();
        assert_eq!(classifier.perform(Point2::new(0.5, 0.5)), State::In);
        assert_eq!(classifier.perform(Point2::new(2.0, 0.5)), State::Out);
        assert_eq!(classifier.perform(Point2::new(1.0, 0.5)), State::Out);
    }

    #[test]
    fn clockwise_wire_is_a_hole() {
        let classifier = BRepMeshClassifier::new();
        classifier
            .register_wire(&square(true), (1e-7, 1e-7), (0.0, 1.0), (0.0, 1.0))
            .unwrap();
        assert_eq!(classifier.perform(Point2::new(0.5, 0.5)), State::Out);
    }

    #[test]
    fn short_wire_is_ignored_and_turn_sum_is_signed() {
        let classifier = BRepMeshClassifier::new();
        classifier
            .register_wire(
                &[Point2::new(0.0, 0.0)],
                (1e-7, 1e-7),
                (0.0, 1.0),
                (0.0, 1.0),
            )
            .unwrap();
        assert_eq!(classifier.perform(Point2::new(5.0, 5.0)), State::In);
        assert!(accumulate_angle(&square(false)).unwrap().0 > 0.0);
        assert!(accumulate_angle(&square(true)).unwrap().0 < 0.0);
    }
}
