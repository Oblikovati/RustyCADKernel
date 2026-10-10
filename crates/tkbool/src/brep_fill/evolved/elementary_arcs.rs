// Rust translation of the bisector loop of BRepFill_Evolved::ElementaryPerform and the
// static VertexFromNode and ComputeIntervals of
// ModelingAlgorithms/TKBool/BRepFill/BRepFill_Evolved.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The edges of the bisectors of the spine on the tubes and revolutions: for each arc
//! of the map and each profile edge, the bisector projected on the two faces it
//! separates between its vertices on the parallel edges and at the nodes.
use super::elementary::{BisectorShapes, VertexParameters};
use super::node_vertices::{NodeVertices, null_item, vertex_from_node};
use super::statics::{
    concave_side, confusion, distance_to_oz, edge_vertices, is_inversed, simple_expression,
};
use super::{BRepFillEvolved, find2, first_of};
use crate::brep_fill::BRepFillTrimSurfaceTool;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, Sequence};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_mat2d::{BRepMAT2dBisectingLocus, BRepMAT2dLinkTopoBilo};
use rustycad_tktopalgo::mat::{MatArc, MatNode};

/// The intervals of a bisector projected on its faces: parameters and end vertices.
#[derive(Default)]
struct Intervals {
    first_par: Vec<f64>,
    last_par: Vec<f64>,
    first_v: Vec<Shape>,
    last_v: Vec<Shape>,
}

impl Intervals {
    /// The interval [`u1`, `u2`] kept when its middle point projects on the faces.
    fn push_if_on_face(
        &mut self,
        trim: &BRepFillTrimSurfaceTool,
        bis: &Handle<dyn Geom2dCurve>,
        (u1, v1): (f64, &Shape),
        (u2, v2): (f64, &Shape),
    ) -> Result<(), Failure> {
        if v1.is_null() || v2.is_same(v1) {
            return Ok(());
        }
        if trim.is_on_face(bis.value((u2 + u1) * 0.5)?)? {
            self.first_par.push(u1);
            self.last_par.push(u2);
            self.first_v.push(v1.clone());
            self.last_v.push(v2.clone());
        }
        Ok(())
    }
}

/// The vertices of a bisector on the parallel edges of the first and last vertices of
/// the profile edge, with their parameters.
#[derive(Clone, Default)]
struct OnParallels {
    v_on_f: Vec<Shape>,
    v_on_l: Vec<Shape>,
    par_on_f: Vec<Point3>,
    par_on_l: Vec<Point3>,
}

/// `ComputeIntervals(VOnF, VOnL, ParOnF, ParOnL, Trim, Bis, VS, VE, ...)` (static): the
/// intervals of the bisector between consecutive vertices (by parameter, from the
/// node vertices `vs` to `ve`) whose middle projects on the faces.
fn compute_intervals(
    on: &OnParallels,
    trim: &BRepFillTrimSurfaceTool,
    bis: &Handle<dyn Geom2dCurve>,
    vs: &Option<Vertex>,
    ve: &Option<Vertex>,
) -> Result<Intervals, Failure> {
    let mut out = Intervals::default();
    let (mut i_on_f, mut i_on_l) = (0, 0);
    let (mut u1, mut v1) = (0., Shape::default());
    if let Some(vs) = vs {
        (u1, v1) = (bis.first_parameter(), vs.as_ref().clone());
    }
    while i_on_f < on.v_on_f.len() || i_on_l < on.v_on_l.len() {
        let take_f = i_on_l >= on.v_on_l.len()
            || (i_on_f < on.v_on_f.len() && on.par_on_f[i_on_f].x() < on.par_on_l[i_on_l].x());
        let (u2, v2) = if take_f {
            i_on_f += 1;
            (on.par_on_f[i_on_f - 1].x(), on.v_on_f[i_on_f - 1].clone())
        } else {
            i_on_l += 1;
            (on.par_on_l[i_on_l - 1].x(), on.v_on_l[i_on_l - 1].clone())
        };
        out.push_if_on_face(trim, bis, (u1, &v1), (u2, &v2))?;
        (u1, v1) = (u2, v2);
    }
    if let Some(ve) = ve {
        out.push_if_on_face(trim, bis, (u1, &v1), (bis.last_parameter(), ve.as_ref()))?;
    }
    Ok(out)
}

