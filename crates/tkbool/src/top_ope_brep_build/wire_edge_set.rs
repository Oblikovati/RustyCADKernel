// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepBuild/TopOpeBRepBuild_WireEdgeSet.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepBuild_WireEdgeSet`: the edges of the wires of a face being built, whose
//! neighbours through a vertex are filtered for closing edges (seams) by the turn
//! direction of their p-curves.
use super::shape_set::{TopOpeBRepBuildShapeSet, TopOpeBRepBuildShapeSetBase};
use crate::top_ope_brep_tool::fc2d_curve_on_surface;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dLine;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom_api::ProjectPointOnSurf;
use rustycad_tkgeombase::extrema::ExtAlgo;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Direction2, Point2, Vector2};

/// The edges of the wires of a face (`TopOpeBRepBuild_WireEdgeSet`);
/// `let mut wes = TopOpeBRepBuildWireEdgeSet::new(&face)?;`.
#[derive(Clone)]
pub struct TopOpeBRepBuildWireEdgeSet {
    base: TopOpeBRepBuildShapeSetBase,
    my_face: Face,
}

impl TopOpeBRepBuildWireEdgeSet {
    /// `TopOpeBRepBuild_WireEdgeSet(F, Addr)` (the address is unused): a set of edges
    /// with their vertices; `TopOpeBRepBuildWireEdgeSet::new(&face)?`.
    pub fn new(f: &Shape) -> Result<Self, Failure> {
        Ok(Self {
            base: TopOpeBRepBuildShapeSetBase::new(ShapeEnum::Vertex, true)?,
            my_face: topods::face(f)?,
        })
    }

    /// `Face()`; `wes.face()`.
    pub fn face(&self) -> &Face {
        &self.my_face
    }

    /// `IsUVISO(E, F, uiso, viso)`: whether the p-curve of the edge on the face (made
    /// if needed) is a U-iso or V-iso line; Standard_ProgramError without p-curve;
    /// `let (uiso, viso) = TopOpeBRepBuildWireEdgeSet::is_uviso(&e, &f)?;`.
    pub fn is_uviso(e: &Edge, f: &Face) -> Result<(bool, bool), Failure> {
        let (pc, _, _, _) = fc2d_curve_on_surface(e, f, true)?;
        if pc.is_null() {
            return Err(Failure::with_type(
                "Standard_ProgramError",
                "TopOpeBRepBuild_WireEdgeSet::IsUVISO: no p-curve; expected an edge with a p-curve on the face",
                "TopOpeBRepBuild_WireEdgeSet",
            ));
        }
        let line: Handle<Geom2dLine> = pc.cast();
        let Some(hl) = line.get() else {
            return Ok((false, false));
        };
        let d = hl.direction();
        let tol = precision::angular();
        if d.is_parallel(Direction2::Y, tol) {
            return Ok((true, false));
        }
        Ok((false, d.is_parallel(Direction2::X, tol)))
    }

    /// `IsClosed(E)` (private): closed on the face.
    fn is_closed(&self, e: &Shape) -> Result<bool, Failure> {
        BRepTool::is_closed_on_face(&topods::edge(e)?, &self.my_face)
    }

    /// `IsUClosed(E)` (private).
    fn is_uclosed(&self, e: &Shape) -> Result<bool, Failure> {
        Ok(Self::is_uviso(&topods::edge(e)?, &self.my_face)?.0)
    }

    /// `IsVClosed(E)` (private).
    fn is_vclosed(&self, e: &Shape) -> Result<bool, Failure> {
        Ok(Self::is_uviso(&topods::edge(e)?, &self.my_face)?.1)
    }

    /// `NbClosingShapes(L)` (private).
    fn nb_closing_shapes(&self, l: &List<Shape>) -> Result<i32, Failure> {
        let mut n = 0;
        for s in l.iter() {
            if self.is_closed(s)? {
                n += 1;
            }
        }
        Ok(n)
    }

