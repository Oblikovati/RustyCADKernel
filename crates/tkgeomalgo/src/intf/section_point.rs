// Rust translation of ModelingAlgorithms/TKGeomAlgo/Intf/Intf_PIType.hxx and
// Intf_SectionPoint.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Intf_PIType and Intf_SectionPoint: an intersection point between polygons and
//! polyhedra, with where it lies on each argument.
use rustycad_tkmath::gp::{Point2, Point3};

/// Intf_PIType: the kind of element an intersection point lies on, ordered by
/// dimension as natively (`Merge` keeps the greater one); `PIType::Edge > PIType::Face`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(i32)]
pub enum PIType {
    /// Intf_EXTERNAL: outside the element (the default).
    #[default]
    External,
    /// Intf_FACE: inside a facet.
    Face,
    /// Intf_EDGE: on an edge (a segment).
    Edge,
    /// Intf_VERTEX: on a vertex.
    Vertex,
}

impl PIType {
    /// Every type in declaration order; `PIType::ALL[3] == PIType::Vertex`.
    pub const ALL: [PIType; 4] = [PIType::External, PIType::Face, PIType::Edge, PIType::Vertex];
}

/// Where a section point lies on one argument: the type of element, its two
/// addresses and the parameter on it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct ElementPlace {
    dimension: PIType,
    first_index: i32,
    second_index: i32,
    parameter: f64,
}

impl ElementPlace {
    /// The cumulated parameter `(second_index - 1) + parameter`; the index difference
    /// is integer arithmetic, as natively.
    fn cumulated(&self) -> f64 {
        f64::from(self.second_index.wrapping_sub(1)) + self.parameter
    }

    fn same_logic(&self, other: &Self) -> bool {
        self.dimension == other.dimension
            && self.first_index == other.first_index
            && self.second_index == other.second_index
    }
}

/// Intf_SectionPoint: an intersection point with its location, where it lies on the
/// first (object) and second (tool) argument, and the incidence (the sine of the
/// angle between the elements).
///
/// ```
/// use rustycad_tkgeomalgo::intf::{PIType, SectionPoint};
/// use rustycad_tkmath::gp::Point2;
/// let point = SectionPoint::with_data_2d(Point2::new(1.0, 2.0), PIType::Edge, 3, 0.5, PIType::Vertex, 7, 0.0, 1.0);
/// assert_eq!((point.param_on_first(), point.param_on_second()), (2.5, 6.0));
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct SectionPoint {
    pnt: Point3,
    object: ElementPlace,
    tool: ElementPlace,
    incidence: f64,
}

impl PartialEq for SectionPoint {
    /// operator==: the logical comparison of `is_equal`.
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other)
    }
}

impl SectionPoint {
    /// Intf_SectionPoint(): the origin, EXTERNAL on both arguments with null
    /// addresses, parameters and incidence; `SectionPoint::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Intf_SectionPoint(Where, DimeO, AddrO1, AddrO2, ParamO, DimeT, AddrT1, AddrT2,
    /// ParamT, Incid): a point with its place on both arguments;
    /// `SectionPoint::with_data(p, PIType::Edge, 0, 2, 0.5, PIType::Face, 4, 0, 0.0, 1.0)`.
    #[allow(clippy::too_many_arguments)]
    pub fn with_data(
        location: Point3,
        dim_o: PIType,
        addr_o1: i32,
        addr_o2: i32,
        param_o: f64,
        dim_t: PIType,
        addr_t1: i32,
        addr_t2: i32,
        param_t: f64,
        incid: f64,
    ) -> Self {
        Self {
            pnt: location,
            object: place(dim_o, addr_o1, addr_o2, param_o),
            tool: place(dim_t, addr_t1, addr_t2, param_t),
            incidence: incid,
        }
    }

    /// Intf_SectionPoint(Where2d, DimeO, AddrO1, ParamO, DimeT, AddrT1, ParamT, Incid):
    /// a 2d point (z = 0) whose single address on each argument is the second one, the
    /// first staying 0; `SectionPoint::with_data_2d(p, PIType::Edge, 1, 0.5, PIType::Edge, 2, 0.5, 1.0)`.
    #[allow(clippy::too_many_arguments)]
    pub fn with_data_2d(
        location: Point2,
        dim_o: PIType,
        addr_o: i32,
        param_o: f64,
        dim_t: PIType,
        addr_t: i32,
        param_t: f64,
        incid: f64,
    ) -> Self {
        Self {
            pnt: Point3::new(location.x(), location.y(), 0.0),
            object: place(dim_o, 0, addr_o, param_o),
            tool: place(dim_t, 0, addr_t, param_t),
            incidence: incid,
        }
    }

    /// Pnt: the location; `point.pnt()`.
    pub fn pnt(&self) -> Point3 {
        self.pnt
    }

