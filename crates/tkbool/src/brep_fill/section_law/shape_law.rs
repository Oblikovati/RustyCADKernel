// Rust translation of ModelingAlgorithms/TKBool/BRepFill/BRepFill_ShapeLaw.hxx/.cxx/.lxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFill_ShapeLaw`: the section law of one wire (or vertex), constant or scaled by
//! a law function.
use super::vertex_tol::vertex_gap;
use super::{BRepFillSectionLaw, BRepFillSectionLawBase, index_error};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::brep_lprop::continuity_with_tolerances;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex, Wire};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBoundedCurve, GeomCurve, GeomLine, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom_fill::{
    GeomFillEvolvedSection, GeomFillSectionLaw, GeomFillUniformSection,
};
use rustycad_tkgeomalgo::law::LawFunction;
use rustycad_tkgeombase::geom_convert::CompCurveToBSplineCurve;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Transform3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPITransform};

/// The section law of a wire or a vertex (`BRepFill_ShapeLaw`): one uniform section
/// law per edge, or evolved sections scaled by a law function;
/// `let law = BRepFillShapeLaw::with_wire(&wire, true)?;`.
pub struct BRepFillShapeLaw {
    parent: StandardTransient,
    base: BRepFillSectionLawBase,
    vertex: bool,
    my_shape: Shape,
    my_edges: Vec<Shape>,
    the_law: Handle<dyn LawFunction>,
}
impl_transient!(
    BRepFillShapeLaw,
    "BRepFill_ShapeLaw",
    dyn BRepFillSectionLaw,
    parent,
    interfaces: [dyn BRepFillSectionLaw]
);

/// The section law of a curve: uniform, or evolved by `law`.
fn section_of(
    curve: &Handle<dyn GeomCurve>,
    law: &Handle<dyn LawFunction>,
) -> Result<Handle<dyn GeomFillSectionLaw>, Failure> {
    if law.is_null() {
        let uniform = GeomFillUniformSection::new(curve, 0., 1.)?;
        return Ok(Handle::new(uniform).cast::<dyn GeomFillSectionLaw>());
    }
    Ok(Handle::new(GeomFillEvolvedSection::new(curve, law)?).cast::<dyn GeomFillSectionLaw>())
}

/// The scaling about the origin by the value of the law at `param`.
fn law_scaling(law: &dyn LawFunction, param: f64) -> Result<Transform3, Failure> {
    let mut trsf = Transform3::default();
    trsf.set_scale(Point3::new(0., 0., 0.), law.value(param)?)?;
    Ok(trsf)
}

/// `BRepBuilderAPI_Transform(S, T)` (no copy): the transformed shape.
fn transformed(shape: &Shape, trsf: Transform3) -> Result<Shape, Failure> {
    let mut transform = BRepBuilderAPITransform::with_shape(shape, trsf, false, false)?;
    Ok(transform.shape()?.clone())
}

impl BRepFillShapeLaw {
    /// `BRepFill_ShapeLaw(V, Build)`: a vertex section, built as the uniform section of
    /// a segment from the vertex along X of length `2 * tolerance + PConfusion`.
    pub fn with_vertex(v: &Vertex, build: bool) -> Result<Self, Failure> {
        let mut law = Self::empty(v.as_ref().clone(), Handle::default(), true);
        law.base.uclosed = false;
        // A constant law.
        law.base.vclosed = true;
        law.my_edges = vec![v.as_ref().clone()];
        if build {
            let line: Handle<dyn GeomCurve> =
                Handle::new(GeomLine::new(Axis1::new(BRepTool::pnt(v)?, Direction3::X))).cast();
            let last: f64 = 2. * BRepTool::vertex_tolerance(v)? + precision::p_confusion();
            let trimmed: Handle<dyn GeomCurve> =
                Handle::new(GeomTrimmedCurve::new(&line, 0., last, true, true)?).cast();
            let section = GeomFillUniformSection::new(&trimmed, 0., 1.)?;
            law.base.laws = Some(vec![Handle::new(section).cast::<dyn GeomFillSectionLaw>()]);
        }
        law.base.done = true;
        Ok(law)
    }

