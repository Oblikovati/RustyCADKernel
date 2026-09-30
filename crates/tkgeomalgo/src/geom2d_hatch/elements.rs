// Rust translation of Geom2dHatch_Elements.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dHatch_Elements`: the elements of a hatcher seen as one face by the
//! classifier (the TopClass face explorer).
use super::{Element, Intersector};
use crate::topclass::FaceExplorer;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Direction2, Line2, Point2, StandardDirection2, Vector2};

// The probing of the classification segments along each element (skv OCC12627):
// the fraction of the element's range where the probing starts, the one past which
// it moves to the next element, and the step between two probes.
const PROBING_START: f64 = 0.123;
const PROBING_END: f64 = 0.8;
const PROBING_STEP: f64 = 0.2111;

/// `Geom2dHatch_Elements`: a map of [`Element`]s by index, which is one face with
/// one wire of every element for the classifier. The classification segments go
/// from the point to probes on the elements (`Segment`, `OtherSegment`); the edge
/// iteration visits the map in its native bucket order.
///
/// The native edge iterator is a map iterator kept in the object; the port keeps
/// its rank in that order (a map changed during an iteration invalidates the native
/// iterator).
///
/// ```
/// use rustycad_tkgeomalgo::geom2d_hatch::{Element, Elements};
/// let mut elements = Elements::new();
/// elements.bind(1, Element::default());
/// assert!(elements.is_bound(1));
/// ```
pub struct Elements {
    my_map: DataMap<i32, Element>,
    iter: Option<usize>,
    num_wire: i32,
    // Reset by InitEdges and never read, as natively.
    #[allow(dead_code)]
    num_edge: i32,
    probe: Probe,
}

// The position of the next probe: the rank of the element and the fraction of its
// range (`myCurEdge`, `myCurEdgePar`).
struct Probe {
    my_cur_edge: i32,
    my_cur_edge_par: f64,
}

impl Default for Elements {
    /// `Geom2dHatch_Elements()`: empty, the probing at the start of the first element.
    fn default() -> Self {
        Self {
            my_map: DataMap::default(),
            iter: None,
            num_wire: 0,
            num_edge: 0,
            probe: Probe {
                my_cur_edge: 1,
                my_cur_edge_par: PROBING_START,
            },
        }
    }
}

impl Elements {
    /// `Geom2dHatch_Elements()`; `Elements::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// The copy constructor `Geom2dHatch_Elements(const Geom2dHatch_Elements&)`, which
    /// natively copies nothing: the result is empty, with the probing at element 0
    /// and fraction 0; `Elements::copy_of(&elements)`.
    pub fn copy_of(_other: &Elements) -> Self {
        Self {
            my_map: DataMap::default(),
            iter: None,
            num_wire: 0,
            num_edge: 0,
            probe: Probe {
                my_cur_edge: 0,
                my_cur_edge_par: 0.0,
            },
        }
    }

    /// Clear(): removes every element; `elements.clear()`.
    pub fn clear(&mut self) {
        self.my_map.clear(false);
    }

    /// Bind(K, I): binds `item` to `key`, replacing a bound one; true when the key
    /// was new; `elements.bind(1, element)`.
    pub fn bind(&mut self, key: i32, item: Element) -> bool {
        self.my_map.bind(key, item)
    }

    /// IsBound(K); `elements.is_bound(1)`.
    pub fn is_bound(&self, key: i32) -> bool {
        self.my_map.is_bound(&key)
    }

    /// UnBind(K): true when the key was bound; `elements.unbind(1)`.
    pub fn unbind(&mut self, key: i32) -> bool {
        self.my_map.unbind(&key)
    }

    /// Find(K) (and `operator()(K) const`): Standard_NoSuchObject for an unbound key;
    /// `elements.find(1)?`.
    pub fn find(&self, key: i32) -> Result<&Element, Failure> {
        self.my_map.find(&key)
    }

    /// ChangeFind(K) (and `operator()(K)`): Standard_NoSuchObject for an unbound key;
    /// `elements.change_find(1)?`.
    pub fn change_find(&mut self, key: i32) -> Result<&mut Element, Failure> {
        self.my_map.change_find(&key)
    }

    /// CheckPoint(P): always true, the point is kept; `elements.check_point(&mut p)`.
    pub fn check_point(&mut self, _point: &mut Point2) -> bool {
        true
    }

    /// Reject(P): always false; `elements.reject(&p)`.
    pub fn reject(&self, _point: &Point2) -> bool {
        false
    }

