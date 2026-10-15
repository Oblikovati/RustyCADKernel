// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepBuild/TopOpeBRepBuild_Pave.hxx/.cxx
// and TopOpeBRepBuild_PaveSet.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepBuild_Pave` (a vertex with its parameter on an edge, a loop of the edge
//! builder) and `TopOpeBRepBuild_PaveSet` (the paves of an edge sorted along it).
use super::loops::{
    TopOpeBRepBuildLoop, TopOpeBRepBuildLoopBase, TopOpeBRepBuildLoopSet,
    TopOpeBRepBuildLoopSetBase,
};
use crate::top_ope_brep_ds::TopOpeBRepDSKind;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::{real_first, real_last};
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// The fields of a pave changed through its handles.
#[derive(Clone)]
struct PaveFields {
    vertex: Shape,
    param: f64,
    has_same_domain: bool,
    same_domain: Shape,
    int_type: TopOpeBRepDSKind,
}

/// A vertex at a parameter of an edge (`TopOpeBRepBuild_Pave`), a loop of the edge
/// builder; its members take `&self`, the paves being shared through handles;
/// `let pv = TopOpeBRepBuildPave::new(&v, 0.5, false);`.
pub struct TopOpeBRepBuildPave {
    parent: TopOpeBRepBuildLoopBase,
    my_is_shape: bool,
    fields: SharedState<PaveFields>,
}
impl_transient!(TopOpeBRepBuildPave, "TopOpeBRepBuild_Pave", TopOpeBRepBuildLoopBase, parent, interfaces: [dyn TopOpeBRepBuildLoop]);

impl TopOpeBRepBuildPave {
    /// `TopOpeBRepBuild_Pave(V, P, B)`: the vertex `v` at `p`, a shape loop when `b`; no
    /// same domain vertex, interference type FACE;
    /// `TopOpeBRepBuildPave::new(&v, 0.5, false)`.
    pub fn new(v: &Shape, p: f64, b: bool) -> Self {
        Self {
            parent: TopOpeBRepBuildLoopBase::from_shape(v),
            my_is_shape: b,
            fields: SharedState::new(PaveFields {
                vertex: v.clone(),
                param: p,
                has_same_domain: false,
                same_domain: Shape::default(),
                int_type: TopOpeBRepDSKind::Face,
            }),
        }
    }

    /// `HasSameDomain(B)`; `pv.set_has_same_domain(true)`.
    pub fn set_has_same_domain(&self, b: bool) {
        self.fields.write().has_same_domain = b;
    }

    /// `SameDomain(VSD)`; `pv.set_same_domain(&v2)`.
    pub fn set_same_domain(&self, vsd: &Shape) {
        self.fields.write().same_domain = vsd.clone();
    }

    /// `HasSameDomain()`; `pv.has_same_domain()`.
    pub fn has_same_domain(&self) -> bool {
        self.fields.read().has_same_domain
    }

    /// `SameDomain()`; `pv.same_domain()`.
    pub fn same_domain(&self) -> Shape {
        self.fields.read().same_domain.clone()
    }

    /// `Vertex()`; `pv.vertex()`.
    pub fn vertex(&self) -> Shape {
        self.fields.read().vertex.clone()
    }

    /// `ChangeVertex()`: the vertex changed in place by `f`;
    /// `pv.change_vertex(|v| v.set_orientation(Orientation::Forward))`.
    pub fn change_vertex<R>(&self, f: impl FnOnce(&mut Shape) -> R) -> R {
        f(&mut self.fields.write().vertex)
    }

    /// `Parameter()`; `pv.parameter()`.
    pub fn parameter(&self) -> f64 {
        self.fields.read().param
    }

    /// `Parameter(Par)`; `pv.set_parameter(0.25)`.
    pub fn set_parameter(&self, par: f64) {
        self.fields.write().param = par;
    }

    /// `InterferenceType()`: the kind of the interference of the vertex (a reference
    /// natively); `pv.interference_type()`.
    pub fn interference_type(&self) -> TopOpeBRepDSKind {
        self.fields.read().int_type
    }

    /// `InterferenceType() = K`; `pv.set_interference_type(TopOpeBRepDSKind::Edge)`.
    pub fn set_interference_type(&self, k: TopOpeBRepDSKind) {
        self.fields.write().int_type = k;
    }
}

impl TopOpeBRepBuildLoop for TopOpeBRepBuildPave {
    /// `IsShape()`: the flag of the constructor.
    fn is_shape(&self) -> bool {
        self.my_is_shape
    }

    /// `Shape()`: the vertex.
    fn shape(&self) -> Shape {
        self.vertex()
    }

    fn block_iterator(&self) -> super::TopOpeBRepBuildBlockIterator {
        self.parent.block_iterator()
    }
}

type Pv = Handle<TopOpeBRepBuildPave>;