    /// `BRepFill_ShapeLaw(W, Build)`: the constant section of a wire.
    pub fn with_wire(w: &Wire, build: bool) -> Result<Self, Failure> {
        Self::with_wire_and_law(w, &Handle::default(), build)
    }

    /// `BRepFill_ShapeLaw(W, L, Build)`: the section of a wire scaled by `L` (constant
    /// for a null law).
    pub fn with_wire_and_law(
        w: &Wire,
        l: &Handle<dyn LawFunction>,
        build: bool,
    ) -> Result<Self, Failure> {
        let mut law = Self::empty(w.as_ref().clone(), l.clone(), false);
        law.init_wire(build)?;
        law.base.done = true;
        Ok(law)
    }

    /// The fields before the constructor bodies.
    fn empty(shape: Shape, the_law: Handle<dyn LawFunction>, vertex: bool) -> Self {
        Self {
            parent: StandardTransient::default(),
            base: BRepFillSectionLawBase::default(),
            vertex,
            my_shape: shape,
            my_edges: Vec::new(),
            the_law,
        }
    }

    /// `Edge(Index)`: the edge of the law `index`; Standard_OutOfRange outside.
    pub fn edge(&self, index: i32) -> Result<Edge, Failure> {
        let found = usize::try_from(index - 1)
            .ok()
            .and_then(|i| self.my_edges.get(i));
        let shape = found
            .ok_or_else(|| index_error("BRepFill_ShapeLaw::Edge", index, self.my_edges.len()))?;
        topods::edge(shape)
    }

    /// `Init(Build)`: the non-degenerated edges with a 3d curve, their laws and the
    /// closure of the wire.
    fn init_wire(&mut self, build: bool) -> Result<(), Failure> {
        self.base.vclosed = true;
        let wire: Wire = topods::wire(&self.my_shape)?;
        let mut laws: Vec<Handle<dyn GeomFillSectionLaw>> = Vec::new();
        let mut explorer = WireExplorer::with_wire(&wire)?;
        while explorer.more() {
            let edge: Edge = explorer.current().clone();
            explorer.next()?;
            if edge.is_null() || BRepTool::degenerated(&edge)? {
                continue;
            }
            let (curve, first, last) = BRepTool::curve(&edge)?;
            if curve.is_null() {
                continue;
            }
            self.my_edges.push(edge.as_ref().clone());
            let ii: i32 = self.my_edges.len() as i32;
            self.base.indices.bind(edge.as_ref().clone(), ii);
            // `myLaws` has one (null when not built) law per edge, as natively.
            laws.push(match build {
                true => self.edge_law(&edge, curve, (first, last), ii)?,
                false => Handle::default(),
            });
        }
        self.base.laws = Some(laws);
        self.base.uclosed = wire.closed()? || self.ends_meet()?;
        Ok(())
    }

    /// The law of one edge: its curve oriented as the edge, trimmed to the edge range
    /// unless it is the closed first edge running over the whole curve.
    fn edge_law(
        &self,
        edge: &Edge,
        mut curve: Handle<dyn GeomCurve>,
        range: (f64, f64),
        ii: i32,
    ) -> Result<Handle<dyn GeomFillSectionLaw>, Failure> {
        let (mut first, mut last) = range;
        let Some(stored) = curve.get() else {
            return Err(Failure::with_type(
                "Standard_NullObject",
                "BRepFill_ShapeLaw: an edge curve is null; expected a curve",
                "BRepFill_ShapeLaw",
            ));
        };
        if edge.orientation() == Orientation::Reversed {
            // A reversed copy, to avoid the deterioration of the topology.
            let reversed: Handle<dyn GeomCurve> = stored.reversed()?;
            let aux: f64 = stored.reversed_parameter(first)?;
            first = stored.reversed_parameter(last)?;
            last = aux;
            curve = reversed;
        }
        let mut is_closed: bool = BRepTool::is_closed(edge)?;
        if is_closed
            && curve
                .get()
                .is_some_and(|c| (c.first_parameter() - first).abs() > precision::p_confusion())
        {
            // The trimmed curve differs.
            is_closed = false;
        }
        if ii > 1 || !is_closed {
            curve = Handle::new(GeomTrimmedCurve::new(&curve, first, last, true, true)?).cast();
        }
        section_of(&curve, &self.the_law)
    }