/// `seq.SetValue(i, v)` (from 1): Standard_OutOfRange outside the sequence.
fn set_value(seq: &mut [Shape], i: usize, v: &Option<Vertex>) -> Result<(), Failure> {
    let len = seq.len();
    let slot = i
        .checked_sub(1)
        .and_then(|k| seq.get_mut(k))
        .ok_or_else(|| {
            Failure::with_type(
                "Standard_OutOfRange",
                &format!("BRepFill_Evolved: vertex index {i}; expected 1..={len}"),
                "NCollection_Sequence",
            )
        })?;
    *slot = v.as_ref().map(|v| v.as_ref().clone()).unwrap_or_default();
    Ok(())
}

/// The data of one arc of the map in ElementaryPerform.
struct ArcData {
    arc: Handle<MatArc>,
    bis: Handle<dyn Geom2dCurve>,
    reverse: bool,
    s: [Shape; 2],
    concave: [bool; 2],
}

/// The faces, parallel edges and inversions of the bisector on one profile edge.
struct ArcFaces {
    f: [Face; 2],
    e: [Edge; 4],
    inv0: [bool; 2],
    inv1: [bool; 2],
}

/// The per-arc maps of the vertices and parameters already made at a profile vertex.
#[derive(Default)]
struct SeenVertices {
    ver: DataMap<Shape, Vec<Shape>, ShapeMapHasher>,
    par: DataMap<Shape, Vec<Point3>, ShapeMapHasher>,
}

impl BRepFillEvolved {
    /// The faces separated by the bisector on the profile edge and their parallel
    /// edges at its vertices.
    fn arc_faces(
        &self,
        a: &ArcData,
        prof: &Edge,
        vf: &Vertex,
        vl: &Vertex,
    ) -> Result<ArcFaces, Failure> {
        let face = |k: usize| -> Result<Face, Failure> {
            topods::face(&first_of(find2(&self.my_map, &a.s[k], prof.as_ref())?)?)
        };
        let edge = |k: usize, v: &Vertex| -> Result<Edge, Failure> {
            topods::edge(&first_of(find2(&self.my_map, &a.s[k], v.as_ref())?)?)
        };
        let f = [face(0)?, face(1)?];
        let e = [edge(0, vf)?, edge(0, vl)?, edge(1, vf)?, edge(1, vl)?];
        let inv0 = if a.concave[0] {
            is_inversed(&a.s[0], &e[0], &e[1])?
        } else {
            [false, false]
        };
        let inv1 = if a.concave[1] {
            is_inversed(&a.s[1], &e[2], &e[3])?
        } else {
            [false, false]
        };
        Ok(ArcFaces { f, e, inv0, inv1 })
    }

    /// The end nodes of the arc from the start of the bisector; a start node on a
    /// basic element at a spine vertex is replaced by the first node of the start arc
    /// of that element.
    fn arc_nodes(a: &ArcData) -> Result<[Handle<MatNode>; 2], Failure> {
        let arc = a.arc.get().ok_or_else(|| null_item("arc"))?;
        let (mut node1, node2) = if a.reverse {
            (arc.second_node(), arc.first_node())
        } else {
            (arc.first_node(), arc.second_node())
        };
        if node1.get().ok_or_else(|| null_item("node"))?.on_basic_elt() {
            let element = if a.s[0].shape_type()? == ShapeEnum::Vertex {
                Some(arc.first_element())
            } else if a.s[1].shape_type()? == ShapeEnum::Vertex {
                Some(arc.second_element())
            } else {
                None
            };
            if let Some(element) = element {
                let start_arc = element
                    .get()
                    .ok_or_else(|| null_item("basic element"))?
                    .start_arc();
                node1 = start_arc
                    .get()
                    .ok_or_else(|| null_item("arc"))?
                    .first_node();
            }
        }
        Ok([node1, node2])
    }

