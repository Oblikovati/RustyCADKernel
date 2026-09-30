// Rust translation of TopClass_FaceClassifier.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopClass_FaceClassifier`: the state of a 2d point against a face, from the
//! classification segments that its explorer proposes.
use super::{Classifier2d, Classifier2dIntersector, FaceExplorer};
use crate::int_res2d::{IntersectionPoint, Position, domain_error};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkmath::gp::{Line2, Point2};

/// `TopClass_FaceClassifier` (the fields of its instances, such as
/// `Geom2dHatch_Classifier`, and the template functions as methods): classifies a
/// point by the [`Classifier2d`] of the segments that the face explorer gives, until
/// one decides the state away from an edge end.
///
/// The position starts at `IntRes2d_Head`: natively it is uninitialized until a
/// classification finds a closest edge.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve};
/// use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
/// use rustycad_tkg3d::top_abs::{Orientation, State};
/// use rustycad_tkgeomalgo::geom2d_hatch::{Classifier, Element, Elements};
/// use rustycad_tkmath::gp::{Axis2d, Point2};
/// let circle = Geom2dCircle::from_axis(Axis2d::default(), 1.0, true).unwrap();
/// let curve: Handle<dyn Geom2dCurve> = Handle::new(circle).cast();
/// let mut disc = Elements::new();
/// let edge = Geom2dAdaptorCurve::with_curve(&curve).unwrap();
/// disc.bind(1, Element::new(edge, Orientation::Forward));
/// let inside = Classifier::with_point(&mut disc, &Point2::new(0.2, 0.1), 1e-7).unwrap();
/// assert_eq!(inside.state(), State::In);
/// let outside = Classifier::with_point(&mut disc, &Point2::new(2.0, 0.1), 1e-7).unwrap();
/// assert_eq!(outside.state(), State::Out);
/// ```
pub struct FaceClassifier<X: FaceExplorer> {
    my_classifier: Classifier2d<X::Intersector>,
    my_edge: X::Edge,
    my_edge_parameter: f64,
    my_position: Position,
    rejected: bool,
    nowires: bool,
}

impl<X: FaceExplorer> Default for FaceClassifier<X> {
    /// The undefined algorithm: not rejected, without wires, edge parameter 0.
    fn default() -> Self {
        Self {
            my_classifier: Classifier2d::new(),
            my_edge: X::Edge::default(),
            my_edge_parameter: 0.0,
            my_position: Position::Head,
            rejected: false,
            nowires: true,
        }
    }
}

// The end of the scan of the wires: the point is ON an edge or OUT of a wire.
struct Decided;

impl<X: FaceExplorer> FaceClassifier<X> {
    /// The undefined algorithm (`Geom2dHatch_Classifier()`); `Classifier::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// The classification of `point` with tolerance `tolerance` on the face that
    /// `explorer` describes (`Geom2dHatch_Classifier(F, P, Tol)`);
    /// `Classifier::with_point(&mut elements, &point, 1e-7)?`.
    pub fn with_point(explorer: &mut X, point: &Point2, tolerance: f64) -> Result<Self, Failure> {
        let mut classifier = Self::new();
        classifier.perform(explorer, point, tolerance)?;
        Ok(classifier)
    }

    /// `Perform(F, P, Tol)`: classifies `point` with tolerance `tolerance` on the face
    /// that `explorer` describes; the explorer's and the intersector's exceptions are
    /// returned; `classifier.perform(&mut elements, &point, 1e-7)?`.
    pub fn perform(
        &mut self,
        explorer: &mut X,
        point: &Point2,
        tolerance: f64,
    ) -> Result<(), Failure> {
        let mut a_point: Point2 = *point;
        while !explorer.check_point(&mut a_point) {}
        self.rejected = explorer.reject(&a_point);
        if self.rejected {
            return Ok(());
        }
        let (mut valid, mut line, mut param) = explorer.segment(&a_point)?;
        let mut state: State = State::Unknown;
        self.nowires = true;
        while valid {
            self.my_classifier.reset(line, param, tolerance);
            if self.scan_wires(explorer, &line, &mut state)?.is_some() {
                return Ok(());
            }
            if !self.my_classifier.is_head_or_end() && state != State::Unknown {
                break;
            }
            // Bad case for classification: try another segment.
            (valid, line, param) = explorer.other_segment(&a_point)?;
        }
        Ok(())
    }

