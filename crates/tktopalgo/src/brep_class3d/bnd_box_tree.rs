// Rust translation of BRepClass3d_BndBoxTree.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The two selectors of the solid explorer's UB-tree of edges and vertices
//! (docs/topalgo-design.md section 5): one finds the edges and vertices whose
//! tolerance zone holds a point, the other those a segment of a line crosses.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::TreeSelector;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::extrema::{ExtCC, ExtPC, ExtPElC};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Line3, Point3};

/// The shape of index `object` in the edge/vertex map, `None` past its end (the
/// native `Accept` rejects it before reading the map).
fn map_shape(map: &IndexedMapOfShape, object: i32) -> Result<Option<&Shape>, Failure> {
    if object > map.extent() as i32 {
        return Ok(None);
    }
    Ok(Some(map.find_key(object as usize)?))
}

/// The square of a tolerance, as `Tol *= Tol` natively.
fn square(value: f64) -> f64 {
    value * value
}

/// `BRepClass3d_BndBoxTreeSelectorPoint`: accepts the first edge or vertex whose
/// tolerance zone holds the current point, then stops the selection.
///
/// ```
/// use rustycad_tkbrep::toptools::IndexedMapOfShape;
/// use rustycad_tkernel::ncollection::UBTree;
/// use rustycad_tkmath::bnd::BoundingBox3;
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tktopalgo::brep_class3d::BRepClass3dBndBoxTreeSelectorPoint;
/// let map = IndexedMapOfShape::default();
/// let mut selector = BRepClass3dBndBoxTreeSelectorPoint::new(&map);
/// selector.set_current_point(Point3::new(0.0, 0.0, 0.0));
/// assert_eq!(UBTree::<i32, BoundingBox3>::default().select(&mut selector), 0);
/// ```
pub struct BRepClass3dBndBoxTreeSelectorPoint<'m> {
    my_map_of_shape: &'m IndexedMapOfShape,
    my_p: Point3,
    my_stop: bool,
    failure: Option<Failure>,
}

impl<'m> BRepClass3dBndBoxTreeSelectorPoint<'m> {
    /// `BRepClass3d_BndBoxTreeSelectorPoint(theMapOfShape)`: a selector over the
    /// edges and vertices of `map`; `BRepClass3dBndBoxTreeSelectorPoint::new(&map)`.
    pub fn new(map: &'m IndexedMapOfShape) -> Self {
        Self {
            my_map_of_shape: map,
            my_p: Point3::default(),
            my_stop: false,
            failure: None,
        }
    }

    /// `SetCurrentPoint(theP)`; `selector.set_current_point(point)`.
    pub fn set_current_point(&mut self, point: Point3) {
        self.my_p = point;
    }

    /// The exception an `Accept` raised during the last selection (natively it leaves
    /// `Select`); `selector.take_failure()`.
    pub fn take_failure(&mut self) -> Option<Failure> {
        self.failure.take()
    }

    /// The edge (its extrema with the point closer than its tolerance) or vertex
    /// (closer than its tolerance) test of `Accept`.
    fn interferes(&self, object: i32) -> Result<bool, Failure> {
        let Some(shape) = map_shape(self.my_map_of_shape, object)? else {
            return Ok(false);
        };
        match shape.shape_type()? {
            ShapeEnum::Edge => edge_holds_point(&topods::edge(shape)?, self.my_p),
            ShapeEnum::Vertex => {
                let vertex = topods::vertex(shape)?;
                let tol_sq = square(BRepTool::vertex_tolerance(&vertex)?);
                Ok(BRepTool::pnt(&vertex)?.square_distance(&self.my_p) < tol_sq)
            }
            _ => Ok(false),
        }
    }
}

/// Whether an extremum of the point on the edge curve is closer than the edge tolerance.
fn edge_holds_point(edge: &Edge, point: Point3) -> Result<bool, Failure> {
    let tol_sq = square(BRepTool::edge_tolerance(edge)?);
    let curve = BRepAdaptorCurve::with_edge(edge)?;
    let (first, last) = BRepTool::range(edge)?;
    let extrema = ExtPC::with_range(point, &curve, first, last, 1.0e-10)?;
    if !extrema.is_done() {
        return Ok(false);
    }
    for i in 1..=extrema.nb_ext()? {
        if extrema.square_distance(i)? < tol_sq {
            return Ok(true);
        }
    }
    Ok(false)
}

