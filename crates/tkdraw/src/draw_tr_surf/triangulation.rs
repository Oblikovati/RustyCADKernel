// Rust translation of Draw/TKDraw/DrawTrSurf/DrawTrSurf_Triangulation.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DrawTrSurf_Triangulation`: a drawable triangulation (free edges in red, internal
//! edges in blue, optional node and triangle numbers).
//!
//! As for the polygons, `Dump` and `Save` hand the Poly writer the stream's number text.
use super::geometry::geometry;
use crate::draw::{
    DrawColor, DrawColorKind, DrawDisplay, DrawIStream, DrawInterpretor, DrawOStream, DrawSession,
    Drawable3D, Drawable3DBase, DrawableHandle,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::poly::{self, Connect, Triangulation};
use std::cell::Cell;
use std::rc::Rc;

/// The edges of a triangulation as node pairs: the free edges (one adjacent
/// triangle) and the internal edges (listed by the triangle of lower index).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TriangulationEdges {
    /// `myFree`: node pairs of the free edges.
    pub free: Vec<(i32, i32)>,
    /// `myInternals`: node pairs of the internal edges.
    pub internal: Vec<(i32, i32)>,
}

impl TriangulationEdges {
    /// The edges the constructor computes with `Poly_Connect`: for each triangle and
    /// each side `j` (nodes `j`, `j + 1`), a free edge when there is no adjacent
    /// triangle, an internal edge when the adjacent triangle has a higher index.
    pub fn compute(triangulation: &Handle<Triangulation>) -> Result<Self, Failure> {
        let mut edges = Self::default();
        let Some(mesh) = triangulation.get() else {
            return Ok(edges);
        };
        let connect = Connect::new(triangulation)?;
        for i in 1..=mesh.nb_triangles() {
            let (t1, t2, t3) = connect.triangles(i)?;
            let (n1, n2, n3) = mesh.triangle(i)?.get();
            let (adjacent, nodes) = ([t1, t2, t3], [n1, n2, n3]);
            for j in 0..3 {
                let side = (nodes[j], nodes[(j + 1) % 3]);
                if adjacent[j] == 0 {
                    edges.free.push(side);
                } else if i < adjacent[j] {
                    edges.internal.push(side);
                }
            }
        }
        Ok(edges)
    }
}

/// `DrawTrSurf_Triangulation`: a `Poly_Triangulation` with its edges classified once.
pub struct DrawTrSurfTriangulation {
    base: Drawable3DBase,
    triangulation: Handle<Triangulation>,
    edges: TriangulationEdges,
    show_nodes: Cell<bool>,
    show_triangles: Cell<bool>,
}

impl DrawTrSurfTriangulation {
    /// `DrawTrSurf_Triangulation(T)`: classifies the edges; no numbers shown. Without a
    /// triangulation (an unreadable record) there are no edges where the C++ code
    /// would dereference the null handle.
    pub fn new(triangulation: Handle<Triangulation>) -> Result<Self, Failure> {
        let edges = TriangulationEdges::compute(&triangulation)?;
        Ok(Self {
            base: Drawable3DBase::new(),
            triangulation,
            edges,
            show_nodes: Cell::new(false),
            show_triangles: Cell::new(false),
        })
    }

    /// `Triangulation`.
    pub fn triangulation(&self) -> Handle<Triangulation> {
        self.triangulation.clone()
    }

    /// The classified edges (`myFree`, `myInternals`).
    pub fn edges(&self) -> &TriangulationEdges {
        &self.edges
    }

    /// `ShowNodes()`.
    pub fn show_nodes(&self) -> bool {
        self.show_nodes.get()
    }

    /// `ShowNodes(bool)`: the node numbers.
    pub fn set_show_nodes(&self, show: bool) {
        self.show_nodes.set(show);
    }

    /// `ShowTriangles()`.
    pub fn show_triangles(&self) -> bool {
        self.show_triangles.get()
    }

    /// `ShowTriangles(bool)`: the triangle numbers at the centroids.
    pub fn set_show_triangles(&self, show: bool) {
        self.show_triangles.set(show);
    }

    /// `RegisterFactory` (the `Draw_Drawable3D_FACTORY` macro).
    pub fn register_factory(session: &DrawSession) {
        session.register_factory("DrawTrSurf_Triangulation", Self::restore);
    }