/// The paves of an edge (`TopOpeBRepBuild_PaveSet`), a loop set of the edge builder;
/// `let mut ps = TopOpeBRepBuildPaveSet::new(&edge)?;`.
pub struct TopOpeBRepBuildPaveSet {
    base: TopOpeBRepBuildLoopSetBase,
    my_edge: Edge,
    my_vertices: Vec<Pv>,
    my_vertices_it: usize,
    my_has_equal_parameters: bool,
    my_equal_parameters: f64,
    my_closed: bool,
    my_prepare_done: bool,
    my_remove_pv: bool,
}

/// `FUN_islook(e)` (static): the vertices of the edge are apart.
fn islook(e: &Edge) -> Result<bool, Failure> {
    let (v1, v2) = topexp::vertices(e, false);
    let d = BRepTool::pnt(&v1)?.distance(&BRepTool::pnt(&v2)?);
    Ok(d.abs() > 1.0e-8)
}

impl TopOpeBRepBuildPaveSet {
    /// `TopOpeBRepBuild_PaveSet(E)`: the paves of the edge `e` (Standard_TypeMismatch for
    /// another shape); `TopOpeBRepBuildPaveSet::new(&edge)?`.
    pub fn new(e: &Shape) -> Result<Self, Failure> {
        Ok(Self {
            base: TopOpeBRepBuildLoopSetBase::default(),
            my_edge: topods::edge(e)?,
            my_vertices: Vec::new(),
            my_vertices_it: 0,
            my_has_equal_parameters: false,
            my_equal_parameters: 0.,
            my_closed: false,
            my_prepare_done: false,
            my_remove_pv: true,
        })
    }

    /// `RemovePV(B)`: whether `Prepare` merges the paves with the edge vertices;
    /// `ps.remove_pv(false)`.
    pub fn remove_pv(&mut self, b: bool) {
        self.my_remove_pv = b;
    }

    /// `Append(PV)`; `ps.append(&pv)`.
    pub fn append(&mut self, pv: &Pv) {
        self.my_vertices.push(pv.clone());
        self.my_prepare_done = false;
    }

    /// `SortPave(List, SortedList)` (static): the paves by increasing parameter, rotated
    /// to start at the first FORWARD vertex; `TopOpeBRepBuildPaveSet::sort_pave(&l, &mut s)`.
    pub fn sort_pave(list: &[Pv], sorted_list: &mut Vec<Pv>) {
        let n_pv = list.len();
        let mut taken = vec![false; n_pv + 1];
        let (mut pv1, mut i_pv): (Pv, usize) = (Handle::default(), 0);
        for _ in 1..=n_pv {
            let mut parmin = real_last();
            for (k, pv2) in list.iter().enumerate() {
                if !taken[k + 1] {
                    let par = pv2.get().map_or(0., |p| p.parameter());
                    if par < parmin {
                        (parmin, pv1, i_pv) = (par, pv2.clone(), k + 1);
                    }
                }
            }
            sorted_list.push(pv1.clone());
            taken[i_pv] = true;
        }
        let first_forward = sorted_list.iter().position(|pv| {
            pv.get()
                .is_some_and(|p| p.vertex().orientation() == Orientation::Forward)
        });
        if let Some(k) = first_forward {
            sorted_list.rotate_left(k);
        }
    }

    /// The edge vertex `ve` merged into the paves (`Prepare` loop body): whether it is
    /// to be added at the head.
    fn merge_edge_vertex(
        &mut self,
        ve: &Shape,
        edge_vertex_count: usize,
        is_ed: bool,
    ) -> Result<bool, Failure> {
        let ve_ori = ve.orientation();
        let ve_bound = ve_ori == Orientation::Forward || ve_ori == Orientation::Reversed;
        for k in edge_vertex_count..self.my_vertices.len() {
            let pv = self.my_vertices[k].clone();
            let pvr = pv
                .get()
                .ok_or_else(|| Failure::new("TopOpeBRepBuild_PaveSet: null pave"))?;
            let vi = topods::vertex(&pvr.vertex())?;
            let has_vsd = pvr.has_same_domain();
            let vsd = if has_vsd {
                topods::vertex(&pvr.same_domain())?.into_shape()
            } else {
                Shape::default()
            };
            let vi_ori = vi.orientation();
            let visameve = vi.is_same(ve);
            let vsdsameve = has_vsd && vsd.is_same(ve);
            if !((visameve || vsdsameve) && !is_ed) {
                continue;
            }
            let mut add = false;
            if ve_bound || vsdsameve {
                match vi_ori {
                    Orientation::External => {
                        self.my_vertices.remove(k);
                    }
                    Orientation::Internal => pvr.change_vertex(|v| v.set_orientation(ve_ori)),
                    _ if vi_ori != ve_ori => {
                        self.my_vertices.remove(k);
                        add = ve_bound && (vsdsameve || visameve) && islook(&self.my_edge)?;
                    }
                    _ => {}
                }
            }
            return Ok(add);
        }
        Ok(true)
    }

