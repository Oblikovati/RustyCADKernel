// Rust translation of BRepFill_OffsetWire::PerformWithBiLo
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_OffsetWire.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The offset with a given bisecting locus: the parallels of the elements, the offset
//! vertices on each bisector, the parallels trimmed between them, the merged vertices
//! substituted, the wires chained and their vertices and 3d curves updated.
use super::BRepFillOffsetWire;
use super::kpart::{compute, k_part_circle};
use super::parallels::{NodeVertices, make_circle, make_offset, vertex_from_node};
use super::trim::{BisectorPoints, Detromp, MergedVertices, ParallelVertices, trim_edge};
use crate::brep_fill::BRepFillTrimEdgeTool;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, ShapeIterator, Vertex};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List, Sequence};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomGeometry, GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkmath::gp::{self, Point3, Vector3};
use rustycad_tktopalgo::brep_mat2d::{BRepMAT2dBisectingLocus, BRepMAT2dLinkTopoBilo};
use rustycad_tktopalgo::mat::{MatArc, MatNode};

/// The vertices and parameters found on each parallel edge, the node vertices and the
/// proofing, accumulated over the arcs of the locus.
#[derive(Default)]
struct OffsetVertices {
    map_node_vertex: NodeVertices,
    map_bis: DataMap<Shape, Vec<Shape>, ShapeMapHasher>,
    map_ver_par: DataMap<Shape, Vec<f64>, ShapeMapHasher>,
    detromp: Detromp,
}

/// The bisector of an arc with its elements, its parallels and its end nodes.
struct ArcElements {
    s: [Shape; 2],
    e: [Edge; 2],
    nodes: [Handle<MatNode>; 2],
}

/// The node of a handle, Standard_NullObject for a null one.
fn node_of(node: &Handle<MatNode>) -> Result<&MatNode, Failure> {
    node.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "BRepFill_OffsetWire: null bisector node; expected a node",
            "MAT_Arc",
        )
    })
}

/// The parameters bound to an edge, appended.
fn push_on<T: Clone>(
    map: &mut DataMap<Shape, Vec<T>, ShapeMapHasher>,
    e: &Edge,
    items: impl Iterator<Item = T>,
) -> Result<(), Failure> {
    let list = map.change_seek(e.as_ref()).ok_or_else(|| {
        Failure::with_type(
            "Standard_NoSuchObject",
            "BRepFill_OffsetWire: parallel edge without vertices; expected a bound edge",
            "NCollection_DataMap",
        )
    })?;
    list.extend(items);
    Ok(())
}