    /// `Restore`: `Poly::ReadTriangulation`; text that does not read gives a drawable
    /// without triangulation.
    pub fn restore(
        _session: &DrawSession,
        stream: &mut DrawIStream,
    ) -> Result<Option<DrawableHandle>, Failure> {
        let triangulation = stream.read_buffered(|input| poly::read_triangulation(input));
        let drawable = Self::new(triangulation.unwrap_or_default())?;
        Ok(Some(Rc::new(drawable)))
    }

    /// The node numbers at the nodes, in yellow.
    fn draw_node_numbers(
        &self,
        mesh: &Triangulation,
        display: &mut DrawDisplay<'_>,
    ) -> Result<(), Failure> {
        display.set_color(DrawColor::new(DrawColorKind::Jaune));
        for i in 1..=mesh.nb_nodes() {
            display.draw_string_3d(mesh.node(i)?, &i.to_string());
        }
        Ok(())
    }

    /// The triangle numbers at the centroids, in green.
    fn draw_triangle_numbers(
        &self,
        mesh: &Triangulation,
        display: &mut DrawDisplay<'_>,
    ) -> Result<(), Failure> {
        display.set_color(DrawColor::new(DrawColorKind::Vert));
        for i in 1..=mesh.nb_triangles() {
            let (n1, n2, n3) = mesh.triangle(i)?.get();
            let centroid = centroid([mesh.node(n1)?, mesh.node(n2)?, mesh.node(n3)?]);
            display.draw_string_3d(centroid, &i.to_string());
        }
        Ok(())
    }
}

/// `gp_XYZ(0, 0, 0)`, `Add` of each node, then `Multiply(1. / 3.)`.
fn centroid(nodes: [Point3; 3]) -> Point3 {
    let (mut x, mut y, mut z) = (0.0, 0.0, 0.0);
    for node in nodes {
        x += node.x();
        y += node.y();
        z += node.z();
    }
    let third = 1. / 3.;
    Point3::new(x * third, y * third, z * third)
}

impl Drawable3D for DrawTrSurfTriangulation {
    fn base(&self) -> &Drawable3DBase {
        &self.base
    }

    fn type_name(&self) -> &'static str {
        "DrawTrSurf_Triangulation"
    }

    /// `DrawOn`: free edges in red, internal edges in blue, then the numbers.
    fn draw_on(&self, display: &mut DrawDisplay<'_>) -> Result<(), Failure> {
        let mesh = geometry(&self.triangulation)?;
        let sets = [
            (DrawColorKind::Rouge, &self.edges.free),
            (DrawColorKind::Bleu, &self.edges.internal),
        ];
        for (color, edges) in sets {
            display.set_color(DrawColor::new(color));
            for (n1, n2) in edges {
                display.draw_3d(mesh.node(*n1)?, mesh.node(*n2)?);
            }
        }
        if self.show_nodes() {
            self.draw_node_numbers(mesh, display)?;
        }
        if self.show_triangles() {
            self.draw_triangle_numbers(mesh, display)?;
        }
        Ok(())
    }

    /// `Copy`: a new drawable on the same triangulation (edges classified again,
    /// numbers hidden).
    fn copy(&self, _this: &DrawableHandle) -> Result<DrawableHandle, Failure> {
        Ok(Rc::new(Self::new(self.triangulation.clone())?))
    }

    /// `Dump`: `Poly::Dump`.
    fn dump(&self, stream: &mut DrawOStream) -> Result<(), Failure> {
        let mesh = geometry(&self.triangulation)?;
        let real_text = stream.real_text();
        stream.write_bytes_through(|out| {
            poly::write_triangulation_with(mesh, out, false, &*real_text)
        })
    }

    /// `Save`: `Poly::Write` in scientific notation with 15 digits; the flags are left
    /// set afterwards, as the native `setf(aFlags)` (which only adds flags) leaves them.
    fn save(&self, stream: &mut DrawOStream) -> Result<(), Failure> {
        let mesh = geometry(&self.triangulation)?;
        stream.set_scientific(true).set_precision(15);
        let real_text = stream.real_text();
        stream
            .write_bytes_through(|out| poly::write_triangulation_with(mesh, out, true, &*real_text))
    }

    fn whatis(&self, di: &DrawInterpretor) -> Result<(), Failure> {
        di.append("triangulation")?;
        Ok(())
    }
}
