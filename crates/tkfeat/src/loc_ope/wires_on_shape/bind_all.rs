// Rust translation of LocOpe_WiresOnShape::BindAll of
// ModelingAlgorithms/TKFeat/LocOpe/LocOpe_WiresOnShape.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BindAll`: the p-curves of the splitting edges, their split at internal
//! intersections, and the binding of their vertices to the vertices or edges of the
//! shape they lie on.
use super::WiresOnShape;
use super::intersections::find_internal_intersections;
use super::pcurves::{put_pcurve, put_pcurves};
use super::project::project_on_face;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve2d;
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point2;

/// The vertex parameters of the projections on edges (`aVertParam`).
type VertexParameters = DataMap<Shape, f64, ShapeMapHasher>;

/// The vertex of `eimg` the vertex `vtx` of a wire edge is bound to: none when it is a
/// vertex of `eimg` itself, a geometrically equal vertex, or else the edge `eimg`.
fn image_of_vertex(vtx: &Vertex, eimg: &Edge) -> Result<Option<Shape>, Failure> {
    for v2 in Explorer::new(eimg, ShapeEnum::Vertex, ShapeEnum::Shape)? {
        let vtx2 = topods::vertex(&v2)?;
        if vtx2.is_same(vtx) {
            return Ok(None);
        }
        if brep_tools::compare_vertices(vtx, &vtx2)? {
            return Ok(Some(v2));
        }
    }
    Ok(Some(eimg.as_ref().clone()))
}

/// The point of `vtx` on the p-curve of `edg` on `fac`, infinite without p-curve.
fn vertex_uv(vtx: &Vertex, edg: &Edge, fac: &Face) -> Result<Point2, Failure> {
    let vtx_param = BRepTool::parameter(vtx, edg)?;
    let ba_curve2d = BRepAdaptorCurve2d::with_edge_on_face(edg, fac)?;
    if ba_curve2d.geom2d_adaptor().curve().is_null() {
        return Ok(Point2::new(precision::infinite(), precision::infinite()));
    }
    ba_curve2d.value(vtx_param)
}

/// The vertex of the projection edge `epro` the vertex `vtx` coincides with: itself, or
/// a geometrically equal vertex at an end of the (non-degenerated) edge range on `fac`.
fn coinciding_vertex(
    vtx: &Vertex,
    epro: &Edge,
    fac: &Face,
    prm: f64,
) -> Result<Option<Shape>, Failure> {
    for v2 in Explorer::new(epro, ShapeEnum::Vertex, ShapeEnum::Shape)? {
        let vtx2 = topods::vertex(&v2)?;
        if vtx2.is_same(vtx) {
            return Ok(Some(v2));
        }
        if !brep_tools::compare_vertices(vtx, &vtx2)? {
            continue;
        }
        let (f1, l1) = BRepTool::range_on_face(epro, fac)?;
        let tol = precision::p_confusion();
        if !BRepTool::degenerated(epro)? && ((prm - f1).abs() <= tol || (prm - l1).abs() <= tol) {
            return Ok(Some(v2));
        }
    }
    Ok(None)
}

impl WiresOnShape {
    /// The p-curves of the wire edges bound to edges of the shape, and the binding of
    /// their vertices (detection des vertex a projeter ou a "binder" avec des vertex
    /// existants).
    fn bind_edges_on_edges(&mut self, the_map: &mut MapOfShape) -> Result<(), Failure> {
        let mut map_v: DataMap<Shape, Shape, ShapeMapHasher> = DataMap::default();
        let pairs: Vec<(Shape, Shape)> = self
            .map
            .items()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (key, value) in &pairs {
            let eref = topods::edge(key)?;
            let eimg = topods::edge(value)?;
            put_pcurves(&eref, &eimg, &self.shape)?;
            for v in Explorer::new(&eref, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                let vtx = topods::vertex(&v)?;
                if the_map.contains(&v) {
                    continue; // deja traite
                }
                if let Some(image) = image_of_vertex(&vtx, &eimg)? {
                    map_v.bind(v.clone(), image);
                }
                the_map.add(v);
            }
        }
        for (key, value) in map_v.items() {
            self.map.bind(key.clone(), value.clone());
        }
        Ok(())
    }