impl BRepFillOffsetWire {
    /// `PerformWithBiLo(WSP, Offset, Locus, Link, Join, Alt)` (natively defaulted to
    /// GeomAbs_Arc and Alt 0): the offset of the face with the given bisecting locus
    /// and its links to the topology; a null offset copies the spine;
    /// `ow.perform_with_bilo(&face, 0.5, &locus, &mut link, JoinType::Arc, 0.)?`.
    pub fn perform_with_bilo(
        &mut self,
        wsp: &Face,
        offset: f64,
        locus: &BRepMAT2dBisectingLocus,
        link: &mut BRepMAT2dLinkTopoBilo,
        join: JoinType,
        alt: f64,
    ) -> Result<(), Failure> {
        self.reset_for(wsp, offset, join)?;
        if self.my_offset.abs() < precision::confusion() {
            compute(&self.my_spine, &mut self.my_shape, &mut self.my_map, alt)?;
            self.my_is_done = true;
            return Ok(());
        }
        let work = self.my_work_spine.clone();
        if k_part_circle(
            &work,
            offset,
            self.my_is_open_result,
            alt,
            (&mut self.my_shape, &mut self.my_map, &mut self.my_is_done),
        )? {
            return Ok(());
        }
        let ref_plane: Handle<dyn GeomSurface> = self.reference_plane(alt)?;
        let ends: [Vertex; 2] = self.open_ends()?;
        if locus.number_of_contours() == 0 {
            return Ok(());
        }
        self.make_parallels(locus, link, &ref_plane, &ends)?;
        self.remove_hanging_arcs()?;
        let mut st = OffsetVertices::default();
        let graph = locus.graph();
        let graph = graph.get().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "BRepFill_OffsetWire: null graph of the locus; expected a computed locus",
                "BRepMAT2d_BisectingLocus",
            )
        })?;
        for i in 1..=graph.number_of_arcs() {
            self.offset_vertices_of_arc(&graph.arc(i)?, (locus, link), &ends, &mut st)?;
        }
        let map_vv: MergedVertices = self.trim_parallels(&mut st)?;
        self.substitute_merged_vertices(&map_vv)?;
        self.make_wires()?;
        self.update_offset_vertices(&ref_plane)?;
        if !self.my_is_open_result {
            self.fix_holes()?;
        }
        self.my_is_done = true;
        Ok(())
    }

    /// The state before an offset: the forward work spine, the join, the offset, the
    /// spine when none, no shape and no parallels.
    fn reset_for(&mut self, wsp: &Face, offset: f64, join: JoinType) -> Result<(), Failure> {
        self.my_is_done = false;
        self.my_work_spine = topods::face(&wsp.oriented(Orientation::Forward))?;
        self.my_join_type = join;
        self.my_offset = offset;
        self.my_shape = Shape::default();
        if self.my_spine.is_null() {
            self.my_spine = topods::face(&wsp.oriented(Orientation::Forward))?;
        }
        self.my_map.clear(true);
        Ok(())
    }

    /// The plane of the work spine, moved by `Alt` along its normal (opposite for a
    /// reversed spine).
    fn reference_plane(&self, alt: f64) -> Result<Handle<dyn GeomSurface>, Failure> {
        let ref_plane: Handle<GeomPlane> = BRepTool::surface(&self.my_work_spine)?.cast();
        if alt.abs() <= gp::resolution() {
            return Ok(ref_plane.cast());
        }
        let an_alt: f64 = if self.my_work_spine.orientation() == Orientation::Reversed {
            -alt
        } else {
            alt
        };
        let plane = ref_plane.get().ok_or_else(not_plane)?;
        let normal: Vector3 = Vector3::from(plane.pln().axis().direction());
        let moved: Handle<dyn GeomGeometry> = plane.translated(normal * an_alt)?;
        Ok(moved.cast::<GeomPlane>().cast())
    }

    /// The end vertices of the first wire of an open result (null otherwise).
    fn open_ends(&self) -> Result<[Vertex; 2], Failure> {
        if !self.my_is_open_result {
            return Ok([Vertex::default(), Vertex::default()]);
        }
        let iter = ShapeIterator::new(self.my_spine.as_ref(), true, true);
        let the_wire = topods::wire(iter.value()?)?;
        let (e0, e1) = topexp::wire_vertices(&the_wire);
        Ok([e0, e1])
    }

    /// The circles at the vertices (but the ends of an open result) and the offsets of
    /// the edges, contour by contour.
    fn make_parallels(
        &mut self,
        locus: &BRepMAT2dBisectingLocus,
        link: &BRepMAT2dLinkTopoBilo,
        ref_plane: &Handle<dyn GeomSurface>,
        ends: &[Vertex; 2],
    ) -> Result<(), Failure> {
        for ic in 1..=locus.number_of_contours() {
            let nb: i32 = locus.number_of_elts(ic)?;
            let mut pe: Shape = link.generating_shape(&locus.basic_elt(ic, nb)?)?;
            for ie in 1..=nb {
                let se: Shape = link.generating_shape(&locus.basic_elt(ic, ie)?)?;
                if se.shape_type()? == ShapeEnum::Vertex {
                    if !se.is_same(&ends[0]) && !se.is_same(&ends[1]) {
                        make_circle(
                            &topods::edge(&pe)?,
                            &topods::vertex(&se)?,
                            &self.my_work_spine,
                            self.my_offset,
                            &mut self.my_map,
                            ref_plane,
                        )?;
                    }
                    continue;
                }
                make_offset(
                    &topods::edge(&se)?,
                    &self.my_work_spine,
                    self.my_offset,
                    &mut self.my_map,
                    ref_plane,
                    self.my_is_open_result,
                    self.my_join_type,
                    ends,
                )?;
                pe = se;
            }
        }
        Ok(())
    }

    /// The possible hanging arcs on the vertices at both ends of an open Arc result.
    fn remove_hanging_arcs(&mut self) -> Result<(), Failure> {
        if !self.my_is_open_result || self.my_join_type != JoinType::Arc {
            return Ok(());
        }
        if !self.my_map.is_empty() && self.my_map.find_key(1)?.shape_type()? == ShapeEnum::Vertex {
            self.my_map.remove_from_index(1)?;
        }
        if !self.my_map.is_empty()
            && self.my_map.find_key(self.my_map.extent())?.shape_type()? == ShapeEnum::Vertex
        {
            self.my_map.remove_last()?;
        }
        Ok(())
    }

    /// The elements, the parallels and the end nodes (from the start of the bisector)
    /// of an arc; `None` when an element has no parallel.
    fn arc_elements(
        &self,
        arc: &MatArc,
        link: &BRepMAT2dLinkTopoBilo,
        reverse: bool,
    ) -> Result<Option<ArcElements>, Failure> {
        let s: [Shape; 2] = [
            link.generating_shape(&arc.first_element())?,
            link.generating_shape(&arc.second_element())?,
        ];
        let (Some(l0), Some(l1)) = (self.my_map.seek(&s[0]), self.my_map.seek(&s[1])) else {
            return Ok(None);
        };
        let first_edge = |l: &List<Shape>| -> Result<Edge, Failure> {
            topods::edge(l.first().ok_or_else(not_plane)?)
        };
        let e: [Edge; 2] = [first_edge(l0)?, first_edge(l1)?];
        let nodes = if reverse {
            [arc.second_node(), arc.first_node()]
        } else {
            [arc.first_node(), arc.second_node()]
        };
        Ok(Some(ArcElements { s, e, nodes }))
    }

    /// The vertex of a finite node on the offset.
    fn node_vertex(
        &self,
        locus: &BRepMAT2dBisectingLocus,
        node: &Handle<MatNode>,
        st: &mut OffsetVertices,
    ) -> Result<Option<Vertex>, Failure> {
        if node_of(node)?.infinite() {
            return Ok(None);
        }
        let a_local_pnt2d = locus.geom_elt_of_node(node)?;
        vertex_from_node(node, self.my_offset, a_local_pnt2d, &mut st.map_node_vertex)
    }

    /// The offset vertices of one arc: its intersections with both parallels and its
    /// end nodes on the offset, recorded on the parallels and in the proofing.
    fn offset_vertices_of_arc(
        &self,
        current_arc: &Handle<MatArc>,
        (locus, link): (&BRepMAT2dBisectingLocus, &BRepMAT2dLinkTopoBilo),
        ends: &[Vertex; 2],
        st: &mut OffsetVertices,
    ) -> Result<(), Failure> {
        let (bisec, reverse) = locus.geom_bis(current_arc)?;
        let arc = current_arc.get().ok_or_else(not_plane)?;
        let Some(el) = self.arc_elements(arc, link, reverse)? else {
            return Ok(());
        };
        let mut vs = self.node_vertex(locus, &el.nodes[0], st)?;
        let mut ve = self.node_vertex(locus, &el.nodes[1], st)?;
        if self.my_join_type == JoinType::Intersection {
            (vs, ve) = (None, None);
        }
        let s1 = locus.geom_elt_of_basic_elt(&arc.first_element())?;
        let s2 = locus.geom_elt_of_basic_elt(&arc.second_element())?;
        let trim = BRepFillTrimEdgeTool::new(&bisec, &s1, &s2, self.my_offset)?;
        let mut params: Sequence<Point3> = Sequence::default();
        trim.intersect_with(
            &el.e[0],
            &el.e[1],
            &el.s[0],
            &el.s[1],
            &ends[0],
            &ends[1],
            self.my_join_type,
            self.my_is_open_result,
            &mut params,
        )?;
        let mut vertices: Vec<Shape> = bisector_vertices(&bisec, &params)?;
        if let Some(vs) = &vs {
            trim.add_or_confuse(true, &el.e[0], &el.e[1], &mut params)?;
            if params.len() == vertices.len() && !params.is_empty() {
                vertices[0] = vs.as_ref().clone();
            } else {
                // The point was not found by IntersectWith.
                vertices.insert(0, vs.as_ref().clone());
            }
        }
        if let Some(ve) = &ve {
            trim.add_or_confuse(false, &el.e[0], &el.e[1], &mut params)?;
            if params.len() == vertices.len() && !params.is_empty() {
                let last = params.len() - 1;
                vertices[last] = ve.as_ref().clone();
            } else {
                // The point was not found by IntersectWith.
                vertices.push(ve.as_ref().clone());
            }
        }
        let params: Vec<Point3> = params.iter().copied().collect();
        for s in &el.s {
            if !st.detromp.is_bound(s) {
                st.detromp.bind(s.clone(), List::default());
            }
        }
        let points = BisectorPoints {
            vertices: &vertices,
            params: &params,
            bisec: &bisec,
            on_edge: (vs.is_some(), ve.is_some()),
            trim: &trim,
        };
        self.update_detromp(&mut st.detromp, (&el.s[0], &el.s[1]), &points)?;
        self.store_on_parallels(&el, (&vertices, &params), st)
    }

    /// The vertices and their parameters stored on both parallels; without coherent
    /// vertices, the parallels of a complete circle kept whole when both nodes are
    /// farther than the offset.
    fn store_on_parallels(
        &self,
        el: &ArcElements,
        (vertices, params): (&[Shape], &[Point3]),
        st: &mut OffsetVertices,
    ) -> Result<(), Failure> {
        if !vertices.is_empty() && params.len() == vertices.len() {
            for (k, e) in el.e.iter().enumerate() {
                if !st.map_bis.is_bound(e.as_ref()) {
                    st.map_bis.bind(e.as_ref().clone(), Vec::new());
                    st.map_ver_par.bind(e.as_ref().clone(), Vec::new());
                }
                push_on(&mut st.map_bis, e, vertices.iter().cloned())?;
                push_on(
                    &mut st.map_ver_par,
                    e,
                    params.iter().map(|p| if k == 0 { p.y() } else { p.z() }),
                )?;
            }
            return Ok(());
        }
        // For complete circles: the parallel can lie in the zone without intersection
        // with the border; it is valid when the offset is below the node distances.
        let far: bool = node_of(&el.nodes[0])?.distance() > self.my_offset
            && node_of(&el.nodes[1])?.distance() > self.my_offset;
        for e in &el.e {
            if !st.map_bis.is_bound(e.as_ref()) && far {
                st.map_bis.bind(e.as_ref().clone(), Vec::new());
                st.map_ver_par.bind(e.as_ref().clone(), Vec::new());
            }
        }
        Ok(())
    }

    /// The parallels trimmed between their kept vertices (whole for complete circles),
    /// replacing them in the map; the merged vertices.
    fn trim_parallels(&mut self, st: &mut OffsetVertices) -> Result<MergedVertices, Failure> {
        let mut map_vv = MergedVertices::default();
        let extent: usize = self.my_map.extent();
        for j in 1..=extent {
            let current_spine: Shape = self.my_map.find_key(j)?.clone();
            let list = self.my_map.change_from_index(j)?;
            let current_edge: Edge = topods::edge(list.first().ok_or_else(not_plane)?)?;
            list.clear();
            let (Some(the_ver), Some(the_par)) = (
                st.map_bis.change_seek(current_edge.as_ref()),
                st.map_ver_par.change_seek(current_edge.as_ref()),
            ) else {
                continue;
            };
            if the_ver.is_empty() {
                // Complete circles.
                self.my_map
                    .change_from_index(j)?
                    .append(current_edge.into_shape());
                continue;
            }
            let ind_of_e: i32 = match (self.my_is_open_result, j) {
                (true, 1) => 1,
                (true, j) if j == extent => -1,
                _ => 0,
            };
            let detromp = st.detromp.seek(&current_spine).ok_or_else(not_plane)?;
            let p = ParallelVertices {
                e: &current_edge,
                pro_e: &current_spine,
                all_spine: &self.my_spine,
                detromp,
                ind_of_e,
            };
            let parts: Vec<Shape> = trim_edge(&p, the_ver, the_par, &mut map_vv)?;
            let list = self.my_map.change_from_index(j)?;
            for part in parts {
                list.append(part);
            }
        }
        Ok(map_vv)
    }
}

/// The vertices on the bisector at the parameters of the intersections (on Z = 0).
fn bisector_vertices(
    bisec: &rustycad_tktopalgo::bisector::Bisec,
    params: &Sequence<Point3>,
) -> Result<Vec<Shape>, Failure> {
    let builder = BRepBuilder::default();
    let bis = bisec.value();
    let bis = bis.get().ok_or_else(not_plane)?;
    let mut vertices: Vec<Shape> = Vec::new();
    for p in params.iter() {
        let p2 = bis.value(p.x())?;
        let mut vc = Vertex::default();
        builder.make_vertex(&mut vc);
        builder.update_vertex_point(
            &vc,
            Point3::new(p2.x(), p2.y(), 0.),
            precision::confusion(),
        )?;
        vertices.push(vc.into_shape());
    }
    Ok(vertices)
}

/// Standard_NullObject of a missing plane, curve or list item (natively a crash).
fn not_plane() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BRepFill_OffsetWire: null plane, curve or item; expected a non-null value",
        "BRepFill_OffsetWire",
    )
}
