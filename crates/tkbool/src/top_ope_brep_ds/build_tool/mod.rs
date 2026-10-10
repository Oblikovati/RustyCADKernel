// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_BuildTool.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepDS_BuildTool`: the construction of the shapes of a topological
//! operation from the data structure (vertices, edges, faces, ...), with the
//! approximation and p-curves of the section edges. The native members casting a
//! `TopoDS_Shape&` in place take `&mut Shape` and raise Standard_TypeMismatch for a
//! non-null shape of another type.
mod curves;
mod pcurve;
mod topology;

use super::{TopOpeBRepDSCurve, TopOpeBRepDSPoint, TopOpeBRepDSSurface};
use crate::top_ope_brep_tool::{
    TopOpeBRepToolCurveTool, TopOpeBRepToolGeomTool, TopOpeBRepToolOutCurveType,
};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Shell, Solid, Vertex, Wire};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;

pub use curves::{
    EdgeCurveTol, fun_get_uv, fun_make_uiso_line_on_sphe, fun_reverse_pc, fun_uiso_line_on_sphe,
};
pub use pcurve::topopebrepds_set_the_pcurve;
pub use topology::OrientedEdgeVertices;

/// The builder of the shapes of a topological operation (`TopOpeBRepDS_BuildTool`);
/// overwrites the p-curves and translates them onto periodic faces by default;
/// `let bt = TopOpeBRepDSBuildTool::new();`.
#[derive(Clone, Copy, Debug)]
pub struct TopOpeBRepDSBuildTool {
    my_builder: BRepBuilder,
    my_curve_tool: TopOpeBRepToolCurveTool,
    my_over_write: bool,
    my_translate: bool,
}

impl Default for TopOpeBRepDSBuildTool {
    /// `TopOpeBRepDS_BuildTool()`: the APPROX curve tool.
    fn default() -> Self {
        Self::with_type(TopOpeBRepToolOutCurveType::Approx)
    }
}

/// The typed shape of an in-out `TopoDS_Shape&`: a new one for a null shape, the
/// shape itself (checked) otherwise.
fn typed<T: Default>(s: &Shape, cast: impl Fn(&Shape) -> Result<T, Failure>) -> Result<T, Failure> {
    if s.is_null() {
        return Ok(T::default());
    }
    cast(s)
}

impl TopOpeBRepDSBuildTool {
    /// `TopOpeBRepDS_BuildTool()`; `TopOpeBRepDSBuildTool::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `TopOpeBRepDS_BuildTool(OutCurveType)`;
    /// `TopOpeBRepDSBuildTool::with_type(TopOpeBRepToolOutCurveType::BSpline1)`.
    pub fn with_type(out_curve_type: TopOpeBRepToolOutCurveType) -> Self {
        Self {
            my_builder: BRepBuilder::default(),
            my_curve_tool: TopOpeBRepToolCurveTool::with_type(out_curve_type),
            my_over_write: true,
            my_translate: true,
        }
    }

    /// `TopOpeBRepDS_BuildTool(GT)`; `TopOpeBRepDSBuildTool::with_geom_tool(&gt)`.
    pub fn with_geom_tool(gt: &TopOpeBRepToolGeomTool) -> Self {
        Self {
            my_builder: BRepBuilder::default(),
            my_curve_tool: TopOpeBRepToolCurveTool::with_geom_tool(gt),
            my_over_write: true,
            my_translate: true,
        }
    }

    /// `GetGeomTool()`; `bt.get_geom_tool()`.
    pub fn get_geom_tool(&self) -> &TopOpeBRepToolGeomTool {
        self.my_curve_tool.get_geom_tool()
    }

    /// `ChangeGeomTool()`; `bt.change_geom_tool().define_curves(false)`.
    pub fn change_geom_tool(&mut self) -> &mut TopOpeBRepToolGeomTool {
        self.my_curve_tool.change_geom_tool()
    }

    /// `OverWrite()`; `bt.over_write()`.
    pub fn over_write(&self) -> bool {
        self.my_over_write
    }

    /// `OverWrite(O)`; `bt.set_over_write(false)`.
    pub fn set_over_write(&mut self, o: bool) {
        self.my_over_write = o;
    }

    /// `Translate()`; `bt.translate()`.
    pub fn translate(&self) -> bool {
        self.my_translate
    }

    /// `Translate(T)`; `bt.set_translate(false)`.
    pub fn set_translate(&mut self, t: bool) {
        self.my_translate = t;
    }

