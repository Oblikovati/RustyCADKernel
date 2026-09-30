// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT/MAT_Edge.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MAT_Edge`: an element of the contour in the MAT algorithm, between two bisectors.
use super::{MatBisector, MatTListItem};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};

struct EdgeState {
    edge_number: i32,
    first_bisector: Handle<MatBisector>,
    second_bisector: Handle<MatBisector>,
    distance: f64,
    intersection_point: i32,
}

/// `MAT_Edge`: a transient edge of the MAT algorithm, with the bisectors on each side
/// and the intersection of those bisectors (its point index and distance). An edge
/// and its bisectors reference each other with strong handles, as natively;
/// `MAT2d_Mat2d` breaks those cycles when it is dropped.
/// ```
/// use rustycad_tktopalgo::mat::MatEdge;
/// let edge = MatEdge::new();
/// edge.set_edge_number(4);
/// assert_eq!((edge.edge_number(), edge.distance()), (4, 0.0));
/// ```
pub struct MatEdge {
    parent: StandardTransient,
    state: SharedState<EdgeState>,
}
impl_transient!(MatEdge, "MAT_Edge", StandardTransient, parent);

impl Default for MatEdge {
    fn default() -> Self {
        Self::new()
    }
}

impl MatEdge {
    /// `MAT_Edge()`: number 0, distance 0, intersection point 0; `MatEdge::new()`.
    pub fn new() -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(EdgeState {
                edge_number: 0,
                first_bisector: Handle::default(),
                second_bisector: Handle::default(),
                distance: 0.0,
                intersection_point: 0,
            }),
        }
    }
    /// `EdgeNumber(anumber)`; `edge.set_edge_number(1)`.
    pub fn set_edge_number(&self, anumber: i32) {
        self.state.write().edge_number = anumber;
    }
    /// `FirstBisector(abisector)`; `edge.set_first_bisector(&b)`.
    pub fn set_first_bisector(&self, abisector: &Handle<MatBisector>) {
        self.state.write().first_bisector = abisector.clone();
    }
    /// `SecondBisector(abisector)`; `edge.set_second_bisector(&b)`.
    pub fn set_second_bisector(&self, abisector: &Handle<MatBisector>) {
        self.state.write().second_bisector = abisector.clone();
    }
    /// `Distance(adistance)`; `edge.set_distance(1.0)`.
    pub fn set_distance(&self, adistance: f64) {
        self.state.write().distance = adistance;
    }
    /// `IntersectionPoint(apoint)`; `edge.set_intersection_point(3)`.
    pub fn set_intersection_point(&self, apoint: i32) {
        self.state.write().intersection_point = apoint;
    }
    /// `EdgeNumber()`: the index of the element; `edge.edge_number()`.
    pub fn edge_number(&self) -> i32 {
        self.state.read().edge_number
    }
    /// `FirstBisector()`; `edge.first_bisector()`.
    pub fn first_bisector(&self) -> Handle<MatBisector> {
        self.state.read().first_bisector.clone()
    }
    /// `SecondBisector()`; `edge.second_bisector()`.
    pub fn second_bisector(&self) -> Handle<MatBisector> {
        self.state.read().second_bisector.clone()
    }
    /// `Distance()`: the distance of the intersection of the bisectors;
    /// `edge.distance()`.
    pub fn distance(&self) -> f64 {
        self.state.read().distance
    }
    /// `IntersectionPoint()`: the index of that intersection; `edge.intersection_point()`.
    pub fn intersection_point(&self) -> i32 {
        self.state.read().intersection_point
    }
    /// `Dump(ashift, alevel)`: prints nothing natively; `edge.dump(0, 0, &mut text)`.
    pub fn dump(&self, _ashift: i32, _alevel: i32, _out: &mut String) {}
}

impl MatTListItem for MatEdge {
    const LIST_NAME: &'static str = "MAT_ListOfEdge";
    const NODE_NAME: &'static str = "MAT_TListNodeOfListOfEdge";
    fn dump_item(&self, ashift: i32, alevel: i32, out: &mut String) -> Result<(), Failure> {
        self.dump(ashift, alevel, out);
        Ok(())
    }
}
