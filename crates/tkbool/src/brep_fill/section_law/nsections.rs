// Rust translation of ModelingAlgorithms/TKBool/BRepFill/BRepFill_NSections.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFill_NSections`: the section law through several wires placed along a path.
use super::nsections_surface::{SkinInput, point_curve, totalsurf};
use super::vertex_tol::vertex_gap;
use super::{BRepFillSectionLaw, BRepFillSectionLawBase};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::brep_lprop::continuity_with_tolerances;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex, Wire};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBSplineSurface, GeomCurve, GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom_fill::{GeomFillNSections, GeomFillSectionLaw};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Point3, Transform3};
use rustycad_tktopalgo::brep_lib::{BRepLibMakeEdge, BRepLibMakeWire};

/// The section law through several wires (`BRepFill_NSections`): a surface skinned
/// through the sections, cut into one GeomFill_NSections law per edge rank;
/// `let law = BRepFillNSections::new(&sections, true)?;`.
pub struct BRepFillNSections {
    parent: StandardTransient,
    base: BRepFillSectionLawBase,
    v_first: f64,
    v_last: f64,
    my_shapes: Vec<Shape>,
    my_trsfs: Vec<Transform3>,
    my_params: Vec<f64>,
    my_edges: Array2<'static, Shape>,
    my_surface: Handle<GeomBSplineSurface>,
}
impl_transient!(
    BRepFillNSections,
    "BRepFill_NSections",
    dyn BRepFillSectionLaw,
    parent,
    interfaces: [dyn BRepFillSectionLaw]
);

/// Standard_OutOfRange for a table access.
fn table_error(what: &str, error: impl std::fmt::Display) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("BRepFill_NSections: {what}: {error}; expected an index of the edge table"),
        "BRepFill_NSections",
    )
}

impl BRepFillNSections {
    /// `BRepFill_NSections(S, Build)`: the sections at the parameters `0, 1, ...`, on
    /// the V range `[0, 1]`.
    pub fn new(s: &[Shape], build: bool) -> Result<Self, Failure> {
        let par: Vec<f64> = (0..s.len()).map(|i| i as f64).collect();
        let mut law = Self::empty(s, &[], &par, (0., 1.));
        law.init_sections(&par, build)?;
        law.base.done = true;
        Ok(law)
    }

    /// `BRepFill_NSections(S, Trsfs, P, VF, VL, Build)`: the sections at the strictly
    /// increasing parameters `P` (else not done), with their placements.
    pub fn with_placements(
        s: &[Shape],
        trsfs: &[Transform3],
        p: &[f64],
        vf: f64,
        vl: f64,
        build: bool,
    ) -> Result<Self, Failure> {
        let ok: bool = p.windows(2).all(|w| w[0] < w[1]);
        if !ok {
            let mut law = Self::empty(&[], &[], &[], (0., 0.));
            law.base.done = false;
            return Ok(law);
        }
        let mut law = Self::empty(s, trsfs, p, (vf, vl));
        law.init_sections(p, build)?;
        law.base.done = true;
        Ok(law)
    }

    /// The fields before the constructor bodies.
    fn empty(s: &[Shape], trsfs: &[Transform3], p: &[f64], v: (f64, f64)) -> Self {
        Self {
            parent: StandardTransient::default(),
            base: BRepFillSectionLawBase::default(),
            v_first: v.0,
            v_last: v.1,
            my_shapes: s.to_vec(),
            my_trsfs: trsfs.to_vec(),
            my_params: p.to_vec(),
            my_edges: Array2::default(),
            my_surface: Handle::default(),
        }
    }

    /// The section `j` (1-based) as a wire.
    fn section(&self, j: usize) -> Result<Wire, Failure> {
        let shape = self
            .my_shapes
            .get(j.wrapping_sub(1))
            .ok_or_else(|| table_error("section", j))?;
        topods::wire(shape)
    }

    /// The edge of the table at `(ii, jj)`.
    fn table_edge(&self, ii: i32, jj: i32) -> Result<Edge, Failure> {
        topods::edge(
            self.my_edges
                .value(ii, jj)
                .map_err(|e| table_error("edge", e))?,
        )
    }

