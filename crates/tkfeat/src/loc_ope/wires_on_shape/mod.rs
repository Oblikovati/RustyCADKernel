// Rust translation of ModelingAlgorithms/TKFeat/LocOpe/LocOpe_WiresOnShape.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LocOpe_WiresOnShape`: the splitting edges and wires bound to the faces (or the
//! edges) of a shape they lie on, with the projection of their vertices on the edges
//! of the shape.
mod add;
mod bind_all;
mod intersections;
mod pcurves;
mod project;

use crate::errors::{construction_error, not_done};
use project::{project_2d, project_on_edge};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve2d;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Compound, Edge, Face, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::{MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{DataMap, IndexedDataMap};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The edges and the faces (or edges) they are bound to (`myMapEF`).
pub(crate) type EdgeFaceMap = IndexedDataMap<Shape, Shape, ShapeMapHasher>;

/// `LocOpe_WiresOnShape`: the splitting edges of a shape and where they lie.
///
/// ```
/// use rustycad_tkfeat::loc_ope::WiresOnShape;
/// use rustycad_tkbrep::topods::Shape;
/// let wires = WiresOnShape::new(&Shape::default());
/// assert!(!wires.is_done());
/// ```
#[derive(Clone)]
pub struct WiresOnShape {
    shape: Shape,
    map_ef: EdgeFaceMap,
    faces_with_section: MapOfShape,
    check_interior: bool,
    /// `myMap`: the edges of the wires and the edges of the shape they lie on, then the
    /// vertices of the wires and the vertex or edge of the shape they are projected on.
    map: DataMap<Shape, Shape, ShapeMapHasher>,
    done: bool,
    index: i32,
}

impl WiresOnShape {
    /// `LocOpe_WiresOnShape(S)`: no edge bound yet, the internal intersections checked;
    /// `WiresOnShape::new(&shape)`.
    pub fn new(s: &Shape) -> Self {
        Self {
            shape: s.clone(),
            map_ef: IndexedDataMap::default(),
            faces_with_section: MapOfShape::default(),
            check_interior: true,
            map: DataMap::default(),
            done: false,
            index: -1,
        }
    }

    /// `Init(S)`: a new shape; the bindings are forgotten (the faces with section are
    /// kept, as natively); `wires.init(&shape)`.
    pub fn init(&mut self, s: &Shape) {
        self.shape = s.clone();
        self.check_interior = true;
        self.done = false;
        self.map.clear(false);
        self.map_ef.clear(false);
    }

    /// `SetCheckInterior(ToCheckInterior)`: whether `BindAll` splits the edges at their
    /// internal intersections with the face edges; `wires.set_check_interior(false)`.
    pub fn set_check_interior(&mut self, to_check_interior: bool) {
        self.check_interior = to_check_interior;
    }

    /// `Bind(W, F)`: every edge of the wire on the face; `wires.bind_wire(&w, &f)?`.
    pub fn bind_wire(&mut self, w: &Wire, f: &Face) -> Result<(), Failure> {
        for e in Explorer::new(w, ShapeEnum::Edge, ShapeEnum::Shape)? {
            self.bind_edge(&topods::edge(&e)?, f)?;
        }
        Ok(())
    }

    /// `Bind(Comp, F)`: every edge of the compound on the face, which is marked as split
    /// by a section; `wires.bind_compound(&c, &f)?`.
    pub fn bind_compound(&mut self, comp: &Compound, f: &Face) -> Result<(), Failure> {
        for e in Explorer::new(comp, ShapeEnum::Edge, ShapeEnum::Shape)? {
            self.bind_edge(&topods::edge(&e)?, f)?;
        }
        self.faces_with_section.add(f.as_ref().clone());
        Ok(())
    }

    /// `Bind(E, F)`: the edge on the face, unless it is an edge of the face;
    /// Standard_ConstructionError when the edge is bound already;
    /// `wires.bind_edge(&e, &f)?`.
    pub fn bind_edge(&mut self, e: &Edge, f: &Face) -> Result<(), Failure> {
        if self.map_ef.contains(e) {
            return Err(construction_error(
                "LocOpe_WiresOnShape::Bind",
                "edge bound twice",
            ));
        }
        let mut edges = Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)?;
        if !edges.any(|x| x.is_same(e)) {
            self.map_ef.add(e.as_ref().clone(), f.as_ref().clone());
        }
        Ok(())
    }

    /// `Bind(EfromW, EonFace)`: the edge of a wire lies on the edge of the shape;
    /// `wires.bind_edge_on_edge(&ew, &es)`.
    pub fn bind_edge_on_edge(&mut self, ewir: &Edge, efac: &Edge) {
        self.map.bind(ewir.as_ref().clone(), efac.as_ref().clone());
    }

    /// `IsDone()`: `BindAll` ran; `wires.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// `InitEdgeIterator()`: binds all, then starts at the first edge;
    /// `wires.init_edge_iterator()?`.
    pub fn init_edge_iterator(&mut self) -> Result<(), Failure> {
        self.bind_all()?;
        self.index = 1;
        Ok(())
    }

    /// `MoreEdge()`; `wires.more_edge()`.
    pub fn more_edge(&self) -> bool {
        self.index >= 1 && self.index as usize <= self.map_ef.extent()
    }

    fn current_key(&self) -> Result<&Shape, Failure> {
        if self.index < 1 {
            return Err(not_done("LocOpe_WiresOnShape::Edge"));
        }
        self.map_ef.find_key(self.index as usize)
    }

    /// `Edge()`: the current edge; `wires.edge()?`.
    pub fn edge(&self) -> Result<Edge, Failure> {
        topods::edge(self.current_key()?)
    }

    /// `OnFace()`: the face the current edge lies on; `wires.on_face()?`.
    pub fn on_face(&self) -> Result<Face, Failure> {
        if self.index < 1 {
            return Err(not_done("LocOpe_WiresOnShape::OnFace"));
        }
        topods::face(self.map_ef.find_from_index(self.index as usize)?)
    }

    /// `OnEdge(E)`: the edge of the shape the current edge lies on, if any;
    /// `wires.on_edge()?`.
    pub fn on_edge(&self) -> Result<Option<Edge>, Failure> {
        match self.map.seek(self.current_key()?) {
            Some(e) => Ok(Some(topods::edge(e)?)),
            None => Ok(None),
        }
    }

    /// `NextEdge()`; `wires.next_edge()`.
    pub fn next_edge(&mut self) {
        self.index += 1;
    }

    /// `OnVertex(Vwire, Vshape)`: the vertex of the shape the vertex of a wire is bound
    /// to, if any; `wires.on_vertex(&v)?`.
    pub fn on_vertex(&self, vw: &Vertex) -> Result<Option<Vertex>, Failure> {
        match self.map.seek(vw) {
            Some(s) if s.shape_type()? == ShapeEnum::Vertex => Ok(Some(topods::vertex(s)?)),
            _ => Ok(None),
        }
    }

    /// The edge of the shape the vertex `v` is projected on.
    fn projected_edge(&self, v: &Vertex) -> Result<Option<Edge>, Failure> {
        match self.map.seek(v) {
            Some(s) if s.shape_type()? != ShapeEnum::Vertex => Ok(Some(topods::edge(s)?)),
            _ => Ok(None),
        }
    }

    /// `OnEdge(V, E, P)`: when the vertex `v` lies on an edge of the shape, that edge in
    /// `e` and the parameter of the projection in `p`, and true;
    /// `wires.on_edge_of_vertex(&v, &mut e, &mut p)?`.
    pub fn on_edge_of_vertex(
        &self,
        v: &Vertex,
        e: &mut Edge,
        p: &mut f64,
    ) -> Result<bool, Failure> {
        let Some(ed) = self.projected_edge(v)? else {
            return Ok(false);
        };
        *p = project_on_edge(v, &ed)?;
        *e = ed;
        Ok(true)
    }

    /// `OnEdge(V, EdgeFrom, E, P)`: as [`Self::on_edge_of_vertex`], the parameter taken on
    /// the p-curve of `edge_from` when the edge of the shape has no 3D curve. False, with
    /// `e` written (as natively), when `edge_from` is not bound;
    /// `wires.on_edge_of_vertex_from(&v, &ef, &mut e, &mut p)?`.
    pub fn on_edge_of_vertex_from(
        &self,
        v: &Vertex,
        edge_from: &Edge,
        e: &mut Edge,
        p: &mut f64,
    ) -> Result<bool, Failure> {
        let Some(ed) = self.projected_edge(v)? else {
            return Ok(false);
        };
        *e = ed.clone();
        let Some(a_shape) = self.map_ef.seek(edge_from) else {
            return Ok(false);
        };
        let (c, _, _) = BRepTool::curve(&ed)?;
        *p = if c.is_null() && a_shape.shape_type()? == ShapeEnum::Face {
            let a_face = topods::face(a_shape)?;
            let vtx_param = BRepTool::parameter(v, edge_from)?;
            let p2d =
                BRepAdaptorCurve2d::with_edge_on_face(edge_from, &a_face)?.value(vtx_param)?;
            project_2d(p2d, &ed, &a_face)?
        } else {
            project_on_edge(v, &ed)?
        };
        Ok(true)
    }

    /// `IsFaceWithSection(aFace)`: the face was bound to a compound of section edges;
    /// `wires.is_face_with_section(&face)`.
    pub fn is_face_with_section(&self, a_face: &Shape) -> bool {
        self.faces_with_section.contains(a_face)
    }
}