    /// Segment(P, L, Par): restarts the probing at the first element and returns
    /// the first classification segment ([`Elements::other_segment`]);
    /// `let (valid, line, length) = elements.segment(&p)?;`.
    pub fn segment(&mut self, point: &Point2) -> Result<(bool, Line2, f64), Failure> {
        self.probe.my_cur_edge = 1;
        self.probe.my_cur_edge_par = PROBING_START;
        self.other_segment(point)
    }

    /// OtherSegment(P, L, Par): the next segment from `point` to a probe on a
    /// FORWARD or REVERSED element, skipping probes at the point, on a degenerate
    /// tangent, nearly tangent to the segment (unless it is the last probe of the
    /// element) or with an end of the element on the line. Without one: false, the
    /// X line through the point and RealLast; `elements.other_segment(&p)?`.
    pub fn other_segment(&mut self, point: &Point2) -> Result<(bool, Line2, f64), Failure> {
        for (rank, (_, item)) in self.my_map.items().enumerate() {
            if (rank as i32 + 1) < self.probe.my_cur_edge {
                continue;
            }
            let orientation = item.orientation();
            let probed =
                orientation == Orientation::Forward || orientation == Orientation::Reversed;
            if probed && let Some((line, length)) = self.probe.along(item.curve(), point)? {
                return Ok((true, line, length));
            }
            self.probe.my_cur_edge += 1;
            self.probe.my_cur_edge_par = PROBING_START;
        }
        let x_axis = Direction2::from_standard(StandardDirection2::X);
        Ok((false, Line2::new(*point, x_axis), real_last()))
    }
}

// The parameter range of an element: an infinite side is replaced by a unit
// range next to the finite one ([-1, 1] when both are infinite).
fn probing_range(curve: &Geom2dAdaptorCurve) -> Result<(f64, f64), Failure> {
    let (mut f_par, mut l_par) = (curve.first_parameter()?, curve.last_parameter()?);
    if precision::is_negative_infinite(f_par) {
        if precision::is_positive_infinite(l_par) {
            f_par = -1.0;
            l_par = 1.0;
        } else {
            f_par = l_par - 1.0;
        }
    } else if precision::is_positive_infinite(l_par) {
        l_par = f_par + 1.0;
    }
    Ok((f_par, l_par))
}

impl Probe {
    // The probes of one element from the current fraction: the first acceptable
    // segment and its length, the probe moved past it.
    fn along(
        &mut self,
        curve: &Geom2dAdaptorCurve,
        point: &Point2,
    ) -> Result<Option<(Line2, f64)>, Failure> {
        let (f_par, l_par) = probing_range(curve)?;
        while self.my_cur_edge_par < PROBING_END {
            let param: f64 = self.my_cur_edge_par * f_par + (1.0 - self.my_cur_edge_par) * l_par;
            if let Some(found) = self.probe_at(curve, point, param, (f_par, l_par))? {
                return Ok(Some(found));
            }
            self.my_cur_edge_par += PROBING_STEP;
        }
        Ok(None)
    }

    // One probe at `param`; Some with the segment when it is acceptable (the
    // negated comparisons reject a NaN length as the native `>` tests do).
    fn probe_at(
        &mut self,
        curve: &Geom2dAdaptorCurve,
        point: &Point2,
        param: f64,
        (f_par, l_par): (f64, f64),
    ) -> Result<Option<(Line2, f64)>, Failure> {
        let Some((lin_dir, square_length)) = self.probe_direction(curve, point, param)? else {
            return Ok(None);
        };
        let line = Line2::new(*point, lin_dir);
        if !ends_off_line(curve, &line, (f_par, l_par))? {
            return Ok(None);
        }
        self.my_cur_edge_par += PROBING_STEP;
        if self.my_cur_edge_par >= PROBING_END {
            self.my_cur_edge += 1;
            self.my_cur_edge_par = PROBING_START;
        }
        Ok(Some((line, square_length.sqrt())))
    }

    // The direction from the point to the probe and the squared length, unless the
    // probe is at the point, its tangent is degenerate, or the segment is nearly
    // tangent to the element (allowed at the last probe of the element).
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    fn probe_direction(
        &self,
        curve: &Geom2dAdaptorCurve,
        point: &Point2,
        param: f64,
    ) -> Result<Option<(Direction2, f64)>, Failure> {
        let (p_on_c, mut tan_vec) = curve.d1(param)?;
        let lin_vec = Vector2::between(*point, p_on_c);
        let square_length: f64 = lin_vec.square_magnitude();
        if !(square_length > precision::square_p_confusion()) {
            return Ok(None);
        }
        let lin_dir = Direction2::from_vector(lin_vec)?;
        let tan_mod: f64 = tan_vec.square_magnitude();
        if tan_mod < precision::square_p_confusion() {
            return Ok(None);
        }
        tan_vec /= tan_mod.sqrt();
        let sin_a: f64 = tan_vec.crossed(Vector2::from(lin_dir));
        // Too small an angle: the line and the edge may be taken as tangent, which is
        // bad for the classifier.
        if sin_a.abs() < 0.001 && self.my_cur_edge_par + PROBING_STEP < PROBING_END {
            return Ok(None);
        }
        Ok(Some((lin_dir, square_length)))
    }
}