    /// Whether the last edge ends where the first edge starts (the same vertex, or
    /// points within the sum of the vertex tolerances).
    fn ends_meet(&self) -> Result<bool, Failure> {
        let edge1: Edge = self.edge(self.my_edges.len() as i32)?;
        let edge2: Edge = self.edge(1)?;
        let reversed = |e: &Edge| e.orientation() == Orientation::Reversed;
        let v1: Vertex = if reversed(&edge1) {
            topexp::first_vertex(&edge1, false)
        } else {
            topexp::last_vertex(&edge1, false)
        };
        let v2: Vertex = if reversed(&edge2) {
            topexp::last_vertex(&edge2, false)
        } else {
            topexp::first_vertex(&edge2, false)
        };
        if v1.is_same(&v2) {
            return Ok(true);
        }
        let curve1 = BRepAdaptorCurve::with_edge(&edge1)?;
        let curve2 = BRepAdaptorCurve::with_edge(&edge2)?;
        let u1: f64 = BRepTool::parameter(&v1, &edge1)?;
        let u2: f64 = BRepTool::parameter(&v2, &edge2)?;
        let eps: f64 = BRepTool::vertex_tolerance(&v2)? + BRepTool::vertex_tolerance(&v1)?;
        Ok(curve1.value(u1)?.is_equal(&curve2.value(u2)?, eps))
    }

    /// The concatenation of the edge curves into one B-spline (a junction is added
    /// with the tolerance of the common vertex, then 200 times it).
    fn concatenation(&self) -> Result<Handle<dyn GeomCurve>, Failure> {
        let bounded = |edge: &Edge| -> Result<Handle<dyn GeomBoundedCurve>, Failure> {
            let (curve, f, l) = BRepTool::curve(edge)?;
            Ok(
                Handle::new(GeomTrimmedCurve::new(&curve, f, l, true, true)?)
                    .cast::<dyn GeomBoundedCurve>(),
            )
        };
        let mut concat = CompCurveToBSplineCurve::with_curve(
            &bounded(&self.edge(1)?)?,
            ParameterisationType::TgtThetaOver2,
        )?;
        let mut bof: bool = true;
        let mut ii: i32 = 2;
        while ii <= self.my_edges.len() as i32 && bof {
            let tc = bounded(&self.edge(ii)?)?;
            let eps_v: f64 = match topexp::common_vertex(&self.edge(ii - 1)?, &self.edge(ii)?) {
                Some(v) => BRepTool::vertex_tolerance(&v)?,
                None => 10. * precision::p_confusion(),
            };
            bof = concat.add(&tc, eps_v, true, false, 20)?;
            if !bof {
                bof = concat.add(&tc, 200. * eps_v, true, false, 20)?;
            }
            ii += 1;
        }
        Ok(concat.bspline_curve().cast::<dyn GeomCurve>())
    }

    /// The ends of the edges `index` and `index + 1` (the last and the first for index
    /// 0 or the last index): the common vertex as each edge sees it (the last ends
    /// when there is none, as natively).
    fn junction(&self, index: i32) -> Result<Option<(Edge, Edge)>, Failure> {
        let n: i32 = self.my_edges.len() as i32;
        if index == 0 || index == n {
            if !self.base.uclosed {
                return Ok(None);
            }
            return Ok(Some((self.edge(n)?, self.edge(1)?)));
        }
        Ok(Some((self.edge(index)?, self.edge(index + 1)?)))
    }
}

/// The common vertex of two edges as `(V1 on edge1, V2 on edge2)`: the first pair of
/// same vertices among (first1, first2), (first1, last2), (last1, first2), else the
/// last ends.
fn common_ends(edge1: &Edge, edge2: &Edge) -> (Vertex, Vertex) {
    let (vv1, vv2) = topexp::vertices(edge1, false);
    let (vv3, vv4) = topexp::vertices(edge2, false);
    if vv1.is_same(&vv3) {
        return (vv1, vv3);
    }
    if vv1.is_same(&vv4) {
        return (vv1, vv4);
    }
    if vv2.is_same(&vv3) {
        return (vv2, vv3);
    }
    (vv2, vv4)
}