    /// The vertices of the bisector on the parallel edges `ea`/`eb` at a profile
    /// vertex: new ones at the intersections (the node vertices at the ends when the
    /// node is on that vertex, status `at`), none when a face is inverted, or the ones
    /// already made at that vertex.
    #[allow(clippy::too_many_arguments)] // The native inline block reads all of them.
    fn vertices_on_parallel(
        &self,
        trim: &BRepFillTrimSurfaceTool,
        (ea, eb): (&Edge, &Edge),
        inverted: bool,
        seen: Option<(&Vec<Shape>, &Vec<Point3>)>,
        at: i32,
        (start_on_f, vs): (i32, &Option<Vertex>),
        (end_on_f, ve): (i32, &Option<Vertex>),
    ) -> Result<(Vec<Shape>, Vec<Point3>), Failure> {
        if let Some((ver, par)) = seen {
            return Ok((ver.clone(), par.clone()));
        }
        if inverted {
            return Ok((Vec::new(), Vec::new()));
        }
        let mut seq: Sequence<Point3> = Sequence::default();
        trim.intersect_with(ea, eb, &mut seq)?;
        let par: Vec<Point3> = seq.iter().copied().collect();
        let mut ver = Vec::with_capacity(par.len());
        for _ in 0..par.len() {
            let mut vc = Vertex::default();
            BRepBuilder::default().make_vertex(&mut vc);
            ver.push(vc.into_shape());
        }
        if start_on_f == at {
            set_value(&mut ver, 1, vs)?;
        }
        if end_on_f == at {
            let n = par.len();
            set_value(&mut ver, n, ve)?;
        }
        Ok((ver, par))
    }

    /// The bisector edge of one interval: on the projected curve, between the interval
    /// vertices, with its p-curves on both faces and their regularity.
    fn bisector_edge(
        trim: &BRepFillTrimSurfaceTool,
        faces: &ArcFaces,
        (u1, v1): (f64, &Shape),
        (u2, v2): (f64, &Shape),
    ) -> Result<Shape, Failure> {
        let ((cbis, pcurve1, pcurve2), continuity) = trim.project(u1, u2)?;
        let b = BRepBuilder::default();
        let c = cbis.get().ok_or_else(|| null_item("bisector curve"))?;
        let (first, last) = (c.first_parameter(), c.last_parameter());
        let mut current_edge = Edge::default();
        b.make_edge_with_curve(&mut current_edge, &cbis, confusion())?;
        let (v1, v2) = (topods::vertex(v1)?, topods::vertex(v2)?);
        b.update_vertex_point(&v1, c.value(first)?, confusion())?;
        b.update_vertex_point(&v2, c.value(last)?, confusion())?;
        let mut ce = current_edge.into_shape();
        b.add(&mut ce, &v1.as_ref().oriented(Orientation::Forward))?;
        b.add(&mut ce, &v2.as_ref().oriented(Orientation::Reversed))?;
        let ce_edge = topods::edge(&ce)?;
        b.range(&ce_edge, first, last, false)?;
        b.update_edge_pcurve_on_face(&ce_edge, &pcurve1, &faces.f[0], confusion())?;
        b.update_edge_pcurve_on_face(&ce_edge, &pcurve2, &faces.f[1], confusion())?;
        b.continuity_on_faces(&ce_edge, &faces.f[0], &faces.f[1], continuity)?;
        Ok(ce)
    }

