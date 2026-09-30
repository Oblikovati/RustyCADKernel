// Rust translation of ModelingAlgorithms/TKHLR/TopBas/TopBas_TestInterference.hxx and TopBas_TestInterference_0.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! TopBas_TestInterference: one interference of a curve with a boundary.
use rustycad_tkg3d::top_abs::Orientation;

/// TopBas_TestInterference: an intersection parameter on a curve, the index of the
/// boundary met there, the orientation of the point on the boundary, the transition of
/// the curve and the boundary transition.
///
/// The native default constructor (`= default`) leaves every field uninitialized; the
/// Rust default is `0.0`, `0` and FORWARD (docs/hlr-design.md section 4).
///
/// ```
/// use rustycad_tkg3d::top_abs::Orientation;
/// use rustycad_tkhlr::top_bas::TestInterference;
/// let mut interference = TestInterference::with_values(
///     0.5, 2, Orientation::Forward, Orientation::Reversed, Orientation::Internal);
/// *interference.change_boundary() += 1;
/// assert_eq!((interference.intersection(), interference.boundary()), (0.5, 3));
/// assert_eq!(interference.transition(), Orientation::Reversed);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TestInterference {
    intersection: f64,
    boundary: i32,
    orientation: Orientation,
    transition: Orientation,
    b_transition: Orientation,
}

impl Default for TestInterference {
    fn default() -> Self {
        Self::new()
    }
}

impl TestInterference {
    /// TopBas_TestInterference(): the zero record (see the type documentation);
    /// `TestInterference::new()`.
    pub fn new() -> Self {
        Self::with_values(
            0.0,
            0,
            Orientation::Forward,
            Orientation::Forward,
            Orientation::Forward,
        )
    }

    /// TopBas_TestInterference(Inters, Bound, Orient, Trans, BTrans): the record of
    /// every value; `TestInterference::with_values(0.5, 1, o, t, b)`.
    pub fn with_values(
        intersection: f64,
        boundary: i32,
        orientation: Orientation,
        transition: Orientation,
        b_transition: Orientation,
    ) -> Self {
        Self {
            intersection,
            boundary,
            orientation,
            transition,
            b_transition,
        }
    }

    /// Intersection(I): sets the intersection parameter; `record.set_intersection(0.5)`.
    pub fn set_intersection(&mut self, intersection: f64) {
        self.intersection = intersection;
    }

    /// Boundary(B): sets the boundary index; `record.set_boundary(2)`.
    pub fn set_boundary(&mut self, boundary: i32) {
        self.boundary = boundary;
    }

    /// Orientation(O): sets the orientation of the point on the boundary;
    /// `record.set_orientation(Orientation::Reversed)`.
    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }

    /// Transition(Tr): sets the transition of the curve;
    /// `record.set_transition(Orientation::Forward)`.
    pub fn set_transition(&mut self, transition: Orientation) {
        self.transition = transition;
    }

    /// BoundaryTransition(BTr): sets the boundary transition;
    /// `record.set_boundary_transition(Orientation::Internal)`.
    pub fn set_boundary_transition(&mut self, b_transition: Orientation) {
        self.b_transition = b_transition;
    }

    /// Intersection(): the intersection parameter; `record.intersection()`.
    pub fn intersection(&self) -> f64 {
        self.intersection
    }

    /// ChangeIntersection: the intersection parameter, writable in place;
    /// `*record.change_intersection() = 1.0`.
    pub fn change_intersection(&mut self) -> &mut f64 {
        &mut self.intersection
    }

    /// Boundary(): the boundary index; `record.boundary()`.
    pub fn boundary(&self) -> i32 {
        self.boundary
    }

    /// ChangeBoundary: the boundary index, writable in place;
    /// `*record.change_boundary() += 1`.
    pub fn change_boundary(&mut self) -> &mut i32 {
        &mut self.boundary
    }

    /// Orientation(): the orientation of the point on the boundary; `record.orientation()`.
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    /// Transition(): the transition of the curve; `record.transition()`.
    pub fn transition(&self) -> Orientation {
        self.transition
    }

    /// BoundaryTransition(): the boundary transition; `record.boundary_transition()`.
    pub fn boundary_transition(&self) -> Orientation {
        self.b_transition
    }
}