impl BRepFillSectionLaw for BRepFillShapeLaw {
    fn section_law_base(&self) -> &BRepFillSectionLawBase {
        &self.base
    }

    /// `IsVertex()`: the section is a vertex.
    fn is_vertex(&self) -> bool {
        self.vertex
    }

    /// `IsConstant()`: no law function scales the section.
    fn is_constant(&self) -> bool {
        self.the_law.is_null()
    }

    /// `Vertex(Index, Param)`: the first vertex of edge `index` in its orientation (the
    /// last vertex of the last edge for `index = count + 1`, null beyond), scaled by
    /// the law at `param`.
    fn vertex(&self, index: i32, param: f64) -> Result<Vertex, Failure> {
        let n: i32 = self.my_edges.len() as i32;
        let mut v = Vertex::default();
        if index <= n {
            let e: Edge = self.edge(index)?;
            v = if e.orientation() == Orientation::Reversed {
                topexp::last_vertex(&e, false)
            } else {
                topexp::first_vertex(&e, false)
            };
        } else if index == n + 1 {
            let e: Edge = self.edge(index - 1)?;
            v = if e.orientation() == Orientation::Reversed {
                topexp::first_vertex(&e, false)
            } else {
                topexp::last_vertex(&e, false)
            };
        }
        if let Some(law) = self.the_law.get() {
            v = topods::vertex(&transformed(&v, law_scaling(law, param)?)?)?;
        }
        Ok(v)
    }

    /// `VertexTol(Index, Param)`: the gap at a vertex (Precision::Confusion() at the ends
    /// of an open section).
    fn vertex_tol(&self, index: i32, param: f64) -> Result<f64, Failure> {
        let n: i32 = self.my_edges.len() as i32;
        let pair: (i32, i32) = if index == 0 || index == n {
            if !self.base.uclosed {
                // The least possible error.
                return Ok(precision::confusion());
            }
            (n, 1)
        } else {
            (index, index + 1)
        };
        vertex_gap(self.base.built_laws()?, pair, param)
    }

    /// `ConcatenedLaw()`: the only law, or the law of the concatenated edge curves.
    fn concatened_law(&self) -> Result<Handle<dyn GeomFillSectionLaw>, Failure> {
        let laws = self.base.built_laws()?;
        if laws.len() == 1 {
            return Ok(laws[0].clone());
        }
        if topods::wire(&self.my_shape)?.is_null() {
            return Ok(Handle::default());
        }
        section_of(&self.concatenation()?, &self.the_law)
    }

    /// `Continuity(Index, TolAngular)`: the continuity at the junction (C0 at the ends
    /// of an open section).
    fn continuity(&self, index: i32, tol_angular: f64) -> Result<Continuity, Failure> {
        let Some((edge1, edge2)) = self.junction(index)? else {
            // The least possible error.
            return Ok(Continuity::C0);
        };
        let (v1, v2) = common_ends(&edge1, &edge2);
        let u1: f64 = BRepTool::parameter(&v1, &edge1)?;
        let u2: f64 = BRepTool::parameter(&v2, &edge2)?;
        let curve1 = BRepAdaptorCurve::with_edge(&edge1)?;
        let curve2 = BRepAdaptorCurve::with_edge(&edge2)?;
        let eps: f64 = BRepTool::vertex_tolerance(&v2)? + BRepTool::vertex_tolerance(&v1)?;
        continuity_with_tolerances(&curve1, &curve2, u1, u2, eps, tol_angular)
    }

    /// `D0(U, S)`: the shape, scaled by the law at `u`.
    fn d0(&self, u: f64) -> Result<Shape, Failure> {
        match self.the_law.get() {
            Some(law) => transformed(&self.my_shape, law_scaling(law, u)?),
            None => Ok(self.my_shape.clone()),
        }
    }
}