    /// The bisector edges of the intervals stored on both faces: with the orientation
    /// of the parallel edge of one face and the complement of the other's (skin when
    /// the profile edge goes away from Oz, contreskin otherwise).
    fn store_bisector_edges(
        &self,
        trim: &BRepFillTrimSurfaceTool,
        faces: &mut ArcFaces,
        intervals: &Intervals,
        skin: bool,
        map_bis: &mut BisectorShapes,
    ) -> Result<(), Failure> {
        for ti in 0..intervals.first_par.len() {
            let ce = Self::bisector_edge(
                trim,
                faces,
                (intervals.first_par[ti], &intervals.first_v[ti]),
                (intervals.last_par[ti], &intervals.last_v[ti]),
            )?;
            for f in &faces.f {
                map_bis.try_emplaced(f.as_ref().clone(), Vec::new);
            }
            let o0 = brep_tools::ori_edge_in_face(&faces.e[0], &faces.f[0])?;
            faces.e[0] = topods::edge(&faces.e[0].as_ref().oriented(o0))?;
            let o2 = brep_tools::ori_edge_in_face(&faces.e[2], &faces.f[1])?;
            faces.e[2] = topods::edge(&faces.e[2].as_ref().oriented(o2))?;
            let (first, second, o_first, o_second) =
                if skin { (0, 1, o0, o2) } else { (1, 0, o2, o0) };
            let push = |map_bis: &mut BisectorShapes, k: usize, s: Shape| {
                map_bis
                    .try_emplaced(faces.f[k].as_ref().clone(), Vec::new)
                    .push(s);
            };
            push(map_bis, first, ce.oriented(o_first));
            push(map_bis, second, ce.oriented(top_abs::complement(o_second)));
        }
        Ok(())
    }

    /// The vertices on the parallel edges `e[k]` (k from `ks`) stored with their
    /// parameters (Y on the first face, Z on the second).
    fn store_parallel_vertices(
        faces: &ArcFaces,
        ks: [usize; 2],
        ver: &[Shape],
        par: &[Point3],
        map_bis: &mut BisectorShapes,
        map_ver_par: &mut VertexParameters,
    ) {
        if ver.is_empty() {
            return;
        }
        for (j, k) in ks.into_iter().enumerate() {
            let key = faces.e[k].as_ref().clone();
            if !map_bis.is_bound(&key) {
                map_bis.bind(key.clone(), Vec::new());
                map_ver_par.bind(key.clone(), Vec::new());
            }
            for (v, p) in ver.iter().zip(par) {
                map_bis.try_emplaced(key.clone(), Vec::new).push(v.clone());
                map_ver_par
                    .try_emplaced(key.clone(), Vec::new)
                    .push(if j == 0 { p.y() } else { p.z() });
            }
        }
    }