impl TreeSelector<i32, BoundingBox3> for BRepClass3dBndBoxTreeSelectorPoint<'_> {
    /// `Reject(theBox)`: the box does not hold the point.
    fn reject(&self, bounds: &BoundingBox3) -> bool {
        bounds.is_out_point(self.my_p)
    }
    /// `Accept(theObj)`: true (and stop) for an edge or vertex whose tolerance zone
    /// holds the point; an exception is kept and stops the selection.
    fn accept(&mut self, object: &i32) -> bool {
        match self.interferes(*object) {
            Ok(found) => {
                self.my_stop |= found;
                found
            }
            Err(failure) => {
                self.failure = Some(failure);
                self.my_stop = true;
                false
            }
        }
    }
    fn stop(&self) -> bool {
        self.my_stop
    }
}

/// `BRepClass3d_BndBoxTreeSelectorLine::EdgeParam`: an edge the line crosses, the
/// parameter on the edge and the parameter on the line.
#[derive(Clone, Debug, Default)]
pub struct EdgeParam {
    /// `myE`.
    pub my_e: Edge,
    /// `myParam`: the parameter on the edge.
    pub my_param: f64,
    /// `myLParam`: the parameter on the line.
    pub my_l_param: f64,
}

/// `BRepClass3d_BndBoxTreeSelectorLine::VertParam`: a vertex on the line and its
/// parameter on the line.
#[derive(Clone, Debug, Default)]
pub struct VertParam {
    /// `myV`.
    pub my_v: Vertex,
    /// `myLParam`: the parameter on the line.
    pub my_l_param: f64,
}

/// `BRepClass3d_BndBoxTreeSelectorLine`: collects the edges (with both parameters)
/// and vertices (with the line parameter) whose tolerance zone the current segment
/// crosses; an edge the segment runs along makes the classification incorrect.
///
/// ```
/// use rustycad_tkbrep::toptools::IndexedMapOfShape;
/// use rustycad_tktopalgo::brep_class3d::BRepClass3dBndBoxTreeSelectorLine;
/// let map = IndexedMapOfShape::default();
/// let selector = BRepClass3dBndBoxTreeSelectorLine::new(&map);
/// assert!(selector.is_correct());
/// assert_eq!(selector.get_nb_edge_param(), 0);
/// ```
pub struct BRepClass3dBndBoxTreeSelectorLine<'m> {
    my_map_of_shape: &'m IndexedMapOfShape,
    my_l: Line3,
    my_ep: Vec<EdgeParam>,
    my_vp: Vec<VertParam>,
    my_lc: GeomAdaptorCurve,
    my_is_valid: bool,
    failure: Option<Failure>,
}

impl<'m> BRepClass3dBndBoxTreeSelectorLine<'m> {
    /// `BRepClass3d_BndBoxTreeSelectorLine(theMapOfShape)`: a valid selector without
    /// results; `BRepClass3dBndBoxTreeSelectorLine::new(&map)`.
    pub fn new(map: &'m IndexedMapOfShape) -> Self {
        Self {
            my_map_of_shape: map,
            my_l: Line3::default(),
            my_ep: Vec::new(),
            my_vp: Vec::new(),
            my_lc: GeomAdaptorCurve::default(),
            my_is_valid: true,
            failure: None,
        }
    }