    /// `Init(P, Build)`: the table of edges (the edge `ii` of each section, the
    /// degenerated edge of a point section at every rank), the closures, the skinned
    /// surface and, when built, the GeomFill_NSections law of each rank.
    fn init_sections(&mut self, p: &[f64], build: bool) -> Result<(), Failure> {
        let nb_sects: usize = p.len();
        let w1_point: bool = punctual(&self.section(1)?)?;
        let w2_point: bool = punctual(&self.section(nb_sects)?)?;
        let ideb: usize = if w1_point { 2 } else { 1 };
        let ifin: usize = if w2_point { nb_sects - 1 } else { nb_sects };
        // Whether the start and end wires are identical.
        self.base.vclosed = self.my_shapes[0].is_same(&self.my_shapes[nb_sects - 1]);
        let nb_edge: i32 = non_degenerated_count(&self.section(ideb)?)?;
        self.my_edges = Array2::with_bounds(1, nb_edge, 1, nb_sects as i32)
            .map_err(|e| table_error("table", e))?;
        self.base.uclosed = true;
        for jj in ideb..=ifin {
            self.fill_section(jj, nb_edge)?;
        }
        if w1_point {
            self.fill_point_section(1, nb_edge)?;
        }
        if w2_point {
            self.fill_point_section(nb_sects, nb_edge)?;
        }
        let input = SkinInput {
            shapes: &self.my_edges,
            nb_sects: self.my_shapes.len() as i32,
            nb_edges: nb_edge,
            params: &self.my_params,
            w1_point,
            w2_point,
            u_closed: self.base.uclosed,
            v_closed: self.base.vclosed,
        };
        self.my_surface = totalsurf(&input, precision::confusion())?;
        // Increase the degree so that the position D2 on GeomFill_NSections is correct.
        let surface = live_surface(&self.my_surface)?;
        if surface.v_degree() < 2 {
            surface.increase_degree(surface.u_degree(), 2)?;
        }
        self.base.laws = Some(self.rank_laws(nb_edge, build)?);
        Ok(())
    }

    /// The edges of section `jj` in the table, their indices (negative for REVERSED
    /// edges) and the U closure of the section.
    fn fill_section(&mut self, jj: usize, nb_edge: i32) -> Result<(), Failure> {
        let w: Wire = self.section(jj)?;
        let mut explorer = WireExplorer::with_wire(&w)?;
        for ii in 1..=nb_edge {
            let edge: Edge = explorer.current().clone();
            if !BRepTool::degenerated(&edge)? {
                *self
                    .my_edges
                    .change_value(ii, jj as i32)
                    .map_err(|e| table_error("edge", e))? = edge.as_ref().clone();
                let index: i32 = if edge.orientation() == Orientation::Forward {
                    ii
                } else {
                    -ii
                };
                self.base.indices.bind(edge.as_ref().clone(), index);
            }
            explorer.next()?;
        }
        // Is the law closed in U?
        let closed: bool = w.closed()? || self.section_ends_meet(nb_edge, jj as i32)?;
        if !closed {
            self.base.uclosed = false;
        }
        Ok(())
    }

    /// Whether the last edge of section `jj` ends where its first edge starts.
    fn section_ends_meet(&self, nb_edge: i32, jj: i32) -> Result<bool, Failure> {
        let edge1: Edge = self.table_edge(nb_edge, jj)?;
        let edge2: Edge = self.table_edge(1, jj)?;
        let (v1, v2) = junction_vertices(&edge1, &edge2);
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

    /// The first (degenerated) edge of a point section at every rank of the table.
    fn fill_point_section(&mut self, jj: usize, nb_edge: i32) -> Result<(), Failure> {
        let edge: Edge = WireExplorer::with_wire(&self.section(jj)?)?
            .current()
            .clone();
        for ii in 1..=nb_edge {
            *self
                .my_edges
                .change_value(ii, jj as i32)
                .map_err(|e| table_error("edge", e))? = edge.as_ref().clone();
        }
        Ok(())
    }

    /// The GeomFill_NSections law of each rank on the U range `[ii - 1, ii]` (null laws
    /// when not built).
    fn rank_laws(
        &self,
        nb_edge: i32,
        build: bool,
    ) -> Result<Vec<Handle<dyn GeomFillSectionLaw>>, Failure> {
        let mut laws: Vec<Handle<dyn GeomFillSectionLaw>> = Vec::with_capacity(nb_edge as usize);
        for ii in 1..=nb_edge {
            if !build {
                laws.push(Handle::default());
                continue;
            }
            let mut nc: Vec<Handle<dyn GeomCurve>> = Vec::new();
            for jj in 1..=self.my_shapes.len() as i32 {
                nc.push(rank_curve(&self.table_edge(ii, jj)?, ii)?);
            }
            let u: (f64, f64) = (f64::from(ii - 1), f64::from(ii));
            let law = GeomFillNSections::with_reference(
                &nc,
                &self.my_trsfs,
                &self.my_params,
                u,
                (self.v_first, self.v_last),
                &self.my_surface,
            )?;
            laws.push(Handle::new(law).cast::<dyn GeomFillSectionLaw>());
        }
        Ok(laws)
    }

    /// The iso V curve of the law `index` at `v` as a B-spline.
    fn law_iso(&self, index: i32, v: f64) -> Result<Handle<GeomBSplineCurve>, Failure> {
        let law = self.law(index)?;
        let law = law.get().ok_or_else(|| table_error("law", index))?;
        let surface = law.bspline_surface()?;
        Ok(live_surface(&surface)?.v_iso(v)?.cast::<GeomBSplineCurve>())
    }
}

/// The surface of a live handle.
fn live_surface(surface: &Handle<GeomBSplineSurface>) -> Result<&GeomBSplineSurface, Failure> {
    surface.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "BRepFill_NSections: null surface; expected the skinned surface",
            "BRepFill_NSections",
        )
    })
}