    /// `VertexConnectsEdges(V, E1, E2, O1, O2)` (private): the orientations of `v` in
    /// both edges when they differ (the last compared ones otherwise).
    fn vertex_connects_edges(
        v: &Shape,
        e1: &Shape,
        e2: &Shape,
    ) -> Result<(bool, Option<(Orientation, Orientation)>), Failure> {
        let mut last = None;
        for x1 in Explorer::new(e1, ShapeEnum::Vertex, ShapeEnum::Shape)? {
            if !v.is_same(&x1) {
                continue;
            }
            for x2 in Explorer::new(e2, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                if v.is_same(&x2) {
                    let (o1, o2) = (x1.orientation(), x2.orientation());
                    last = Some((o1, o2));
                    if o1 != o2 {
                        return Ok((true, last));
                    }
                }
            }
        }
        Ok((false, last))
    }

    /// `VertexConnectsEdgesClosing(V, E1, E2)` (private): two closing edges along
    /// different isos (or one closing edge) connect through `v` with different
    /// orientations; two closing edges along the same iso connect when they are
    /// distinct with the same orientation.
    fn vertex_connects_edges_closing(
        &self,
        v: &Shape,
        e1: &Shape,
        e2: &Shape,
    ) -> Result<bool, Failure> {
        let (c1, c2) = (self.is_closed(e1)?, self.is_closed(e2)?);
        let mut testconnect = c1 || c2;
        if c1 && c2 {
            let (u1, v1) = (self.is_uclosed(e1)?, self.is_vclosed(e1)?);
            let (u2, v2) = (self.is_uclosed(e2)?, self.is_vclosed(e2)?);
            testconnect = (u1 && v2) || (u2 && v1);
        }
        if testconnect {
            return Ok(Self::vertex_connects_edges(v, e1, e2)?.0);
        }
        if c1 && c2 && e1.orientation() == e2.orientation() && !e1.is_equal(e2) {
            return Ok(Self::vertex_connects_edges(v, e1, e2)?.0);
        }
        Ok(false)
    }

    /// `LocalD1(F, E, V, p2, v2)` (private): the projection of the edge point at the
    /// vertex on the surface and the 3d tangent expressed on the surface derivatives.
    fn local_d1(&self, e: &Edge, v: &Vertex) -> Result<(Point2, Vector2), Failure> {
        let par_e = BRepTool::parameter(v, e)?;
        let (ce, _, _) = BRepTool::curve(e)?;
        let (p3d_e, d3d_e) = ce
            .get()
            .ok_or_else(|| Failure::new("TopOpeBRepBuild_WireEdgeSet::LocalD1: null curve"))?
            .d1(par_e)?;
        let s = BRepTool::surface(&self.my_face)?;
        let proj = ProjectPointOnSurf::with_surface(p3d_e, &s, ExtAlgo::Grad)?;
        let (u, v) = proj.lower_distance_parameters()?;
        let d1 = s
            .get()
            .ok_or_else(|| Failure::new("TopOpeBRepBuild_WireEdgeSet::LocalD1: null surface"))?
            .d1(u, v)?;
        Ok((
            Point2::new(u, v),
            Vector2::new(d3d_e.dot(d1.d1u), d3d_e.dot(d1.d1v)),
        ))
    }

    /// The oriented first derivative of the p-curve of the edge at the vertex.
    fn oriented_d1(&self, e: &Edge, v: &Vertex) -> Result<Vector2, Failure> {
        let par = BRepTool::parameter(v, e)?;
        let (pc, _, _, _) = fc2d_curve_on_surface(e, &self.my_face, true)?;
        let mut d1 = match pc.get() {
            Some(c) => c.d1(par)?.1,
            None => self.local_d1(e, v)?.1,
        };
        if e.orientation() == Orientation::Reversed {
            d1.reverse();
        }
        Ok(d1)
    }

