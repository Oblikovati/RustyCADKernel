// Rust translation of BRepTopAdaptor_TopolTool.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTopAdaptor_TopolTool`: the topology of a face seen from its surface adaptor:
//! the pcurves of its edges, the vertices of an edge, the classification of UV points
//! (through `BRepTopAdaptor_FClass2d`) and the sample points of the intersections.
use super::errors::{
    construction_error, domain_error, no_such_object, not_implemented, null_object,
};
use super::fclass2d::BRepTopAdaptorFClass2d;
use super::h_vertex::BRepTopAdaptorHVertex;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve2d, BRepAdaptorSurface};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::{Adaptor3dHVertex, Adaptor3dSurface, Adaptor3dTopolTool};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkmath::gp::{Point2, Point3};

/// The fields of one `BRepTopAdaptor_TopolTool` (its own and the base's protected
/// `myS`, `myNbSamplesU` and `myNbSamplesV`).
pub(super) struct BRepTopolState {
    pub(super) surface: Handle<dyn Adaptor3dSurface>,
    pub(super) nb_samples_u: i32,
    pub(super) nb_samples_v: i32,
    v_iterator: Explorer,
    face: Face,
    curve: Handle<BRepAdaptorCurve2d>,
    curves: Vec<Handle<BRepAdaptorCurve2d>>,
    // The list iterator: None before Init (More is false), else the position.
    c_iterator: Option<usize>,
    pub(super) u0: f64,
    pub(super) v0: f64,
    pub(super) du: f64,
    pub(super) dv: f64,
}

/// `BRepTopAdaptor_TopolTool` (docs/topalgo-design.md section 4): the face of a
/// `BRepAdaptor_Surface` with the pcurves of its edges as arcs, its vertices as
/// `BRepTopAdaptor_HVertex`, a lazily built `BRepTopAdaptor_FClass2d` for Classify
/// and IsThePointOn, and sample counts by surface kind. Every member takes `&self`,
/// as the base does; the base part (the parent field) only carries the RTTI chain.
///
/// ```
/// use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorTopolTool;
/// let tool = BRepTopAdaptorTopolTool::new();
/// assert!(!tool.more());
/// assert!(tool.has_3d());
/// ```
pub struct BRepTopAdaptorTopolTool {
    parent: Adaptor3dTopolTool,
    pub(super) state: SharedState<BRepTopolState>,
    fclass2d: SharedState<Option<BRepTopAdaptorFClass2d>>,
}
impl_transient!(
    BRepTopAdaptorTopolTool,
    "BRepTopAdaptor_TopolTool",
    Adaptor3dTopolTool,
    parent
);

impl Default for BRepTopAdaptorTopolTool {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepTopAdaptorTopolTool {
    /// `BRepTopAdaptor_TopolTool()`: no face, no curve, samples not computed;
    /// `BRepTopAdaptorTopolTool::new()`.
    pub fn new() -> Self {
        let state = BRepTopolState {
            surface: Handle::default(),
            nb_samples_u: -1,
            nb_samples_v: -1,
            v_iterator: Explorer::default(),
            face: Face::default(),
            curve: Handle::default(),
            curves: Vec::new(),
            c_iterator: None,
            u0: 0.0,
            v0: 0.0,
            du: 0.0,
            dv: 0.0,
        };
        Self {
            parent: Adaptor3dTopolTool::default(),
            state: SharedState::new(state),
            fclass2d: SharedState::new(None),
        }
    }

    /// `BRepTopAdaptor_TopolTool(S)`: [`Self::initialize_surface`] on `surface`. The
    /// sample grid origin and steps stay 0 until the samples are computed
    /// (uninitialized natively); `BRepTopAdaptorTopolTool::with_surface(&surface)?`.
    pub fn with_surface(surface: &Handle<dyn Adaptor3dSurface>) -> Result<Self, Failure> {
        let tool = Self::new();
        tool.initialize_surface(surface)?;
        Ok(tool)
    }