    /// The bisector of one arc on one profile edge: its vertices on the parallel edges
    /// and at the nodes, its edges on the two faces, the vertices recorded on the
    /// parallel edges.
    fn arc_on_profile_edge(
        &self,
        a: &ArcData,
        prof: &Edge,
        on: &mut OnParallels,
        seen: &mut SeenVertices,
        node_vertices: &mut NodeVertices,
        maps: (&mut BisectorShapes, &mut VertexParameters),
    ) -> Result<(), Failure> {
        let (map_bis, map_ver_par) = maps;
        let (vf, vl) = edge_vertices(prof);
        let mut faces = self.arc_faces(a, prof, &vf, &vl)?;
        let trim = BRepFillTrimSurfaceTool::new(
            &a.bis,
            &faces.f[0],
            &faces.f[1],
            &faces.e[0],
            &faces.e[2],
            faces.inv0[0],
            faces.inv1[0],
        );
        let [node1, node2] = Self::arc_nodes(a)?;
        let (start_on_f, vs) = vertex_from_node(&node1, prof, &vf, &vl, node_vertices)?;
        let (end_on_f, ve) = vertex_from_node(&node2, prof, &vf, &vl, node_vertices)?;
        let seen_f = seen.ver.seek(vf.as_ref()).zip(seen.par.seek(vf.as_ref()));
        let first_new = seen_f.is_none();
        (on.v_on_f, on.par_on_f) = self.vertices_on_parallel(
            &trim,
            (&faces.e[0], &faces.e[2]),
            faces.inv0[0] || faces.inv1[0],
            seen_f,
            1,
            (start_on_f, &vs),
            (end_on_f, &ve),
        )?;
        let seen_l = seen.ver.seek(vl.as_ref()).zip(seen.par.seek(vl.as_ref()));
        let last_new = seen_l.is_none();
        (on.v_on_l, on.par_on_l) = self.vertices_on_parallel(
            &trim,
            (&faces.e[1], &faces.e[3]),
            faces.inv0[1] || faces.inv1[1],
            seen_l,
            3,
            (start_on_f, &vs),
            (end_on_f, &ve),
        )?;
        if start_on_f == 0 && end_on_f == 0 && on.v_on_l.len() + on.v_on_f.len() <= 1 {
            // No trace of the bisector on the face, or only a node on an edge.
            return Ok(());
        }
        let intervals = compute_intervals(on, &trim, &a.bis, &vs, &ve)?;
        let skin = distance_to_oz(&vf)? < distance_to_oz(&vl)?;
        self.store_bisector_edges(&trim, &mut faces, &intervals, skin, map_bis)?;
        if first_new {
            Self::store_parallel_vertices(
                &faces,
                [0, 2],
                &on.v_on_f,
                &on.par_on_f,
                map_bis,
                map_ver_par,
            );
        }
        if last_new {
            Self::store_parallel_vertices(
                &faces,
                [1, 3],
                &on.v_on_l,
                &on.par_on_l,
                map_bis,
                map_ver_par,
            );
        }
        // Edge [1] of the current face will be edge [0] of the next face.
        seen.par.bind(vf.as_ref().clone(), on.par_on_f.clone());
        seen.ver.bind(vf.as_ref().clone(), on.v_on_f.clone());
        seen.par.bind(vl.as_ref().clone(), on.par_on_l.clone());
        seen.ver.bind(vl.as_ref().clone(), on.v_on_l.clone());
        Ok(())
    }

    /// The edges of the bisectors of every arc of the map on the faces it separates.
    pub(super) fn bisector_edges(
        &self,
        locus: &BRepMAT2dBisectingLocus,
        link: &BRepMAT2dLinkTopoBilo,
        map_bis: &mut BisectorShapes,
        map_ver_par: &mut VertexParameters,
    ) -> Result<(), Failure> {
        let graph = locus.graph();
        let graph = graph.get().ok_or_else(|| null_item("graph"))?;
        let mut node_vertices = NodeVertices::default();
        for i in 1..=graph.number_of_arcs() {
            let arc = graph.arc(i)?;
            let (bisec, reverse) = locus.geom_bis(&arc)?;
            let bis = simple_expression(&bisec)?;
            let arc_ref = arc.get().ok_or_else(|| null_item("arc"))?;
            let s = [
                link.generating_shape(&arc_ref.first_element())?,
                link.generating_shape(&arc_ref.second_element())?,
            ];
            let concave = [
                concave_side(&s[0], &self.my_spine)?,
                concave_side(&s[1], &self.my_spine)?,
            ];
            let a = ArcData {
                arc: arc.clone(),
                bis,
                reverse,
                s,
                concave,
            };
            let (mut on, mut seen) = (OnParallels::default(), SeenVertices::default());
            let mut prof_exp =
                rustycad_tkbrep::brep_tools::WireExplorer::with_wire(&self.my_profile)?;
            while prof_exp.more() {
                let prof = prof_exp.current().clone();
                self.arc_on_profile_edge(
                    &a,
                    &prof,
                    &mut on,
                    &mut seen,
                    &mut node_vertices,
                    (map_bis, map_ver_par),
                )?;
                prof_exp.next()?;
            }
        }
        Ok(())
    }
}