/// Whether every edge of the wire is degenerated.
fn punctual(w: &Wire) -> Result<bool, Failure> {
    let mut point: bool = true;
    let mut explorer = WireExplorer::with_wire(w)?;
    while explorer.more() {
        point = point && BRepTool::degenerated(explorer.current())?;
        explorer.next()?;
    }
    Ok(point)
}

/// The number of non-degenerated edges of a wire.
fn non_degenerated_count(w: &Wire) -> Result<i32, Failure> {
    let mut count: i32 = 0;
    let mut explorer = WireExplorer::with_wire(w)?;
    while explorer.more() {
        if !BRepTool::degenerated(explorer.current())? {
            count += 1;
        }
        explorer.next()?;
    }
    Ok(count)
}

/// The end vertex of `edge1` and the start vertex of `edge2`, in their orientations.
fn junction_vertices(edge1: &Edge, edge2: &Edge) -> (Vertex, Vertex) {
    let reversed = |e: &Edge| e.orientation() == Orientation::Reversed;
    let v1 = if reversed(edge1) {
        topexp::first_vertex(edge1, false)
    } else {
        topexp::last_vertex(edge1, false)
    };
    let v2 = if reversed(edge2) {
        topexp::last_vertex(edge2, false)
    } else {
        topexp::first_vertex(edge2, false)
    };
    (v1, v2)
}

/// The curve of one edge of the rank `ii`: a degree 1 curve for a degenerated edge,
/// else its 3d curve oriented as the edge and trimmed (unless it is the closed first
/// edge).
fn rank_curve(edge: &Edge, ii: i32) -> Result<Handle<dyn GeomCurve>, Failure> {
    if BRepTool::degenerated(edge)? {
        let (vl, vf) = topexp::vertices(edge, false);
        return Ok(point_curve(BRepTool::pnt(&vf)?, BRepTool::pnt(&vl)?)?.cast());
    }
    let (mut curve, mut first, mut last) = BRepTool::curve(edge)?;
    if edge.orientation() == Orientation::Reversed {
        let stored = curve.get().ok_or_else(|| table_error("curve", ii))?;
        // A reversed copy, to avoid spoiling the topology.
        let reversed = stored.reversed()?;
        let aux: f64 = stored.reversed_parameter(first)?;
        first = stored.reversed_parameter(last)?;
        last = aux;
        curve = reversed;
    }
    if ii > 1 || !BRepTool::is_closed(edge)? {
        curve = Handle::new(GeomTrimmedCurve::new(&curve, first, last, true, true)?).cast();
    }
    Ok(curve)
}

impl BRepFillSectionLaw for BRepFillNSections {
    fn section_law_base(&self) -> &BRepFillSectionLawBase {
        &self.base
    }

    /// `IsVertex()`: never.
    fn is_vertex(&self) -> bool {
        false
    }

    /// `IsConstant()`: never.
    fn is_constant(&self) -> bool {
        false
    }