    /// `Approximation()`: the 3d curves are not B-splines of degree 1;
    /// `bt.approximation()`.
    pub fn approximation(&self) -> bool {
        self.my_curve_tool.get_geom_tool().type_c3d() != TopOpeBRepToolOutCurveType::BSpline1
    }

    /// `MakeVertex(V, P)`: a vertex at the point with its tolerance;
    /// `bt.make_vertex(&mut v, &p)?`.
    pub fn make_vertex(&self, v: &mut Shape, p: &TopOpeBRepDSPoint) -> Result<(), Failure> {
        let mut vertex: Vertex = typed(v, topods::vertex)?;
        self.my_builder
            .make_vertex_at(&mut vertex, p.point(), p.tolerance())?;
        *v = vertex.into_shape();
        Ok(())
    }

    /// `MakeEdge(E, C)`: an edge on the curve with its tolerance, or a degenerated
    /// edge without curve; `bt.make_edge(&mut e, &c)?`.
    pub fn make_edge(&self, e: &mut Shape, c: &TopOpeBRepDSCurve) -> Result<(), Failure> {
        let mut edge: Edge = typed(e, topods::edge)?;
        if c.curve().is_null() {
            self.my_builder.make_edge(&mut edge)?;
            self.my_builder.degenerated(&edge, true)?;
        } else {
            self.my_builder
                .make_edge_with_curve(&mut edge, c.curve(), c.tolerance())?;
        }
        *e = edge.into_shape();
        Ok(())
    }

    /// `MakeEdge(E, C, Tol)`; `bt.make_edge_with_curve(&mut e, &curve, 1e-7)?`.
    pub fn make_edge_with_curve(
        &self,
        e: &mut Shape,
        c: &Handle<dyn GeomCurve>,
        tol: f64,
    ) -> Result<(), Failure> {
        let mut edge: Edge = typed(e, topods::edge)?;
        self.my_builder.make_edge_with_curve(&mut edge, c, tol)?;
        *e = edge.into_shape();
        Ok(())
    }

    /// `MakeEdge(E)`: an empty edge; `bt.make_empty_edge(&mut e)?`.
    pub fn make_empty_edge(&self, e: &mut Shape) -> Result<(), Failure> {
        let mut edge: Edge = typed(e, topods::edge)?;
        self.my_builder.make_edge(&mut edge)?;
        *e = edge.into_shape();
        Ok(())
    }

    /// `MakeWire(W)`; `bt.make_wire(&mut w)?`.
    pub fn make_wire(&self, w: &mut Shape) -> Result<(), Failure> {
        let mut wire: Wire = typed(w, topods::wire)?;
        self.my_builder.make_wire(&mut wire);
        *w = wire.into_shape();
        Ok(())
    }

    /// `MakeFace(F, S)`: a face on the surface with its tolerance; `bt.make_face(&mut f, &s)?`.
    pub fn make_face(&self, f: &mut Shape, s: &TopOpeBRepDSSurface) -> Result<(), Failure> {
        let mut face: Face = typed(f, topods::face)?;
        self.my_builder
            .make_face_on_surface(&mut face, s.surface(), s.tolerance())?;
        *f = face.into_shape();
        Ok(())
    }

    /// `MakeShell(Sh)`; `bt.make_shell(&mut sh)?`.
    pub fn make_shell(&self, sh: &mut Shape) -> Result<(), Failure> {
        let mut shell: Shell = typed(sh, topods::shell)?;
        self.my_builder.make_shell(&mut shell);
        *sh = shell.into_shape();
        Ok(())
    }

    /// `MakeSolid(S)`; `bt.make_solid(&mut s)?`.
    pub fn make_solid(&self, s: &mut Shape) -> Result<(), Failure> {
        let mut solid: Solid = typed(s, topods::solid)?;
        self.my_builder.make_solid(&mut solid);
        *s = solid.into_shape();
        Ok(())
    }

    /// `CopyEdge(Ein, Eou)`: an empty copy of the edge with its range;
    /// `bt.copy_edge(&e, &mut copy)?`.
    pub fn copy_edge(&self, ein: &Shape, eou: &mut Shape) -> Result<(), Failure> {
        let e1 = topods::edge(ein)?;
        let (f, l) = rustycad_tkbrep::brep::BRepTool::range(&e1)?;
        *eou = ein.empty_copied()?;
        self.my_builder.range(&topods::edge(eou)?, f, l, false)
    }

    /// `CopyFace(Fin, Fou)`: an empty copy of the face; `bt.copy_face(&f, &mut copy)?`.
    pub fn copy_face(&self, fin: &Shape, fou: &mut Shape) -> Result<(), Failure> {
        *fou = fin.empty_copied()?;
        Ok(())
    }
}