    /// `Initialize()`: Standard_NotImplemented; `tool.initialize()`.
    pub fn initialize(&self) -> Result<(), Failure> {
        Err(not_implemented("BRepTopAdaptor_TopolTool::Initialize()"))
    }

    /// `Initialize(S)`: the face of the BRepAdaptor_Surface `surface` (FORWARD) and a
    /// pcurve adaptor per edge; drops the classifier and the sample counts;
    /// Standard_ConstructionError when `surface` is not a BRepAdaptor_Surface;
    /// `tool.initialize_surface(&surface)?`.
    pub fn initialize_surface(
        &self,
        surface: &Handle<dyn Adaptor3dSurface>,
    ) -> Result<(), Failure> {
        let brhs: Handle<BRepAdaptorSurface> = surface.cast();
        let Some(adaptor) = brhs.get() else {
            return Err(construction_error(
                "BRepTopAdaptor_TopolTool::Initialize: the surface is not a BRepAdaptor_Surface; expected one",
            ));
        };
        let face: Face = topods::face(&adaptor.face().oriented(Orientation::Forward))?;
        *self.fclass2d.write() = None;
        {
            let mut state = self.state.write();
            state.face = face.clone();
            state.nb_samples_u = -1;
            state.surface = surface.clone();
            state.curves.clear();
        }
        // An adaptor that raises leaves the curves of the edges before it, as natively.
        let mut explorer = Explorer::new(&face, ShapeEnum::Edge, ShapeEnum::Shape)?;
        while explorer.more() {
            let edge: Edge = topods::edge(explorer.current())?;
            let curve = Handle::new(BRepAdaptorCurve2d::with_edge_on_face(&edge, &face)?);
            self.state.write().curves.push(curve);
            explorer.next()?;
        }
        self.state.write().c_iterator = None;
        Ok(())
    }

    /// `Initialize(C)`: the arc whose vertices the vertex iterator runs on;
    /// Standard_ConstructionError when `curve` is not a BRepAdaptor_Curve2d;
    /// `tool.initialize_curve(&curve)?`.
    pub fn initialize_curve(&self, curve: &Handle<dyn Adaptor2dCurve2d>) -> Result<(), Failure> {
        let brhc: Handle<BRepAdaptorCurve2d> = curve.cast();
        let is_null: bool = brhc.is_null();
        self.state.write().curve = brhc;
        if is_null {
            return Err(construction_error(
                "BRepTopAdaptor_TopolTool::Initialize: the curve is not a BRepAdaptor_Curve2d; expected one",
            ));
        }
        Ok(())
    }

    /// `Init()`: restarts the arc iterator on the edge pcurves; `tool.init()`.
    pub fn init(&self) {
        self.state.write().c_iterator = Some(0);
    }

    /// `More()`: whether an arc remains; `tool.more()`.
    pub fn more(&self) -> bool {
        let state = self.state.read();
        state
            .c_iterator
            .is_some_and(|index| index < state.curves.len())
    }

    /// `Next()`: moves to the next arc (a null dereference natively past the end, a
    /// no-op here); `tool.next()`.
    pub fn next(&self) {
        let mut state = self.state.write();
        let len: usize = state.curves.len();
        if let Some(index) = state.c_iterator.as_mut().filter(|index| **index < len) {
            *index += 1;
        }
    }

    /// `Value()`: the pcurve adaptor of the current arc; Standard_NoSuchObject past
    /// the end (a null dereference natively); `tool.value()?`.
    pub fn value(&self) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
        Ok(self.current_curve("Value")?.cast())
    }

    /// `Edge()`: the edge of the current arc (a pointer to it natively);
    /// Standard_NoSuchObject past the end; `tool.edge()?`.
    pub fn edge(&self) -> Result<Edge, Failure> {
        let curve = self.current_curve("Edge")?;
        Ok(curve.get().expect("non-null current curve").edge())
    }