    /// `Prepare()` (protected): the edge vertices merged into the paves (unless
    /// `RemovePV(false)`), then the paves sorted; nothing when only edge vertices.
    fn prepare(&mut self) -> Result<(), Failure> {
        if self.my_prepare_done {
            return Ok(());
        }
        let is_ed = BRepTool::degenerated(&self.my_edge)?;
        let mut edge_vertex_count = 0;
        if self.my_remove_pv {
            for ve in Explorer::new(&self.my_edge, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                if self.merge_edge_vertex(&ve, edge_vertex_count, is_ed)? {
                    let par_ve = BRepTool::parameter(&topods::vertex(&ve)?, &self.my_edge)?;
                    self.my_vertices
                        .insert(0, Handle::new(TopOpeBRepBuildPave::new(&ve, par_ve, true)));
                    edge_vertex_count += 1;
                }
            }
        }
        let ll = self.my_vertices.len();
        if ll == edge_vertex_count {
            self.my_vertices.clear();
        } else if ll >= 2 {
            let list = std::mem::take(&mut self.my_vertices);
            Self::sort_pave(&list, &mut self.my_vertices);
        }
        self.my_prepare_done = true;
        Ok(())
    }

    /// `Edge()`; `ps.edge()`.
    pub fn edge(&self) -> &Edge {
        &self.my_edge
    }

    /// `HasEqualParameters()`: two different paves at the same parameter (within
    /// PConfusion), or a pave at the first parameter of the curve of the edge;
    /// `ps.has_equal_parameters()?`.
    pub fn has_equal_parameters(&mut self) -> Result<bool, Failure> {
        self.my_has_equal_parameters = false;
        'outer: for pv1 in &self.my_vertices {
            let (v1, p1) = pv1
                .get()
                .map_or((Shape::default(), 0.), |p| (p.vertex(), p.parameter()));
            for pv2 in &self.my_vertices {
                let (v2, p2) = pv2
                    .get()
                    .map_or((Shape::default(), 0.), |p| (p.vertex(), p.parameter()));
                if v2.is_equal(&v1) {
                    continue;
                }
                if (p1 - p2).abs() < precision::p_confusion() {
                    (self.my_has_equal_parameters, self.my_equal_parameters) = (true, p1);
                    break 'outer;
                }
            }
        }
        if !self.my_has_equal_parameters {
            let (c, ff, _) = BRepTool::curve(&self.my_edge)?;
            if !c.is_null() {
                for pv1 in &self.my_vertices {
                    let p1 = pv1.get().map_or(0., |p| p.parameter());
                    if (p1 - ff).abs() < precision::p_confusion() {
                        (self.my_has_equal_parameters, self.my_equal_parameters) = (true, ff);
                        break;
                    }
                }
            }
        }
        Ok(self.my_has_equal_parameters)
    }

    /// `EqualParameters()`: the parameter found by `HasEqualParameters` (0 without);
    /// `ps.equal_parameters()`.
    pub fn equal_parameters(&self) -> f64 {
        if self.my_has_equal_parameters {
            self.my_equal_parameters
        } else {
            0.
        }
    }

    /// `ClosedVertices()`: the paves of least and greatest parameters are the same
    /// vertex; `ps.closed_vertices()`.
    pub fn closed_vertices(&mut self) -> bool {
        if self.my_vertices.is_empty() {
            return false;
        }
        let (mut vmin, mut vmax) = (Shape::default(), Shape::default());
        let (mut parmin, mut parmax) = (real_last(), real_first());
        for pv in &self.my_vertices {
            let (v, par) = pv
                .get()
                .map_or((Shape::default(), 0.), |p| (p.vertex(), p.parameter()));
            if par > parmax {
                (vmax, parmax) = (v.clone(), par);
            }
            if par < parmin {
                (vmin, parmin) = (v, par);
            }
        }
        self.my_closed = vmin.is_same(&vmax);
        self.my_closed
    }
}

impl TopOpeBRepBuildLoopSet for TopOpeBRepBuildPaveSet {
    fn loop_set_base(&self) -> &TopOpeBRepBuildLoopSetBase {
        &self.base
    }

    fn loop_set_base_mut(&mut self) -> &mut TopOpeBRepBuildLoopSetBase {
        &mut self.base
    }

    /// `InitLoop()`: `Prepare` when needed, then at the first pave.
    fn init_loop(&mut self) -> Result<(), Failure> {
        if !self.my_prepare_done {
            self.prepare()?;
        }
        self.my_vertices_it = 0;
        Ok(())
    }

    fn more_loop(&self) -> bool {
        self.my_vertices_it < self.my_vertices.len()
    }

    fn next_loop(&mut self) -> Result<(), Failure> {
        self.my_vertices_it += 1;
        Ok(())
    }

    /// `Loop()`: the current pave.
    fn loop_(&self) -> Result<Handle<dyn TopOpeBRepBuildLoop>, Failure> {
        self.my_vertices
            .get(self.my_vertices_it)
            .map(|p| p.cast())
            .ok_or_else(|| {
                Failure::with_type(
                    "Standard_NoSuchObject",
                    "TopOpeBRepBuild_PaveSet::Loop: no current pave; expected MoreLoop()",
                    "TopOpeBRepBuild_PaveSet",
                )
            })
    }
}