    /// `SetCurrentLine(theL, theMaxParam)`: the line and its segment
    /// `[-Precision::PConfusion(), max_param]`; Standard_ConstructionError when the
    /// segment is reversed; `selector.set_current_line(line, 10.0)?`.
    pub fn set_current_line(&mut self, line: Line3, max_param: f64) -> Result<(), Failure> {
        self.my_l = line;
        let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_lin(line)).cast();
        self.my_lc
            .load_range(&curve, -precision::p_confusion(), max_param)
    }

    /// `GetEdgeParam(i, theOutE, theOutParam, outLParam)`: the i-th edge (1-based) and
    /// its parameters; Standard_OutOfRange outside `[1, GetNbEdgeParam()]`.
    pub fn get_edge_param(&self, i: i32) -> Result<(Edge, f64, f64), Failure> {
        let ep = sequence_value(&self.my_ep, i)?;
        Ok((ep.my_e.clone(), ep.my_param, ep.my_l_param))
    }

    /// `GetVertParam(i, theOutV, outLParam)`: the i-th vertex (1-based) and its line
    /// parameter; Standard_OutOfRange outside `[1, GetNbVertParam()]`.
    pub fn get_vert_param(&self, i: i32) -> Result<(Vertex, f64), Failure> {
        let vp = sequence_value(&self.my_vp, i)?;
        Ok((vp.my_v.clone(), vp.my_l_param))
    }

    /// `GetNbEdgeParam()`; `selector.get_nb_edge_param()`.
    pub fn get_nb_edge_param(&self) -> i32 {
        self.my_ep.len() as i32
    }

    /// `GetNbVertParam()`; `selector.get_nb_vert_param()`.
    pub fn get_nb_vert_param(&self) -> i32 {
        self.my_vp.len() as i32
    }

    /// `ClearResults()`: no results, valid again; `selector.clear_results()`.
    pub fn clear_results(&mut self) {
        self.my_ep.clear();
        self.my_vp.clear();
        self.my_is_valid = true;
    }

    /// `IsCorrect()`: false once an edge ran along the segment; `selector.is_correct()`.
    pub fn is_correct(&self) -> bool {
        self.my_is_valid
    }

    /// The exception an `Accept` raised during the last selection (natively it leaves
    /// `Select`); `selector.take_failure()`.
    pub fn take_failure(&mut self) -> Option<Failure> {
        self.failure.take()
    }

    /// The edge or vertex test of `Accept`, appending its results.
    fn collect(&mut self, object: i32) -> Result<bool, Failure> {
        let Some(shape) = map_shape(self.my_map_of_shape, object)? else {
            return Ok(false);
        };
        match shape.shape_type()? {
            ShapeEnum::Edge => self.collect_edge(&topods::edge(shape)?),
            ShapeEnum::Vertex => self.collect_vertex(&topods::vertex(shape)?),
            _ => Ok(false),
        }
    }

    /// Edge-line interference: the extrema of the edge curve and the segment closer
    /// than the edge tolerance; a parallel pair makes the selector incorrect.
    fn collect_edge(&mut self, edge: &Edge) -> Result<bool, Failure> {
        let tol_sq = square(BRepTool::edge_tolerance(edge)?);
        let curve = BRepAdaptorCurve::with_edge(edge)?;
        let (first, last) = BRepTool::range(edge)?;
        let segment = (self.my_lc.first_parameter()?, self.my_lc.last_parameter()?);
        let extrema = ExtCC::with_ranges(
            &curve,
            &self.my_lc,
            (first, last),
            segment,
            1.0e-10,
            1.0e-10,
        )?;
        if !extrema.is_done() {
            return Ok(false);
        }
        if extrema.is_parallel()? {
            self.my_is_valid = false; // Tangent case is invalid for classification
            return Ok(false);
        }
        let mut is_inside = false;
        for i in 1..=extrema.nb_ext()? {
            if extrema.square_distance(i)? < tol_sq {
                let (on_edge, on_line) = extrema.points(i)?;
                self.my_ep.push(EdgeParam {
                    my_e: edge.clone(),
                    my_param: on_edge.parameter(),
                    my_l_param: on_line.parameter(),
                });
                is_inside = true;
            }
        }
        Ok(is_inside)
    }

    /// Vertex-line interference: the projection of the vertex on the whole line
    /// closer than the vertex tolerance.
    fn collect_vertex(&mut self, vertex: &Vertex) -> Result<bool, Failure> {
        let tol_sq = square(BRepTool::vertex_tolerance(vertex)?);
        let infinite = precision::infinite();
        let extrema = ExtPElC::with_line(
            BRepTool::pnt(vertex)?,
            self.my_l,
            precision::confusion(),
            -infinite,
            infinite,
        );
        if !(extrema.is_done() && extrema.nb_ext()? > 0) || extrema.square_distance(1)? >= tol_sq {
            return Ok(false);
        }
        let param_l = extrema.point(1)?.parameter();
        self.my_vp.push(VertParam {
            my_v: vertex.clone(),
            my_l_param: param_l,
        });
        Ok(true)
    }
}

/// `NCollection_Sequence::Value(i)`: Standard_OutOfRange outside `[1, Length()]`.
fn sequence_value<T>(sequence: &[T], i: i32) -> Result<&T, Failure> {
    usize::try_from(i - 1)
        .ok()
        .and_then(|index| sequence.get(index))
        .ok_or_else(|| {
            Failure::with_message(
                "Standard_OutOfRange",
                &format!(
                    "NCollection_Sequence::Value: index {i} outside [1, {}]",
                    sequence.len()
                ),
            )
        })
}

impl TreeSelector<i32, BoundingBox3> for BRepClass3dBndBoxTreeSelectorLine<'_> {
    /// `Reject(theBox)`: the box does not meet the whole line.
    fn reject(&self, bounds: &BoundingBox3) -> bool {
        bounds.is_out_line(self.my_l)
    }
    /// `Accept(theObj)`: collects the interferences of an edge or vertex; an
    /// exception is kept and stops the selection.
    fn accept(&mut self, object: &i32) -> bool {
        self.collect(*object).unwrap_or_else(|failure| {
            self.failure = Some(failure);
            false
        })
    }
    fn stop(&self) -> bool {
        self.failure.is_some()
    }
}