    /// ParamOnFirst: the cumulated parameter on the first argument (second address
    /// minus one, plus the parameter); `point.param_on_first()`.
    pub fn param_on_first(&self) -> f64 {
        self.object.cumulated()
    }

    /// ParamOnSecond: the cumulated parameter on the second argument;
    /// `point.param_on_second()`.
    pub fn param_on_second(&self) -> f64 {
        self.tool.cumulated()
    }

    /// TypeOnFirst: the element type on the first argument; `point.type_on_first()`.
    pub fn type_on_first(&self) -> PIType {
        self.object.dimension
    }

    /// TypeOnSecond: the element type on the second argument; `point.type_on_second()`.
    pub fn type_on_second(&self) -> PIType {
        self.tool.dimension
    }

    /// InfoFirst(Dim, Add1, Add2, Param): the type, both addresses and the parameter
    /// on the first argument; `let (dim, a1, a2, par) = point.info_first();`.
    pub fn info_first(&self) -> (PIType, i32, i32, f64) {
        info(&self.object)
    }

    /// InfoFirst(Dim, Addr, Param): the type, the second address and the parameter on
    /// the first argument; `let (dim, addr, par) = point.info_first_address();`.
    pub fn info_first_address(&self) -> (PIType, i32, f64) {
        (
            self.object.dimension,
            self.object.second_index,
            self.object.parameter,
        )
    }

    /// InfoSecond(Dim, Add1, Add2, Param): the same data on the second argument;
    /// `let (dim, a1, a2, par) = point.info_second();`.
    pub fn info_second(&self) -> (PIType, i32, i32, f64) {
        info(&self.tool)
    }

    /// InfoSecond(Dim, Addr, Param): the type, the second address and the parameter on
    /// the second argument; `let (dim, addr, par) = point.info_second_address();`.
    pub fn info_second_address(&self) -> (PIType, i32, f64) {
        (
            self.tool.dimension,
            self.tool.second_index,
            self.tool.parameter,
        )
    }

    /// Incidence: the cosine-like incidence given at construction; `point.incidence()`.
    pub fn incidence(&self) -> f64 {
        self.incidence
    }

    /// IsEqual: the same types and addresses on both arguments; locations and
    /// parameters are not compared; `first.is_equal(&second)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        self.object.same_logic(&other.object) && self.tool.same_logic(&other.tool)
    }

    /// IsOnSameEdge: whether both points lie on one edge of the first argument, or
    /// else of the second. Natively the vertex-vertex case of the first argument
    /// compares the second argument's first addresses; `first.is_on_same_edge(&second)`.
    pub fn is_on_same_edge(&self, other: &Self) -> bool {
        let first_vertices_share = self.tool.first_index == other.tool.first_index;
        same_edge(&self.object, &other.object, first_vertices_share)
            || same_edge(&self.tool, &other.tool, first_vertices_share)
    }

    /// Merge: gives both points the location of `self` and, on each argument, the
    /// place of higher dimension (ties keep `self`'s); `first.merge(&mut second)`.
    pub fn merge(&mut self, other: &mut Self) {
        other.pnt = self.pnt;
        merge_place(&mut self.object, &mut other.object);
        merge_place(&mut self.tool, &mut other.tool);
    }

    /// Dump: prints nothing, as natively (its body is compiled out by
    /// DEBUG_INTFSECTIONPOINT 0); `point.dump(2)`.
    pub fn dump(&self, _indent: i32) {}
}

fn place(dimension: PIType, first_index: i32, second_index: i32, parameter: f64) -> ElementPlace {
    ElementPlace {
        dimension,
        first_index,
        second_index,
        parameter,
    }
}

fn info(place: &ElementPlace) -> (PIType, i32, i32, f64) {
    (
        place.dimension,
        place.first_index,
        place.second_index,
        place.parameter,
    )
}

/// One argument's part of IsOnSameEdge; `vertices_share` is the native test of the
/// vertex-vertex case, which reads the second argument's first addresses on both sides.
fn same_edge(mine: &ElementPlace, other: &ElementPlace, vertices_share: bool) -> bool {
    match (mine.dimension, other.dimension) {
        (PIType::Edge, PIType::Edge) => {
            mine.first_index == other.first_index && mine.second_index == other.second_index
        }
        (PIType::Edge, PIType::Vertex) => {
            mine.first_index == other.first_index || mine.second_index == other.first_index
        }
        (PIType::Vertex, PIType::Edge) => {
            mine.first_index == other.first_index || mine.first_index == other.second_index
        }
        (PIType::Vertex, PIType::Vertex) => vertices_share,
        _ => false,
    }
}

fn merge_place(mine: &mut ElementPlace, other: &mut ElementPlace) {
    if mine.dimension >= other.dimension {
        *other = *mine;
    } else {
        *mine = *other;
    }
}