    /// `Vertex(Index, Param)`: the point at the start of the iso V curve of law
    /// `index` (its end for `index = count + 1`; an empty vertex beyond).
    fn vertex(&self, index: i32, param: f64) -> Result<Vertex, Failure> {
        let builder = BRepBuilder::default();
        let mut v = Vertex::default();
        builder.make_vertex(&mut v);
        let col_length: i32 = self.my_edges.upper_row();
        let point: Option<Point3> = if index <= col_length {
            let curve = self.law_iso(index, param)?;
            let c = curve.get().ok_or_else(|| table_error("iso", index))?;
            Some(c.d0(c.first_parameter())?)
        } else if index == col_length + 1 {
            let curve = self.law_iso(index - 1, param)?;
            let c = curve.get().ok_or_else(|| table_error("iso", index))?;
            Some(c.d0(c.last_parameter())?)
        } else {
            None
        };
        if let Some(p) = point {
            builder.update_vertex_point(&v, p, precision::confusion())?;
        }
        Ok(v)
    }

    /// `VertexTol(Index, Param)`: the gap at a vertex (Precision::Confusion() at the ends
    /// of an open section).
    fn vertex_tol(&self, index: i32, param: f64) -> Result<f64, Failure> {
        let n: i32 = self.my_edges.upper_row();
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

    /// `ConcatenedLaw()`: the only law, or the GeomFill_NSections of the iso V curves of
    /// the whole surface.
    fn concatened_law(&self) -> Result<Handle<dyn GeomFillSectionLaw>, Failure> {
        let laws = self.base.built_laws()?;
        if laws.len() == 1 {
            return Ok(laws[0].clone());
        }
        let surface = live_surface(&self.my_surface)?;
        let (u_first, u_last, v_first, v_last) = surface.bounds();
        let mut compo: Vec<Handle<dyn GeomCurve>> = Vec::new();
        for &param in &self.my_params {
            compo.push(surface.v_iso(param)?);
        }
        let law = GeomFillNSections::with_reference(
            &compo,
            &self.my_trsfs,
            &self.my_params,
            (u_first, u_last),
            (v_first, v_last),
            &self.my_surface,
        )?;
        Ok(Handle::new(law).cast::<dyn GeomFillSectionLaw>())
    }

    /// `Continuity(Index, TolAngular)`: the lowest continuity over the sections at the
    /// junction (CN at a degenerated edge, C0 at the ends of an open section).
    fn continuity(&self, index: i32, tol_angular: f64) -> Result<Continuity, Failure> {
        let n: i32 = self.my_edges.upper_row();
        let mut cont = Continuity::C0;
        for jj in 1..=self.my_shapes.len() as i32 {
            let (edge1, edge2) = if index == 0 || index == n {
                if !self.base.uclosed {
                    // The least possible error.
                    return Ok(Continuity::C0);
                }
                (self.table_edge(n, jj)?, self.table_edge(1, jj)?)
            } else {
                (self.table_edge(index, jj)?, self.table_edge(index + 1, jj)?)
            };
            let cont_jj = junction_continuity(&edge1, &edge2, tol_angular)?;
            if jj == 1 || cont > cont_jj {
                cont = cont_jj;
            }
        }
        Ok(cont)
    }

    /// `D0(V, S)`: the wire of the iso V curves of the laws.
    fn d0(&self, v: f64) -> Result<Shape, Failure> {
        let mut maker = BRepLibMakeWire::new();
        for ii in 1..=self.nb_law()? {
            let curve = self.law_iso(ii, v)?;
            let c = curve.get().ok_or_else(|| table_error("iso", ii))?;
            let (first, last) = (c.first_parameter(), c.last_parameter());
            let edge = BRepLibMakeEdge::from_curve_params(&curve.cast(), first, last)?.edge()?;
            maker.add_edge(&edge)?;
        }
        Ok(maker.wire()?.into_shape().oriented(Orientation::Forward))
    }
}

/// The continuity of two edges at their junction; CN when one is degenerated.
fn junction_continuity(
    edge1: &Edge,
    edge2: &Edge,
    tol_angular: f64,
) -> Result<Continuity, Failure> {
    let (v1, v2) = junction_vertices(edge1, edge2);
    if BRepTool::degenerated(edge1)? || BRepTool::degenerated(edge2)? {
        return Ok(Continuity::CN);
    }
    let u1: f64 = BRepTool::parameter(&v1, edge1)?;
    let u2: f64 = BRepTool::parameter(&v2, edge2)?;
    let curve1 = BRepAdaptorCurve::with_edge(edge1)?;
    let curve2 = BRepAdaptorCurve::with_edge(edge2)?;
    let eps: f64 = BRepTool::vertex_tolerance(&v2)? + BRepTool::vertex_tolerance(&v1)?;
    continuity_with_tolerances(&curve1, &curve2, u1, u2, eps, tol_angular)
}