// Whether both ends of the probing range are off the line (the first one is
// tested first, the last one only when the first is off); a NaN distance is on it.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn ends_off_line(
    curve: &Geom2dAdaptorCurve,
    line: &Line2,
    (f_par, l_par): (f64, f64),
) -> Result<bool, Failure> {
    if !(line.square_distance(curve.value(f_par)?) > precision::square_p_confusion()) {
        return Ok(false);
    }
    Ok(line.square_distance(curve.value(l_par)?) > precision::square_p_confusion())
}

impl Elements {
    /// InitWires(): one wire holds every element; `elements.init_wires()`.
    pub fn init_wires(&mut self) {
        self.num_wire = 0;
    }

    /// MoreWires(): true before the only wire is passed; `elements.more_wires()`.
    pub fn more_wires(&self) -> bool {
        self.num_wire == 0
    }

    /// NextWire(); `elements.next_wire()`.
    pub fn next_wire(&mut self) {
        self.num_wire += 1;
    }

    /// RejectWire(L, Par): always false; `elements.reject_wire(&line, 1.0)`.
    pub fn reject_wire(&self, _line: &Line2, _param: f64) -> bool {
        false
    }

    /// InitEdges(): starts the iteration over the elements; `elements.init_edges()`.
    pub fn init_edges(&mut self) {
        self.num_edge = 0;
        self.iter = Some(0);
    }

    /// MoreEdges(); `elements.more_edges()`.
    pub fn more_edges(&self) -> bool {
        self.iter.is_some_and(|rank| rank < self.my_map.len())
    }

    /// NextEdge(); `elements.next_edge()`.
    pub fn next_edge(&mut self) {
        if self.more_edges() {
            self.iter = self.iter.map(|rank| rank + 1);
        }
    }

    /// RejectEdge(L, Par): always false; `elements.reject_edge(&line, 1.0)`.
    pub fn reject_edge(&self, _line: &Line2, _param: f64) -> bool {
        false
    }

    /// CurrentEdge(E, Or): a copy of the curve of the current element and its
    /// orientation; Standard_NoSuchObject past the last element (the iterator's
    /// `Key`); `let (curve, orientation) = elements.current_edge()?;`.
    pub fn current_edge(&self) -> Result<(Geom2dAdaptorCurve, Orientation), Failure> {
        let current = self
            .iter
            .and_then(|rank| self.my_map.items().nth(rank))
            .map(|(key, _)| *key)
            .ok_or_else(|| {
                Failure::with_type(
                    "Standard_NoSuchObject",
                    "NCollection_DataMap::Iterator::Key: the edge iteration is past the last element; expected MoreEdges()",
                    "",
                )
            })?;
        let item = self.my_map.find(&current)?;
        Ok((item.curve().shallow_adaptor()?, item.orientation()))
    }
}

impl FaceExplorer for Elements {
    type Edge = Geom2dAdaptorCurve;
    type Intersector = Intersector;
    fn check_point(&mut self, point: &mut Point2) -> bool {
        Elements::check_point(self, point)
    }
    fn reject(&self, point: &Point2) -> bool {
        Elements::reject(self, point)
    }
    fn segment(&mut self, point: &Point2) -> Result<(bool, Line2, f64), Failure> {
        Elements::segment(self, point)
    }
    fn other_segment(&mut self, point: &Point2) -> Result<(bool, Line2, f64), Failure> {
        Elements::other_segment(self, point)
    }
    fn init_wires(&mut self) {
        Elements::init_wires(self)
    }
    fn more_wires(&self) -> bool {
        Elements::more_wires(self)
    }
    fn next_wire(&mut self) {
        Elements::next_wire(self)
    }
    fn reject_wire(&self, line: &Line2, param: f64) -> bool {
        Elements::reject_wire(self, line, param)
    }
    fn init_edges(&mut self) {
        Elements::init_edges(self)
    }
    fn more_edges(&self) -> bool {
        Elements::more_edges(self)
    }
    fn next_edge(&mut self) {
        Elements::next_edge(self)
    }
    fn reject_edge(&self, line: &Line2, param: f64) -> bool {
        Elements::reject_edge(self, line, param)
    }
    fn current_edge(&self) -> Result<(Geom2dAdaptorCurve, Orientation), Failure> {
        Elements::current_edge(self)
    }
}