    // The wires of one segment; Some when the point is ON an edge or OUT of a wire.
    fn scan_wires(
        &mut self,
        explorer: &mut X,
        line: &Line2,
        state: &mut State,
    ) -> Result<Option<Decided>, Failure> {
        explorer.init_wires();
        while explorer.more_wires() {
            self.nowires = false;
            if !explorer.reject_wire(line, self.my_classifier.parameter()) {
                if self.scan_edges(explorer, line, state)?.is_some() {
                    return Ok(Some(Decided));
                }
                *state = self.my_classifier.state();
                if *state == State::Out {
                    return Ok(Some(Decided));
                }
            }
            explorer.next_wire();
        }
        Ok(None)
    }

    // The edges of the current wire; Some when the point is ON an edge.
    fn scan_edges(
        &mut self,
        explorer: &mut X,
        line: &Line2,
        state: &mut State,
    ) -> Result<Option<Decided>, Failure> {
        explorer.init_edges();
        while explorer.more_edges() {
            if !explorer.reject_edge(line, self.my_classifier.parameter())
                && self.compare_edge(explorer, state)?.is_some()
            {
                return Ok(Some(Decided));
            }
            explorer.next_edge();
        }
        Ok(None)
    }

    // Compares the current edge when it is FORWARD or REVERSED and keeps it when it
    // holds the closest intersection; Some when the point is ON it.
    fn compare_edge(
        &mut self,
        explorer: &mut X,
        state: &mut State,
    ) -> Result<Option<Decided>, Failure> {
        let (edge, orientation) = explorer.current_edge()?;
        if orientation != Orientation::Forward && orientation != Orientation::Reversed {
            return Ok(None);
        }
        self.my_classifier.compare(&edge, orientation)?;
        let closest: i32 = self.my_classifier.closest_intersection();
        if closest != 0 {
            self.save_closest_edge(edge, closest)?;
        }
        *state = self.my_classifier.state();
        Ok((*state == State::On).then_some(Decided))
    }

    // Saves the closest edge and the position and parameter of the closest
    // intersection on it (the first or last point of a segment by the parity of
    // the index past the points).
    fn save_closest_edge(&mut self, edge: X::Edge, closest: i32) -> Result<(), Failure> {
        let results = self.my_classifier.intersector().results();
        let nb_pnts: i32 = results.nb_points()?;
        self.my_edge = edge;
        let results = self.my_classifier.intersector().results();
        let p_inter: IntersectionPoint = if closest <= nb_pnts {
            results.point(closest)?
        } else {
            let past: i32 = closest - nb_pnts;
            let segment = results.segment((past + 1) / 2)?;
            if past & 1 != 0 {
                segment.first_point()?
            } else {
                segment.last_point()?
            }
        };
        self.my_position = p_inter.transition_of_second().position_on_curve();
        self.my_edge_parameter = p_inter.param_on_second();
        Ok(())
    }

    /// `State()`: OUT when rejected, IN without wires, else the classifier's state;
    /// `classifier.state()`.
    pub fn state(&self) -> State {
        if self.rejected {
            return State::Out;
        }
        if self.nowires {
            return State::In;
        }
        self.my_classifier.state()
    }

    /// `Rejected()`: the state was computed by a rejection (it is OUT);
    /// `classifier.rejected()`.
    pub fn rejected(&self) -> bool {
        self.rejected
    }

    /// `NoWires()`: the face has no wire (the state is IN); `classifier.no_wires()`.
    pub fn no_wires(&self) -> bool {
        self.nowires
    }

    /// `Edge()`: the edge of the closest intersection (the edge holding the point
    /// when the state is ON); Standard_DomainError when rejected; `classifier.edge()?`.
    pub fn edge(&self) -> Result<&X::Edge, Failure> {
        if self.rejected {
            return Err(domain_error("TopClass_FaceClassifier::Edge", "rejected"));
        }
        Ok(&self.my_edge)
    }

    /// `EdgeParameter()`: the parameter on [`Self::edge`] of the closest
    /// intersection; Standard_DomainError when rejected; `classifier.edge_parameter()?`.
    pub fn edge_parameter(&self) -> Result<f64, Failure> {
        if self.rejected {
            return Err(domain_error(
                "TopClass_FaceClassifier::EdgeParameter",
                "rejected",
            ));
        }
        Ok(self.my_edge_parameter)
    }

    /// `Position()`: the position of the closest intersection on [`Self::edge`];
    /// `classifier.position()`.
    pub fn position(&self) -> Position {
        self.my_position
    }
}