    /// The closing neighbours kept by the turn direction from `e` at `v`.
    fn filter_closing(&mut self, e: &Edge, v: &Vertex) -> Result<(), Failure> {
        let d1e = self.oriented_d1(e, v)?;
        let neighbours: Vec<Shape> = self
            .base
            .my_current_shape_neighbours
            .iter()
            .cloned()
            .collect();
        let mut kept = List::default();
        for n in neighbours {
            if !self.is_closed(&n)? {
                kept.append(n);
                continue;
            }
            let ee = topods::edge(&n)?;
            let cross = d1e.crossed(self.oriented_d1(&ee, v)?);
            let (_, orientations) = Self::vertex_connects_edges(v.as_ref(), e.as_ref(), &n)?;
            let o_ve = orientations.map_or(Orientation::Forward, |o| o.0);
            let t2 = (cross > 0. && o_ve == Orientation::Reversed)
                || (cross < 0. && o_ve == Orientation::Forward);
            if t2 {
                kept.append(n);
            }
        }
        self.base.my_current_shape_neighbours = kept;
        Ok(())
    }
}

impl TopOpeBRepBuildShapeSet for TopOpeBRepBuildWireEdgeSet {
    fn shape_set_base(&self) -> &TopOpeBRepBuildShapeSetBase {
        &self.base
    }

    fn shape_set_base_mut(&mut self) -> &mut TopOpeBRepBuildShapeSetBase {
        &mut self.base
    }

    /// `AddShape(S)`: no check for a wire on a U- or V-closed face.
    fn add_shape(&mut self, s: &Shape) -> Result<(), Failure> {
        let mut tocheck = true;
        if s.shape_type()? == ShapeEnum::Wire {
            let bas = BRepAdaptorSurface::with_face(&self.my_face, false)?;
            if bas.is_u_closed()? || bas.is_v_closed()? {
                tocheck = false;
            }
        }
        if tocheck && !self.check_shape_of(s, false)? {
            return Ok(());
        }
        self.process_add_shape(s);
        Ok(())
    }

    /// `AddStartElement(S)`: no check for a B-spline or Bezier edge.
    fn add_start_element(&mut self, s: &Shape) -> Result<(), Failure> {
        let mut tocheck = true;
        if s.shape_type()? == ShapeEnum::Edge {
            let t = BRepAdaptorCurve::with_edge(&topods::edge(s)?)?.get_type()?;
            tocheck = !(t == CurveType::BSplineCurve || t == CurveType::BezierCurve);
        }
        if tocheck && !self.check_shape_of(s, false)? {
            return Ok(());
        }
        self.process_add_start_element(s)
    }

    /// `MakeNeighboursList(E, V)`: the edges of the vertex; with closing edges among
    /// them, those connecting to `e` (VertexConnectsEdgesClosing), and with two closing
    /// ones at least, the closing ones turning the right way from `e`.
    fn make_neighbours_list(&mut self, earg: &Shape, varg: &Shape) -> Result<List<Shape>, Failure> {
        let e = topods::edge(earg)?;
        let v = topods::vertex(varg)?;
        let l = self.base.my_sub_shape_map.find_from_key(varg)?.clone();
        if self.nb_closing_shapes(&l)? == 0 {
            return Ok(l);
        }
        self.base.my_current_shape_neighbours = List::default();
        for curn in l.iter() {
            if self.vertex_connects_edges_closing(varg, earg, curn)? {
                self.base.my_current_shape_neighbours.append(curn.clone());
            }
        }
        let current = self.base.my_current_shape_neighbours.clone();
        if self.nb_closing_shapes(&current)? >= 2 {
            self.filter_closing(&e, &v)?;
        }
        Ok(self.base.my_current_shape_neighbours.clone())
    }

    /// `SName(S, sb, sa)`: `sb` in a release build.
    fn sname(&self, _s: &Shape, sb: &str, _sa: &str) -> String {
        sb.to_owned()
    }

    /// `SNameori(S, sb, sa)`: `sb` in a release build.
    fn snameori(&self, _s: &Shape, sb: &str, _sa: &str) -> String {
        sb.to_owned()
    }
}