    // The current arc of the iterator.
    fn current_curve(&self, member: &str) -> Result<Handle<BRepAdaptorCurve2d>, Failure> {
        let state = self.state.read();
        state
            .c_iterator
            .and_then(|index| state.curves.get(index))
            .cloned()
            .ok_or_else(|| {
                no_such_object(&format!(
                    "BRepTopAdaptor_TopolTool::{member}: arc iterator at {:?} of {} arcs; expected More()",
                    state.c_iterator,
                    state.curves.len()
                ))
            })
    }

    /// `InitVertexIterator()`: the vertices of the edge of the arc given to
    /// [`Self::initialize_curve`]; Standard_NullObject without one;
    /// `tool.init_vertex_iterator()?`.
    pub fn init_vertex_iterator(&self) -> Result<(), Failure> {
        let mut state = self.state.write();
        let Some(curve) = state.curve.get() else {
            return Err(null_object(
                "BRepTopAdaptor_TopolTool::InitVertexIterator: no arc; expected Initialize(C) first",
            ));
        };
        let edge: Edge = curve.edge();
        state.v_iterator = Explorer::new(&edge, ShapeEnum::Vertex, ShapeEnum::Shape)?;
        Ok(())
    }

    /// `MoreVertex()`; `tool.more_vertex()`.
    pub fn more_vertex(&self) -> bool {
        self.state.read().v_iterator.more()
    }

    /// `NextVertex()`; `tool.next_vertex()?`.
    pub fn next_vertex(&self) -> Result<(), Failure> {
        self.state.write().v_iterator.next()
    }

    /// `Vertex()`: a new BRepTopAdaptor_HVertex of the current vertex on the arc, as
    /// its base handle; Standard_TypeMismatch past the last vertex (the explorer's
    /// current shape is then the edge); `tool.vertex()?`.
    pub fn vertex(&self) -> Result<Handle<Adaptor3dHVertex>, Failure> {
        let state = self.state.read();
        let vertex = topods::vertex(state.v_iterator.current())?;
        let hvertex = Handle::new(BRepTopAdaptorHVertex::new(&vertex, &state.curve));
        Ok(hvertex.cast())
    }

    /// `Classify(P2d, Tol, RecadreOnPeriodic)`: UNKNOWN without a face, else the
    /// state from the classifier built with the first tolerance given to Classify or
    /// IsThePointOn; `tool.classify(Point2::new(0.5, 0.5), 1e-7, true)?`.
    pub fn classify(
        &self,
        point: Point2,
        tolerance: f64,
        recadre_on_periodic: bool,
    ) -> Result<State, Failure> {
        if self.state.read().face.is_null() {
            return Ok(State::Unknown);
        }
        self.with_classifier(tolerance, |classifier| {
            classifier.perform(point, recadre_on_periodic)
        })
    }

    /// `IsThePointOn(P2d, Tol, RecadreOnPeriodic)`: whether TestOnRestriction answers
    /// ON (the classifier is built as in [`Self::classify`], face or not);
    /// `tool.is_the_point_on(Point2::new(0.0, 0.5), 1e-7, true)?`.
    pub fn is_the_point_on(
        &self,
        point: Point2,
        tolerance: f64,
        recadre_on_periodic: bool,
    ) -> Result<bool, Failure> {
        self.with_classifier(tolerance, |classifier| {
            let state = classifier.test_on_restriction(point, tolerance, recadre_on_periodic)?;
            Ok(state == State::On)
        })
    }

    // Runs `body` on the classifier, building it with `tolerance` when there is none.
    fn with_classifier<T>(
        &self,
        tolerance: f64,
        body: impl FnOnce(&BRepTopAdaptorFClass2d) -> Result<T, Failure>,
    ) -> Result<T, Failure> {
        if self.fclass2d.read().is_none() {
            let face: Face = self.state.read().face.clone();
            let classifier = BRepTopAdaptorFClass2d::new(&face, tolerance)?;
            *self.fclass2d.write() = Some(classifier);
        }
        let guard = self.fclass2d.read();
        body(guard.as_ref().expect("classifier built above"))
    }

    /// `Destroy()`: drops the classifier; `tool.destroy()`.
    pub fn destroy(&self) {
        *self.fclass2d.write() = None;
    }