    /// The p-curves of the edges bound to faces and the split of those crossing an edge
    /// of their face (overlapping ones kept whole).
    fn split_at_internal_intersections(&mut self) -> Result<(), Failure> {
        let mut splits = IndexedDataMapOfShapeListOfShape::default();
        let mut overlapped = MapOfShape::default();
        for ind in 1..=self.map_ef.extent() {
            let edg = topods::edge(self.map_ef.find_key(ind)?)?;
            let fac = topods::face(self.map_ef.find_from_index(ind)?)?;
            put_pcurve(&edg, &fac)?;
            if BRepTool::curve_on_face(&edg, &fac)?.pcurve.is_null() {
                continue;
            }
            if self.check_interior {
                let mut is_overlapped = false;
                find_internal_intersections(&edg, &fac, &mut splits, &mut is_overlapped)?;
                if is_overlapped {
                    overlapped.add(edg.into_shape());
                }
            }
        }
        for ind in 1..=splits.extent() {
            let an_edge = splits.find_key(ind)?.clone();
            if overlapped.contains(&an_edge) {
                continue;
            }
            let a_face = self.map_ef.find_from_key(&an_edge)?.clone();
            // Remove "anEdge" from "myMapEF"
            self.map_ef.remove_key(&an_edge);
            for piece in splits.find_from_index(ind)?.iter() {
                self.map_ef.add(piece.clone(), a_face.clone());
            }
        }
        Ok(())
    }

    /// Binds the vertex `vtx` of the edge `edg` on `fac` to the vertex or edge of the
    /// face it projects on.
    fn project_vertex(
        &mut self,
        vtx: &Vertex,
        edg: &Edge,
        fac: &Face,
        the_map: &mut MapOfShape,
        vert_param: &mut VertexParameters,
    ) -> Result<(), Failure> {
        let p2d = vertex_uv(vtx, edg, fac)?;
        let mut epro = Edge::default();
        let mut prm = precision::infinite();
        let is_projected = self.map.is_bound(vtx);
        // if vertex was already projected on the current edge on the previous face
        // it is necessary to check tolerance of the vertex in the 2D space on the current
        // face without projection and update tolerance of vertex if it is necessary
        if is_projected {
            let a_sh = self.map.find(vtx)?;
            if a_sh.shape_type()? != ShapeEnum::Edge {
                return Ok(());
            }
            epro = topods::edge(a_sh)?;
            if let Some(param) = vert_param.seek(vtx) {
                prm = *param;
            }
        }
        let ok = project_on_face(vtx, p2d, fac, &mut epro, &mut prm)?;
        if !ok || is_projected {
            return Ok(());
        }
        let key = vtx.as_ref().clone();
        if let Some(vtx2) = coinciding_vertex(vtx, &epro, fac, prm)? {
            self.map.bind(key.clone(), vtx2);
            the_map.add(key);
        } else {
            self.map.bind(key.clone(), epro.into_shape());
            vert_param.bind(key, prm);
        }
        Ok(())
    }

    /// The projection of the vertices of the edges bound to faces.
    fn project_vertices(&mut self, the_map: &mut MapOfShape) -> Result<(), Failure> {
        let mut vert_param: VertexParameters = DataMap::default();
        for ind in 1..=self.map_ef.extent() {
            let edg = topods::edge(self.map_ef.find_key(ind)?)?;
            let fac = topods::face(self.map_ef.find_from_index(ind)?)?;
            // JAG 02.02.96 : On verifie les pcurves...
            for v in Explorer::new(&edg, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                if the_map.contains(&v) {
                    continue;
                }
                self.project_vertex(&topods::vertex(&v)?, &edg, &fac, the_map, &mut vert_param)?;
            }
        }
        Ok(())
    }

    /// `BindAll()`: once, the p-curves of the splitting edges, their split at internal
    /// intersections (when checked), the projections of their vertices, then the edges
    /// bound to edges join the edge iteration; `wires.bind_all()?`.
    pub fn bind_all(&mut self) -> Result<(), Failure> {
        if self.done {
            return Ok(());
        }
        let mut the_map = MapOfShape::default();
        self.bind_edges_on_edges(&mut the_map)?;
        self.split_at_internal_intersections()?;
        self.project_vertices(&mut the_map)?;
        //  Modified by Sergey KHROMOV - Mon Feb 12 16:26:50 2001 Begin
        let edges: Vec<(Shape, Shape)> = self
            .map
            .items()
            .filter(|(k, _)| k.shape_type().ok() == Some(ShapeEnum::Edge))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (key, value) in edges {
            self.map_ef.add(key, value);
        }
        //  Modified by Sergey KHROMOV - Mon Feb 12 16:26:52 2001 End
        self.done = true;
        Ok(())
    }
}