    /// `Orientation(C)`: the orientation of the edge of the arc; Standard_NullObject
    /// when `curve` is not a BRepAdaptor_Curve2d; `tool.curve_orientation(&arc)?`.
    pub fn curve_orientation(
        &self,
        curve: &Handle<dyn Adaptor2dCurve2d>,
    ) -> Result<Orientation, Failure> {
        let brhc: Handle<BRepAdaptorCurve2d> = curve.cast();
        let Some(adaptor) = brhc.get() else {
            return Err(null_object(
                "BRepTopAdaptor_TopolTool::Orientation: the arc is not a BRepAdaptor_Curve2d; expected one",
            ));
        };
        Ok(adaptor.edge().orientation())
    }

    /// `Orientation(V)`: the base member, `V->Orientation()` (the TopoDS orientation of
    /// a BRepTopAdaptor_HVertex); Standard_NullObject for a null handle;
    /// `tool.vertex_orientation(&vertex)?`.
    pub fn vertex_orientation(
        &self,
        vertex: &Handle<Adaptor3dHVertex>,
    ) -> Result<Orientation, Failure> {
        let brhv: Handle<BRepTopAdaptorHVertex> = vertex.cast();
        if let Some(brep_vertex) = brhv.get() {
            return Ok(brep_vertex.orientation());
        }
        vertex
            .get()
            .map(Adaptor3dHVertex::orientation)
            .ok_or_else(|| {
                null_object("BRepTopAdaptor_TopolTool::Orientation: null vertex; expected a vertex")
            })
    }

    /// `Has3d()`: the arcs and vertices have 3d tolerances; `tool.has_3d()`.
    pub fn has_3d(&self) -> bool {
        true
    }

    /// `Tol3d(C)`: the tolerance of the edge of the arc; Standard_DomainError when the
    /// arc is not a BRepAdaptor_Curve2d or has no edge; `tool.curve_tol_3d(&arc)?`.
    pub fn curve_tol_3d(&self, curve: &Handle<dyn Adaptor2dCurve2d>) -> Result<f64, Failure> {
        let no_3d = || domain_error("BRepTopAdaptor_TopolTool: arc has no 3d representation");
        let brhc: Handle<BRepAdaptorCurve2d> = curve.cast();
        let edge: Edge = brhc.get().ok_or_else(no_3d)?.edge();
        if edge.is_null() {
            return Err(no_3d());
        }
        BRepTool::edge_tolerance(&edge)
    }

    /// `Tol3d(V)`: the tolerance of the TopoDS vertex; Standard_DomainError when the
    /// vertex is not a BRepTopAdaptor_HVertex or its vertex is null;
    /// `tool.vertex_tol_3d(&vertex)?`.
    pub fn vertex_tol_3d(&self, vertex: &Handle<Adaptor3dHVertex>) -> Result<f64, Failure> {
        BRepTool::vertex_tolerance(&brep_vertex(vertex)?)
    }

    /// `Pnt(V)`: the point of the TopoDS vertex; Standard_DomainError as
    /// [`Self::vertex_tol_3d`]; `tool.pnt(&vertex)?`.
    pub fn pnt(&self, vertex: &Handle<Adaptor3dHVertex>) -> Result<Point3, Failure> {
        BRepTool::pnt(&brep_vertex(vertex)?)
    }
}

// The TopoDS vertex of a BRepTopAdaptor_HVertex handle, for Tol3d(V) and Pnt(V).
fn brep_vertex(
    vertex: &Handle<Adaptor3dHVertex>,
) -> Result<rustycad_tkbrep::topods::Vertex, Failure> {
    let no_3d = || domain_error("BRepTopAdaptor_TopolTool: vertex has no 3d representation");
    let brhv: Handle<BRepTopAdaptorHVertex> = vertex.cast();
    let topo = brhv.get().ok_or_else(no_3d)?.vertex().clone();
    if topo.is_null() {
        return Err(no_3d());
    }
    Ok(topo)
}
